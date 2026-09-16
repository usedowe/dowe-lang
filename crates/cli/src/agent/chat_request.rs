pub(super) async fn run_agent_chat_command(
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut parsed = parse_agent_args(args, true)?;
    let auth = {
        let store = AgentAuthStore::from_default_path()?;
        let preferences = open_preferences()?;
        let provider = resolve_request_provider(&parsed, &store, &preferences)?;
        parsed.provider = Some(provider.clone());
        parsed.options.provider = Some(provider.clone());
        let (model, thinking) = restore_selection(
            Some(&provider),
            parsed.options.model.as_deref(),
            &preferences,
        )?;
        parsed.options.model = model;
        parsed.options.thinking_level = thinking;
        dowe_agent::validate_agent_model(
            &provider,
            parsed
                .options
                .model
                .as_deref()
                .unwrap_or(provider_default_model(&provider)?),
        )?;
        if parsed.options.request_type == Some(AgentRequestType::Conversation) {
            None
        } else {
            let Some(auth) =
                ensure_provider_auth(&store, &provider, parsed.api_key.as_deref(), true).await?
            else {
                return Ok(());
            };
            Some(auth)
        }
    };
    if parsed.options.request_type == Some(AgentRequestType::Conversation) {
        let provider = parsed.provider.as_deref().ok_or("provider missing")?;
        let selection = dowe_agent::native_harness::ModelSelection {
            provider: provider.into(),
            model: parsed
                .options
                .model
                .clone()
                .unwrap_or(provider_default_model(provider)?.into()),
            thinking: parsed.options.thinking_level,
        };
        let mut native = super::native::NativeSession::open()?;
        native.image_paths = parsed.options.image_paths.clone();
        let outcome = native
            .run(
                &parsed.prompt,
                selection,
                parsed.explicit_model,
                parsed.api_key.clone(),
                parsed.json_output,
                &mut AgentUsageTotals::default(),
            )
            .await;
        let outcome = match outcome {
            Ok(outcome) => outcome,
            Err(error) => {
                if parsed.json_output {
                    println!(
                        "{}",
                        serde_json::json!({
                            "event": "error",
                            "payload": {"error": {"message": error.to_string()}}
                        })
                    );
                }
                return Err(error.into());
            }
        };
        if !matches!(
            outcome,
            dowe_agent::native_harness::HarnessOutcome::Completed
        ) {
            return Err(
                "Agent paused without completing the task; inspect its session events".into(),
            );
        }
        return Ok(());
    }
    let event = send_parsed_request(parsed, auth).await?;
    if event.event == AgentDesktopEventKind::Error {
        return Err(event
            .payload
            .pointer("/error/message")
            .and_then(Value::as_str)
            .unwrap_or("agent request failed")
            .to_string()
            .into());
    }
    Ok(())
}

async fn send_parsed_request(
    parsed: ParsedAgentChatArgs,
    auth: Option<ResolvedProviderAuth>,
) -> Result<AgentDesktopEvent, Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    let prepared = prepare_agent_request(root, &parsed.prompt, parsed.options)?;
    let request = prepared.request;
    let prepared_payload = json!({
        "requestId": request.request_id,
        "requestType": request.request_type,
        "provider": request.provider,
        "model": request.model,
        "skillCount": prepared.context.skills.len(),
        "imageCount": prepared.context.images.len(),
        "needsReferenceImage": prepared.context.needs_reference_image,
        "codegraphMode": prepared.context.codegraph.as_ref().map(|summary| summary.mode.clone())
    });
    let prepared_event = AgentDesktopEvent {
        event: AgentDesktopEventKind::RequestPrepared,
        request_id: request.request_id.clone(),
        request_type: request.request_type,
        model: request.model.clone(),
        payload: prepared_payload,
    };
    print_agent_event(&prepared_event, parsed.json_output)?;

    let response = send_native_agent_request(
        &request,
        auth.as_ref().ok_or("provider authentication is required")?,
    )
    .await;
    let response = match response {
        Ok(response) => response,
        Err(error) => {
            let event = AgentDesktopEvent {
                event: AgentDesktopEventKind::Error,
                request_id: request.request_id.clone(),
                request_type: request.request_type,
                model: request.model.clone(),
                payload: json!({
                    "error": {
                        "code": "provider_request_failed",
                        "message": error.to_string()
                    }
                }),
            };
            print_agent_event(&event, parsed.json_output)?;
            return Ok(event);
        }
    };
    let event = AgentDesktopEvent {
        event: AgentDesktopEventKind::ResponseReceived,
        request_id: response.request_id,
        request_type: response.request_type,
        model: response.model,
        payload: response.payload,
    };
    print_agent_event(&event, parsed.json_output)?;
    Ok(event)
}
