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

}
