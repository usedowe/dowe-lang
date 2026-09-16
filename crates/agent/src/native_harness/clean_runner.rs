use super::clean_runner_context::record_codegraph_context;
use super::clean_runner_execution::execute_clean_call;
pub(crate) use super::clean_runner_read::run_clean_read_task;
use super::{
    HarnessConfig, HarnessHost, HarnessOutcome, HarnessRole, HarnessSession, HarnessStore,
    HarnessTask, HarnessTools, HarnessTurn, ToolResult,
};
use crate::{
    AgentError, AgentPrepareOptions, AgentRequestType, AgentResult, prepare_agent_request,
};
use serde_json::json;
use std::collections::BTreeSet;

pub async fn compact_clean_session(
    store: &HarnessStore,
    session: &mut HarnessSession,
    _config: &HarnessConfig,
    _active: &super::ModelSelection,
    _explicit: Option<&super::ModelSelection>,
    _host: &mut impl HarnessHost,
) -> AgentResult<()> {
    if session.interrupted {
        return Err(AgentError::new("cannot compact an interrupted workflow"));
    }
    if session.turns.is_empty() {
        return Err(AgentError::new(
            "no durable turns are available for compaction",
        ));
    }
    let start = session.context_start;
    let end = session.turns.len();
    let summary = json!({
        "schema": 1,
        "objective": session.initial_prompt_preview,
        "turns": end.saturating_sub(start),
        "events": session.events.len(),
        "references": [start, end],
        "policy": "deterministic bounded projection; durable turns remain available"
    });
    session.summary = Some(summary.to_string());
    session.context_start = end;
    session.events.push(json!({
        "event": "clean_context_compacted",
        "start": start,
        "end": end,
        "turns": end.saturating_sub(start)
    }));
    store.save_session(session)
}

pub async fn run_clean_build_task(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    task: HarnessTask<'_>,
    host: &mut impl HarnessHost,
) -> AgentResult<HarnessOutcome> {
    if task.role != HarnessRole::Execute {
        return Err(AgentError::new(
            "clean build runner requires the Execute role",
        ));
    }
    if let Some(expected) = task.expected_codegraph_binding.as_ref() {
        let snapshot = dowe_codegraph::clean::read_persistent_clean_codegraph(store.root())
            .map_err(|error| {
                AgentError::new(format!("CodeGraph binding cannot be verified: {error}"))
            })?;
        let actual = dowe_codegraph::clean::clean_binding(&snapshot);
        if &actual != expected {
            return Err(AgentError::new(
                "CodeGraph binding changed since plan approval; refresh and re-approve before BUILD",
            ));
        }
        if matches!(
            snapshot.freshness,
            dowe_codegraph::GraphFreshness::Stale | dowe_codegraph::GraphFreshness::Error
        ) {
            return Err(AgentError::new(
                "CodeGraph is stale since plan approval; refresh and re-approve before BUILD",
            ));
        }
    }
    let scopes = task.edit_scope.clone().unwrap_or_default();
    let scoped_execution = task.edit_scope.is_some();
    let mut tools = HarnessTools::with_scope(
        store.root(),
        &session.id,
        config.clone(),
        scoped_execution.then_some(scopes),
    )?;
    tools.set_permission_mode(task.permission_mode);
    let selection = task.explicit.unwrap_or(task.active);
    let mut prepared = prepare_agent_request(
        store.root(),
        task.prompt,
        AgentPrepareOptions {
            provider: Some(selection.provider.clone()),
            request_type: Some(AgentRequestType::Implementation),
            model: Some(selection.model.clone()),
            thinking_level: selection.thinking,
            image_paths: task.image_paths.to_vec(),
            stream: false,
        },
    )?;
    prepared
        .request
        .extra
        .insert("session_id".into(), json!(session.id.clone()));
    host.event(&json!({
        "event": "request_prepared",
        "session": session.id,
        "role": "execute",
        "requestType": prepared.request.request_type,
        "model": prepared.request.model.as_str(),
    }))?;
    record_codegraph_context(host, session, prepared.context.codegraph.as_ref())?;
    let mut history = Vec::<HarnessTurn>::new();
    let mut mutation_seen = false;
    let mut visual_attempted = false;
    let max_rounds = config.max_rounds.min(32);
    for round in 0..max_rounds {
        let history_scope = format!("{}/{}", selection.provider, selection.model);
        let (bounded_history, truncated) = super::continuation::request_turns_bounded(
            &history,
            &history_scope,
            config.max_output_bytes.max(8192),
            false,
        )?;
        let bounded_history: Vec<HarnessTurn> = serde_json::from_value(bounded_history)?;
        prepared.request.extra.insert(
            "dowe_harness_turns".into(),
            serde_json::to_value(&bounded_history)?,
        );
        if truncated {
            session
                .events
                .push(json!({"event":"clean_context_truncated","round":round + 1}));
        }
        host.event(&json!({"event":"clean_workflow_round","session":session.id,"round":round + 1,"mutation":true}))?;
        let response = match host.send(&prepared.request).await {
            Ok(response) => response,
            Err(error) => {
                for event in host.take_request_events() {
                    host.event(&event)?;
                }
                host.event(&json!({
                    "event": "error",
                    "message": error.to_string(),
                    "requestType": prepared.request.request_type,
                    "model": prepared.request.model
                }))?;
                return Err(error);
            }
        };
        for event in host.take_request_events() {
            host.event(&event)?;
        }
        if let Ok(text) = crate::agent_response_text(&response.payload) {
            host.event(&json!({
                "event": "response_received",
                "requestId": response.request_id,
                "requestType": response.request_type,
                "model": response.model,
                "role": "execute",
                "text": text,
                "payload": {"output_text": text}
            }))?;
        }
        let turn = super::response_turn(&response.payload)?;
        if turn.calls.is_empty() {
            if !task.image_paths.is_empty() && mutation_seen && !visual_attempted {
                let event = json!({
                    "event":"visualQA",
                    "status":"not_run",
                    "reason":"attached-reference task completed without a post-write screenshot attempt"
                });
                host.event(&event)?;
                session.events.push(event);
                session.interrupted = false;
                store.save_session(session)?;
                return Ok(HarnessOutcome::ValidationFailed);
            }
            session.set_initial_prompt_metadata(task.prompt);
            session.turns.push(turn);
            session.events.push(json!({"event":"clean_workflow_completed","requestId":response.request_id,"requestType":response.request_type,"model":response.model,"mutation":true}));
            session.interrupted = false;
            store.save_session(session)?;
            return Ok(HarnessOutcome::Completed);
        }
        let calls = turn.calls.clone();
        session.turns.push(turn.clone());
        history.push(turn);
        let mut results = Vec::new();
        let batch_calls = calls
            .iter()
            .filter(|call| matches!(call.name.as_str(), "write_file" | "edit_file"))
            .cloned()
            .collect::<Vec<_>>();
        if batch_calls
            .iter()
            .any(|call| matches!(call.name.as_str(), "write_file" | "edit_file"))
        {
            mutation_seen = true;
        }
        let batch_ids = if batch_calls.len() > 1 {
            let approvals = tools.prepare_text_write_batch(&batch_calls, HarnessRole::Execute)?;
            host.event(&json!({
                "event": "approval_required",
                "session": session.id,
                "tool": "write_batch",
                "count": approvals.len(),
                "state": "pending"
            }))?;
            let decision = host.approve_batch(&approvals).await?;
            let outputs = match decision {
                Some(true) => {
                    for call in &batch_calls {
                        host.event(&json!({
                            "event": "operation_started",
                            "call_id": call.id,
                            "operation": call.name,
                            "state": "executing_tool"
                        }))?;
                    }
                    tools.apply_text_write_batch(approvals)
                }
                Some(false) => {
                    let mut outputs = Vec::new();
                    for approval in approvals {
                        tools.reject(approval)?;
                        outputs.push(Ok(json!({"status":"rejected"})));
                    }
                    outputs
                }
                None => {
                    for approval in approvals {
                        tools.reject(approval)?;
                    }
                    session.interrupted = false;
                    store.save_session(session)?;
                    return Ok(HarnessOutcome::ApprovalRequired);
                }
            };
            for (call, output) in batch_calls.iter().zip(outputs) {
                let failed = output.is_err();
                let output = output.unwrap_or_else(|error| json!({"error": error.to_string()}));
                host.event(&json!({
                    "event": "tool_result",
                    "result": {"id": call.id.clone(), "name": call.name.clone(), "failed": failed, "output": output.clone()}
                }))?;
                let finished = json!({"event":"operation_finished","call_id":call.id.clone(),"name":call.name.clone(),"failed":failed,"output":output.clone()});
                session.events.push(finished.clone());
                host.event(&finished)?;
                results.push(ToolResult {
                    id: call.id.clone(),
                    name: call.name.clone(),
                    failed,
                    output,
                });
            }
            batch_calls
                .iter()
                .map(|call| call.id.clone())
                .collect::<BTreeSet<_>>()
        } else {
            BTreeSet::new()
        };
        for call in calls {
            if batch_ids.contains(&call.id) {
                continue;
            }
            if matches!(
                call.name.as_str(),
                "write_file" | "edit_file" | "write_asset" | "shell" | "run_validation"
            ) {
                host.event(&json!({
                    "event": "approval_required",
                    "session": session.id,
                    "tool": call.name,
                    "state": "pending"
                }))?;
            }
            let result = execute_clean_call(&mut tools, call.clone(), host).await;
            if matches!(
                call.name.as_str(),
                "write_file" | "edit_file" | "write_asset" | "generate_image"
            ) {
                mutation_seen = true;
            }
            if call.name == "capture_web_screenshot" {
                visual_attempted = true;
            }
            if let Err(error) = &result
                && error.to_string().contains("approval required")
            {
                host.event(&json!({
                    "event": "approval_required",
                    "session": session.id,
                    "tool": call.name,
                    "reason": error.to_string()
                }))?;
                // A pending approval is a resumable pause, not an interrupted
                // provider/process effect; recovery must not be required.
                session.interrupted = false;
                store.save_session(session)?;
                return Ok(HarnessOutcome::ApprovalRequired);
            }
            let failed = result.is_err();
            let output = result.unwrap_or_else(|error| json!({"error": error.to_string()}));
            host.event(&json!({
                "event": "tool_result",
                "result": {
                    "id": call.id.clone(),
                    "name": call.name.clone(),
                    "failed": failed,
                    "output": output.clone()
                }
            }))?;
            let finished = json!({"event":"operation_finished","call_id":call.id.clone(),"name":call.name.clone(),"failed":failed,"output":output.clone()});
            session.events.push(finished.clone());
            host.event(&finished)?;
            if call.name == "shell" && output["canceled"].as_bool() == Some(true) {
                host.event(&json!({
                    "event": "task_canceled",
                    "session": session.id,
                    "reason": "shell operation canceled"
                }))?;
                session.interrupted = false;
                store.save_session(session)?;
                return Ok(HarnessOutcome::Canceled);
            }
            results.push(ToolResult {
                id: call.id,
                name: call.name,
                failed,
                output,
            });
        }
        let result_turn = HarnessTurn {
            results,
            ..Default::default()
        };
        history.push(result_turn.clone());
        session.turns.push(result_turn);
        store.save_session(session)?;
    }
    host.event(&json!({"event":"clean_workflow_budget_exhausted","session":session.id,"maxRounds":max_rounds}))?;
    Ok(HarnessOutcome::BudgetExhausted)
}
