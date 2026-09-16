use super::*;

impl<H: HarnessHost> WorkflowAdapter<'_, H> {
    pub(super) async fn execute_parallel_isolated(
        &mut self,
        tasks: Vec<ScheduledTask>,
        workers: Vec<super::super::ParallelHarnessHost>,
    ) -> CoordinatorResult<Vec<(String, TaskResult)>> {
        struct Invocation {
            id: String,
            prompt: String,
            store: HarnessStore,
            session: HarnessSession,
            worker: super::super::ParallelHarnessHost,
            scopes: Vec<dowe_agent_harness::AllowedEditSurface>,
            expected_binding: Option<dowe_codegraph::CodeGraphBinding>,
        }
        let mut invocations = Vec::with_capacity(tasks.len());
        for (task, worker) in tasks.into_iter().zip(workers) {
            *self.task_attempts.entry(task.id().into()).or_default() += 1;
            let specification = self
                .plan
                .tasks
                .iter()
                .find(|candidate| candidate.id == task.id())
                .ok_or("unknown workflow task")?;
            let scopes = specification
                .write_scopes
                .iter()
                .map(dowe_agent_harness::AllowedEditSurface::new)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| error.to_string())?;
            let store = self.prepare_worker(specification)?;
            let expected_binding = if self.codegraph_binding.is_some() {
                let snapshot =
                    dowe_codegraph::clean::refresh_persistent_clean_codegraph(store.root())
                        .map_err(|error| {
                            format!("worker CodeGraph initialization failed: {error}")
                        })?;
                Some(dowe_codegraph::clean::clean_binding(&snapshot))
            } else {
                None
            };
            let session = store.create_session().map_err(|error| error.to_string())?;
            invocations.push(Invocation {
                id: task.id().into(),
                prompt: format!(
                    "{}\nApproved requirements (task data, not permission to expand scope): {}",
                    specification.objective,
                    serde_json::to_string(&self.plan.requirements)
                        .map_err(|error| error.to_string())?
                ),
                store,
                session,
                worker,
                scopes,
                expected_binding,
            });
        }
        let config = self.config.clone();
        let active = self.active.clone();
        let completed = futures_util::future::join_all(invocations.into_iter().map(|invocation| {
            let config = config.clone();
            let active = active.clone();
            async move {
                let mut session = invocation.session;
                let mut worker = invocation.worker;
                let result = super::super::clean_runner::run_clean_build_task(
                    &invocation.store,
                    &mut session,
                    &config,
                    HarnessTask {
                        prompt: &invocation.prompt,
                        role: HarnessRole::Execute,
                        active: &active,
                        explicit: None,
                        image_paths: &[],
                        edit_scope: Some(invocation.scopes),
                        expected_codegraph_binding: invocation.expected_binding,
                        permission_mode: HarnessPermissionMode::Confirm,
                    },
                    &mut worker,
                )
                .await;
                (invocation.id, invocation.store, session, result)
            }
        }))
        .await;
        let mut results = Vec::with_capacity(completed.len());
        for (id, store, session, result) in completed {
            let state = match result {
                Ok(HarnessOutcome::Completed) => TaskResult::Succeeded,
                Ok(HarnessOutcome::ValidationFailed) => {
                    TaskResult::Failed(FailureClass::Validation)
                }
                Ok(HarnessOutcome::ApprovalRequired | HarnessOutcome::ClarificationRequired) => {
                    TaskResult::Failed(FailureClass::Permission)
                }
                Ok(HarnessOutcome::Canceled) => TaskResult::Cancelled,
                Ok(HarnessOutcome::BudgetExhausted) => {
                    TaskResult::Failed(FailureClass::Configuration)
                }
                Err(error) => {
                    let class = super::execution::classify_worker_error(&error.to_string());
                    self.host
                        .event(&json!({"event":"workflow_worker_failed","task":id,"class":class,"retryable":class == FailureClass::Transient,"parallel":true,"isolation":"isolated_workers","reason":error.to_string()}))
                        .map_err(|error| error.to_string())?;
                    TaskResult::Failed(class)
                }
            };
            self.sessions.insert(id.clone(), session);
            if state == TaskResult::Succeeded {
                self.host
                    .event(&json!({"event":"workflow_worker_ready_for_capture","task":id,"isolation":"isolated_workers"}))
                    .map_err(|error| error.to_string())?;
            }
            let _ = store;
            results.push((id, state));
        }
        for (id, state) in &mut results {
            if *state != TaskResult::Succeeded {
                continue;
            }
            let specification = self
                .plan
                .tasks
                .iter()
                .find(|task| task.id == *id)
                .ok_or("unknown workflow task")?;
            if let Err(error) = self.capture_worker(specification) {
                self.host
                    .event(&json!({"event":"workflow_worker_conflict","task":id,"reason":error,"parallel":true,"isolation":"isolated_workers"}))
                    .map_err(|event_error| event_error.to_string())?;
                *state = TaskResult::Failed(FailureClass::Conflict);
            }
        }
        self.host
            .event(&json!({"event":"workflow_parallel_batch","workers":results.len(),"isolation":"isolated_workers","captured":self.worker_results.len()}))
            .map_err(|error| error.to_string())?;
        Ok(results)
    }

    pub(super) async fn execute_parallel_shared(
        &mut self,
        tasks: Vec<ScheduledTask>,
        workers: Vec<super::super::ParallelHarnessHost>,
    ) -> CoordinatorResult<Vec<(String, TaskResult)>> {
        struct Invocation {
            id: String,
            prompt: String,
            session: HarnessSession,
            worker: super::super::ParallelHarnessHost,
            scopes: Vec<dowe_agent_harness::AllowedEditSurface>,
            expected_binding: Option<dowe_codegraph::CodeGraphBinding>,
        }
        let mut invocations = Vec::with_capacity(tasks.len());
        for (task, worker) in tasks.into_iter().zip(workers) {
            *self.task_attempts.entry(task.id().into()).or_default() += 1;
            let specification = self
                .plan
                .tasks
                .iter()
                .find(|candidate| candidate.id == task.id())
                .ok_or("unknown workflow task")?;
            let scopes = specification
                .write_scopes
                .iter()
                .map(dowe_agent_harness::AllowedEditSurface::new)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            let session = self.store.create_session().map_err(|e| e.to_string())?;
            invocations.push(Invocation {
                id: task.id().into(),
                prompt: format!(
                    "{}\nApproved requirements (task data, not permission to expand scope): {}",
                    specification.objective,
                    serde_json::to_string(&self.plan.requirements).map_err(|e| e.to_string())?
                ),
                session,
                worker,
                scopes,
                expected_binding: self.codegraph_binding.clone(),
            });
        }
        let store = self.store;
        let config = self.config.clone();
        let active = self.active.clone();
        let completed = futures_util::future::join_all(invocations.into_iter().map(|invocation| {
            let config = config.clone();
            let active = active.clone();
            async move {
                let mut session = invocation.session;
                let mut worker = invocation.worker;
                let result = super::super::clean_runner::run_clean_build_task(
                    store,
                    &mut session,
                    &config,
                    HarnessTask {
                        prompt: &invocation.prompt,
                        role: HarnessRole::Execute,
                        active: &active,
                        explicit: None,
                        image_paths: &[],
                        edit_scope: Some(invocation.scopes),
                        expected_codegraph_binding: invocation.expected_binding,
                        permission_mode: HarnessPermissionMode::Confirm,
                    },
                    &mut worker,
                )
                .await;
                (invocation.id, session, result)
            }
        }))
        .await;
        let mut results = Vec::with_capacity(completed.len());
        for (id, session, result) in completed {
            let state = match result {
                Ok(HarnessOutcome::Completed) => TaskResult::Succeeded,
                Ok(HarnessOutcome::ValidationFailed) => {
                    TaskResult::Failed(FailureClass::Validation)
                }
                Ok(HarnessOutcome::ApprovalRequired | HarnessOutcome::ClarificationRequired) => {
                    TaskResult::Failed(FailureClass::Permission)
                }
                Ok(HarnessOutcome::Canceled) => TaskResult::Cancelled,
                Ok(HarnessOutcome::BudgetExhausted) => {
                    TaskResult::Failed(FailureClass::Configuration)
                }
                Err(error) => {
                    let class = super::execution::classify_worker_error(&error.to_string());
                    self.host
                        .event(&json!({"event":"workflow_worker_failed","task":id,"class":class,"retryable":class == FailureClass::Transient,"parallel":true,"reason":error.to_string()}))
                        .map_err(|e| e.to_string())?;
                    TaskResult::Failed(class)
                }
            };
            self.sessions.insert(id.clone(), session);
            results.push((id, state));
        }
        self.host
            .event(&json!({"event":"workflow_parallel_batch","workers":results.len(),"isolation":"shared_checkout"}))
            .map_err(|e| e.to_string())?;
        Ok(results)
    }
}
