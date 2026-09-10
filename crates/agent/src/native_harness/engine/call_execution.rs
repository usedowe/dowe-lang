struct HarnessCallResults {
    results: Vec<ToolResult>,
    screenshot_message: Option<AgentMessage>,
    approval_required: bool,
    canceled: bool,
    graph_dirty: bool,
    generated_image_for_continuation: Option<GeneratedImage>,
    outcome: Option<HarnessOutcome>,
}

async fn execute_harness_calls(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    role: HarnessRole,
    tools: &mut HarnessTools,
    calls: Vec<super::ToolCall>,
    host: &mut impl HarnessHost,
    started: Instant,
    usage: &AgentUsageTotals,
    persist: bool,
    turn_codegraph_binding: Option<&CodeGraphBinding>,
    image_selection: &ModelSelection,
) -> AgentResult<HarnessCallResults> {
    let mut generated_image_for_continuation = None;
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
                return Ok(HarnessCallResults {
                    results,
                    screenshot_message,
                    approval_required,
                    canceled: false,
                    graph_dirty: false,
                    generated_image_for_continuation,
                    outcome: Some(HarnessOutcome::ClarificationRequired),
                });
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
                                        turn_codegraph_binding,
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

    Ok(HarnessCallResults {
        results,
        screenshot_message,
        approval_required,
        canceled,
        graph_dirty,
        generated_image_for_continuation,
        outcome: None,
    })
}
