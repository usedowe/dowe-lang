async fn run_harness_round(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    role: HarnessRole,
    active: &ModelSelection,
    explicit: Option<&ModelSelection>,
    selected: &ModelSelection,
    prompt: &str,
    semantic: &SemanticEnrichment,
    image_selection: &ModelSelection,
    tools: &mut HarnessTools,
    turn_codegraph_binding: Option<&CodeGraphBinding>,
    expected_codegraph_binding: Option<&CodeGraphBinding>,
    host: &mut impl HarnessHost,
    round: usize,
    persist: bool,
    state: &mut HarnessTurnState,
) -> AgentResult<Option<HarnessOutcome>> {
    if exhausted(config, &state.usage, 0, state.started) {
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"budget_exhausted","session":session.id}),
        )?;
        return Ok(Some(HarnessOutcome::BudgetExhausted));
    }
    let mut request = build_request(
        store,
        session,
        config,
        role,
        selected,
        prompt,
        Some(semantic),
    )?;
    if let Some(image) = &state.generated_image_for_continuation {
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
            Duration::from_secs(config.duration_seconds).saturating_sub(state.started.elapsed()),
            compact(
                store,
                session,
                &compact_config,
                active,
                explicit,
                host,
                &mut state.usage,
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
            selected,
            prompt,
            Some(semantic),
        )?;
        estimate = super::task::estimate_request(&request)?;
        if estimate + 6144 >= limit {
            return Err(AgentError::new(
                "compacted context still exceeds destination window; start a new session",
            ));
        }
    }
    if exhausted(config, &state.usage, 0, state.started)
        || estimate.saturating_add(4096) > config.token_budget
    {
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"budget_exhausted","reason":"estimated_next_request","estimated_input_tokens":estimate}),
        )?;
        return Ok(Some(HarnessOutcome::BudgetExhausted));
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
            Duration::from_secs(config.duration_seconds).saturating_sub(state.started.elapsed());
        let result =
            tokio::time::timeout(remaining, transport::send(host, &request, store, session)).await;
        match result {
            Ok(Ok(response)) => break response,
            Ok(Err(error))
                if !state.has_tool_result
                    && retry < transport::MAX_TRANSIENT_RETRIES
                    && let Some(classification) =
                        transport::transient_failure_classification(&error) =>
            {
                retry += 1;
                let remaining = Duration::from_secs(config.duration_seconds)
                    .saturating_sub(state.started.elapsed());
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
    state.usage = session.usage_since(state.usage_start);
    session.turns.push(turn);
    session.set_initial_prompt_metadata(prompt);
    session.interrupted = !calls.is_empty();
    if persist {
        store.save_session(session)?;
    }
    if calls.is_empty() {
        if state.visual_failed_seen {
            if state.visual_repair_attempts < 1 {
                state.visual_repair_attempts += 1;
                session.turns.push(HarnessTurn {
                    message: Some(AgentMessage {
                        role: "user".into(),
                        content: AgentMessageContent::Text(
                            "Native visual comparison found a significant difference from the attached reference. Inspect the captured screenshot and diff, refine the Dowe view, and capture it again before declaring the task complete.".into(),
                        ),
                    }),
                    ..Default::default()
                });
                state.has_tool_result = true;
                session.interrupted = false;
                if persist {
                    store.save_session(session)?;
                }
                return Ok(None);
            }
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"validation_failed","kind":"visual_comparison","reason":"visual_comparison_failed"}),
            )?;
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"task_state","status":"failed","reason":"visual_comparison_failed"}),
            )?;
            return Ok(Some(HarnessOutcome::ValidationFailed));
        }
        if state.validation_failures > 0 && !state.validation_attempted && !state.mutation_seen {
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"validation_failed","kind":"dowe_project","reason":"dowe_validation_failed","validation":state.last_validation}),
            )?;
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"task_state","status":"failed","reason":"dowe_validation_failed","validation":state.last_validation}),
            )?;
            return Ok(Some(HarnessOutcome::ValidationFailed));
        }
        if crate::is_dowe_project(store.root())
            && state.mutation_seen
            && !state.validation_attempted
        {
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"validation_started","scope":"project","automatic":true}),
            )?;
            let validation = validate_dowe_project(store, host, "project").await?;
            let failed = validation["status"] == "failed";
            let mut validation_for_model = validation.clone();
            tools.redactor.value(&mut validation_for_model);
            validation_for_model =
                super::privacy::bounded_value(validation_for_model, config.max_output_bytes);
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"validation_finished","automatic":true,"result":validation_for_model.clone()}),
            )?;
            state.last_validation = Some(validation_for_model.clone());
            let result = ToolResult {
                id: format!("automatic-validation-{}", super::identifier()),
                name: "validate_dowe_project".into(),
                failed,
                output: validation_for_model,
            };
            session.turns.push(HarnessTurn {
                results: vec![result],
                ..Default::default()
            });
            if failed {
                if state.validation_failures < 1 {
                    state.validation_failures += 1;
                    state.validation_attempted = false;
                    session.turns.push(HarnessTurn {
                        message: Some(AgentMessage {
                            role: "user".into(),
                            content: AgentMessageContent::Text(
                                "Automatic Dowe validation failed. Inspect the compiler diagnostics and quality findings above, repair the source, and validate again before declaring the task complete.".into(),
                            ),
                        }),
                        ..Default::default()
                    });
                    state.has_tool_result = true;
                    session.interrupted = false;
                    if persist {
                        store.save_session(session)?;
                    }
                    return Ok(None);
                }
                emit_persist(
                    store,
                    session,
                    persist,
                    host,
                    json!({"event":"validation_failed","kind":"dowe_project","reason":"dowe_validation_failed","validation":state.last_validation}),
                )?;
                emit_persist(
                    store,
                    session,
                    persist,
                    host,
                    json!({"event":"task_state","status":"failed","reason":"dowe_validation_failed","validation":state.last_validation}),
                )?;
                return Ok(Some(HarnessOutcome::ValidationFailed));
            }
        }
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"task_state","status":"done","validation":state.last_validation}),
        )?;
        return Ok(Some(HarnessOutcome::Completed));
    }
    let call_results = execute_harness_calls(
        store,
        session,
        config,
        role,
        tools,
        calls,
        host,
        state.started,
        &state.usage,
        persist,
        turn_codegraph_binding,
        image_selection,
    )
    .await?;
    let HarnessCallResults {
        results,
        screenshot_message,
        approval_required,
        canceled,
        graph_dirty,
        generated_image_for_continuation: generated_image,
        validation_attempted: call_validation_attempted,
        validation_failed: call_validation_failed,
        visual_checked: call_visual_checked,
        visual_failed: call_visual_failed,
        outcome,
    } = call_results;
    state.generated_image_for_continuation = generated_image;
    state.mutation_seen |= graph_dirty;
    state.validation_attempted |= call_validation_attempted && !call_validation_failed;
    if call_validation_failed {
        state.validation_attempted = false;
        state.validation_failures = state.validation_failures.saturating_add(1);
    }
    if call_validation_attempted || call_validation_failed {
        state.last_validation = results
            .iter()
            .rev()
            .find(|result| result.name == "validate_dowe_project")
            .map(|result| result.output.clone());
    }
    if call_visual_checked {
        state.visual_failed_seen = call_visual_failed;
        if !call_visual_failed {
            state.visual_repair_attempts = 0;
        }
    }
    if let Some(outcome) = outcome {
        return Ok(Some(outcome));
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
    state.has_tool_result = true;
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
        return Ok(Some(HarnessOutcome::Canceled));
    }
    if approval_required {
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"task_state","status":"awaiting_approval"}),
        )?;
        return Ok(Some(HarnessOutcome::ApprovalRequired));
    }
    Ok(None)
}
