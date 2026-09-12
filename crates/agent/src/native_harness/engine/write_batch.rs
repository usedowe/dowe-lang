struct TextWriteBatchResults {
    results: Vec<ToolResult>,
    approval_required: bool,
    graph_dirty: bool,
}

const MAX_TEXT_WRITE_BATCH_OPERATIONS: usize = 8;

fn is_batchable_text_write(call: &super::ToolCall) -> bool {
    matches!(call.name.as_str(), "write_file" | "edit_file")
}

async fn execute_text_write_batch(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    role: HarnessRole,
    calls: &[super::ToolCall],
    tools: &mut HarnessTools,
    host: &mut impl HarnessHost,
    started: Instant,
    usage: &AgentUsageTotals,
    persist: bool,
    turn_codegraph_binding: Option<&CodeGraphBinding>,
) -> AgentResult<TextWriteBatchResults> {
    let approvals = match tools.prepare_text_write_batch(calls, role) {
        Ok(approvals) => approvals,
        Err(error) => {
            let results = calls
                .iter()
                .map(|call| {
                    text_write_result(
                        tools,
                        config,
                        call,
                        true,
                        json!({"error":error.to_string()}),
                    )
                })
                .collect::<Vec<_>>();
            for result in &results {
                emit_persist(
                    store,
                    session,
                    persist,
                    host,
                    json!({"event":"tool_result","result":result}),
                )?;
            }
            return Ok(TextWriteBatchResults {
                results,
                approval_required: false,
                graph_dirty: false,
            });
        }
    };
    let mut result_slots = vec![None; calls.len()];
    let mut changed_indices = Vec::new();
    let mut changed_approvals = Vec::new();
    for (index, approval) in approvals.into_iter().enumerate() {
        if approval.details["unchanged"] == true {
            tools.reject(approval)?;
            result_slots[index] = Some(text_write_result(
                tools,
                config,
                &calls[index],
                false,
                json!({
                    "status":"unchanged",
                    "path":calls[index].arguments["path"]
                }),
            ));
        } else {
            changed_indices.push(index);
            changed_approvals.push(approval);
        }
    }

    let mut approval_required = false;
    let mut graph_dirty = false;
    if !changed_approvals.is_empty() {
        let batch_id = format!("batch-{}", super::identifier());
        for (position, approval) in changed_approvals.iter().enumerate() {
            let approval_view = serde_json::to_value(approval)?;
            emit_persist(
                store,
                session,
                persist,
                host,
                json!({
                    "event":"approval_required",
                    "approval":approval_view,
                    "batchId":batch_id,
                    "batchIndex":position,
                    "batchSize":changed_approvals.len()
                }),
            )?;
        }
        let decision = host.approve_batch(&changed_approvals).await?;
        let approval_missing = decision.is_none();
        match decision {
            Some(true) if exhausted(config, usage, 0, started) => {
                for (index, approval) in changed_indices
                    .iter()
                    .copied()
                    .zip(changed_approvals.into_iter())
                {
                    tools.reject(approval)?;
                    result_slots[index] = Some(text_write_result(
                        tools,
                        config,
                        &calls[index],
                        false,
                        json!({"status":"not_executed","reason":"budget_exhausted"}),
                    ));
                }
            }
            Some(true) => {
                let approval_ids = changed_approvals
                    .iter()
                    .map(|approval| approval.id.clone())
                    .collect::<Vec<_>>();
                let receipts = changed_indices
                    .iter()
                    .zip(changed_approvals.iter())
                    .map(|(index, approval)| {
                        operation_receipt(
                            &calls[*index],
                            "started",
                            turn_codegraph_binding,
                            approval,
                        )
                    })
                    .collect::<Vec<_>>();
                for (position, index) in changed_indices.iter().copied().enumerate() {
                    emit_persist(
                        store,
                        session,
                        persist,
                        host,
                        json!({
                            "event":"operation_started",
                            "call_id":calls[index].id,
                            "approval_id":approval_ids[position],
                            "receipt":receipts[position]
                        }),
                    )?;
                }
                let applied = tools.apply_text_write_batch(changed_approvals);
                for (position, index) in changed_indices.iter().copied().enumerate() {
                    let output = applied.get(position).cloned().unwrap_or_else(|| {
                        Err(AgentError::new(
                            "text write batch returned an incomplete result",
                        ))
                    });
                    let failed = output.is_err();
                    let mut output_value =
                        output.unwrap_or_else(|error| json!({"error":error.to_string()}));
                    let receipt = finish_operation_receipt(
                        store,
                        &calls[index],
                        receipts[position].clone(),
                        failed,
                    );
                    emit_persist(
                        store,
                        session,
                        persist,
                        host,
                        json!({
                            "event":"operation_finished",
                            "call_id":calls[index].id,
                            "result":(!failed).then_some(&output_value),
                            "failed":failed,
                            "receipt":receipt
                        }),
                    )?;
                    if !failed {
                        graph_dirty = true;
                        sync_text_write_capability_map(
                            store,
                            session,
                            persist,
                            host,
                            calls[index].arguments["path"].as_str(),
                        )?;
                    }
                    result_slots[index] = Some(text_write_result(
                        tools,
                        config,
                        &calls[index],
                        failed,
                        std::mem::take(&mut output_value),
                    ));
                }
            }
            Some(false) | None => {
                approval_required = approval_missing;
                let reason = if approval_missing {
                    "approval_required"
                } else {
                    "user_rejected"
                };
                for (index, approval) in changed_indices
                    .iter()
                    .copied()
                    .zip(changed_approvals.into_iter())
                {
                    tools.reject(approval)?;
                    result_slots[index] = Some(text_write_result(
                        tools,
                        config,
                        &calls[index],
                        false,
                        json!({"status":"not_executed","reason":reason}),
                    ));
                }
            }
        }
    }

    let mut results = Vec::with_capacity(calls.len());
    for (index, result) in result_slots.into_iter().enumerate() {
        let result = result.unwrap_or_else(|| {
            text_write_result(
                tools,
                config,
                &calls[index],
                true,
                json!({"error":"text write batch did not produce a result"}),
            )
        });
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"tool_result","result":result}),
        )?;
        results.push(result);
    }
    Ok(TextWriteBatchResults {
        results,
        approval_required,
        graph_dirty,
    })
}

fn text_write_result(
    tools: &HarnessTools,
    config: &HarnessConfig,
    call: &super::ToolCall,
    failed: bool,
    mut output: Value,
) -> ToolResult {
    tools.redactor.value(&mut output);
    ToolResult {
        id: call.id.clone(),
        name: call.name.clone(),
        failed,
        output: super::privacy::bounded_value(output, config.max_output_bytes),
    }
}

fn sync_text_write_capability_map(
    store: &HarnessStore,
    session: &mut HarnessSession,
    persist: bool,
    host: &mut impl HarnessHost,
    path: Option<&str>,
) -> AgentResult<()> {
    let Some(path) = path else {
        return Ok(());
    };
    match super::sync_capability_map(store.root(), &[path.to_string()]) {
        Ok(update) if !update.changed.is_empty() => emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"capability_map_updated","paths":update.changed}),
        ),
        Ok(_) => Ok(()),
        Err(error) => emit_persist(
            store,
            session,
            persist,
            host,
            json!({"event":"capability_map_warning","message":error.to_string()}),
        ),
    }
}
