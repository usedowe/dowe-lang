impl Drop for DoweSubscription {
    fn drop(&mut self) {
        let _ = self.close_sync();
    }
}

impl LocalReceipt {
    fn settle(&mut self, requeue: bool) -> QueueResult<()> {
        if self.resolved {
            return Err(QueueError::InvalidReceipt(
                "Queue delivery receipt is already resolved".to_string(),
            ));
        }
        self.resolved = true;
        self.engine
            .settle(&self.queue, &self.session, &self.receipt, requeue)
    }
}

impl DeliveryReceipt for LocalReceipt {
    fn ack<'a>(
        &'a mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = QueueResult<()>> + Send + 'a>> {
        Box::pin(async move { self.settle(false) })
    }

    fn nack<'a>(
        &'a mut self,
        requeue: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = QueueResult<()>> + Send + 'a>> {
        Box::pin(async move { self.settle(requeue) })
    }
}

pub use crate::storage::queue_root;

pub fn init_namespace(project_root: &Path, name: &str) -> QueueResult<()> {
    open_namespace(project_root, name).map(|_| ())
}

pub fn open_namespace(project_root: &Path, name: &str) -> QueueResult<DoweQueue> {
    validate_namespace(name)?;
    let path = queue_root(project_root).join(name);
    fs::create_dir_all(&path)?;
    let registry = ENGINES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut registry = registry
        .lock()
        .map_err(|_| QueueError::DurabilityError("Queue registry lock failed".to_string()))?;
    if let Some(shared) = registry.get(&path).and_then(Weak::upgrade) {
        return Ok(DoweQueue { shared });
    }
    let lock = acquire_namespace_lock(&path)?;
    let mut state = read_state(&path, name)?;
    if requeue_recovered(&mut state) {
        persist(&path, &state)?;
    } else if !path.join("state.json").exists() {
        persist(&path, &state)?;
    }
    let shared = Arc::new(SharedEngine {
        path: path.clone(),
        _lock: lock,
        state: Mutex::new(state),
        notify: Notify::new(),
    });
    registry.insert(path, Arc::downgrade(&shared));
    Ok(DoweQueue { shared })
}

fn acquire_namespace_lock(path: &Path) -> QueueResult<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path.join(".lock"))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(QueueError::DurabilityError(
            "Queue namespace is already in use".to_string(),
        )),
        Err(TryLockError::Error(_)) => Err(QueueError::DurabilityError(
            "Queue namespace lock cannot be acquired".to_string(),
        )),
    }
}

pub fn list_namespaces(project_root: &Path) -> QueueResult<Vec<String>> {
    let root = queue_root(project_root);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() || entry.file_name() == "_auth" {
            continue;
        }
        if !entry.path().join("state.json").exists() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        validate_namespace(&name)?;
        names.push(name);
    }
    names.sort();
    Ok(names)
}
