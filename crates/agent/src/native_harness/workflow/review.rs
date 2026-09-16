use super::reviewer::reviewer_lenses;
use super::*;

impl<H: HarnessHost> WorkflowAdapter<'_, H> {
    pub(super) async fn review_profiles(
        &mut self,
        coordinator: &Coordinator,
    ) -> CoordinatorResult<(String, bool)> {
        let profiles = self
            .plan
            .verification_graph
            .as_ref()
            .map(|graph| {
                graph
                    .nodes
                    .iter()
                    .filter(|node| node.kind == "review")
                    .map(|node| node.id.clone())
                    .collect::<Vec<_>>()
            })
            .filter(|profiles| !profiles.is_empty())
            .unwrap_or_else(|| vec!["independent_review".into()]);
        let isolated_store = self
            .worktrees
            .values()
            .next()
            .map(|candidate| self.store.for_project_root(&candidate.path))
            .transpose()
            .map_err(|e| e.to_string())?;
        let review_store = isolated_store.as_ref().unwrap_or(self.store);
        let changes: Vec<_> = self
            .sessions
            .values()
            .map(|session| json!({"id":session.id,"events":session.events.iter().filter(|event| event["event"] == "operation_finished").collect::<Vec<_>>() }))
            .collect();
        let mut context = json!({"plan":self.plan,"coordinator":coordinator,"changes":changes});
        self.path_tools.redactor.value(&mut context);
        let context = super::super::privacy::bounded_value(context, self.config.max_output_bytes);
        let lenses = reviewer_lenses(self.plan);
        let mut ids = Vec::new();
        let mut all_approved = true;
        for profile in profiles {
            self.review_sessions += 1;
            self.host.event(&json!({"event":"workflow_reviewer_started","profile":profile,"independent":true,"lenses":lenses,"evidence":"source reads, verification reports and candidate state"})).map_err(|e| e.to_string())?;
            let mut session = review_store.create_session().map_err(|e| e.to_string())?;
            let prompt = format!(
                "Act as independent reviewer profile `{profile}`. Review the completed workflow using source reads and verification evidence. Review every lens: {}. Return exactly JSON with approved (boolean) and findings (array). Never approve based solely on worker claims. {}",
                lenses.join(", "),
                context
            );
            let outcome = super::super::clean_runner::run_clean_read_task(
                review_store,
                &mut session,
                HarnessTask {
                    prompt: &prompt,
                    role: HarnessRole::Review,
                    active: self.active,
                    explicit: None,
                    image_paths: &[],
                    edit_scope: None,
                    expected_codegraph_binding: None,
                    permission_mode: HarnessPermissionMode::Confirm,
                },
                self.host,
            )
            .await
            .map_err(|e| e.to_string())?;
            let approved = matches!(outcome, HarnessOutcome::Completed)
                && session
                    .turns
                    .iter()
                    .rev()
                    .filter_map(|turn| turn.message.as_ref())
                    .find(|message| message.role == "assistant")
                    .and_then(|message| match &message.content {
                        crate::AgentMessageContent::Text(text) => {
                            serde_json::from_str::<serde_json::Value>(text).ok()
                        }
                        _ => None,
                    })
                    .is_some_and(|value| {
                        value["approved"] == true
                            && value["findings"].as_array().is_some_and(Vec::is_empty)
                    });
            all_approved &= approved;
            ids.push(format!("reviewer:{profile}"));
            self.host.event(&json!({"event":"workflow_reviewer_finished","profile":profile,"approved":approved})).map_err(|e| e.to_string())?;
        }
        Ok((ids.join(","), all_approved))
    }
}
