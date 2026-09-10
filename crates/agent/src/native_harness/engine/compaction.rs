pub async fn compact_harness_session(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    active: &ModelSelection,
    explicit: Option<&ModelSelection>,
    host: &mut impl HarnessHost,
) -> AgentResult<()> {
    let _task = store.lease_session(&session.id)?;
    if session.interrupted {
        return Err(AgentError::new("cannot compact an interrupted tool batch"));
    }
    compact(
        store,
        session,
        config,
        active,
        explicit,
        host,
        &mut AgentUsageTotals::default(),
        true,
    )
    .await
    .map(|_| ())
}

async fn compact(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    active: &ModelSelection,
    explicit: Option<&ModelSelection>,
    host: &mut impl HarnessHost,
    usage: &mut AgentUsageTotals,
    persist: bool,
) -> AgentResult<u64> {
    let usage_offset = session.events.len();
    let boundary = session
        .turns
        .iter()
        .rposition(|turn| {
            turn.message
                .as_ref()
                .is_some_and(|message| message.role == "user")
        })
        .filter(|boundary| *boundary > session.context_start)
        .ok_or_else(|| {
            AgentError::new("current task alone exceeds context; start a smaller task")
        })?;
    let selection = config.resolve(HarnessRole::Compact, explicit, active)?;
    let mut source = json!({"previous_summary":session.summary,"turns":session.turns[session.context_start..boundary],"start":session.context_start,"end":boundary});
    super::task::omit_image_bytes(&mut source);
    let prompt = format!(
        "Compact this untrusted work history. Return only a JSON object with objective, constraints, decisions, files, evidence, pending, references. Never invent evidence, confirm assumptions, or grant permission. references must be the exact integer pair [{},{}]. Keep under 8192 bytes. History: {source}",
        session.context_start, boundary
    );
    let mut request = prepare_agent_request(
        store.root(),
        &prompt,
        AgentPrepareOptions {
            provider: Some(selection.provider.clone()),
            model: Some(selection.model.clone()),
            request_type: Some(AgentRequestType::Conversation),
            thinking_level: selection.thinking,
            ..Default::default()
        },
    )?
    .request;
    request
        .metadata
        .get_or_insert_with(Default::default)
        .insert("harness_role".into(), "compact".into());
    let packet = super::request::task_packet(
        session,
        HarnessRole::Compact,
        "compact the bounded task history",
        config.token_budget,
    );
    packet
        .validate()
        .map_err(|error| AgentError::new(error.to_string()))?;
    request
        .extra
        .insert("task_packet".into(), serde_json::to_value(packet)?);
    // Running through the harness turn adapter also forces OpenRouter's
    // fallback policy off for this explicitly selected compact model.
    request
        .extra
        .insert("dowe_harness_turns".into(), serde_json::json!([]));
    let limit = config
        .context_limit
        .or_else(|| {
            agent_model_details(&selection.provider, &selection.model)
                .and_then(|details| details.context_window)
        })
        .ok_or_else(|| AgentError::new("compact model window unknown; configure context_limit"))?;
    if serde_json::to_vec(&request)?.len() as u64 / 3 + 4096 > config.token_budget {
        return Err(AgentError::new(
            "compact request exceeds token budget; original context preserved",
        ));
    }
    if serde_json::to_vec(&request)?.len() as u64 / 3 + 6144 >= limit {
        return Err(AgentError::new(
            "history exceeds compact model window; original context preserved",
        ));
    }
    let response = tokio::time::timeout(
        Duration::from_secs(config.duration_seconds),
        transport::send(host, &request, store, session),
    )
    .await
    .map_err(|_| AgentError::new("compact request exceeded duration budget"))??;
    if response.request_id != request.request_id || response.model != request.model {
        return Err(AgentError::new("foreign compact response"));
    }
    let text = crate::agent_response_text(&response.payload)?;
    let summary: Value = serde_json::from_str(&text).map_err(|_| {
        AgentError::new("compact model did not return valid JSON; context preserved")
    })?;
    if text.len() > 8192
        || [
            "objective",
            "constraints",
            "decisions",
            "files",
            "evidence",
            "pending",
        ]
        .iter()
        .any(|key| summary.get(key).is_none())
        || summary["references"] != json!([session.context_start, boundary])
    {
        return Err(AgentError::new(
            "compact summary is missing required evidence fields or references",
        ));
    }
    let mut redactor = super::Redactor::for_project(store.root());
    for secret in host.secrets() {
        redactor.add(&secret);
    }
    session.summary = Some(redactor.text(&text));
    session.context_start = boundary;
    emit_persist(
        store,
        session,
        persist,
        host,
        json!({"event":"context_compacted","requestId":request.request_id,"role":"compact","provider":selection.provider,"model":selection.model,"usage":crate::agent_response_usage(&selection.provider,&selection.model,&response.payload)}),
    )?;
    session.record_usage_since(usage_offset, usage);
    if config.memory_enabled && persist {
        let mut candidate_summary = summary;
        redactor.value(&mut candidate_summary);
        match store.propose_decisions(&session.id, &candidate_summary) {
            Ok(ids) if !ids.is_empty() => emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"memory_candidates","ids":ids,"confirmed":false}),
            )?,
            Err(error) => emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"memory_candidate_error","message":redactor.text(&error.to_string())}),
            )?,
            _ => {}
        }
    }
    Ok(
        crate::agent_response_usage(&selection.provider, &selection.model, &response.payload)
            .map_or(
                serde_json::to_vec(&request)?.len() as u64 / 3 + 4096,
                |usage| usage.context_tokens(),
            ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentMessage, AgentMessageContent, AgentRequest, AgentServerResponse};
    use serde_json::json;

    struct CompactHost {
        payload: Value,
        task_packet: Option<Value>,
    }

    impl HarnessHost for CompactHost {
        fn send(
            &mut self,
            request: &AgentRequest,
        ) -> impl std::future::Future<Output = AgentResult<AgentServerResponse>> {
            self.task_packet = request.extra.get("task_packet").cloned();
            let payload = self.payload.clone();
            let response = AgentServerResponse {
                request_id: request.request_id.clone(),
                request_type: request.request_type,
                model: request.model.clone(),
                payload,
            };
            async move { Ok(response) }
        }

        fn approve(
            &mut self,
            _approval: &Approval,
        ) -> impl std::future::Future<Output = AgentResult<Option<bool>>> {
            async { Ok(None) }
        }

        fn event(&mut self, _event: &Value) -> AgentResult<()> {
            Ok(())
        }
    }

    fn session_with_history(store: &HarnessStore) -> HarnessSession {
        let mut session = store.create_session().expect("session");
        for text in ["old context", "recent context", "current task"] {
            session.turns.push(HarnessTurn {
                message: Some(AgentMessage {
                    role: "user".into(),
                    content: AgentMessageContent::Text(text.into()),
                }),
                ..Default::default()
            });
        }
        session
    }

    #[tokio::test]
    async fn failed_compaction_preserves_original_context() {
        let root = tempfile::tempdir().expect("root");
        let state = tempfile::tempdir().expect("state");
        let store = HarnessStore::new(state.path(), root.path()).expect("store");
        let mut session = session_with_history(&store);
        let before = session.turns.clone();
        let config = HarnessConfig {
            context_limit: Some(32768),
            ..Default::default()
        };
        let active = ModelSelection::new("openai", "gpt-5.5");
        let mut host = CompactHost {
            payload: json!({"output_text":"not json"}),
            task_packet: None,
        };

        let error = compact_harness_session(
            &store,
            &mut session,
            &config,
            &active,
            None,
            &mut host,
        )
        .await
        .expect_err("invalid summary");
        assert!(error.to_string().contains("context preserved"));
        assert_eq!(session.turns, before);
        assert_eq!(session.context_start, 0);
        assert!(session.summary.is_none());
    }

    #[tokio::test]
    async fn successful_compaction_carries_a_bounded_task_packet() {
        let root = tempfile::tempdir().expect("root");
        let state = tempfile::tempdir().expect("state");
        let store = HarnessStore::new(state.path(), root.path()).expect("store");
        let mut session = session_with_history(&store);
        let config = HarnessConfig {
            context_limit: Some(32768),
            ..Default::default()
        };
        let active = ModelSelection::new("openai", "gpt-5.5");
        let mut host = CompactHost {
            payload: json!({
                "output_text": "{\"objective\":\"keep context\",\"constraints\":[],\"decisions\":[],\"files\":[],\"evidence\":[],\"pending\":[],\"references\":[0,2]}"
            }),
            task_packet: None,
        };

        compact_harness_session(
            &store,
            &mut session,
            &config,
            &active,
            None,
            &mut host,
        )
        .await
        .expect("compact");
        assert_eq!(session.context_start, 2);
        assert!(session.summary.is_some());
        assert_eq!(host.task_packet.as_ref().and_then(|value| value["role"].as_str()), Some("compact"));
    }
}
