pub(super) fn run_agent_providers_command(
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let json_output = args.iter().any(|value| value == "--json");
    if args.iter().any(|value| value != "--json") {
        return Err(USAGE.into());
    }
    let providers = builtin_provider_info(&AgentAuthStore::from_default_path()?)?;
    if json_output {
        println!("{}", serde_json::to_string(&providers)?);
    } else {
        for provider in providers {
            println!("{}\t{}", provider.id, provider_label(&provider));
        }
    }
    Ok(())
}

pub(super) async fn run_agent_chat_or_session(
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    if parse_agent_args(args, false)?.prompt.trim().is_empty() {
        run_agent_session_with_args(args).await
    } else {
        run_agent_chat_command(args).await
    }
}

pub(super) async fn run_agent_session() -> Result<(), Box<dyn std::error::Error>> {
    run_agent_session_with_args(&[]).await
}

pub(super) async fn run_agent_session_with_args(
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut parsed = parse_agent_args(args, false)?;
    parsed.options.stream = true;
    let store = AgentAuthStore::from_default_path()?;
    let preferences = open_preferences()?;
    let mut provider = preferred_request_provider(&parsed, &store, &preferences)?;
    let (mut model, mut thinking) = restore_selection(
        provider.as_deref(),
        parsed.options.model.as_deref(),
        &preferences,
    )?;
    let mut usage = AgentUsageTotals::default();
    let mut conversation = AgentConversation::default();
    let mut native: Option<super::native::NativeSession> = None;
    let native_enabled = !parsed.uses_legacy_server
        && parsed.options.request_type == Some(AgentRequestType::Conversation);
    let mut api_key = parsed.api_key.clone();

    if !parsed.json_output {
        println!("Dowe Agent");
        println!("Dowe Agent is ready — built-in skills loaded. Type /exit to quit.");
    }

    loop {
        let effective_model = model.clone().or_else(|| {
            provider
                .as_deref()
                .and_then(|p| provider_default_model(p).ok().map(str::to_string))
        });
        let footer = super::footer::Footer {
            provider: provider.as_deref(),
            model: effective_model.as_deref(),
            thinking,
            usage: &usage,
        };
        let queued = match native.as_mut() {
            Some(session) => session.take_queued()?,
            None => None,
        };
        let queued_id = queued.as_ref().map(|task| task.id.clone());
        let Some(prompt) = (match queued {
            Some(task) => Some(task.prompt),
            None => read_agent_prompt(&footer, parsed.json_output)?,
        }) else {
            if !parsed.json_output {
                println!();
            }
            return Ok(());
        };
        let mut command = prompt.trim().to_string();
        if !command.starts_with('/') {
            command = attach_prompt_references(&command, &env::current_dir()?, &mut parsed.options);
        }
        let command = command.as_str();
        if is_exit_command(command) {
            return Ok(());
        }
        if command.is_empty() {
            if let Some(id) = queued_id.as_deref()
                && let Some(native) = native.as_ref()
            {
                native.fail_queued(id, "queued prompt was empty")?;
            }
            continue;
        }
        if command.starts_with('/') {
            if let Some(id) = queued_id.as_deref()
                && let Some(native) = native.as_ref()
            {
                native.fail_queued(id, "queued commands are not executed as agent prompts")?;
            }
        }
        if command == "/new" {
            if let Some(native) = &mut native {
                native.reset()?;
            }
            conversation.reset();
            usage = AgentUsageTotals::default();
            eprintln!("Started a new conversation.");
            continue;
        }
        if command == "/login" {
            let Some(configured) = configure_provider(&store, None).await? else {
                continue;
            };
            provider = Some(configured.provider);
            api_key = configured.api_key;
            (model, thinking) = restore_selection(provider.as_deref(), None, &preferences)?;
            usage.clear_context();
            continue;
        }
        if command == "/logout" {
            let selected = provider
                .clone()
                .ok_or_else(|| "No provider selected".to_string())?;
            store.delete(&selected)?;
            api_key = None;
            eprintln!("Logged out from {selected}.");
            continue;
        }
        if command == "/provider" {
            let Some(selected) = select_provider(&store)? else {
                continue;
            };
            preferences.select_provider(&selected)?;
            provider = Some(selected);
            api_key = None;
            (model, thinking) = restore_selection(provider.as_deref(), None, &preferences)?;
            usage.clear_context();
            continue;
        }
        if command == "/model" {
            if provider.is_none() {
                let Some(selected) = select_provider(&store)? else {
                    continue;
                };
                preferences.select_provider(&selected)?;
                provider = Some(selected);
            }
            if let Some(selected) =
                select_model_for_session(&store, provider.as_deref(), model.as_deref())?
            {
                if let Err(error) = preferences.select_model(&selected.provider, &selected.model) {
                    eprintln!("{error}");
                    continue;
                }
                if provider.as_deref() != Some(selected.provider.as_str()) {
                    api_key = None;
                }
                provider = Some(selected.provider);
                (model, thinking) = restore_selection(provider.as_deref(), None, &preferences)?;
                usage.clear_context();
            }
            continue;
        }
        if command == "/thinking" {
            if parsed.uses_legacy_server {
                eprintln!("Thinking selection is only available for native provider requests.");
                continue;
            }
            let Some(selected_provider) = provider.as_deref() else {
                eprintln!("Select a provider with /login or /model first.");
                continue;
            };
            let selected_model = model
                .as_deref()
                .unwrap_or(provider_default_model(selected_provider)?);
            let Some(details) = agent_model_details(selected_provider, selected_model) else {
                eprintln!(
                    "Thinking selection is not available for {selected_provider}/{selected_model}."
                );
                continue;
            };
            let levels = details.thinking_levels;
            let thinking_items = aligned_menu_items(
                &levels
                    .iter()
                    .map(|level| level.as_str())
                    .collect::<Vec<_>>(),
                menu_width(),
            );
            if let Some(index) = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select thinking level")
                .items(&thinking_items)
                .default(
                    thinking
                        .and_then(|level| levels.iter().position(|candidate| *candidate == level))
                        .unwrap_or(0),
                )
                .interact_opt()?
            {
                preferences.select_thinking(selected_provider, selected_model, levels[index])?;
                thinking = Some(levels[index]);
            }
            continue;
        }

        if native_enabled
            && matches!(
                command.split_whitespace().next(),
                Some(
                    "/session"
                        | "/sessions"
                        | "/resume"
                        | "/delete-session"
                        | "/memory"
                        | "/models"
                        | "/shell"
                        | "/env"
                        | "/evaluate"
                        | "/processes"
                        | "/watch"
                        | "/queue"
                        | "/inspect"
                        | "/recover"
                        | "/capabilities"
                        | "/governance"
                        | "/sdd"
                )
            )
        {
            if native.is_none() {
                native = Some(super::native::NativeSession::open()?);
            }
            match native.as_mut().expect("native session").local_command(
                command,
                parsed.json_output,
                parsed.api_key.as_deref(),
            ) {
                Ok(false) => {
                    return Err("approval_required: pending operation was not executed".into());
                }
                Err(error) => eprintln!("{}", super::markdown::terminal_text(&error.to_string())),
                Ok(true)
                    if matches!(
                        command.split_whitespace().next(),
                        Some("/resume" | "/recover")
                    ) =>
                {
                    usage = native.as_ref().expect("native session").usage();
                }
                _ => {}
            }
            continue;
        }
        let selected_provider = match provider.clone() {
            Some(provider) => provider,
            None => {
                if let Some(configured) = first_configured_provider(&store)? {
                    provider = Some(configured.clone());
                    configured
                } else {
                    let Some(configured) = configure_provider(&store, None).await? else {
                        continue;
                    };
                    api_key = configured.api_key;
                    provider = Some(configured.provider.clone());
                    configured.provider
                }
            }
        };
        if native_enabled {
            if native.is_none() {
                native = Some(super::native::NativeSession::open()?);
            }
            let selection = dowe_agent::native_harness::ModelSelection {
                provider: selected_provider.clone(),
                model: model
                    .clone()
                    .unwrap_or(provider_default_model(&selected_provider)?.into()),
                thinking,
            };
            native.as_mut().expect("native session").image_paths =
                parsed.options.image_paths.clone();
            let work = native.as_mut().expect("native session").run(
                command,
                selection,
                parsed.explicit_model,
                api_key.clone(),
                parsed.json_output,
                &mut usage,
            );
            let result = tokio::select! {
                result = work => result,
                _ = tokio::signal::ctrl_c() => Err(dowe_agent::AgentError::new("Agent task canceled; interrupted operations are not replayed")),
            };
            if let Some(id) = queued_id.as_deref()
                && let Some(native) = native.as_ref()
            {
                match &result {
                    Ok(dowe_agent::native_harness::HarnessOutcome::Completed) => {
                        native.complete_queued(id, json!({"status":"completed"}))?
                    }
                    Ok(outcome) => native
                        .fail_queued(id, &format!("queued task did not complete: {outcome:?}"))?,
                    Err(error) => native.fail_queued(id, &error.to_string())?,
                }
            }
            if matches!(
                result,
                Ok(dowe_agent::native_harness::HarnessOutcome::Completed)
            ) {
                parsed.options.image_paths.clear();
            }
            if matches!(
                result,
                Ok(dowe_agent::native_harness::HarnessOutcome::ApprovalRequired)
            ) && (parsed.json_output || !is_interactive_terminal())
            {
                return Err("approval_required: pending operation was not executed".into());
            }
            if let Err(error) = result {
                eprintln!("{}", super::markdown::terminal_text(&error.to_string()));
                eprintln!(
                    "Request failed. Use /model, /provider or /login to adjust the session, then try again."
                );
            }
            continue;
        }
        let Some(auth) =
            ensure_provider_auth(&store, &selected_provider, api_key.as_deref(), true).await?
        else {
            continue;
        };
        let mut request = parsed.clone();
        request.prompt = command.to_string();
        request.provider = Some(selected_provider.clone());
        request.api_key = api_key.clone();
        request.options.provider = Some(selected_provider.clone());
        request.options.model = model.clone();
        request.options.thinking_level = if parsed.uses_legacy_server {
            None
        } else {
            thinking
        };
        match send_parsed_request(request, Some(auth), &mut conversation).await {
            Ok(event) if event.event == AgentDesktopEventKind::ResponseReceived => {
                usage.record(&selected_provider, &event.model, &event.payload);
                if event.request_type == AgentRequestType::Conversation {
                    parsed.options.image_paths.clear();
                }
            }
            result => {
                usage.clear_context();
                if let Err(error) = result {
                    eprintln!("{error}");
                }
                eprintln!(
                    "Request failed. Use /model, /provider or /login to adjust the session, then try again."
                );
            }
        }
    }
}
