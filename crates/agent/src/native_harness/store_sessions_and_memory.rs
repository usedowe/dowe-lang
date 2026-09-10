impl HarnessStore {

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
