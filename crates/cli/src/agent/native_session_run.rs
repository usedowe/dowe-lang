impl NativeSession {
    pub(super) async fn run(
        &mut self,
        prompt: &str,
        active: ModelSelection,
        explicit: bool,
        api_key: Option<String>,
        json_output: bool,
        usage: &mut AgentUsageTotals,
    ) -> AgentResult<HarnessOutcome> {
        let auth = AgentAuthStore::from_default_path()?;
        let activity = activity::Activity::new(json_output)?;
        if activity.enabled() {
            let width = crossterm::terminal::size()?.0.saturating_sub(1) as usize;
            activity.footer(
                super::footer::Footer {
                    provider: Some(&active.provider),
                    model: Some(&active.model),
                    thinking: active.thinking,
                    usage,
                }
                .lines(width),
            );
        }
        let shared_usage = std::sync::Arc::new(std::sync::Mutex::new(std::mem::take(usage)));
        let mut host = TerminalHost {
            auth,
            api_key,
            key_provider: active.provider.clone(),
            json_output,
            usage: shared_usage.clone(),
            root: self.store.root().to_path_buf(),
            request_events: Vec::new(),
            pending_responses: Vec::new(),
            activity: activity.clone(),
            automatic_approval: self.permission_mode.is_full_access(),
        };
        let explicit = explicit.then_some(&active);
        if prompt == "/compact" {
            let result = activity
                .drive(compact_clean_session(
                    &self.store,
                    &mut self.session,
                    &self.config,
                    &active,
                    explicit,
                    &mut host,
                ))
                .await;
            if let Some(draft) = activity.take_draft() {
                self.activity_draft = Some(draft);
            }
            self.transfer_activity_queue(&activity);
            host.flush_pending_responses()?;
            drop(host);
            *usage = std::sync::Arc::try_unwrap(shared_usage)
                .map_err(|_| AgentError::new("usage ledger still has active workers"))?
                .into_inner()
                .map_err(|_| AgentError::new("usage ledger is poisoned"))?;
            result?;
            return Ok(HarnessOutcome::Completed);
        }
        let result = activity
            .drive(dowe_agent::native_harness::run_agent_task(
                &self.store,
                &mut self.session,
                &self.config,
                dowe_agent::native_harness::HarnessTask {
                    role: HarnessRole::Execute,
                    active: &active,
                    explicit,
                    prompt,
                    image_paths: &self.image_paths,
                    edit_scope: None,
                    expected_codegraph_binding: None,
                    permission_mode: self.permission_mode,
                },
                &mut host,
            ))
            .await;
        if let Some(draft) = activity.take_draft() {
            self.activity_draft = Some(draft);
        }
        self.transfer_activity_queue(&activity);
        host.flush_pending_responses()?;
        drop(host);
        *usage = std::sync::Arc::try_unwrap(shared_usage)
            .map_err(|_| AgentError::new("usage ledger still has active workers"))?
            .into_inner()
            .map_err(|_| AgentError::new("usage ledger is poisoned"))?;
        result
    }
}
