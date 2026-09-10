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
    let super::HarnessTask {
        prompt,
        role,
        active,
        explicit,
        image_paths,
        edit_scope,
        expected_codegraph_binding,
    } = task;
    config.validate()?;
    let _task = store.lease_session(&session.id)?;
    if session.interrupted {
        return Err(AgentError::new(
            "session has an interrupted operation; inspect its events and start a new session without replaying tools",
        ));
    }
    let selected = config.resolve(role, explicit, active)?;
    // Refresh ordinary sessions at the task boundary. Orchestrated sessions
    // retain their baseline binding until the coordinator advances it.
    let graph_snapshot = if expected_codegraph_binding.is_none() {
        ensure_persistent_codegraph(store.root())
            .ok()
            .or_else(|| read_persistent_codegraph(store.root()).ok())
    } else {
        read_persistent_codegraph(store.root()).ok()
    };
    // Capture the persisted graph once; receipts retain this turn-start binding.
    let turn_codegraph_binding = graph_snapshot.as_ref().and_then(|snapshot| {
        Some(CodeGraphBinding {
            generation: snapshot.generation.clone()?,
            revision: snapshot.manifest.revision,
            root: snapshot.manifest.root.clone(),
            mode: snapshot.manifest.mode.clone(),
        })
    });
    if let Some(expected) = expected_codegraph_binding.as_ref()
        && turn_codegraph_binding.as_ref() != Some(expected)
    {
        return Err(AgentError::new(
            "persisted CodeGraphBinding does not match the expected orchestration binding",
        ));
    }
    let semantic = if matches!(role, HarnessRole::Execute | HarnessRole::Plan)
        && config.roles.contains_key(&HarnessRole::Codegraph)
    {
        match graph_snapshot.as_ref() {
            Some(snapshot) => {
                enrich_codegraph(
                    store.root(),
                    &config.resolve(HarnessRole::Codegraph, None, active)?,
                    &snapshot,
                )
                .await
            }
            None => SemanticEnrichment {
                status: SemanticStatus::Unavailable,
                provider: None,
                model: None,
                context: "[]".into(),
                usage: None,
                cache_warning: None,
            },
        }
    } else {
        SemanticEnrichment {
            status: SemanticStatus::Disabled,
            provider: None,
            model: None,
            context: "[]".into(),
            usage: None,
            cache_warning: None,
        }
    };
    let image_selection = config.resolve(HarnessRole::ImageGeneration, None, active)?;
    let historical_images = role == HarnessRole::Execute && session.turns.iter().skip(session.context_start).any(|turn| {
        turn.message.as_ref().is_some_and(|message| matches!(&message.content, AgentMessageContent::Parts(parts) if parts.iter().any(|part| matches!(part, crate::AgentMessagePart::ImageUrl { .. }))))
    });
    config.require_capabilities(
        &selected,
        role,
        !image_paths.is_empty() || historical_images,
    )?;
    let mut tools =
        HarnessTools::with_scope(store.root(), &session.id, config.clone(), edit_scope)?;
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
    let task_id = super::identifier();
    let baseline = super::request::task_baseline(store.root());
    emit_persist(
        store,
        session,
        persist,
        host,
        json!({"event":"task_started","taskId":task_id,"role":role,"baseline":baseline}),
    )?;
    if matches!(role, HarnessRole::Codegraph | HarnessRole::Research)
        && let Some(snapshot) = graph_snapshot.as_ref()
    {
        let indexed_files = snapshot
            .graph
            .nodes
            .iter()
            .filter(|node| node.path.as_deref().is_some_and(|path| path != "."))
            .count();
        let unknown_language_files = snapshot
            .graph
            .nodes
            .iter()
            .filter(|node| {
                node.path.as_deref().is_some_and(|path| path != ".")
                    && node.language == "unknown"
            })
            .count();
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({
                "event":"codegraph_coverage",
                "mode":snapshot.manifest.mode,
                "indexed_files":indexed_files,
                "unknown_language_files":unknown_language_files,
                "relationships":"deterministic_edges_only",
                "semantic_enrichment":"advisory_and_optional",
            }),
        )?;
    }
    if expected_codegraph_binding.is_none() {
        match super::refresh_capability_map(store.root()) {
            Ok(paths) if !paths.is_empty() => emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"capability_map_stale","paths":paths}),
            )?,
            Ok(_) => {}
            Err(error) => emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"capability_map_warning","message":error.to_string()}),
            )?,
        }
        if !store.root().join(".agents/capabilities/index.md").is_file() {
            match super::bootstrap_capability_map(store.root()) {
                Ok(update) if !update.changed.is_empty() => emit_persist(
                    store,
                    session,
                    persist,
                    host,
                    json!({"event":"capability_map_bootstrapped","paths":update.changed}),
                )?,
                Ok(_) => {}
                Err(error) => emit_persist(
                    store,
                    session,
                    persist,
                    host,
                    json!({"event":"capability_map_warning","message":error.to_string()}),
                )?,
            }
        }
    }
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
            generated_image_for_continuation = None;
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
                    active,
                    explicit,
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
        let mut results = Vec::new();
        let mut screenshot_message = None;
        let mut approval_required = false;
        let mut graph_dirty = false;
        for call in &calls {
            if call.name != "ask_user" && call.name != "question" {
                continue;
            }
            let value = call
                .arguments
                .get("question")
                .cloned()
                .unwrap_or_else(|| call.arguments.clone());
            let question: crate::ClarificationQuestion = serde_json::from_value(value)
                .map_err(|_| AgentError::new("malformed clarification question"))?;
            question.validate()?;
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"clarification_required","call_id":call.id,"question":question}),
            )?;
            session.interrupted = true;
            if persist {
                store.save_session(session)?;
            }
            let Some(answer) = host.ask_clarification(&question).await? else {
                emit_persist(
                    store,
                    session,
                    persist,
                    host,
                    json!({"event":"task_state","status":"blocked","reason":"clarification_required"}),
                )?;
                return Ok(HarnessOutcome::ClarificationRequired);
            };
            if answer.trim().is_empty() || answer.len() > 1024 {
                return Err(AgentError::new("clarification answer exceeds limits"));
            }
            let result = ToolResult {
                id: call.id.clone(),
                name: call.name.clone(),
                failed: false,
                output: json!({"answer":tools.redactor.text(&answer)}),
            };
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"tool_result","result":result}),
            )?;
            results.push(result);
        }
        let calls = calls
            .into_iter()
            .filter(|call| call.name != "ask_user" && call.name != "question")
            .collect::<Vec<_>>();
        let mut canceled = false;
        let mut call_index = 0;
        while call_index < calls.len() {
            let batch_start = call_index;
            let mut batch_end = batch_start + 1;
            while batch_end < calls.len()
                && is_parallel_read(&calls[batch_end])
                && is_parallel_read(&calls[batch_end - 1])
            {
                batch_end += 1;
            }
            let parallel_outputs = if batch_end - batch_start > 1
                && !canceled
                && !approval_required
                && !exhausted(config, &usage, 0, started)
            {
                Some(execute_read_batch(&tools, &calls[batch_start..batch_end]))
            } else {
                None
            };
            for index in batch_start..batch_end {
                let call = calls[index].clone();
                let output = if let Some(outputs) = &parallel_outputs {
                    outputs[index - batch_start].clone()
                } else if canceled || approval_required {
                    Ok(
                        json!({"status":"not_executed","reason":if canceled {"task_canceled"} else {"approval_required"}}),
                    )
                } else if exhausted(config, &usage, 0, started) {
                    Ok(json!({"status":"not_executed","reason":"budget_exhausted"}))
                } else {
                    match tools.prepare(&call, role) {
                        Ok(Some(approval)) => {
                            emit_persist(
                                store,
                                session,
                                persist,
                                host,
                                json!({"event":"approval_required","approval":approval}),
                            )?;
                            match host.approve(&approval).await? {
                                Some(true)
                                    if exhausted(config, &usage, 0, started) =>
                                {
                                    tools.reject(approval)?;
                                    Ok(json!({"status":"not_executed","reason":"budget_exhausted"}))
                                }
                                Some(true) => {
                                    let receipt_start = operation_receipt(
                                        &call,
                                        "started",
                                        turn_codegraph_binding.as_ref(),
                                        &approval,
                                    );
                                    session.interrupted = true;
                                    emit_persist(
                                        store,
                                        session,
                                        persist,
                                        host,
                                        json!({"event":"operation_started","call_id":call.id,"approval_id":approval.id,"receipt":receipt_start}),
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
                                        if call.name == "generate_image" {
                                            let generated = host
                                                .generate_image(&image_selection, &approval)
                                                .await;
                                            match generated {
                                                Ok(image) => {
                                                    let output = tools.apply_generated_image(
                                                        approval,
                                                        image.clone(),
                                                    );
                                                    if output.is_ok() {
                                                        generated_image_for_continuation =
                                                            Some(image);
                                                    }
                                                    output
                                                }
                                                Err(error) => {
                                                    tools.reject(approval)?;
                                                    Err(error)
                                                }
                                            }
                                        } else if call.name == "capture_web_screenshot" {
                                            let output = tools.capture_web_screenshot(approval);
                                            if let Ok(value) = &output {
                                                screenshot_message =
                                                    Some(tools.screenshot_image(value)?);
                                            }
                                            output
                                        } else {
                                            tools.apply_write(approval)
                                        }
                                    };
                                    emit_persist(
                                        store,
                                        session,
                                        persist,
                                        host,
                                        json!({"event":"operation_finished","call_id":call.id,"result":output.as_ref().ok(),"failed":output.is_err(),"receipt":finish_operation_receipt(store, &call, receipt_start, output.is_err())}),
                                    )?;
                                    if output.is_ok()
                                        && matches!(
                                            call.name.as_str(),
                                            "write_file"
                                                | "edit_file"
                                                | "write_asset"
                                                | "generate_image"
                                        )
                                    {
                                        graph_dirty = true;
                                        if let Some(path) = call.arguments["path"]
                                            .as_str()
                                            .or_else(|| call.arguments["destination"].as_str())
                                        {
                                            match super::sync_capability_map(
                                                store.root(),
                                                &[path.to_string()],
                                            ) {
                                                Ok(update) if !update.changed.is_empty() => {
                                                    emit_persist(
                                                        store,
                                                        session,
                                                        persist,
                                                        host,
                                                        json!({"event":"capability_map_updated","paths":update.changed}),
                                                    )?
                                                }
                                                Ok(_) => {}
                                                Err(error) => emit_persist(
                                                    store,
                                                    session,
                                                    persist,
                                                    host,
                                                    json!({"event":"capability_map_warning","message":tools.redactor.text(&error.to_string())}),
                                                )?,
                                            }
                                        }
                                    }
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
                        Ok(None) if is_parallel_read(&call) => tools.execute_read(&call),
                        Ok(None) => tools.execute_skill(&call),
                        Err(error) => Err(error),
                    }
                };
                let failed = output.is_err();
                let mut output = output.unwrap_or_else(
                    |error| json!({"error":tools.redactor.text(&error.to_string())}),
                );
                canceled |= output["canceled"] == true;
                tools.redactor.value(&mut output);
                output = super::privacy::bounded_value(output, config.max_output_bytes);
                let result = ToolResult {
                    id: call.id,
                    name: call.name,
                    failed,
                    output,
                };
                emit_persist(
                    store,
                    session,
                    persist,
                    host,
                    json!({"event":"tool_result","result":result}),
                )?;
                results.push(result);
            }
            call_index = batch_end;
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

fn operation_receipt(
    call: &super::ToolCall,
    status: &str,
    binding: Option<&CodeGraphBinding>,
    approval: &Approval,
) -> Value {
    let mut receipt = json!({
        "operation": call.name,
        "status": status,
        "codegraphBinding": binding,
    });
    let path = match call.name.as_str() {
        "write_file" | "edit_file" | "write_asset" => call.arguments["path"].as_str(),
        "generate_image" => call.arguments["destination"].as_str(),
        _ => None,
    };
    if let Some(path) = path {
        // Start receipts record the approved base; afterFingerprint is unknown
        // until the operation has completed.
        let before = approval.details["before_sha256"].clone();
        receipt["path"] = json!(path);
        receipt["beforeFingerprint"] = if before.is_null() {
            Value::Null
        } else {
            before
        };
        receipt["afterFingerprint"] = Value::Null;
    }
    receipt
}

fn finish_operation_receipt(
    store: &HarnessStore,
    call: &super::ToolCall,
    mut receipt: Value,
    failed: bool,
) -> Value {
    receipt["status"] = json!(if failed { "failed" } else { "succeeded" });
    let path = match call.name.as_str() {
        "write_file" | "edit_file" | "write_asset" => call.arguments["path"].as_str(),
        "generate_image" => call.arguments["destination"].as_str(),
        _ => None,
    };
    if let Some(path) = path {
        let fingerprint = HarnessTools::checked_path(store.root(), path)
            .ok()
            .and_then(|resolved| fs::read(resolved).ok())
            .map(|bytes| super::digest(&bytes));
        receipt["afterFingerprint"] = fingerprint.map_or(Value::Null, Value::String);
    }
    receipt
}

fn is_parallel_read(call: &super::ToolCall) -> bool {
    matches!(call.name.as_str(), "read_file" | "list_files" | "search")
}

fn execute_read_batch(tools: &HarnessTools, calls: &[super::ToolCall]) -> Vec<AgentResult<Value>> {
    std::thread::scope(|scope| {
        let workers = calls
            .iter()
            .map(|call| scope.spawn(|| tools.execute_parallel_read(call)))
            .collect::<Vec<_>>();
        workers
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .unwrap_or_else(|_| Err(AgentError::new("read tool worker panicked")))
            })
            .collect()
    })
}

fn exhausted(
    config: &HarnessConfig,
    usage: &AgentUsageTotals,
    estimated: u64,
    started: Instant,
) -> bool {
    started.elapsed().as_secs() >= config.duration_seconds
        || estimated >= config.token_budget
        || config
            .cost_budget_usd
            .is_some_and(|cost| usage.cost_usd >= cost || usage.incomplete_cost)
}

fn emit(
    store: &HarnessStore,
    session: &mut HarnessSession,
    host: &mut impl HarnessHost,
    event: Value,
) -> AgentResult<()> {
    emit_persist(store, session, true, host, event)
}

fn emit_persist(
    store: &HarnessStore,
    session: &mut HarnessSession,
    persist: bool,
    host: &mut impl HarnessHost,
    event: Value,
) -> AgentResult<()> {
    session.events.push(event.clone());
    if persist {
        store.save_session(session)?;
    }
    let mut public_event = event.clone();
    if public_event["event"] == "task_started"
        && let Some(object) = public_event.as_object_mut()
    {
        object.remove("baseline");
    }
    if matches!(
        public_event["event"].as_str(),
        Some("task_started" | "capability_map_bootstrapped")
    ) {
        return Ok(());
    }
    host.event(&public_event)
}

include!("engine/compaction.rs");
