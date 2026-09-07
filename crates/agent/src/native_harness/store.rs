use super::lock::DataLock as AuthFileLock;
mod candidates;
mod processes;
mod recovery;
mod validity;
use super::{HarnessConfig, HarnessTurn, Redactor, digest, identifier};
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
    pub revision: u64,
    pub turns: Vec<HarnessTurn>,
    pub events: Vec<Value>,
    pub summary: Option<String>,
    pub context_start: usize,
    pub interrupted: bool,
}

impl HarnessSession {
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
                    Some("response_received" | "context_compacted")
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
        AuthFileLock::acquire(&path).map_err(|_| {
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
            revision: 0,
            turns: vec![],
            events: vec![],
            summary: None,
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
        let path = self.directory().join("memory.json");
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
        let path = self.directory().join("memory.json");
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
        let path = self.directory().join("memory.json");
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
        let path = self.directory().join("memory.json");
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
