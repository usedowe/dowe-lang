use super::clean_runner_context::record_codegraph_context;
use super::{
    HarnessConfig, HarnessHost, HarnessOutcome, HarnessRole, HarnessSession, HarnessStore,
    HarnessTask, HarnessTools, HarnessTurn, ToolResult,
};
use crate::{
    AgentError, AgentPrepareOptions, AgentRequestType, AgentResult, prepare_agent_request,
};
use serde_json::json;

pub async fn run_clean_read_task(
    store: &HarnessStore,
    session: &mut HarnessSession,
    task: HarnessTask<'_>,
    host: &mut impl HarnessHost,
) -> AgentResult<HarnessOutcome> {
    ensure_capability_index(store)?;
    if store.sessions()?.len() == 1
        && !session
            .events
            .iter()
            .any(|event| event["event"] == "capability_map_stale")
    {
        let event = json!({
            "event": "capability_map_stale",
            "session": session.id,
            "reason": "capability evidence is refreshed by the clean harness"
        });
        session.events.push(event.clone());
        host.event(&event)?;
    }
    let strict_read_only = task.prompt.starts_with("/research ")
        || task.prompt.starts_with("/plan ")
        || task.prompt.starts_with("/review ");
    let request_prompt = task
        .prompt
        .strip_prefix("/research ")
        .or_else(|| task.prompt.strip_prefix("/plan "))
        .or_else(|| task.prompt.strip_prefix("/review "))
        .unwrap_or(task.prompt);
    let request_type = match task.role {
        HarnessRole::Plan => AgentRequestType::SpecPlan,
        HarnessRole::Research | HarnessRole::Review => AgentRequestType::Conversation,
        _ => {
            return Err(AgentError::new(
                "clean read runner only accepts plan or research roles",
            ));
        }
    };
    if let Some(expected) = task.expected_codegraph_binding.as_ref() {
        let snapshot = dowe_codegraph::clean::read_persistent_clean_codegraph(store.root())
            .map_err(|error| {
                AgentError::new(format!(
                    "persisted CodeGraphBinding cannot be read: {error}"
                ))
            })?;
        let actual = dowe_codegraph::clean::clean_binding(&snapshot);
        if &actual != expected {
            return Err(AgentError::new(
                "persisted CodeGraphBinding does not match the requested child turn",
            ));
        }
        if matches!(
            snapshot.freshness,
            dowe_codegraph::GraphFreshness::Stale | dowe_codegraph::GraphFreshness::Error
        ) {
            return Err(AgentError::new(
                "persisted CodeGraphBinding is stale; refresh CodeGraph before the read turn",
            ));
        }
    }
    if task.edit_scope.is_some() {
        return Err(AgentError::new(
            "clean read runner cannot receive mutation scopes",
        ));
    }
    let selection = task.explicit.unwrap_or(task.active);
    let mut prepared = prepare_agent_request(
        store.root(),
        request_prompt,
        AgentPrepareOptions {
            provider: Some(selection.provider.clone()),
            request_type: Some(request_type),
            model: Some(selection.model.clone()),
            thinking_level: selection.thinking,
            image_paths: task.image_paths.to_vec(),
            stream: false,
        },
    )?;
    prepared.request.tools = crate::read_only_agent_tool_definitions();
    if task.role == HarnessRole::Research && !strict_read_only {
        prepared
            .request
            .tools
            .extend(crate::conversation_compatibility_tool_definitions());
    }
    prepared
        .request
        .extra
        .insert("session_id".into(), json!(session.id.clone()));
    let prior_turns: Vec<super::HarnessTurn> = if session.turns.is_empty() {
        Vec::new()
    } else {
        let scope = format!("{}/{}", selection.provider, selection.model);
        let (bounded, truncated) =
            super::continuation::request_turns_bounded(&session.turns, &scope, 32 * 1024, true)?;
        if truncated {
            session.events.push(
                json!({"event":"clean_context_truncated","session":session.id,"role":"read"}),
            );
        }
        serde_json::from_value(bounded)?
    };
    if !prior_turns.is_empty() {
        let current = prepared.request.messages.pop();
        let mut messages = prior_turns
            .into_iter()
            .filter_map(|turn| turn.message)
            .collect::<Vec<_>>();
        if let Some(current) = current {
            messages.push(current);
        }
        prepared.request.messages = messages;
    }
    host.event(&json!({"event":"request_prepared","session":session.id,"role":if task.role == HarnessRole::Plan {"plan"} else {"research"},"requestType":prepared.request.request_type,"model":prepared.request.model.as_str()}))?;
    record_codegraph_context(host, session, prepared.context.codegraph.as_ref())?;
    session.events.push(json!({"event":"clean_workflow_started","session":session.id,"intent":if task.role == HarnessRole::Plan {"plan"} else {"ask"},"mutation":false,"context":"selective"}));
    if let Some(message) = prepared.request.messages.last().cloned() {
        session.turns.push(super::HarnessTurn {
            message: Some(message),
            ..Default::default()
        });
    }
    let read_tools = HarnessTools::new(store.root(), &session.id, HarnessConfig::default())?;
    let mut history = Vec::<HarnessTurn>::new();
    for round in 0..8 {
        if round > 0 {
            prepared
                .request
                .extra
                .insert("dowe_harness_turns".into(), serde_json::to_value(&history)?);
        }
        let response = match host.send(&prepared.request).await {
            Ok(response) => response,
            Err(error) => {
                if round == 0 {
                    let _ = session.turns.pop();
                }
                for event in host.take_request_events() {
                    host.event(&event)?;
                }
                host.event(&json!({"event":"error","message":error.to_string(),"requestType":prepared.request.request_type,"model":prepared.request.model}))?;
                return Err(error);
            }
        };
        for event in host.take_request_events() {
            host.event(&event)?;
        }
        if let Ok(text) = crate::agent_response_text(&response.payload) {
            let event = json!({"event":"response_received","requestId":response.request_id,"requestType":response.request_type,"model":response.model,"role":"execute","text":text,"payload":{"output_text":text}});
            session.events.push(event.clone());
            host.event(&event)?;
            host.event(&json!({"event":"clean_workflow_completed","session":session.id,"requestType":response.request_type,"model":response.model,"mutation":false}))?;
        }
        let turn = super::response_turn(&response.payload)?;
        if turn.calls.is_empty() {
            session.set_initial_prompt_metadata(request_prompt);
            session.turns.push(turn);
            session.events.push(json!({"event":"clean_workflow_completed","requestId":response.request_id,"requestType":response.request_type,"model":response.model,"mutation":false}));
            session.interrupted = false;
            store.save_session(session)?;
            return Ok(HarnessOutcome::Completed);
        }
        session.turns.push(turn.clone());
        history.push(turn.clone());
        let mut results = Vec::new();
        for call in turn.calls {
            let output = read_tools.execute_read(&call)?;
            host.event(&json!({"event":"tool_result","result":{"id":call.id,"name":call.name,"failed":false,"output":output.clone()}}))?;
            results.push(ToolResult {
                id: call.id,
                name: call.name,
                failed: false,
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
    Err(AgentError::new(
        "read-only workflow exceeded its tool-round budget",
    ))
}

fn ensure_capability_index(store: &HarnessStore) -> AgentResult<()> {
    let directory = store.root().join(".agents/capabilities");
    std::fs::create_dir_all(&directory)?;
    let path = directory.join("index.md");
    if !path.exists() {
        std::fs::write(
            path,
            "# Capability index\n\nGenerated by the clean harness. Capability declarations are advisory metadata; execution still requires the configured permission and approval gates.\n",
        )?;
    }
    Ok(())
}
