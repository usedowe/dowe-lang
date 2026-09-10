use super::request::build_request;
use super::{
    Approval, HarnessConfig, HarnessRole, HarnessSession, HarnessStore, HarnessTools, HarnessTurn,
    ModelSelection, ToolResult, response_turn,
};
use crate::codegraph_enrichment::{SemanticEnrichment, SemanticStatus, enrich_codegraph};
use crate::{
    AgentError, AgentMessage, AgentMessageContent, AgentPrepareOptions, AgentRequest,
    AgentRequestType, AgentResult, AgentServerResponse, AgentUsageTotals, GeneratedImage,
    agent_model_details, prepare_agent_request,
};
use base64::Engine;
use dowe_codegraph::{CodeGraphBinding, ensure_persistent_codegraph, read_persistent_codegraph};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
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
    fn generate_image(
        &mut self,
        selection: &ModelSelection,
        approval: &Approval,
    ) -> impl std::future::Future<Output = AgentResult<GeneratedImage>> {
        let _ = (selection, approval);
        async { Err(AgentError::new("image generation provider is unavailable")) }
    }
    fn ask_clarification(
        &mut self,
        question: &crate::ClarificationQuestion,
    ) -> impl std::future::Future<Output = AgentResult<Option<String>>> {
        let _ = question;
        async { Ok(None) }
    }
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
    ClarificationRequired,
}

pub async fn run_harness_turn(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    task: super::HarnessTask<'_>,
    host: &mut impl HarnessHost,
) -> AgentResult<HarnessOutcome> {
    run_harness_turn_with_persistence(store, session, config, task, host, true).await
}

pub(crate) async fn run_harness_turn_without_persistence(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    task: super::HarnessTask<'_>,
    host: &mut impl HarnessHost,
) -> AgentResult<HarnessOutcome> {
    run_harness_turn_with_persistence(store, session, config, task, host, false).await
}

async fn run_harness_turn_with_persistence(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    task: super::HarnessTask<'_>,
    host: &mut impl HarnessHost,
    persist: bool,
) -> AgentResult<HarnessOutcome> {
    let PreparedHarnessTurn {
        role,
        active,
        explicit,
        selected,
        prompt,
        semantic,
        image_selection,
        mut tools,
        turn_codegraph_binding,
        expected_codegraph_binding,
    } = prepare_harness_turn(store, session, config, host, persist, task).await?;

    let started = Instant::now();
    let usage_start = session.events.len();
    let mut usage = AgentUsageTotals::default();
    if semantic.status == SemanticStatus::Miss {
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({
                "event":"auxiliary_response",
                "requestId":format!("codegraph-enrichment-{}", super::digest(semantic.context.as_bytes())),
                "provider":semantic.provider.clone(),
                "model":semantic.model.clone(),
                "role":"codegraph",
                "usage":semantic.usage,
            }),
        )?;
        usage = session.usage_since(usage_start);
    }
    // Retrying is safe only before any tool result exists; otherwise a
    // repeated request could duplicate an observed side effect.
    let mut has_tool_result = false;
    let mut generated_image_for_continuation: Option<GeneratedImage> = None;
    for round in 0..config.max_rounds {
        if exhausted(config, &usage, 0, started) {
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"budget_exhausted","session":session.id}),
            )?;
            return Ok(HarnessOutcome::BudgetExhausted);
        }
        let mut request = build_request(
            store,
            session,
            config,
            role,
            &selected,
            &prompt,
            Some(&semantic),
        )?;
        if let Some(image) = &generated_image_for_continuation {
            request.messages.push(AgentMessage {
                role: "user".into(),
                content: AgentMessageContent::Parts(vec![
                    crate::AgentMessagePart::Text {
                        text: "Generated image asset (inspect it before continuing):".into(),
                    },
                    crate::AgentMessagePart::ImageUrl {
                        image_url: crate::ImageUrl {
                            url: format!(
                                "data:{};base64,{}",
                                image.mime_type,
                                base64::engine::general_purpose::STANDARD.encode(&image.bytes)
                            ),
                        },
                    },
                ]),
            });
        }
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
            // token_budget is a per-request safety bound. Compaction gets the
            // same bound instead of the remaining cumulative total so an
            // otherwise valid task cannot fail while preparing its summary.
            compact_config.token_budget = config.token_budget;
            let _compact_charge = tokio::time::timeout(
                Duration::from_secs(config.duration_seconds).saturating_sub(started.elapsed()),
                compact(
                    store,
                    session,
                    &compact_config,
                    &active,
                    explicit.as_ref(),
                    host,
                    &mut usage,
                    persist,
                ),
            )
            .await
            .map_err(|_| {
                AgentError::new(
                    "task duration exhausted during compaction; inspect the preserved history",
                )
            })??;
            tools.restore_loaded_skills(&session.turns[session.context_start..]);
            request = build_request(
                store,
                session,
                config,
                role,
                &selected,
                &prompt,
                Some(&semantic),
            )?;
            estimate = super::task::estimate_request(&request)?;
            if estimate + 6144 >= limit {
                return Err(AgentError::new(
                    "compacted context still exceeds destination window; start a new session",
                ));
            }
        }
        if exhausted(config, &usage, 0, started)
            || estimate.saturating_add(4096) > config.token_budget
        {
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"budget_exhausted","reason":"estimated_next_request","estimated_input_tokens":estimate}),
            )?;
            return Ok(HarnessOutcome::BudgetExhausted);
        }
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"request_prepared","requestType":"conversation","requestId":request.request_id,"session":session.id,"role":role,"provider":selected.provider,"model":selected.model,"round":round,"estimated_input_tokens":estimate}),
        )?;
        let mut retry = 0_u8;
        let response = loop {
            let remaining =
                Duration::from_secs(config.duration_seconds).saturating_sub(started.elapsed());
            let result =
                tokio::time::timeout(remaining, transport::send(host, &request, store, session))
                    .await;
            match result {
                Ok(Ok(response)) => break response,
                Ok(Err(error))
                    if !has_tool_result
                        && retry < transport::MAX_TRANSIENT_RETRIES
                        && let Some(classification) =
                            transport::transient_failure_classification(&error) =>
                {
                    retry += 1;
                    let remaining = Duration::from_secs(config.duration_seconds)
                        .saturating_sub(started.elapsed());
                    let delay = transport::retry_delay(retry).min(remaining);
                    // Never let backoff consume more than the task's duration budget.
                    if !delay.is_zero() {
                        tokio::time::sleep(delay).await;
                    }
                    // Persist only a stable classification, never provider error text.
                    emit_persist(
                        store,
                        session,
                        persist,
                        host,
                        json!({"event":"provider_retry","requestType":"conversation","requestId":request.request_id,"provider":selected.provider,"model":selected.model,"attempt":retry + 1,"maxAttempts":transport::MAX_TRANSIENT_RETRIES + 1,"classification":classification}),
                    )?;
                }
                failed => {
                    let error = match failed {
                        Ok(Err(error)) => error,
                        _ => AgentError::new("task duration exhausted during provider request"),
                    };
                    if round == 0 {
                        session.turns.pop();
                    }
                    emit_persist(
                        store,
                        session,
                        persist,
                        host,
                        json!({"event":"error","requestType":"conversation","requestId":request.request_id,"model":selected.model,"payload":{"error":{"code":"provider_request_failed","message":tools.redactor.text(&error.to_string())}}}),
                    )?;
                    return Err(AgentError::new(tools.redactor.text(&error.to_string())));
                }
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

        let mut turn = response_turn(&response.payload)?;
        if !turn.continuation.is_empty() || turn.calls.iter().any(|call| call.signature.is_some()) {
            turn.continuation_scope = Some(format!("{}/{}", selected.provider, selected.model));
        }
        // Learn asset encodings before projecting the response, but retain the
        // original calls for execution so approval/apply still receives the
        // exact decoded bytes.
        let calls = turn.calls.clone();
        for call in &calls {
            if call.name == "write_asset"
                && let Some(content) = call.arguments["content_base64"].as_str()
            {
                tools.redactor.add(content);
            }
        }
        let continuation = std::mem::take(&mut turn.continuation);
        let mut projection = serde_json::to_value(&turn)?;
        tools.redactor.value(&mut projection);
        turn = serde_json::from_value(projection)?;
        turn.continuation = continuation;
        let text = turn.message.as_ref().map(|message| &message.content);
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"response_received","requestType":"conversation","requestId":request.request_id,"payload":{"output_text":text},"role":role,"provider":selected.provider,"model":selected.model,"text":text,"usage":crate::agent_response_usage(&selected.provider,&selected.model,&response.payload)}),
        )?;
        usage = session.usage_since(usage_start);
        session.turns.push(turn);
        session.set_initial_prompt_metadata(&prompt);
        session.interrupted = !calls.is_empty();
        if persist {
            store.save_session(session)?;
        }
        if calls.is_empty() {
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"task_state","status":"done"}),
            )?;
            return Ok(HarnessOutcome::Completed);
        }
        let call_results = execute_harness_calls(
            store,
            session,
            config,
            role,
            &mut tools,
            calls,
            host,
            started,
            &usage,
            persist,
            turn_codegraph_binding.as_ref(),
            &image_selection,
        )
        .await?;
        let HarnessCallResults {
            results,
            screenshot_message,
            approval_required,
            canceled,
            graph_dirty,
            generated_image_for_continuation: generated_image,
            outcome,
        } = call_results;
        generated_image_for_continuation = generated_image;
        if let Some(outcome) = outcome {
            return Ok(outcome);
        }
        if graph_dirty {
            if expected_codegraph_binding.is_some() {
                emit_persist(
                    store,
                    session,
                    persist,
                    host,
                    json!({"event":"codegraph_refresh_deferred","reason":"orchestration binding remains anchored to the task baseline"}),
                )?;
            } else {
                match ensure_persistent_codegraph(store.root()) {
                    Ok(snapshot) => emit_persist(
                        store,
                        session,
                        persist,
                        host,
                        json!({"event":"codegraph_updated","generation":snapshot.generation,"revision":snapshot.manifest.revision,"changed":snapshot.changed}),
                    )?,
                    Err(error) => emit_persist(
                        store,
                        session,
                        persist,
                        host,
                        json!({"event":"codegraph_warning","message":tools.redactor.text(&error.to_string())}),
                    )?,
                }
            }
        }
        session.turns.push(HarnessTurn {
            results,
            ..Default::default()
        });
        if let Some(message) = screenshot_message {
            session.turns.push(HarnessTurn {
                message: Some(message),
                ..Default::default()
            });
        }
        has_tool_result = true;
        session.interrupted = false;
        if persist {
            store.save_session(session)?;
        }
        if canceled {
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"task_canceled","session":session.id}),
            )?;
            return Ok(HarnessOutcome::Canceled);
        }
        if approval_required {
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"task_state","status":"awaiting_approval"}),
            )?;
            return Ok(HarnessOutcome::ApprovalRequired);
        }
    }
    emit_persist(
        store,
        session,
        persist,
        host,
        json!({"event":"budget_exhausted","reason":"tool_rounds"}),
    )?;
    Ok(HarnessOutcome::BudgetExhausted)
}

include!("engine/preparation.rs");
include!("engine/call_execution.rs");
include!("engine/receipts_and_events.rs");
include!("engine/compaction.rs");
