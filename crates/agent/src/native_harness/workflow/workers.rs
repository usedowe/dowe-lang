use super::*;

impl<H: HarnessHost> WorkflowAdapter<'_, H> {
    fn dependency_results(
        &self,
        task: &WorkflowTask,
    ) -> CoordinatorResult<Vec<dowe_agent_harness::WorkerResult>> {
        task.dependencies
            .iter()
            .map(|id| {
                self.worker_results
                    .get(id)
                    .cloned()
                    .ok_or_else(|| format!("dependency result is unavailable: {id}"))
            })
            .collect()
    }

    pub(super) fn prepare_worker(
        &mut self,
        task: &WorkflowTask,
    ) -> CoordinatorResult<HarnessStore> {
        let dependencies = self.dependency_results(task)?;
        let worker = dowe_agent_harness::create_worktree_from_results(
            self.store.root(),
            &identifier(),
            &dependencies,
        )
        .map_err(|e| e.to_string())?;
        self.worker_worktrees
            .insert(task.id.clone(), worker.clone());
        self.host.event(&json!({"event":"workflow_worker_created","task":task.id,"path":worker.path,"baseRevision":worker.base_revision,"dependencies":task.dependencies})).map_err(|e| e.to_string())?;
        self.store
            .for_project_root(&worker.path)
            .map_err(|e| e.to_string())
    }

    pub(super) fn capture_worker(&mut self, task: &WorkflowTask) -> CoordinatorResult<()> {
        let dependencies = self.dependency_results(task)?;
        let scopes = task
            .write_scopes
            .iter()
            .map(dowe_agent_harness::AllowedEditSurface::new)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let worker = self
            .worker_worktrees
            .get(&task.id)
            .ok_or("worker workspace is missing")?;
        let result = dowe_agent_harness::capture_worker_result(
            self.store.root(),
            worker,
            &task.id,
            &dependencies,
            &scopes,
        )
        .map_err(|e| e.to_string())?;
        self.host.event(&json!({"event":"workflow_worker_captured","task":result.task_id(),"files":result.changed_files()})).map_err(|e| e.to_string())?;
        self.worker_results.insert(task.id.clone(), result);
        if self.worker_results.len() == self.plan.tasks.len() {
            let candidate = dowe_agent_harness::create_worktree_from_results(
                self.store.root(),
                &identifier(),
                &self.worker_results.values().cloned().collect::<Vec<_>>(),
            )
            .map_err(|e| e.to_string())?;
            self.worktrees.insert("candidate".into(), candidate.clone());
            self.host.event(&json!({"event":"workflow_candidate_created","path":candidate.path,"workers":self.worker_results.len()})).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
