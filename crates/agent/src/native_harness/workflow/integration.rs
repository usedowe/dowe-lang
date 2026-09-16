use super::*;

impl<H: HarnessHost> WorkflowAdapter<'_, H> {
    pub(super) async fn integrate_results(
        &mut self,
        coordinator: &mut Coordinator,
    ) -> CoordinatorResult<bool> {
        if !self.worktrees.is_empty() {
            let worktrees = self.worktrees.values().cloned().collect::<Vec<_>>();
            let prepared =
                dowe_agent_harness::prepare_isolated_worktrees(self.store.root(), &worktrees)
                    .map_err(|error| error.to_string())?;
            let manifest = prepared.approval_manifest();
            let scopes = self
                .plan
                .tasks
                .iter()
                .flat_map(|task| &task.write_scopes)
                .map(dowe_agent_harness::AllowedEditSurface::new)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            for path in manifest["report"]["files"]
                .as_array()
                .ok_or("integration manifest has no files")?
            {
                let path = path.as_str().ok_or("invalid integration path")?;
                if !scopes.iter().any(|scope| scope.allows(path)) {
                    return Err(format!(
                        "integration path is outside approved task scopes: {path}"
                    ));
                }
            }
            if self.verified_manifest.as_ref() != Some(&manifest) {
                return Err(
                    "candidate changed after verification; tests and review must run again".into(),
                );
            }
            let approval = Approval {
                id: identifier(),
                session: coordinator.id().into(),
                call: ToolCall::new(
                    "workflow-integration",
                    "integrate_worktrees",
                    manifest.clone(),
                ),
                details: json!({
                    "workflow": coordinator.id(),
                    "policy": "Approval covers these exact patch and new-file hashes; any change requires new approval. Worktrees remain for recovery.",
                    "manifest": manifest,
                    "worktrees": self.worktrees.values().collect::<Vec<_>>(),
                }),
                before: None,
                after: None,
                before_bytes: None,
                after_bytes: None,
                reference_images: vec![],
            };
            if self
                .host
                .approve(&approval)
                .await
                .map_err(|error| error.to_string())?
                != Some(true)
            {
                coordinator.record_integration(IntegrationSummary {
                    ready: false,
                    ..IntegrationSummary::default()
                });
                self.host
                    .event(&json!({"event":"workflow_integration_required","workflow":coordinator.id()}))
                    .map_err(|error| error.to_string())?;
                return Ok(false);
            }
            let report = match prepared.apply() {
                Ok(report) => report,
                Err(error) => {
                    self.host
                        .event(&json!({
                            "event": "workflow_integration_failed",
                            "workflow": coordinator.id(),
                            "reason": error.to_string(),
                            "rollback": "See failure details; inspect retained worktrees before retrying",
                        }))
                        .map_err(|event_error| event_error.to_string())?;
                    return Err(error.to_string());
                }
            };
            self.refresh_integrated_codegraph(coordinator.id())?;
            coordinator.record_integration(IntegrationSummary {
                ready: true,
                applied: true,
                files: report.files.clone(),
                new_files: report.new_files.clone(),
                patch_bytes: report.patch_bytes,
                new_file_bytes: report.new_file_bytes,
            });
            self.checkpoint(coordinator)?;
            let retained = std::mem::take(&mut self.worktrees);
            let criteria = self
                .plan
                .checks
                .iter()
                .map(|check| check.criterion.clone())
                .collect::<Vec<_>>();
            let verification = self.verify(&criteria).await;
            self.worktrees = retained;
            if !verification?.iter().all(|check| check.passed) {
                return Err("integrated checkout failed verification; changes and worktrees retained for recovery".into());
            }
            self.record_product_update(
                coordinator.id(),
                report
                    .files
                    .clone()
                    .into_iter()
                    .chain(report.new_files.clone())
                    .collect(),
            )?;
            self.host
                .event(&json!({
                    "event": "workflow_integration_applied",
                    "workflow": coordinator.id(),
                    "files": report.files,
                    "newFiles": report.new_files,
                    "patchBytes": report.patch_bytes,
                    "newFileBytes": report.new_file_bytes,
                    "worktrees": worktrees,
                }))
                .map_err(|error| error.to_string())?;
            if self.cleanup_after_integration {
                let cleanup =
                    dowe_agent_harness::cleanup_isolated_worktrees(self.store.root(), &worktrees);
                self.host
                    .event(&json!({
                        "event": if cleanup.is_ok() {"workflow_worktrees_cleaned"} else {"workflow_worktrees_retained"},
                        "reason": cleanup.err().map(|error| error.to_string()),
                        "workflow": coordinator.id(),
                        "worktrees": worktrees,
                    }))
                    .map_err(|error| error.to_string())?;
            }
            return Ok(true);
        }
        let mut baseline = std::collections::BTreeMap::new();
        let mut tasks = Vec::new();
        for (task_id, session) in &self.sessions {
            let mut changes = Vec::new();
            for event in &session.events {
                if event["event"] != "operation_finished" {
                    continue;
                }
                let Some(receipt) = event.get("receipt") else {
                    continue;
                };
                let Some(path) = receipt["path"].as_str() else {
                    continue;
                };
                let before = string_field(receipt, "beforeFingerprint");
                let after = string_field(receipt, "afterFingerprint");
                if let Some(fingerprint) = &before {
                    baseline
                        .entry(path.to_owned())
                        .or_insert_with(|| fingerprint.clone());
                }
                changes.push(dowe_agent_harness::IntegrationChange {
                    path: path.into(),
                    before_fingerprint: before,
                    after_fingerprint: after,
                });
            }
            if !changes.is_empty() {
                let task_baseline = changes
                    .iter()
                    .filter_map(|change| {
                        change
                            .before_fingerprint
                            .as_ref()
                            .map(|fingerprint| (change.path.clone(), fingerprint.clone()))
                    })
                    .collect();
                tasks.push(dowe_agent_harness::IntegrationTask {
                    task_id: task_id.clone(),
                    baseline: task_baseline,
                    changes,
                });
            }
        }
        let report = dowe_agent_harness::analyze_integration(&baseline, &tasks)
            .map_err(|error| error.to_string())?;
        if report.ready {
            self.refresh_integrated_codegraph(coordinator.id())?;
            self.record_product_update(coordinator.id(), report.files.keys().cloned().collect())?;
        }
        coordinator.record_integration(IntegrationSummary {
            ready: report.ready,
            applied: report.ready,
            files: report.files.keys().cloned().collect(),
            ..IntegrationSummary::default()
        });
        self.host
            .event(&json!({
                "event": "workflow_integration",
                "workflow": coordinator.id(),
                "ready": report.ready,
                "issues": report.issues,
                "files": report.files,
            }))
            .map_err(|error| error.to_string())?;
        Ok(report.ready)
    }
}

impl<H: HarnessHost> WorkflowAdapter<'_, H> {
    fn refresh_integrated_codegraph(&mut self, workflow: &str) -> CoordinatorResult<()> {
        let snapshot = dowe_codegraph::clean::refresh_persistent_clean_codegraph(self.store.root())
            .map_err(|error| format!("integrated CodeGraph refresh failed: {error}"))?;
        let binding = dowe_codegraph::clean::clean_binding(&snapshot);
        self.codegraph_binding = Some(binding.clone());
        self.host
            .event(&json!({
                "event": "workflow_codegraph_refreshed",
                "workflow": workflow,
                "revision": binding.revision,
                "generation": binding.generation,
                "freshness": snapshot.freshness,
                "files": snapshot.manifest.fingerprints.len(),
            }))
            .map_err(|error| error.to_string())
    }

    fn record_product_update(
        &mut self,
        workflow: &str,
        changed_files: Vec<String>,
    ) -> CoordinatorResult<()> {
        let update = dowe_agent_harness::VerifiedProductUpdate {
            workflow_id: workflow.to_string(),
            objective: self.plan.objective.clone(),
            specification_path: self
                .plan
                .specification
                .as_ref()
                .map(|binding| binding.path.clone()),
            contract_paths: self
                .plan
                .contracts
                .iter()
                .map(|binding| binding.path.clone())
                .collect(),
            task_ids: self.plan.tasks.iter().map(|task| task.id.clone()).collect(),
            check_names: self
                .plan
                .checks
                .iter()
                .map(|check| check.criterion.clone())
                .collect(),
            changed_files,
            use_cases: self.plan.product_use_cases.clone(),
        };
        let path = dowe_agent_harness::record_verified_product_update(self.store.root(), &update)
            .map_err(|error| format!("product knowledge update failed: {error}"))?;
        self.host
            .event(&json!({
                "event": "workflow_product_knowledge_updated",
                "workflow": workflow,
                "path": path.strip_prefix(self.store.root()).unwrap_or(&path),
                "evidence": "verified",
            }))
            .map_err(|error| error.to_string())
    }
}

fn string_field(value: &serde_json::Value, name: &str) -> Option<String> {
    value
        .get(name)
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}
