
/// Pure, in-memory coordinator for session-owned orchestration state.
///
/// This type intentionally owns data and typed transitions only. Native execution,
/// providers, persistence, and child-agent management belong to a future adapter.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Orchestrator {
    sessions: BTreeMap<String, SessionRecord>,
}

impl Orchestrator {
    pub fn new() -> Self { Self::default() }

    pub fn create_session(
        &mut self,
        id: impl Into<String>,
        codegraph_binding: CodeGraphBinding,
    ) -> OrchestrationResult<()> {
        let session = SessionRecord::new(id, codegraph_binding)?;
        self.register_session(session)
    }

    pub fn register_session(&mut self, session: SessionRecord) -> OrchestrationResult<()> {
        if self.sessions.contains_key(&session.id) {
            return Err(OrchestrationError::new("session ID must be unique"));
        }
        self.sessions.insert(session.id.clone(), session);
        Ok(())
    }

    pub fn add_agent(&mut self, session_id: &str, agent: AgentRecord) -> OrchestrationResult<()> {
        self.session_mut(session_id)?.add_agent(agent)
    }

    pub fn add_task(&mut self, session_id: &str, task: TaskRecord) -> OrchestrationResult<()> {
        self.session_mut(session_id)?.add_task(task)
    }

    pub fn start_task(&mut self, session_id: &str, task_id: &str) -> OrchestrationResult<()> {
        let session = self.session_mut(session_id)?;
        session.ensure_active()?;
        Self::task_mut(session, task_id)?.start()
    }

    pub fn finish_task(&mut self, session_id: &str, task_id: &str) -> OrchestrationResult<()> {
        let session = self.session_mut(session_id)?;
        session.ensure_active()?;
        Self::task_mut(session, task_id)?.complete()
    }

    pub fn cancel_task(&mut self, session_id: &str, task_id: &str) -> OrchestrationResult<()> {
        let session = self.session_mut(session_id)?;
        session.ensure_active()?;
        Self::task_mut(session, task_id)?.cancel()
    }

    /// Starts a task only when the caller presents the exact session binding.
    pub fn start_task_with_binding(
        &mut self,
        session_id: &str,
        task_id: &str,
        binding: &CodeGraphBinding,
    ) -> OrchestrationResult<()> {
        self.ensure_task_binding(session_id, task_id, binding)?;
        self.start_task(session_id, task_id)
    }

    pub fn finish_task_with_binding(
        &mut self,
        session_id: &str,
        task_id: &str,
        binding: &CodeGraphBinding,
    ) -> OrchestrationResult<()> {
        self.ensure_task_binding(session_id, task_id, binding)?;
        self.finish_task(session_id, task_id)
    }

    pub fn cancel_task_with_binding(
        &mut self,
        session_id: &str,
        task_id: &str,
        binding: &CodeGraphBinding,
    ) -> OrchestrationResult<()> {
        self.ensure_task_binding(session_id, task_id, binding)?;
        self.cancel_task(session_id, task_id)
    }

    pub fn record_receipt(
        &mut self,
        session_id: &str,
        task_id: &str,
        receipt: Receipt,
    ) -> OrchestrationResult<()> {
        self.session_mut(session_id)?.record_receipt(task_id, receipt)
    }

    pub fn complete_session(&mut self, session_id: &str) -> OrchestrationResult<()> {
        self.session_mut(session_id)?.complete()
    }

    pub fn interrupt_session(&mut self, session_id: &str) -> OrchestrationResult<()> {
        self.session_mut(session_id)?.interrupt()
    }

    pub fn close_session(&mut self, session_id: &str) -> OrchestrationResult<()> {
        self.session_mut(session_id)?.close()
    }

    /// Returns an owned snapshot so later coordinator mutations cannot alter it.
    pub fn inspect_session(&self, session_id: &str) -> OrchestrationResult<SessionRecord> {
        self.session(session_id).cloned()
    }

    pub fn session_count(&self) -> usize { self.sessions.len() }

    fn session(&self, session_id: &str) -> OrchestrationResult<&SessionRecord> {
        self.sessions.get(session_id)
            .ok_or_else(|| OrchestrationError::new("session is not registered"))
    }

    fn session_mut(&mut self, session_id: &str) -> OrchestrationResult<&mut SessionRecord> {
        self.sessions.get_mut(session_id)
            .ok_or_else(|| OrchestrationError::new("session is not registered"))
    }

    fn task_mut<'a>(session: &'a mut SessionRecord, task_id: &str) -> OrchestrationResult<&'a mut TaskRecord> {
        session.tasks.iter_mut().find(|task| task.id == task_id)
            .ok_or_else(|| OrchestrationError::new("task is not owned by session"))
    }

    fn ensure_task_binding(
        &self,
        session_id: &str,
        task_id: &str,
        binding: &CodeGraphBinding,
    ) -> OrchestrationResult<()> {
        let session = self.session(session_id)?;
        if &session.codegraph_binding != binding {
            return Err(OrchestrationError::new("CodeGraphBinding does not match session"));
        }
        let task = session.tasks.iter().find(|task| task.id == task_id)
            .ok_or_else(|| OrchestrationError::new("task is not owned by session"))?;
        if &task.codegraph_binding != binding {
            return Err(OrchestrationError::new("CodeGraphBinding does not match task"));
        }
        Ok(())
    }
}

fn normalize_scope(raw: &str) -> OrchestrationResult<String> {
    let raw = raw.trim().replace('\\', "/");
    if raw.is_empty() || raw.starts_with('/') || raw.starts_with("//") || raw.split('/').next().is_some_and(|part| part.contains(':')) {
        return Err(OrchestrationError::new("scope must be a non-empty project-relative path"));
    }
    let mut parts = Vec::new();
    for part in raw.split('/') {
        if part.is_empty() || part == "." { continue; }
        if part == ".." { return Err(OrchestrationError::new("scope must not contain traversal")); }
        parts.push(part);
    }
    if parts.is_empty() { return Ok(".".into()); }
    Ok(parts.join("/"))
}

fn worker_scope_allowed(worker: &AllowedEditSurface, task: &[AllowedEditSurface]) -> bool {
    task.iter().any(|surface| surface.allows(&worker.path) && (!worker.recursive || surface.recursive || worker.path == surface.path))
}


