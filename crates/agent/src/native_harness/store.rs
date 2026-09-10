use super::lock::DataLock as AuthFileLock;
mod candidates;
mod processes;
mod recovery;
mod validity;
use super::{HarnessConfig, HarnessTurn, Redactor, SessionRecord, digest, identifier};
use crate::auth::write_private_json;
use crate::{AgentError, AgentResult};
use candidates::now;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
pub use validity::MemoryValidity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessSession {
    pub schema: u32,
    pub id: String,
    pub project: String,
    #[serde(default)]
    pub catalog: String,
    /// Embedded provider/model/capability/skill authority used to validate replay compatibility.
    #[serde(default)]
    pub authority_fingerprint: String,
    pub revision: u64,
    pub turns: Vec<HarnessTurn>,
    pub events: Vec<Value>,
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_prompt_preview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orchestration: Option<SessionRecord>,
    pub context_start: usize,
    pub interrupted: bool,
}

impl HarnessSession {
    /// Record metadata only after the first provider response is accepted.
    pub(crate) fn set_initial_prompt_metadata(&mut self, prompt: &str) {
        if self.initial_prompt_preview.is_some() {
            return;
        }
        let preview = sanitize_session_text(prompt, 240);
        if preview.is_empty() {
            return;
        }
        self.initial_prompt_preview = Some(preview);
        let title_source = prompt.lines().find(|line| !line.trim().is_empty()).unwrap_or(prompt);
        self.title = Some(sanitize_session_text(title_source, 80));
    }

    pub fn orchestration(&self) -> Option<&SessionRecord> {
        self.orchestration.as_ref()
    }

    pub fn attach_orchestration(&mut self, record: SessionRecord) -> AgentResult<()> {
        self.validate_orchestration_id(&record)?;
        if self.orchestration.is_some() {
            return Err(AgentError::new("orchestration record is already attached"));
        }
        self.orchestration = Some(record);
        Ok(())
    }

    pub fn update_orchestration(&mut self, record: SessionRecord) -> AgentResult<()> {
        self.validate_orchestration_id(&record)?;
        if self.orchestration.is_none() {
            return Err(AgentError::new("orchestration record is not attached"));
        }
        self.orchestration = Some(record);
        Ok(())
    }

    fn validate_orchestration_id(&self, record: &SessionRecord) -> AgentResult<()> {
        if record.id != self.id {
            return Err(AgentError::new(
                "orchestration record belongs to a different native session",
            ));
        }
        Ok(())
    }

    pub fn usage(&self) -> crate::AgentUsageTotals {
        self.usage_since(0)
    }

    pub(crate) fn usage_since(&self, offset: usize) -> crate::AgentUsageTotals {
        let mut totals = crate::AgentUsageTotals::default();
        self.record_usage_since(offset, &mut totals);
        totals
    }

    pub(crate) fn record_usage_since(&self, offset: usize, totals: &mut crate::AgentUsageTotals) {
        let mut requests = std::collections::BTreeSet::new();
        for event in self.events.iter().skip(offset) {
            let attempt = event["event"] == "request_attempt";
            let id = event["requestId"].as_str();
            if !attempt && id.is_some_and(|id| requests.contains(id)) {
                continue;
            }
            if attempt
                || matches!(
                    event["event"].as_str(),
                    Some("response_received" | "context_compacted" | "auxiliary_response")
                )
            {
                if let Some(id) = id {
                    requests.insert(id);
                }
                let usage =
                    serde_json::from_value::<crate::AgentUsage>(event["usage"].clone()).ok();
                totals.record_usage(
                    event["provider"].as_str().unwrap_or_default(),
                    event["model"].as_str().unwrap_or_default(),
                    usage,
                );
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HarnessTaskState {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessQueuedTask {
    pub id: String,
    pub session: String,
    pub prompt: String,
    pub state: HarnessTaskState,
    pub created_at: u64,
    pub updated_at: u64,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub result: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HarnessTaskQueue {
    schema: u32,
    tasks: Vec<HarnessQueuedTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryObservation {
    pub id: String,
    pub project: String,
    pub title: String,
    pub content: String,
    pub source: String,
    pub confirmed: bool,
    #[serde(default)]
    pub validity: MemoryValidity,
    pub created: u64,
    #[serde(default)]
    pub updated: u64,
    #[serde(default)]
    pub files: std::collections::BTreeMap<String, String>,
    #[serde(default = "default_memory_kind")]
    pub kind: String,
}

fn default_memory_kind() -> String {
    "decision".into()
}

fn sanitize_session_text(text: &str, max_bytes: usize) -> String {
    let mut output = String::new();
    for character in text.chars() {
        if character.is_control() {
            if character.is_whitespace() {
                if !output.ends_with(' ') {
                    output.push(' ');
                }
            }
        } else {
            output.push(character);
        }
        if output.len() >= max_bytes {
            while output.len() > max_bytes {
                output.pop();
            }
            break;
        }
    }
    output.trim().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Memories {
    schema: u32,
    observations: Vec<MemoryObservation>,
}

#[derive(Clone)]
pub struct HarnessStore {
    base: PathBuf,
    root: PathBuf,
    project: String,
}

impl HarnessStore {
    pub fn new(base: impl AsRef<Path>, root: impl AsRef<Path>) -> AgentResult<Self> {
        let root = fs::canonicalize(root)?;
        if !root.is_dir() {
            return Err(AgentError::new("harness project must be a directory"));
        }
        let project = digest(root.as_os_str().as_encoded_bytes());
        let base_root = if base.as_ref().exists() {
            fs::canonicalize(base.as_ref())?
        } else {
            base.as_ref().to_path_buf()
        };
        let base = base_root.join("harness");
        reject_symlink_ancestors(&base)?;
        fs::create_dir_all(&base)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&base, fs::Permissions::from_mode(0o700))?;
        }
        Ok(Self {
            base,
            root,
            project,
        })
    }

    pub fn from_default_path(root: impl AsRef<Path>) -> AgentResult<Self> {
        let path = crate::default_auth_path()?;
        Self::new(
            path.parent()
                .ok_or_else(|| AgentError::new("invalid agent directory"))?,
            root,
        )
    }

    pub(super) fn lease_session(&self, id: &str) -> AgentResult<AuthFileLock> {
        let path = self.session_path(id)?.with_extension("task.lock");
        AuthFileLock::acquire_nowait(&path).map_err(|_| {
            AgentError::new(
                "session already has an active task; wait for its owner instead of replaying work",
            )
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn directory(&self) -> PathBuf {
        self.base.join(&self.project)
    }

    fn task_queue_path(&self) -> PathBuf {
        self.directory().join("tasks.json")
    }

    pub(super) fn memory_path(&self) -> AgentResult<PathBuf> {
        let directory = self.root.join(".agents");
        reject_symlink_ancestors(&directory)?;
        fs::create_dir_all(&directory)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
        }
        reject_symlink_ancestors(&directory)?;
        Ok(directory.join("memory.json"))
    }

    fn task_queue(&self) -> AgentResult<HarnessTaskQueue> {
        let path = self.task_queue_path();
        reject_symlink_ancestors(&path)?;
        if !path.exists() {
            return Ok(HarnessTaskQueue {
                schema: 1,
                tasks: Vec::new(),
            });
        }
        let bytes = read_bounded(&path)?;
        serde_json::from_slice(&bytes)
            .map_err(|_| AgentError::new("invalid task queue; original data preserved"))
    }
    fn save_task_queue(&self, queue: &HarnessTaskQueue) -> AgentResult<()> {
        if queue.tasks.len() > 1000 {
            return Err(AgentError::new("task queue storage limit reached"));
        }
        let path = self.task_queue_path();
        reject_symlink_ancestors(&path)?;
        write_private_json(&path, queue)
    }
    pub fn enqueue_task(&self, session: &str, prompt: &str) -> AgentResult<HarnessQueuedTask> {
        self.session_path(session)?;
        let prompt = prompt.trim();
        if prompt.is_empty() || prompt.len() > 8192 {
            return Err(AgentError::new("queued prompt must be 1..8192 bytes"));
        }
        let path = self.task_queue_path();
        let _lock = AuthFileLock::acquire(&path.with_extension("lock"))?;
        let mut queue = self.task_queue()?;
        if queue
            .tasks
            .iter()
            .filter(|task| task.session == session && task.state == HarnessTaskState::Pending)
            .count()
            >= 8
        {
            return Err(AgentError::new("prompt queue is full (limit 8)"));
        }
        let t = now();
        let task = HarnessQueuedTask {
            id: identifier(),
            session: session.into(),
            prompt: prompt.into(),
            state: HarnessTaskState::Pending,
            created_at: t,
            updated_at: t,
            error: None,
            result: None,
        };
        queue.tasks.push(task.clone());
        self.save_task_queue(&queue)?;
        Ok(task)
    }
    pub fn clear_tasks(&self, session: &str) -> AgentResult<usize> {
        self.session_path(session)?;
        let path = self.task_queue_path();
        let _lock = AuthFileLock::acquire(&path.with_extension("lock"))?;
        let mut queue = self.task_queue()?;
        let before = queue.tasks.len();
        queue.tasks.retain(|task| task.session != session);
        self.save_task_queue(&queue)?;
        Ok(before - queue.tasks.len())
    }

    pub fn list_tasks(&self, session: &str) -> AgentResult<Vec<HarnessQueuedTask>> {
        self.session_path(session)?;
        Ok(self
            .task_queue()?
            .tasks
            .into_iter()
            .filter(|t| t.session == session)
            .collect())
    }
    pub fn claim_task(&self, session: &str) -> AgentResult<Option<HarnessQueuedTask>> {
        self.session_path(session)?;
        let path = self.task_queue_path();
        let _lock = AuthFileLock::acquire(&path.with_extension("lock"))?;
        let mut q = self.task_queue()?;
        let Some(t) = q
            .tasks
            .iter_mut()
            .find(|t| t.session == session && t.state == HarnessTaskState::Pending)
        else {
            return Ok(None);
        };
        t.state = HarnessTaskState::Running;
        t.updated_at = now();
        let out = t.clone();
        self.save_task_queue(&q)?;
        Ok(Some(out))
    }
    pub fn complete_task(&self, s: &str, id: &str, r: Value) -> AgentResult<HarnessQueuedTask> {
        self.update_task(s, id, HarnessTaskState::Completed, None, Some(r))
    }
    pub fn fail_task(&self, s: &str, id: &str, e: &str) -> AgentResult<HarnessQueuedTask> {
        if e.trim().is_empty() || e.len() > 16384 {
            return Err(AgentError::new("task error must be nonempty and bounded"));
        }
        self.update_task(s, id, HarnessTaskState::Failed, Some(e.into()), None)
    }
    fn update_task(
        &self,
        s: &str,
        id: &str,
        state: HarnessTaskState,
        error: Option<String>,
        result: Option<Value>,
    ) -> AgentResult<HarnessQueuedTask> {
        self.session_path(s)?;
        let p = self.task_queue_path();
        let _lock = AuthFileLock::acquire(&p.with_extension("lock"))?;
        let mut q = self.task_queue()?;
        let t = q
            .tasks
            .iter_mut()
            .find(|t| t.session == s && t.id == id && t.state == HarnessTaskState::Running)
            .ok_or_else(|| AgentError::new("running task not found"))?;
        t.state = state;
        t.updated_at = now();
        t.error = error;
        t.result = result;
        let out = t.clone();
        self.save_task_queue(&q)?;
        Ok(out)
    }

    fn session_path(&self, id: &str) -> AgentResult<PathBuf> {
        if id.len() != 32 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(AgentError::new("invalid session id"));
        }
        let path = self.directory().join("sessions").join(format!("{id}.json"));
        reject_symlink_ancestors(&path)?;
        Ok(path)
    }

    pub fn config(&self) -> AgentResult<HarnessConfig> {
        let path = self.base.join("config.json");
        reject_symlink_ancestors(&path)?;
        let config = match fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|_| AgentError::new("invalid harness config"))?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => HarnessConfig::default(),
            Err(error) => return Err(error.into()),
        };
        config.validate()?;
        Ok(config)
    }

    pub fn save_config(&self, config: &HarnessConfig) -> AgentResult<()> {
        config.validate()?;
        let path = self.base.join("config.json");
        reject_symlink_ancestors(&path)?;
        let _lock = AuthFileLock::acquire(&path.with_extension("lock"))?;
        write_private_json(&path, config)
    }

    pub fn create_session(&self) -> AgentResult<HarnessSession> {
        let mut session = HarnessSession {
            schema: 1,
            id: identifier(),
            project: self.project.clone(),
            catalog: super::catalog::catalog_fingerprint()?,
            authority_fingerprint: super::catalog::authority_fingerprint_with_registry(&self.config()?.providers)?,
            revision: 0,
            turns: vec![],
            events: vec![],
            summary: None,
            title: None,
            initial_prompt_preview: None,
            orchestration: None,
            context_start: 0,
            interrupted: false,
        };
        self.save_session(&mut session)?;
        Ok(session)
    }

    pub fn load_session(&self, id: &str) -> AgentResult<HarnessSession> {
        self.read_session(id, true)
    }

    fn read_session(&self, id: &str, check_catalog: bool) -> AgentResult<HarnessSession> {
        let bytes = read_bounded(&self.session_path(id)?)?;
        self.decode_session(id, check_catalog, &bytes)
    }

    fn decode_session(
        &self,
        id: &str,
        check_catalog: bool,
        bytes: &[u8],
    ) -> AgentResult<HarnessSession> {
        let mut session: HarnessSession = serde_json::from_slice(bytes)
            .map_err(|_| AgentError::new("invalid session file; original data preserved"))?;
        if check_catalog && session.catalog != super::catalog::catalog_fingerprint()? {
            return Err(AgentError::new(
                "embedded catalog changed; preserve this history for inspection and start a new session",
            ));
        }
        if check_catalog
            && session.authority_fingerprint != super::catalog::authority_fingerprint_with_registry(&self.config()?.providers)?
        {
            return Err(AgentError::new(
                "embedded provider/model authority changed; preserve this history for inspection and start a new session",
            ));
        }
        if session.schema != 1
            || session.project != self.project
            || session.id != id
            || session.context_start > session.turns.len()
        {
            return Err(AgentError::new(
                "session schema, identity or context is incompatible",
            ));
        }
        let redactor = Redactor::for_project(&self.root);
        let mut projection = serde_json::to_value(&session)?;
        redactor.value(&mut projection);
        session = serde_json::from_value(projection)?;
        Ok(session)
    }

    pub fn save_session(&self, session: &mut HarnessSession) -> AgentResult<()> {
        if session.project != self.project
            || session.schema != 1
            || session.catalog != super::catalog::catalog_fingerprint()?
            || session.authority_fingerprint != super::catalog::authority_fingerprint_with_registry(&self.config()?.providers)?
        {
            return Err(AgentError::new("foreign session"));
        }
        let path = self.session_path(&session.id)?;
        let _lock = AuthFileLock::acquire(&path.with_extension("lock"))?;
        let revision = if path.exists() {
            self.load_session(&session.id)?.revision
        } else {
            0
        };
        if revision != session.revision {
            return Err(AgentError::new(
                "session changed concurrently; reload without replaying tools",
            ));
        }
        let mut next = session.clone();
        next.revision += 1;
        let mut value = serde_json::to_value(&next)?;
        Redactor::for_project(&self.root).value(&mut value);
        if serde_json::to_vec(&value)?.len() > 16777216 {
            return Err(AgentError::new(
                "session storage limit reached; start a new session",
            ));
        }
        write_private_json(&path, &value)?;
        session.revision = next.revision;
        Ok(())
    }

    pub fn sessions(&self) -> AgentResult<Vec<String>> {
        Ok(self
            .session_inventory()?
            .into_iter()
            .filter(|row| row["state"] != "unreadable")
            .filter_map(|row| row["id"].as_str().map(String::from))
            .collect())
    }

    pub fn delete_session(&self, id: &str) -> AgentResult<()> {
        let _task = self.lease_session(id)?;
        if self
            .processes()?
            .iter()
            .any(|record| record["session"] == id && record["state"] == "active")
        {
            return Err(AgentError::new(
                "stop the session's owned processes before deleting its history",
            ));
        }
        let path = self.session_path(id)?;
        let _lock = AuthFileLock::acquire(&path.with_extension("lock"))?;
        self.read_session(id, false)?;
        fs::remove_file(path)?;
        Ok(())
    }

    fn memories(&self) -> AgentResult<Memories> {
        let path = self.memory_path()?;
        reject_symlink_ancestors(&path)?;
        let memories: Memories = match read_bounded(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|_| AgentError::new("invalid memory file; original data preserved"))?,
            Err(_) if !path.exists() => Memories {
                schema: 1,
                observations: vec![],
            },
            Err(error) => return Err(error),
        };
        if memories.schema != 1
            || memories
                .observations
                .iter()
                .any(|observation| observation.project != self.project)
        {
            return Err(AgentError::new("incompatible memory schema or project"));
        }
        Ok(memories)
    }

    pub fn observations(&self) -> AgentResult<Vec<MemoryObservation>> {
        let mut value = serde_json::to_value(self.memories()?.observations)?;
        Redactor::for_project(&self.root).value(&mut value);
        Ok(serde_json::from_value(value)?)
    }

    pub fn remember(
        &self,
        title: &str,
        content: &str,
        source: &str,
        confirmed: bool,
    ) -> AgentResult<String> {
        if title.trim().is_empty()
            || title.len() > 256
            || content.trim().is_empty()
            || content.len() > 4096
            || source.len() > 256
        {
            return Err(AgentError::new(
                "memory title/content/source must be nonempty and bounded",
            ));
        }
        let path = self.memory_path()?;
        reject_symlink_ancestors(&path)?;
        let _lock = AuthFileLock::acquire(&path.with_extension("lock"))?;
        let mut memories = self.memories()?;
        if memories.observations.len() >= 1000 {
            return Err(AgentError::new(
                "memory limit reached; delete obsolete observations",
            ));
        }
        let id = identifier();
        let redactor = Redactor::for_project(&self.root);
        memories.observations.push(MemoryObservation {
            id: id.clone(),
            project: self.project.clone(),
            title: redactor.text(title),
            content: redactor.text(content),
            source: redactor.text(source),
            confirmed,
            validity: Default::default(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            updated: now(),
            files: Default::default(),
            kind: if source == "user" {
                "decision"
            } else {
                "discovery"
            }
            .into(),
        });
        write_private_json(&path, &memories)?;
        Ok(id)
    }

    pub fn forget(&self, id: &str) -> AgentResult<()> {
        let path = self.memory_path()?;
        reject_symlink_ancestors(&path)?;
        let _lock = AuthFileLock::acquire(&path.with_extension("lock"))?;
        let mut memories = self.memories()?;
        memories
            .observations
            .retain(|observation| observation.id != id);
        write_private_json(&path, &memories)
    }

    pub fn update_memory(
        &self,
        id: &str,
        title: &str,
        content: &str,
        files: &[String],
    ) -> AgentResult<()> {
        if title.is_empty()
            || title.len() > 256
            || content.is_empty()
            || content.len() > 4096
            || files.len() > 32
        {
            return Err(AgentError::new(
                "memory update exceeds title/content limits",
            ));
        }
        let fingerprints = files
            .iter()
            .map(|path| Ok((path.clone(), self.file_fingerprint(path)?)))
            .collect::<AgentResult<std::collections::BTreeMap<_, _>>>()?;
        let path = self.memory_path()?;
        reject_symlink_ancestors(&path)?;
        let _lock = AuthFileLock::acquire(&path.with_extension("lock"))?;
        let mut memories = self.memories()?;
        let observation = memories
            .observations
            .iter_mut()
            .find(|observation| observation.id == id)
            .ok_or_else(|| AgentError::new("memory id not found"))?;
        let redactor = Redactor::for_project(&self.root);
        observation.title = redactor.text(title);
        observation.content = redactor.text(content);
        observation.files = fingerprints;
        observation.source = "user".into();
        observation.validity = Default::default();
        observation.kind = "decision".into();
        observation.confirmed = true;
        observation.updated = now();
        write_private_json(&path, &memories)
    }

    pub fn recall(&self, query: &str) -> AgentResult<Vec<MemoryObservation>> {
        let query = query.to_lowercase();
        let terms: Vec<_> = query
            .split(|ch: char| !ch.is_alphanumeric())
            .filter(|word| word.len() > 2)
            .collect();
        let mut review = validity::MemoryReview::new(self)?;
        let mut matches = self
            .observations()?
            .into_iter()
            .filter(|observation| observation.confirmed && review.reason(observation).is_none())
            .filter_map(|observation| {
                let text = format!("{} {}", observation.title, observation.content).to_lowercase();
                let score = terms.iter().filter(|term| text.contains(**term)).count();
                (score > 0).then_some((score, observation))
            })
            .collect::<Vec<_>>();
        matches
            .sort_by_key(|(score, observation)| std::cmp::Reverse((*score, observation.created)));
        let mut budget = 6144;
        Ok(matches
            .into_iter()
            .filter_map(|(_, observation)| {
                let size = serde_json::to_vec(&observation).ok()?.len();
                if size > budget {
                    return None;
                }
                budget -= size;
                Some(observation)
            })
            .take(8)
            .collect())
    }
}

fn read_bounded(path: &Path) -> AgentResult<Vec<u8>> {
    if fs::metadata(path)?.len() > 16777216 {
        return Err(AgentError::new("agent file exceeds storage limit"));
    }
    Ok(fs::read(path)?)
}

fn reject_symlink_ancestors(path: &Path) -> AgentResult<()> {
    for ancestor in path.ancestors() {
        if fs::symlink_metadata(ancestor).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err(AgentError::new("agent storage cannot traverse symlinks"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod queue_tests {
    use super::*;

    #[test]
    fn queue_lifecycle_persists_and_is_session_scoped() {
        let home = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        let store = HarnessStore::new(home.path(), root.path()).unwrap();
        let session = store.create_session().unwrap();
        let other = store.create_session().unwrap();
        let task = store.enqueue_task(&session.id, "do the work").unwrap();
        store.enqueue_task(&other.id, "other work").unwrap();
        assert_eq!(task.state, HarnessTaskState::Pending);
        assert_eq!(store.list_tasks(&session.id).unwrap().len(), 1);
        assert_eq!(store.list_tasks(&other.id).unwrap().len(), 1);
        assert_eq!(store.list_tasks(&session.id).unwrap().len(), 1);
        let claimed = store.claim_task(&session.id).unwrap().unwrap();
        assert_eq!(claimed.state, HarnessTaskState::Running);
        let completed = store
            .complete_task(&session.id, &claimed.id, serde_json::json!({"ok":true}))
            .unwrap();
        assert_eq!(completed.state, HarnessTaskState::Completed);
        let reopened = HarnessStore::new(home.path(), root.path()).unwrap();
        let persisted = reopened.list_tasks(&session.id).unwrap();
        assert_eq!(persisted[0].result, Some(serde_json::json!({"ok":true})));
    }
}
