impl NativeSession {
    pub(super) fn open() -> AgentResult<Self> {
        let store = HarnessStore::from_default_path(std::env::current_dir()?)?;
        let config = store.config()?;
        let session = store.create_session()?;
        Ok(Self {
            store,
            session,
            config,
            image_paths: Vec::new(),
            watchers: Default::default(),
        })
    }

    pub(super) fn take_queued(
        &mut self,
    ) -> AgentResult<Option<dowe_agent::native_harness::HarnessQueuedTask>> {
        self.store.claim_task(&self.session.id)
    }

    pub(super) fn complete_queued(&self, id: &str, result: Value) -> AgentResult<()> {
        self.store
            .complete_task(&self.session.id, id, result)
            .map(|_| ())
    }
    pub(super) fn fail_queued(&self, id: &str, error: &str) -> AgentResult<()> {
        self.store
            .fail_task(&self.session.id, id, error)
            .map(|_| ())
    }

    fn transfer_activity_queue(&mut self, activity: &activity::Activity) {
        for prompt in activity.take_pending() {
            if let Err(error) = self.store.enqueue_task(&self.session.id, &prompt) {
                eprintln!("Input not queued: {error}");
            }
        }
    }

    pub(super) fn usage(&self) -> AgentUsageTotals {
        self.session.usage()
    }

    pub(super) fn reset(&mut self) -> AgentResult<()> {
        self.close_watchers()?;
        self.session = self.store.create_session()?;
        Ok(())
    }

}
