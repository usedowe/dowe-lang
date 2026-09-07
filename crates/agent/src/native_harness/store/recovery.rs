use super::*;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

impl HarnessStore {
    pub fn session_inventory(&self) -> AgentResult<Vec<Value>> {
        let directory = self.directory().join("sessions");
        reject_symlink_ancestors(&directory)?;
        if !directory.exists() {
            return Ok(vec![]);
        }
        let mut rows = Vec::new();
        for entry in fs::read_dir(directory)? {
            let path = entry?.path();
            if path.extension().and_then(|part| part.to_str()) != Some("json") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|part| part.to_str()) else {
                continue;
            };
            if id.len() != 32 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                continue;
            }
            let row = match self.read_session(id, false) {
                Ok(session) => {
                    json!({"id":id,"state":if session.interrupted {"interrupted"} else {"recorded"},"revision":session.revision,"catalog_compatible":session.catalog == super::super::catalog::catalog_fingerprint()?})
                }
                Err(_) => json!({"id":id,"state":"unreadable","original_preserved":true}),
            };
            rows.push(row);
        }
        rows.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        Ok(rows)
    }

    pub fn inspect_session(&self, id: &str, offset: usize) -> AgentResult<Value> {
        let task = self.lease_session(id);
        let busy = task.is_err();
        self.inspection(id, offset, busy)
    }

    fn inspection(&self, id: &str, offset: usize, busy: bool) -> AgentResult<Value> {
        let path = self.session_path(id)?;
        let _read = AuthFileLock::acquire(&path.with_extension("lock"))?;
        let bytes = read_bounded(&path)?;
        let session = self.decode_session(id, false, &bytes)?;
        if offset > session.events.len() {
            return Err(AgentError::new("inspection offset exceeds event history"));
        }
        let mut paths = BTreeSet::new();
        let mut pending: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        let mut pending_watches = Vec::new();
        let mut watches = BTreeSet::new();
        for (index, event) in session.events.iter().enumerate() {
            match event["event"].as_str() {
                Some("approval_required")
                    if matches!(
                        event["approval"]["call"]["name"].as_str(),
                        Some("write_file" | "edit_file")
                    ) =>
                {
                    if let Some(path) = event["approval"]["details"]["path"].as_str() {
                        paths.insert(path.to_string());
                    }
                }
                Some("operation_started") => {
                    pending
                        .entry(event["call_id"].as_str().unwrap_or("unknown").into())
                        .or_default()
                        .push(index);
                }
                Some("operation_finished")
                    if event["failed"] == false && event["result"].is_object() =>
                {
                    if let Some(entries) =
                        pending.get_mut(event["call_id"].as_str().unwrap_or("unknown"))
                        && entries.len() == 1
                    {
                        entries.clear();
                    }
                }
                Some("watch_starting") => pending_watches.push(index),
                Some("watch_started") => {
                    if pending_watches.len() == 1 {
                        pending_watches.clear();
                    }
                    if let Some(id) = event["id"].as_str() {
                        watches.insert(id.to_string());
                    }
                }
                Some("watch_stopped")
                    if event["result"]["success"].is_boolean()
                        && event["result"]["error"].is_null() =>
                {
                    if let Some(id) = event["id"].as_str() {
                        watches.remove(id);
                    }
                }
                _ => {}
            }
        }
        let unresolved =
            pending.values().map(Vec::len).sum::<usize>() + pending_watches.len() + watches.len();
        let files = paths
            .iter()
            .take(32)
            .map(|path| match self.file_fingerprint(path) {
                Ok(hash) => json!({"path":path,"fingerprint":hash,"state":"current_bytes_only"}),
                Err(_) => json!({"path":path,"state":"unavailable_or_out_of_scope"}),
            })
            .collect::<Vec<_>>();
        let processes = self
            .processes()?
            .into_iter()
            .filter(|record| record["session"] == id)
            .collect::<Vec<_>>();
        let ticket = digest(&serde_json::to_vec(
            &json!({"history":digest(&bytes),"files":files,"processes":processes,"busy":busy}),
        )?);
        let mut events = Vec::new();
        for (index, event) in session.events.iter().enumerate().skip(offset).take(32) {
            let value = super::super::privacy::bounded_value(event.clone(), 8192);
            let preview = json!({"index":index,"truncated":value != *event,"value":value});
            events.push(preview);
            if serde_json::to_vec(&events)?.len() > 16384 {
                events.pop();
                break;
            }
        }
        let next = offset + events.len();
        Ok(
            json!({"id":id,"revision":session.revision,"ticket":ticket,"busy":busy,"interrupted":session.interrupted,"catalog_compatible":session.catalog == super::super::catalog::catalog_fingerprint()?,"event_count":session.events.len(),"offset":offset,"next_offset":(next < session.events.len()).then_some(next),"events":events,"unresolved_operations":unresolved,"files":files,"files_not_checked":paths.len().saturating_sub(32),"processes":processes,"policy":"Recorded receipts and current file hashes are not proof of causation or permission. Inspect uncertain effects manually; never replay operations."}),
        )
    }

    pub fn recover_session(
        &self,
        id: &str,
        ticket: &str,
        objective: &str,
    ) -> AgentResult<HarnessSession> {
        if objective.trim().is_empty() || objective.len() > 4096 {
            return Err(AgentError::new(
                "recovery requires a local objective of 1..4096 bytes",
            ));
        }
        let _task = self.lease_session(id)?;
        let inspection = self.inspection(id, 0, false)?;
        if inspection["ticket"] != ticket {
            return Err(AgentError::new(
                "recovery evidence changed; inspect again before confirming",
            ));
        }
        if inspection["processes"]
            .as_array()
            .is_some_and(|records| records.iter().any(|record| record["state"] == "active"))
        {
            return Err(AgentError::new(
                "source session still owns active processes; inspect its owner before recovering",
            ));
        }
        let mut session = self.create_session()?;
        session.summary = Some(json!({"objective":Redactor::for_project(&self.root).text(objective.trim()),"source_session":id,"source_revision":inspection["revision"],"unresolved_operations":inspection["unresolved_operations"],"policy":"Untrusted recovery reference only; never replay old operations. Inspect current files and uncertain effects. No previous approval or execution context was transferred."}).to_string());
        session.events.push(json!({"event":"session_recovered","source_session":id,"source_revision":inspection["revision"],"inspection_ticket":ticket,"replayed_operations":0}));
        self.save_session(&mut session)?;
        Ok(session)
    }
}

#[cfg(test)]
#[path = "recovery_tests.rs"]
mod tests;
