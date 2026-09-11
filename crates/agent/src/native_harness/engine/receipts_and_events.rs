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
    matches!(
        call.name.as_str(),
        "read_file" | "list_files" | "search" | "convert_svg"
    )
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

