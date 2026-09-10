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
        let mut host = TerminalHost {
            auth,
            api_key,
            key_provider: active.provider.clone(),
            json_output,
            usage,
            root: self.store.root().to_path_buf(),
            request_events: Vec::new(),
            activity: activity.clone(),
        };
        let explicit = explicit.then_some(&active);
        if prompt == "/compact" {
            let result = activity
                .drive(compact_harness_session(
                    &self.store,
                    &mut self.session,
                    &self.config,
                    &active,
                    explicit,
                    &mut host,
                ))
                .await;
            self.transfer_activity_queue(&activity);
            result?;
            return Ok(HarnessOutcome::Completed);
        }
        if matches!(prompt, "/plan" | "/review" | "/research") {
            return Err(AgentError::new("Use /plan <task>, /research <question> or /review <task>"));
        }
        let (role, prompt) = if let Some(prompt) = prompt.strip_prefix("/plan ") {
            (HarnessRole::Plan, prompt)
        } else if let Some(prompt) = prompt.strip_prefix("/research ") {
            (HarnessRole::Research, prompt)
        } else if let Some(prompt) = prompt.strip_prefix("/review ") {
            (HarnessRole::Review, prompt)
        } else {
            (HarnessRole::Execute, prompt)
        };
        let result = activity
            .drive(run_harness_turn(
                &self.store,
                &mut self.session,
                &self.config,
                dowe_agent::native_harness::HarnessTask {
                    role,
                    active: &active,
                    explicit,
                    prompt,
                    image_paths: &self.image_paths,
                    edit_scope: None,
                    expected_codegraph_binding: None,
                },
                &mut host,
            ))
            .await;
        self.transfer_activity_queue(&activity);
        result
    }
}
