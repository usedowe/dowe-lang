async fn finish_no_tool_round(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    strict_ui: bool,
    tools: &mut HarnessTools,
    host: &mut impl HarnessHost,
    persist: bool,
    state: &mut HarnessTurnState,
) -> AgentResult<Option<HarnessOutcome>> {
    const MAX_VISUAL_REPAIRS: u8 = 3;
    if state.visual_failed_seen {
        if state.visual_repair_attempts < MAX_VISUAL_REPAIRS {
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
            json!({"event":"validation_failed","kind":"visual_comparison","reason":"visual_comparison_failed","validationMetrics":validation_metrics(state)}),
        )?;
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"task_state","status":"failed","reason":"visual_comparison_failed","validationMetrics":validation_metrics(state),"visualQA":visual_qa_value(state)}),
        )?;
        return Ok(Some(HarnessOutcome::ValidationFailed));
    }
    if state.validation_failures > 0 && !state.validation_attempted && !state.mutation_seen {
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"validation_failed","kind":"dowe_project","reason":"dowe_validation_failed","validation":state.last_validation,"validationMetrics":validation_metrics(state)}),
        )?;
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"task_state","status":"failed","reason":"dowe_validation_failed","validation":state.last_validation,"validationMetrics":validation_metrics(state),"visualQA":visual_qa_value(state)}),
        )?;
        return Ok(Some(HarnessOutcome::ValidationFailed));
    }
    if crate::is_dowe_project(store.root()) && state.mutation_seen && !state.validation_attempted {
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"validation_started","scope":"project","automatic":true}),
        )?;
        let validation = validate_dowe_project(store, host, "project", strict_ui).await?;
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
            json!({"event":"validation_finished","automatic":true,"attempt":state.validation_attempts.saturating_add(1),"result":validation_for_model.clone()}),
        )?;
        state.record_validation(validation_for_model.clone());
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
            state.mutation_seen = false;
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
                json!({"event":"validation_failed","kind":"dowe_project","reason":"dowe_validation_failed","validation":state.last_validation,"validationMetrics":validation_metrics(state)}),
            )?;
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"task_state","status":"failed","reason":"dowe_validation_failed","validation":state.last_validation,"validationMetrics":validation_metrics(state),"visualQA":visual_qa_value(state)}),
            )?;
            return Ok(Some(HarnessOutcome::ValidationFailed));
        }
        state.validation_attempted = true;
    }
    if state.visual_qa_required && state.mutation_seen && !state.visual_qa_attempted {
        if state.visual_gate_prompts < 1 {
            state.visual_gate_prompts += 1;
            session.turns.push(HarnessTurn {
                message: Some(AgentMessage {
                    role: "user".into(),
                    content: AgentMessageContent::Text(
                        "Attached reference visual QA is still required. Call capture_web_screenshot after the final mutation at the reference viewport, inspect the screenshot and report, and repair any significant difference. If the browser, page, or matching PNG is unavailable, make the capture attempt and report visualQA:not_run; do not claim visual parity.".into(),
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
            json!({"event":"validation_failed","kind":"visual_qa","reason":"visual_qa_required","validationMetrics":validation_metrics(state),"visualQA":visual_qa_value(state)}),
        )?;
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"task_state","status":"failed","reason":"visual_qa_required","validationMetrics":validation_metrics(state),"visualQA":visual_qa_value(state)}),
        )?;
        return Ok(Some(HarnessOutcome::ValidationFailed));
    }
    if state.visual_qa_required
        && state.mutation_seen
        && state.visual_qa_attempted
        && state.visual_status.as_deref() == Some("not_run")
    {
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"validation_failed","kind":"visual_qa","reason":"visual_qa_not_run","visualQA":visual_qa_value(state)}),
        )?;
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"task_state","status":"failed","reason":"visual_qa_not_run","validationMetrics":validation_metrics(state),"visualQA":visual_qa_value(state)}),
        )?;
        return Ok(Some(HarnessOutcome::ValidationFailed));
    }
    emit_persist(
        store,
        session,
        persist,
        host,
        json!({"event":"task_state","status":"done","validation":state.last_validation,"validationMetrics":validation_metrics(state),"visualQA":visual_qa_value(state)}),
    )?;
    Ok(Some(HarnessOutcome::Completed))
}

fn visual_qa_value(state: &HarnessTurnState) -> Option<Value> {
    state.visual_qa_required.then(|| {
        json!({
            "status": state.visual_status.as_deref().unwrap_or("not_run"),
            "attempted": state.visual_qa_attempted,
        })
    })
}

fn validation_metrics(state: &HarnessTurnState) -> Value {
    let status = |value: &Option<Value>| {
        value
            .as_ref()
            .and_then(|validation| validation["status"].as_str())
            .map(str::to_owned)
    };
    json!({
        "attempts": state.validation_attempts,
        "firstStatus": status(&state.first_validation),
        "finalStatus": status(&state.last_validation),
        "first": state.first_validation,
    })
}
