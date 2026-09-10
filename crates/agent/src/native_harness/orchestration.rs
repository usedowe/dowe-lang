use super::{run_harness_turn, run_harness_turn_without_persistence, ChildExecutionRequest, HarnessConfig, HarnessHost, HarnessOutcome, HarnessRole, HarnessStore, HarnessTask, ModelSelection};
use crate::{AgentError, AgentResult};
use dowe_agent_harness::{project_native_receipt_event, AgentExecutionKind, AgentRole, TaskState, WorkerRole, WorkerState};
use dowe_codegraph::CodeGraphBinding;
use std::path::PathBuf;

/// Execute one explicitly-owned native turn and project only terminal native
/// operation receipts into the embedded orchestration record. This adapter is
/// deliberately not a delivery or approval authority.
pub async fn run_orchestrated_turn(
    store: &HarnessStore,
    session_id: &str,
    task_id: &str,
    worker_id: &str,
    binding: CodeGraphBinding,
    prompt: &str,
    role: HarnessRole,
    active: &ModelSelection,
    explicit: Option<&ModelSelection>,
    image_paths: &[PathBuf],
    config: &HarnessConfig,
    host: &mut impl HarnessHost,
) -> AgentResult<HarnessOutcome> {
    let mut session = store.load_session(session_id)?;
    let mut record = session
        .orchestration()
        .cloned()
        .ok_or_else(|| AgentError::new("native session has no embedded orchestration record"))?;
    if record.id != session_id || record.codegraph_binding != binding {
        return Err(AgentError::new("orchestration session identity or CodeGraphBinding mismatch"));
    }
    let task = record
        .tasks
        .iter_mut()
        .find(|task| task.id == task_id)
        .ok_or_else(|| AgentError::new("orchestration task is not owned by session"))?;
    if task.codegraph_binding != binding || record.state != dowe_agent_harness::SessionState::Active {
        return Err(AgentError::new("orchestration task or session binding/state mismatch"));
    }
    let worker_index = task
        .workers
        .iter()
        .position(|worker| worker.id == worker_id)
        .ok_or_else(|| AgentError::new("orchestration worker is not owned by task"))?;
    if record.agents.iter().find(|agent| agent.id == worker_id).is_none() {
        return Err(AgentError::new("orchestration worker is not owned by session"));
    }
    if task.workers[worker_index].role == WorkerRole::ReadOnly && role == HarnessRole::Execute {
        return Err(AgentError::new("read-only worker cannot execute the mutating role"));
    }
    if task.state == TaskState::Planned {
        task.start().map_err(orchestration_error)?;
    }
    let worker = &mut task.workers[worker_index];
    if worker.state == WorkerState::Pending {
        worker.start().map_err(orchestration_error)?;
    } else if worker.state != WorkerState::Running {
        return Err(AgentError::new("worker is not available for execution"));
    }
    let edit_scope = Some(worker.edit_surfaces.clone());
    let event_start = session.events.len();
    session.update_orchestration(record.clone())?;
    store.save_session(&mut session)?;

    let result = run_harness_turn(
        store,
        &mut session,
        config,
        HarnessTask {
            prompt,
            role,
            active,
            explicit,
            image_paths,
            edit_scope,
            expected_codegraph_binding: Some(binding.clone()),
        },
        host,
    )
    .await;

    let mut record = session
        .orchestration()
        .cloned()
        .ok_or_else(|| AgentError::new("native orchestration record disappeared during turn"))?;
    let task_view = record
        .tasks
        .iter()
        .find(|task| task.id == task_id)
        .ok_or_else(|| AgentError::new("orchestration task disappeared during turn"))?;
    if task_view.codegraph_binding != binding {
        return Err(AgentError::new("orchestration task binding changed during turn"));
    }
    for event in session.events.iter().skip(event_start) {
        if event["event"] != "operation_finished" {
            continue;
        }
        let projected = project_native_receipt_event(event.clone(), &binding, worker_id)
            .map_err(orchestration_error)?;
        let task = record.tasks.iter().find(|task| task.id == task_id).unwrap();
        if task.receipts.iter().any(|receipt| receipt.id == projected.receipt.id) {
            continue;
        }
        let worker = task.workers.iter().find(|worker| worker.id == worker_id).unwrap();
        if !worker.edit_surfaces.iter().any(|scope| scope.allows(&projected.receipt.path.path)) {
            return Err(AgentError::new("native receipt path is outside the worker edit scope"));
        }
        record.record_receipt(task_id, projected.receipt).map_err(orchestration_error)?;
    }

    let outcome = match &result {
        Ok(HarnessOutcome::Completed) => {
            let task = record.tasks.iter_mut().find(|task| task.id == task_id).unwrap();
            task.workers.iter_mut().find(|worker| worker.id == worker_id).unwrap().succeed().map_err(orchestration_error)?;
            if task.complete().is_err() { /* Other workers may still be active. */ }
            Ok(())
        }
        Ok(HarnessOutcome::Canceled) => {
            let task = record.tasks.iter_mut().find(|task| task.id == task_id).unwrap();
            task.workers.iter_mut().find(|worker| worker.id == worker_id).unwrap().cancel().map_err(orchestration_error)?;
            task.cancel().map_err(orchestration_error)
        }
        Ok(HarnessOutcome::ApprovalRequired) | Ok(HarnessOutcome::ClarificationRequired) | Ok(HarnessOutcome::BudgetExhausted) => Ok(()),
        Err(_) => {
            let task = record.tasks.iter_mut().find(|task| task.id == task_id).unwrap();
            task.workers.iter_mut().find(|worker| worker.id == worker_id).unwrap().fail().map_err(orchestration_error)?;
            if task.state == TaskState::Running { let _ = task.fail(); }
            Ok(())
        }
    };
    outcome.map_err(|error| error)?;
    session.update_orchestration(record)?;
    store.save_session(&mut session)?;
    result
}

/// Runs a child turn with a fresh in-memory conversation while retaining the
/// parent native session as the only persisted session record.
pub async fn run_child_turn(
    store: &HarnessStore,
    request: ChildExecutionRequest,
    config: &HarnessConfig,
    host: &mut impl HarnessHost,
) -> AgentResult<HarnessOutcome> {
    let mut parent = store.load_session(&request.session_id)?;
    let record = parent.orchestration().cloned().ok_or_else(|| AgentError::new("native session has no embedded orchestration record"))?;
    if record.id != request.session_id || record.codegraph_binding != request.codegraph_binding || record.state != dowe_agent_harness::SessionState::Active {
        return Err(AgentError::new("child session identity, state, or CodeGraphBinding mismatch"));
    }
    let parent_agent = record.agents.iter().find(|agent| agent.id == request.parent_agent_id).ok_or_else(|| AgentError::new("child parent agent is not owned by session"))?;
    if parent_agent.role != AgentRole::Coordinator || parent_agent.execution_kind == AgentExecutionKind::Child || parent_agent.parent_id.is_some() {
        return Err(AgentError::new("child parent must be a same-session coordinator"));
    }
    let child = record.agents.iter().find(|agent| agent.id == request.child_agent_id).ok_or_else(|| AgentError::new("child agent is not owned by session"))?;
    if child.parent_id.as_deref() != Some(request.parent_agent_id.as_str()) || child.execution_kind != AgentExecutionKind::Child || child.role != AgentRole::Worker || request.worker_id != request.child_agent_id {
        return Err(AgentError::new("child agent identity or parent ownership mismatch"));
    }
    let task = record.tasks.iter().find(|task| task.id == request.task_id).ok_or_else(|| AgentError::new("child task is not owned by session"))?;
    if task.codegraph_binding != request.codegraph_binding {
        return Err(AgentError::new("child task CodeGraphBinding does not match request"));
    }
    let worker = task.workers.iter().find(|worker| worker.id == request.worker_id).ok_or_else(|| AgentError::new("child worker is not owned by task"))?;
    if worker.edit_surfaces != request.edit_scope {
        return Err(AgentError::new("child edit scope does not match task worker scope"));
    }
    config.resolve(request.role, request.explicit.as_ref(), &request.active)?;

    let parent_event_count = parent.events.len();
    let mut child_session = parent.clone();
    child_session.turns.clear();
    child_session.events.clear();
    child_session.context_start = 0;
    child_session.summary = None;
    // The child uses the same bounded compaction policy in memory. Its
    // compacted summary and events are projected back to the parent only after
    // the turn completes; no child save can overwrite the parent's session.
    let result = run_harness_turn_without_persistence(store, &mut child_session, config, HarnessTask {
        prompt: &request.prompt, role: request.role, active: &request.active,
        explicit: request.explicit.as_ref(), image_paths: &[],
        edit_scope: Some(request.edit_scope.clone()), expected_codegraph_binding: Some(request.codegraph_binding.clone()),
    }, host).await;

    for mut event in child_session.events {
        if let Some(object) = event.as_object_mut() {
            object.insert("session_id".into(), serde_json::json!(request.session_id));
            object.insert("task_id".into(), serde_json::json!(request.task_id));
            object.insert("worker_id".into(), serde_json::json!(request.worker_id));
            object.insert("agent_id".into(), serde_json::json!(request.child_agent_id));
        }
        parent.events.push(event);
    }
    let mut record = parent.orchestration().cloned().unwrap_or(record);
    if let Some(task) = record.tasks.iter_mut().find(|task| task.id == request.task_id) {
        for event in parent.events.iter().skip(parent_event_count) {
            if event["event"] != "operation_finished" { continue; }
            let mut projected = project_native_receipt_event(event.clone(), &request.codegraph_binding, &request.worker_id).map_err(orchestration_error)?;
            projected.receipt.agent_id = Some(request.child_agent_id.clone());
            if !task.receipts.iter().any(|receipt| receipt.id == projected.receipt.id) {
                task.record_receipt(projected.receipt).map_err(orchestration_error)?;
            }
        }
    }
    parent.update_orchestration(record)?;
    store.save_session(&mut parent)?;
    result
}

fn orchestration_error(error: impl std::fmt::Display) -> AgentError {
    AgentError::new(error.to_string())
}
