use super::*;

pub(super) fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl HarnessStore {
    pub(super) fn file_fingerprint(&self, relative: &str) -> AgentResult<String> {
        let path = Path::new(relative);
        if path.is_absolute()
            || path.components().any(|part| {
                !matches!(part, std::path::Component::Normal(_))
                    || part.as_os_str().to_string_lossy().starts_with('.')
            })
        {
            return Err(AgentError::new(
                "memory evidence requires a relative non-private source path",
            ));
        }
        let path = fs::canonicalize(super::super::HarnessTools::checked_path(
            &self.root, relative,
        )?)?;
        if !path.starts_with(&self.root) {
            return Err(AgentError::new("memory evidence escapes project"));
        }
        Ok(digest(&read_bounded(&path)?))
    }

    #[cfg(test)]
    pub(crate) fn propose_decisions(
        &self,
        session: &str,
        summary: &Value,
    ) -> AgentResult<Vec<String>> {
        let source = self.load_session(session)?;
        let source_summary = source.summary.as_ref().ok_or_else(|| {
            AgentError::new("automatic memory requires a persisted source summary")
        })?;
        if serde_json::from_str::<Value>(source_summary).ok().as_ref() != Some(summary) {
            return Err(AgentError::new(
                "memory candidate summary does not match persisted provenance",
            ));
        }
        let decisions = summary["decisions"].as_array().cloned().unwrap_or_default();
        let mut ids = Vec::new();
        for decision in decisions.iter().filter_map(Value::as_str).take(3) {
            if decision.trim().is_empty() {
                continue;
            }
            let title = decision.chars().take(60).collect::<String>();
            let id = self.remember(&title, decision, &format!("session:{session}"), false)?;
            ids.push(id);
        }
        let path = self.memory_path()?;
        reject_symlink_ancestors(&path)?;
        let _lock = AuthFileLock::acquire(&path.with_extension("lock"))?;
        let mut memories = self.memories()?;
        let validity = MemoryValidity {
            catalog: Some(source.catalog),
            source_summary_hash: Some(digest(source_summary.as_bytes())),
            invalidated: None,
        };
        for id in &ids {
            if let Some(item) = memories.observations.iter_mut().find(|item| &item.id == id) {
                item.validity = validity.clone();
            }
        }
        write_private_json(&path, &memories)?;
        Ok(ids)
    }

    pub fn confirm_memory(&self, id: &str) -> AgentResult<()> {
        let path = self.memory_path()?;
        reject_symlink_ancestors(&path)?;
        let _lock = AuthFileLock::acquire(&path.with_extension("lock"))?;
        let mut memories = self.memories()?;
        let observation = memories
            .observations
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| AgentError::new("memory id not found"))?;
        if let Some(reason) = validity::MemoryReview::new(self)?.reason(observation) {
            return Err(AgentError::new(format!(
                "memory requires review ({reason}); use an explicit update before confirming"
            )));
        }
        observation.confirmed = true;
        observation.updated = now();
        write_private_json(&path, &memories)
    }

    pub fn link_memory(&self, id: &str, files: &[String]) -> AgentResult<()> {
        if files.len() > 32 {
            return Err(AgentError::new(
                "memory supports at most 32 source references",
            ));
        }
        let fingerprints = files
            .iter()
            .map(|file| Ok((file.clone(), self.file_fingerprint(file)?)))
            .collect::<AgentResult<std::collections::BTreeMap<_, _>>>()?;
        let path = self.memory_path()?;
        reject_symlink_ancestors(&path)?;
        let _lock = AuthFileLock::acquire(&path.with_extension("lock"))?;
        let mut memories = self.memories()?;
        let observation = memories
            .observations
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| AgentError::new("memory id not found"))?;
        if validity::MemoryReview::new(self)?
            .reason(observation)
            .is_some()
        {
            observation.confirmed = false;
        }
        observation.files = fingerprints;
        observation.updated = now();
        write_private_json(&path, &memories)
    }
}
