use super::request::build_request;
use super::{
    Approval, HarnessConfig, HarnessRole, HarnessSession, HarnessStore, HarnessTools, HarnessTurn,
    ModelSelection, ToolResult, response_turn,
};
use crate::{
    AgentError, AgentMessage, AgentMessageContent, AgentPrepareOptions, AgentRequest,
    AgentRequestType, AgentResult, AgentServerResponse, AgentUsageTotals, agent_model_details,
    prepare_agent_request,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

mod shell_host;
mod transport;

pub trait HarnessHost {
    fn supervisor(&self) -> AgentResult<Option<dowe_runtime::SupervisorCommand>> {
        Ok(None)
    }
    fn take_request_events(&mut self) -> Vec<Value> {
        Vec::new()
    }
    fn open_terminal(&mut self) -> AgentResult<Box<dyn super::HarnessTerminal>> {
        Err(AgentError::new(
            "interactive shell requires a local terminal adapter; no process started",
        ))
    }
    fn send(
        &mut self,
        request: &AgentRequest,
    ) -> impl std::future::Future<Output = AgentResult<AgentServerResponse>>;
    fn approve(
        &mut self,
        approval: &Approval,
    ) -> impl std::future::Future<Output = AgentResult<Option<bool>>>;
    fn event(&mut self, event: &Value) -> AgentResult<()>;
    fn secrets(&self) -> Vec<String> {
        Vec::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessOutcome {
    Completed,
    ApprovalRequired,
    BudgetExhausted,
    Canceled,
}

pub async fn run_harness_turn(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    task: super::HarnessTask<'_>,
    host: &mut impl HarnessHost,
) -> AgentResult<HarnessOutcome> {
    let super::HarnessTask {
        prompt,
        role,
        active,
        explicit,
        image_paths,
    } = task;
    config.validate()?;
    let _task = store.lease_session(&session.id)?;
    if session.interrupted {
        return Err(AgentError::new(
            "session has an interrupted operation; inspect its events and start a new session without replaying tools",
        ));
    }
    let selected = config.resolve(role, explicit, active)?;
    let historical_images = role == HarnessRole::Execute && session.turns.iter().skip(session.context_start).any(|turn| {
        turn.message.as_ref().is_some_and(|message| matches!(&message.content, AgentMessageContent::Parts(parts) if parts.iter().any(|part| matches!(part, crate::AgentMessagePart::ImageUrl { .. }))))
    });
    config.require_capabilities(
        &selected,
        role,
        !image_paths.is_empty() || historical_images,
    )?;
    let mut tools = HarnessTools::new(store.root(), &session.id, config.clone())?;
    tools.set_supervisor(host.supervisor()?);
    tools.restore_loaded_skills(&session.turns[session.context_start..]);
    for secret in host.secrets() {
        tools.redactor.add(&secret);
    }
    let prompt = tools.redactor.text(prompt);
    let user = HarnessTurn {
        message: Some(AgentMessage {
            role: "user".into(),
            content: if image_paths.is_empty() {
                AgentMessageContent::Text(prompt.clone())
            } else {
                let mut parts = vec![crate::AgentMessagePart::Text {
                    text: prompt.clone(),
                }];
                for image in crate::encode_image_paths(image_paths)? {
                    parts.push(crate::AgentMessagePart::ImageUrl {
                        image_url: crate::ImageUrl {
                            url: image.data_url,
                        },
                    });
                }
                AgentMessageContent::Parts(parts)
            },
        }),
        ..Default::default()
    };
    session.turns.push(user);
    store.save_session(session)?;
    let started = Instant::now();
    let usage_start = session.events.len();
    let mut usage = AgentUsageTotals::default();
    let mut charged_estimate = 0_u64;
    for round in 0..config.max_rounds {
        if exhausted(config, &usage, charged_estimate, started) {
            emit(
                store,
                session,
                host,
                json!({"event":"budget_exhausted","session":session.id}),
            )?;
            return Ok(HarnessOutcome::BudgetExhausted);
        }
        let mut request = build_request(store, session, config, role, &selected, &prompt)?;
        let limit = config.context_limit.or_else(|| {
            agent_model_details(&selected.provider, &selected.model)
                .and_then(|details| details.context_window)
        });
        let mut estimate = super::task::estimate_request(&request)?;
        if limit.is_none() && estimate > 8192 {
            return Err(AgentError::new(
                "model context window unknown and request exceeds the small-task safety budget; configure context_limit before automatic compaction",
            ));
        }
        if let Some(limit) = limit
            && estimate >= limit.saturating_sub(6144) * 80 / 100
        {
            let mut compact_config = config.clone();
            compact_config.token_budget = config
                .token_budget
                .saturating_sub(charged_estimate.max(known_tokens(&usage)));
            let compact_charge = tokio::time::timeout(
                Duration::from_secs(config.duration_seconds).saturating_sub(started.elapsed()),
                compact(
                    store,
                    session,
                    &compact_config,
                    active,
                    explicit,
                    host,
                    &mut usage,
                ),
            )
            .await
            .map_err(|_| {
                AgentError::new(
                    "task duration exhausted during compaction; inspect the preserved history",
                )
            })??;
            charged_estimate = charged_estimate.saturating_add(compact_charge);
            tools.restore_loaded_skills(&session.turns[session.context_start..]);
            request = build_request(store, session, config, role, &selected, &prompt)?;
            estimate = super::task::estimate_request(&request)?;
            if estimate + 6144 >= limit {
                return Err(AgentError::new(
                    "compacted context still exceeds destination window; start a new session",
                ));
            }
        }
        if exhausted(config, &usage, charged_estimate, started)
            || charged_estimate
                .max(known_tokens(&usage))
                .saturating_add(estimate)
                .saturating_add(4096)
                > config.token_budget
        {
            emit(
                store,
                session,
                host,
                json!({"event":"budget_exhausted","reason":"estimated_next_request","estimated_input_tokens":estimate}),
            )?;
            return Ok(HarnessOutcome::BudgetExhausted);
        }
        emit(
            store,
            session,
            host,
            json!({"event":"request_prepared","requestType":"conversation","requestId":request.request_id,"session":session.id,"role":role,"provider":selected.provider,"model":selected.model,"round":round,"estimated_input_tokens":estimate}),
        )?;
        let remaining = config
            .duration_seconds
            .saturating_sub(started.elapsed().as_secs());
        let response = match tokio::time::timeout(
            Duration::from_secs(remaining),
            transport::send(host, &request, store, session),
        )
        .await
        {
            Ok(Ok(response)) => response,
            failed => {
                let error = match failed {
                    Ok(Err(error)) => error,
                    _ => AgentError::new("task duration exhausted during provider request"),
                };
                if round == 0 {
                    session.turns.pop();
                }
                emit(
                    store,
                    session,
                    host,
                    json!({"event":"error","requestType":"conversation","requestId":request.request_id,"model":selected.model,"payload":{"error":{"code":"provider_request_failed","message":tools.redactor.text(&error.to_string())}}}),
                )?;
                return Err(AgentError::new(tools.redactor.text(&error.to_string())));
            }
        };
        if response.request_id != request.request_id
            || response.model != request.model
            || response.request_type != request.request_type
        {
            return Err(AgentError::new(
                "provider response does not match harness request",
            ));
        }

        charged_estimate = charged_estimate.saturating_add(
            crate::agent_response_usage(&selected.provider, &selected.model, &response.payload)
                .map_or(estimate + 4096, |usage| usage.context_tokens()),
        );
        let mut turn = response_turn(&response.payload)?;
        if !turn.continuation.is_empty() || turn.calls.iter().any(|call| call.signature.is_some()) {
            turn.continuation_scope = Some(format!("{}/{}", selected.provider, selected.model));
        }
        let continuation = std::mem::take(&mut turn.continuation);
        let mut projection = serde_json::to_value(&turn)?;
        tools.redactor.value(&mut projection);
        turn = serde_json::from_value(projection)?;
        turn.continuation = continuation;
        let calls = turn.calls.clone();
        let text = turn.message.as_ref().map(|message| &message.content);
        emit(
            store,
            session,
            host,
            json!({"event":"response_received","requestType":"conversation","requestId":request.request_id,"payload":{"output_text":text},"role":role,"provider":selected.provider,"model":selected.model,"text":text,"usage":crate::agent_response_usage(&selected.provider,&selected.model,&response.payload)}),
        )?;
        usage = session.usage_since(usage_start);
        session.turns.push(turn);
        session.interrupted = !calls.is_empty();
        store.save_session(session)?;
        if calls.is_empty() {
            return Ok(HarnessOutcome::Completed);
        }
        let mut results = Vec::new();
        let mut approval_required = false;
        let mut canceled = false;
        for call in calls {
            let output = if canceled || approval_required {
                Ok(
                    json!({"status":"not_executed","reason":if canceled {"task_canceled"} else {"approval_required"}}),
                )
            } else if exhausted(config, &usage, charged_estimate, started) {
                Ok(json!({"status":"not_executed","reason":"budget_exhausted"}))
            } else {
                match tools.prepare(&call, role) {
                    Ok(Some(approval)) => {
                        emit(
                            store,
                            session,
                            host,
                            json!({"event":"approval_required","approval":approval}),
                        )?;
                        match host.approve(&approval).await? {
                            Some(true) if exhausted(config, &usage, charged_estimate, started) => {
                                tools.reject(approval)?;
                                Ok(json!({"status":"not_executed","reason":"budget_exhausted"}))
                            }
                            Some(true) => {
                                session.interrupted = true;
                                emit(
                                    store,
                                    session,
                                    host,
                                    json!({"event":"operation_started","call_id":call.id,"approval_id":approval.id}),
                                )?;
                                let output = if call.name == "shell" {
                                    tokio::time::timeout(
                                        Duration::from_secs(
                                            config
                                                .duration_seconds
                                                .saturating_sub(started.elapsed().as_secs()),
                                        ),
                                        tools.run_shell_observed(
                                            approval,
                                            &mut shell_host::ObservedShell {
                                                host,
                                                store,
                                                session,
                                            },
                                        ),
                                    )
                                    .await
                                    .unwrap_or_else(|_| {
                                        Err(AgentError::new(
                                            "task duration exhausted; shell cancellation requested",
                                        ))
                                    })
                                } else {
                                    tools.apply_write(approval)
                                };
                                emit(
                                    store,
                                    session,
                                    host,
                                    json!({"event":"operation_finished","call_id":call.id,"result":output.as_ref().ok(),"failed":output.is_err()}),
                                )?;
                                output
                            }
                            decision => {
                                tools.reject(approval)?;
                                if decision.is_none() {
                                    approval_required = true;
                                }
                                Ok(
                                    json!({"status":"not_executed","reason":if decision.is_none() {"approval_required"} else {"user_rejected"}}),
                                )
                            }
                        }
                    }
                    Ok(None) => tools.execute_read(&call),
                    Err(error) => Err(error),
                }
            };
            let failed = output.is_err();
            let mut output = output
                .unwrap_or_else(|error| json!({"error":tools.redactor.text(&error.to_string())}));
            canceled |= output["canceled"] == true;
            tools.redactor.value(&mut output);
            output = super::privacy::bounded_value(output, config.max_output_bytes);
            let result = ToolResult {
                id: call.id,
                name: call.name,
                failed,
                output,
            };
            emit(
                store,
                session,
                host,
                json!({"event":"tool_result","result":result}),
            )?;
            results.push(result);
        }
        session.turns.push(HarnessTurn {
            results,
            ..Default::default()
        });
        session.interrupted = false;
        store.save_session(session)?;
        if canceled {
            emit(
                store,
                session,
                host,
                json!({"event":"task_canceled","session":session.id}),
            )?;
            return Ok(HarnessOutcome::Canceled);
        }
        if approval_required {
            return Ok(HarnessOutcome::ApprovalRequired);
        }
    }
    emit(
        store,
        session,
        host,
        json!({"event":"budget_exhausted","reason":"tool_rounds"}),
    )?;
    Ok(HarnessOutcome::BudgetExhausted)
}

fn exhausted(
    config: &HarnessConfig,
    usage: &AgentUsageTotals,
    estimated: u64,
    started: Instant,
) -> bool {
    started.elapsed().as_secs() >= config.duration_seconds
        || known_tokens(usage).max(estimated) >= config.token_budget
        || config
            .cost_budget_usd
            .is_some_and(|cost| usage.cost_usd >= cost || usage.incomplete_cost)
}

fn known_tokens(usage: &AgentUsageTotals) -> u64 {
    usage
        .input
        .saturating_add(usage.output)
        .saturating_add(usage.cache_read)
        .saturating_add(usage.cache_write)
}

fn emit(
    store: &HarnessStore,
    session: &mut HarnessSession,
    host: &mut impl HarnessHost,
    event: Value,
) -> AgentResult<()> {
    session.events.push(event.clone());
    store.save_session(session)?;
    host.event(&event)
}

include!("engine/compaction.rs");
