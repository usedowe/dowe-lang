use super::*;
use crate::native_harness::observability;

pub(super) struct WorkflowAdapter<'a, H> {
    pub store: &'a HarnessStore,
    pub config: &'a HarnessConfig,
    pub active: &'a ModelSelection,
    pub plan: &'a WorkflowPlan,
    pub host: &'a mut H,
    pub sequence: Option<u64>,
    pub sessions: BTreeMap<String, HarnessSession>,
    pub path_tools: HarnessTools,
    pub isolation: WorkflowIsolation,
    pub worktrees: BTreeMap<String, dowe_agent_harness::IsolatedWorktree>,
    pub worker_worktrees: BTreeMap<String, dowe_agent_harness::IsolatedWorktree>,
    pub worker_results: BTreeMap<String, dowe_agent_harness::WorkerResult>,
    pub cleanup_after_integration: bool,
    pub task_attempts: BTreeMap<String, u32>,
    pub review_sessions: u32,
    pub verified_manifest: Option<serde_json::Value>,
    pub configuration: String,
    pub elapsed_ms: u64,
    pub resumable: bool,
    pub codegraph_binding: Option<dowe_codegraph::CodeGraphBinding>,
    pub trace: dowe_agent_harness::ObservabilityTrace,
}

impl<H: HarnessHost> WorkflowAdapter<'_, H> {
    async fn approve(&mut self, coordinator: &Coordinator) -> CoordinatorResult<bool> {
        if !autonomy_allowed(self.plan.autonomy, self.plan.risk, true) {
            self.host
                .event(&json!({"event":"autonomy_gate","status":"denied","risk":self.plan.risk,"autonomy":self.plan.autonomy}))
                .map_err(|e| e.to_string())?;
            return Ok(false);
        }
        let approval = Approval {
            id: identifier(),
            session: coordinator.id().into(),
            call: ToolCall::new(
                "workflow-plan",
                "approve_plan",
                serde_json::to_value(self.plan).map_err(|e| e.to_string())?,
            ),
            details: json!({"plan":self.plan,"revision":coordinator.revision(),"schedule":coordinator.schedule(),"policy":"Approval applies to this plan revision and remaining stages; succeeded tasks are not replayed. Individual execution tools retain their approval checks."}),
            before: None,
            after: None,
            before_bytes: None,
            after_bytes: None,
            reference_images: vec![],
        };
        Ok(self
            .host
            .approve(&approval)
            .await
            .map_err(|e| e.to_string())?
            == Some(true))
    }

    async fn execute_batch(
        &mut self,
        tasks: Vec<ScheduledTask>,
    ) -> CoordinatorResult<Vec<(String, TaskResult)>> {
        if tasks.len() > 1
            && matches!(
                self.isolation,
                WorkflowIsolation::SharedCheckout | WorkflowIsolation::IsolatedWorkers
            )
        {
            let mut workers = Vec::with_capacity(tasks.len());
            for _ in 0..tasks.len() {
                let Some(worker) = self.host.parallel_worker().map_err(|e| e.to_string())? else {
                    workers.clear();
                    break;
                };
                workers.push(worker);
            }
            if workers.len() == tasks.len() {
                let started = std::time::Instant::now();
                let results = if matches!(self.isolation, WorkflowIsolation::IsolatedWorkers) {
                    self.execute_parallel_isolated(tasks, workers).await?
                } else {
                    self.execute_parallel_shared(tasks, workers).await?
                };
                observability::record_stage(
                    &mut self.trace,
                    &self.active.model,
                    started.elapsed().as_millis().min(u64::MAX as u128) as u64,
                    format!("parallel_batch:{}", results.len()),
                );
                return Ok(results);
            }
        }
        let mut results = Vec::new();
        for task in tasks {
            *self.task_attempts.entry(task.id().into()).or_default() += 1;
            let specification = self
                .plan
                .tasks
                .iter()
                .find(|t| t.id == task.id())
                .ok_or("unknown workflow task")?;
            let scopes = specification
                .write_scopes
                .iter()
                .map(dowe_agent_harness::AllowedEditSurface::new)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            let isolated_store = if matches!(self.isolation, WorkflowIsolation::IsolatedWorkers) {
                Some(self.prepare_worker(specification)?)
            } else if matches!(self.isolation, WorkflowIsolation::DetachedWorktrees) {
                let worktree = if let Some(candidate) = self.worktrees.values().next() {
                    candidate.clone()
                } else {
                    let candidate = dowe_agent_harness::create_isolated_worktree(
                        self.store.root(),
                        &identifier(),
                    )
                    .map_err(|e| e.to_string())?;
                    self.host.event(&json!({"event":"workflow_worktree_created","task":task.id(),"path":candidate.path,"baseRevision":candidate.base_revision,"isolation":"serial_workflow_candidate"}))
                    .map_err(|e| e.to_string())?;
                    self.worktrees.insert("candidate".into(), candidate.clone());
                    candidate
                };
                let path = std::path::PathBuf::from(&worktree.path);
                Some(
                    self.store
                        .for_project_root(path)
                        .map_err(|e| e.to_string())?,
                )
            } else {
                None
            };
            let task_store = isolated_store.as_ref().unwrap_or(self.store);
            let expected_binding = if task_store.root() == self.store.root() {
                self.codegraph_binding.clone()
            } else if self.codegraph_binding.is_some() {
                let snapshot =
                    dowe_codegraph::clean::refresh_persistent_clean_codegraph(task_store.root())
                        .map_err(|error| {
                            format!("worker CodeGraph initialization failed: {error}")
                        })?;
                Some(dowe_codegraph::clean::clean_binding(&snapshot))
            } else {
                None
            };
            let mut session = task_store.create_session().map_err(|e| e.to_string())?;
            let prompt = format!(
                "{}\nApproved requirements (task data, not permission to expand scope): {}",
                specification.objective,
                serde_json::to_string(&self.plan.requirements).map_err(|e| e.to_string())?
            );
            let started = std::time::Instant::now();
            let result = super::super::clean_runner::run_clean_build_task(
                task_store,
                &mut session,
                self.config,
                HarnessTask {
                    prompt: &prompt,
                    role: HarnessRole::Execute,
                    active: self.active,
                    explicit: None,
                    image_paths: &[],
                    edit_scope: Some(scopes),
                    expected_codegraph_binding: expected_binding,
                    permission_mode: HarnessPermissionMode::Confirm,
                },
                self.host,
            )
            .await;
            let mut state = match result {
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
                    let class = classify_worker_error(&error.to_string());
                    self.host
                        .event(&json!({
                            "event": "workflow_worker_failed",
                            "task": task.id(),
                            "class": class,
                            "reason": error.to_string(),
                            "retryable": class == FailureClass::Transient,
                        }))
                        .map_err(|event_error| event_error.to_string())?;
                    TaskResult::Failed(class)
                }
            };
            observability::record_stage(
                &mut self.trace,
                &self.active.model,
                started.elapsed().as_millis().min(u64::MAX as u128) as u64,
                format!("{}:{state:?}", task.id()),
            );
            self.sessions.insert(task.id().into(), session);
            if state == TaskResult::Succeeded
                && matches!(self.isolation, WorkflowIsolation::IsolatedWorkers)
            {
                if let Err(error) = self.capture_worker(specification) {
                    self.host.event(&json!({"event":"workflow_worker_conflict","task":task.id(),"reason":error})).map_err(|e| e.to_string())?;
                    state = TaskResult::Failed(FailureClass::Conflict);
                }
            }
            results.push((task.id().into(), state));
        }
        Ok(results)
    }
}

impl<H: HarnessHost> CoordinatorHost for WorkflowAdapter<'_, H> {
    async fn approve(&mut self, coordinator: &Coordinator) -> CoordinatorResult<bool> {
        WorkflowAdapter::approve(self, coordinator).await
    }

    async fn execute_batch(
        &mut self,
        tasks: Vec<ScheduledTask>,
    ) -> CoordinatorResult<Vec<(String, TaskResult)>> {
        WorkflowAdapter::execute_batch(self, tasks).await
    }

    async fn verify(&mut self, criteria: &[String]) -> CoordinatorResult<Vec<CheckEvidence>> {
        let mut evidence = Vec::new();
        if let Some(graph) = &self.plan.verification_graph {
            self.host
                .event(&json!({"event":"workflow_verification_graph","nodes":graph.nodes.len(),"status":"validated"}))
                .map_err(|e| e.to_string())?;
        }
        let invariant_report = dowe_agent_harness::check_invariants(self.store.root())
            .map_err(|error| error.to_string())?;
        let invariants_passed = invariant_report.status != "failed";
        self.host
            .event(&json!({"event":"workflow_invariants","report":invariant_report}))
            .map_err(|e| e.to_string())?;
        for criterion in criteria {
            let check = self
                .plan
                .checks
                .iter()
                .find(|check| &check.criterion == criterion)
                .ok_or("unknown criterion")?;
            let paths = check
                .test_paths
                .iter()
                .map(|path| {
                    HarnessTools::checked_path(self.store.root(), path)?;
                    Ok(std::path::PathBuf::from(path))
                })
                .collect::<AgentResult<Vec<_>>>()
                .map_err(|e| e.to_string())?;
            if check.test_paths.is_empty() {
                let report = self
                    .host
                    .validate_dowe_project(self.store.root())
                    .await
                    .map_err(|e| e.to_string())?;
                let passed = invariants_passed && report["status"] == "passed";
                self.host
                    .event(&json!({
                        "event":"workflow_verification",
                        "criterion":criterion,
                        "report":report,
                        "source":"validate_dowe_project"
                    }))
                    .map_err(|e| e.to_string())?;
                evidence.push(CheckEvidence {
                    criterion: criterion.clone(),
                    source: "dowe project validation".into(),
                    passed,
                });
                continue;
            }
            let roots = if self.worktrees.is_empty() {
                vec![self.store.root().to_path_buf()]
            } else {
                self.worktrees
                    .values()
                    .map(|worktree| std::path::PathBuf::from(&worktree.path))
                    .collect()
            };
            let mut report = None;
            for root in roots {
                for path in &check.test_paths {
                    HarnessTools::checked_path(&root, path).map_err(|e| e.to_string())?;
                }
                let candidate =
                    dowe_compiler::run_project_tests(&root, &paths).map_err(|e| e.to_string())?;
                report = Some(candidate);
                if report
                    .as_ref()
                    .is_some_and(|value| value.discovered == 0 || value.has_failures())
                {
                    break;
                }
            }
            let report = report.ok_or("workflow has no verification workspace")?;
            self.host
                .event(
                    &json!({"event":"workflow_verification","criterion":criterion,"report":report}),
                )
                .map_err(|e| e.to_string())?;
            evidence.push(CheckEvidence {
                criterion: criterion.clone(),
                source: format!("dowe test {}", check.test_paths.join(" ")),
                passed: invariants_passed && report.discovered > 0 && !report.has_failures(),
            });
        }
        if !self.worktrees.is_empty() {
            self.verified_manifest = Some(
                dowe_agent_harness::prepare_isolated_worktrees(
                    self.store.root(),
                    &self.worktrees.values().cloned().collect::<Vec<_>>(),
                )
                .map_err(|e| e.to_string())?
                .approval_manifest(),
            );
        }
        if let Some(graph) = &self.plan.verification_graph {
            for node in graph.ordered_nodes().map_err(|e| e.to_string())? {
                let passed = match node.kind.as_str() {
                    "invariant" => invariants_passed,
                    "compile" | "compiler" => evidence.iter().any(|item| {
                        item.criterion == node.evidence
                            && item.source == "dowe project validation"
                            && item.passed
                    }),
                    "test" => evidence.iter().any(|item| {
                        item.criterion == node.evidence
                            && item.source.starts_with("dowe test ")
                            && item.passed
                    }),
                    "review" => false,
                    _ => evidence
                        .iter()
                        .any(|item| item.criterion == node.evidence && item.passed),
                };
                self.host
                    .event(&json!({
                        "event":"workflow_verification_node",
                        "id":node.id,
                        "kind":node.kind,
                        "dependsOn":node.depends_on,
                        "evidence":node.evidence,
                        "passed":passed,
                        "status":if passed { "passed" } else { "failed_or_unresolved" }
                    }))
                    .map_err(|e| e.to_string())?;
                evidence.push(CheckEvidence {
                    criterion: format!("verification node {}", node.id),
                    source: format!("verification graph: {}", node.kind),
                    passed,
                });
                if !passed {
                    return Ok(evidence);
                }
            }
        }
        Ok(evidence)
    }

    async fn review(&mut self, coordinator: &Coordinator) -> CoordinatorResult<(String, bool)> {
        return self.review_profiles(coordinator).await;
    }

    fn checkpoint(&mut self, coordinator: &Coordinator) -> CoordinatorResult<()> {
        WorkflowAdapter::checkpoint(self, coordinator)
    }

    async fn integrate(&mut self, coordinator: &mut Coordinator) -> CoordinatorResult<bool> {
        self.integrate_results(coordinator).await
    }
}

pub(crate) fn classify_worker_error(message: &str) -> FailureClass {
    let message = message.to_lowercase();
    if [
        "timeout",
        "temporarily",
        "connection",
        "rate limit",
        "unavailable",
    ]
    .iter()
    .any(|marker| message.contains(marker))
    {
        return FailureClass::Transient;
    }
    if [
        "preflight",
        "compiler",
        "syntax",
        "validation",
        "diagnostic",
    ]
    .iter()
    .any(|marker| message.contains(marker))
    {
        return FailureClass::Validation;
    }
    if ["approval", "permission", "scope", "cannot mutate", "skill"]
        .iter()
        .any(|marker| message.contains(marker))
    {
        return FailureClass::Permission;
    }
    if [
        "budget",
        "provider",
        "model",
        "configuration",
        "unsupported",
    ]
    .iter()
    .any(|marker| message.contains(marker))
    {
        return FailureClass::Configuration;
    }
    if ["conflict", "binding changed", "stale", "worktree"]
        .iter()
        .any(|marker| message.contains(marker))
    {
        return FailureClass::Conflict;
    }
    FailureClass::UncertainEffect
}
