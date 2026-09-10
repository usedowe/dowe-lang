use dowe_agent::native_harness::HarnessStore;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

const MAX_LINE_BYTES: usize = 64 * 1024;

/// Run the local, one-shot inspection protocol. The process owns no state beyond
/// the request currently being handled and never executes a harness operation.
pub(super) fn run_agent_rpc_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if !args.is_empty() { return Err("Usage: dowe agent rpc".into()); }
    let root = std::env::current_dir()?;
    let store = HarnessStore::from_default_path(root)?;
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let mut line = String::new();
    loop {
        line.clear();
        let read = stdin.lock().read_line(&mut line)?;
        if read == 0 { break; }
        if read > MAX_LINE_BYTES || !line.ends_with('\n') {
            write_response(&mut stdout, Value::Null, Err(("invalid_framing", "request must be one LF-delimited JSON object")))?;
            continue;
        }
        let mut record = line.trim_end_matches('\n');
        if record.ends_with('\r') { record = &record[..record.len() - 1]; }
        let response = handle_inspection_rpc(&store, record);
        let id = serde_json::from_str::<Value>(record).ok().and_then(|v| v.get("id").cloned()).unwrap_or(Value::Null);
        write_response(&mut stdout, id, response)?;
    }
    Ok(())
}

fn write_response<W: Write>(out: &mut W, id: Value, result: Result<Value, (&str, &str)>) -> io::Result<()> {
    let response = match result {
        Ok(result) => json!({"id": id, "ok": true, "result": result}),
        Err((code, message)) => json!({"id": id, "ok": false, "error":{"code":code,"message":message}}),
    };
    writeln!(out, "{}", response)
}

/// Handle one already framed request. Kept small and pure at the protocol edge so
/// integration tests can verify correlation and mutation rejection without a daemon.
pub(crate) fn handle_inspection_rpc(store: &HarnessStore, line: &str) -> Result<Value, (&'static str, &'static str)> {
    let request: Value = serde_json::from_str(line).map_err(|_| ("malformed_json", "request must be valid JSON"))?;
    let object = request.as_object().ok_or(("invalid_request", "request must be a JSON object"))?;
    let command = object.get("command").and_then(Value::as_str).ok_or(("invalid_request", "command is required"))?;
    let session = object.get("sessionId").and_then(Value::as_str).ok_or(("invalid_request", "sessionId is required"))?;
    let since = match object.get("since") {
        None => 0,
        Some(Value::Number(value)) => value.as_u64().ok_or(("invalid_request", "since must be a non-negative integer"))?,
        _ => return Err(("invalid_request", "since must be a non-negative integer")),
    };
    let limits = dowe_agent::native_harness::SessionEventLimits {
        max_events: object.get("maxEvents").and_then(Value::as_u64).unwrap_or(dowe_agent::native_harness::MAX_EVENTS as u64) as usize,
        max_bytes: object.get("maxBytes").and_then(Value::as_u64).unwrap_or(dowe_agent::native_harness::MAX_PAGE_BYTES as u64) as usize,
    };
    match command {
        "get_session" => serde_json::to_value(store.inspect_session_view(session).map_err(|_| ("inspection_failed", "session inspection failed"))?).map_err(|_| ("encoding_failed", "inspection response could not be encoded")),
        "get_tasks" => serde_json::to_value(store.inspect_tasks(session).map_err(|_| ("inspection_failed", "task inspection failed"))?).map_err(|_| ("encoding_failed", "inspection response could not be encoded")),
        "get_governance" => serde_json::to_value(store.inspect_governance(session).map_err(|_| ("inspection_failed", "governance inspection failed"))?).map_err(|_| ("encoding_failed", "inspection response could not be encoded")),
        "subscribe" | "poll" => serde_json::to_value(store.poll_events(session, since, limits).map_err(|_| ("inspection_failed", "event inspection failed"))?).map_err(|_| ("encoding_failed", "inspection response could not be encoded")),
        "approve" | "acknowledge" | "recover" | "execute" | "deliver" | "request_delivery" | "mutate" => Err(("mutation_rejected", "RPC is read-only; command was not executed")),
        _ => Err(("unknown_command", "unknown inspection command")),
    }
}

