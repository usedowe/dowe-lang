use crate::menus::is_interactive_terminal;
use crate::usage::USAGE;
use dialoguer::{Input, Password, Select, theme::ColorfulTheme};
use dowe_agent::{
    AgentAuthStore, AgentConversation, AgentCredential, AgentDesktopEvent, AgentDesktopEventKind,
    AgentPreferencesStore, AgentPrepareOptions, AgentProviderInfo, AgentRequestType,
    AgentUsageTotals, ResolvedProviderAuth, ThinkingLevel, agent_model_details,
    agent_response_text, builtin_provider_info, default_llm_server_url, login_openai_codex,
    normalize_model_id, prepare_agent_request, provider_default_model, provider_definition,
    provider_exists, provider_models, refresh_openai_codex_credential, resolve_provider_auth,
    send_agent_request, send_native_agent_request, token_needs_refresh,
};
use serde_json::{Value, json};
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

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
        println!("Skills are embedded in the Dowe binary. Type /exit to leave.");
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
        let queued = native.as_mut().and_then(|session| session.take_queued());
            let Some(prompt) = (match queued {
                Some(prompt) => Some(prompt),
                None => read_agent_prompt(&footer, parsed.json_output)?,
            }) else {
            if !parsed.json_output {
                println!();
            }
            return Ok(());
        };
        let command = prompt.trim();
        if is_exit_command(command) {
            return Ok(());
        }
        if command.is_empty() {
            continue;
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
            let Some(selected) = select_provider(&store, false)? else {
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
                let Some(selected) = select_provider(&store, false)? else {
                    continue;
                };
                preferences.select_provider(&selected)?;
                provider = Some(selected);
            }
            let selected_provider = provider.as_deref().expect("selected provider");
            if let Some(selected) = select_model(selected_provider, model.as_deref())? {
                if let Err(error) = preferences.select_model(selected_provider, &selected) {
                    eprintln!("{error}");
                    continue;
                }
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
                eprintln!("Select a provider with /provider first.");
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
            if let Some(index) = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select thinking level")
                .items(
                    levels
                        .iter()
                        .map(|level| level.as_str())
                        .collect::<Vec<_>>(),
                )
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
                        | "/budget"
                        | "/env"
                        | "/evaluate"
                        | "/processes"
                        | "/watch"
                        | "/queue"
                            | "/inspect"
                        | "/recover"
                        | "/capabilities"
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

pub(super) async fn run_agent_chat_command(
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut parsed = parse_agent_args(args, true)?;
    let auth = if parsed.uses_legacy_server {
        None
    } else {
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
    if !parsed.uses_legacy_server
        && parsed.options.request_type == Some(AgentRequestType::Conversation)
    {
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
            .await?;
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
    let event = send_parsed_request(parsed, auth, &mut AgentConversation::default()).await?;
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
    conversation: &mut AgentConversation,
) -> Result<AgentDesktopEvent, Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    let conversational = parsed.options.request_type == Some(AgentRequestType::Conversation);
    let prepared = if conversational {
        conversation.prepare(root, &parsed.prompt, parsed.options)?
    } else {
        prepare_agent_request(root, &parsed.prompt, parsed.options)?
    };
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

    let response = if parsed.uses_legacy_server {
        send_agent_request(&parsed.server_url, &request).await
    } else {
        send_native_agent_request(
            &request,
            auth.as_ref().ok_or("provider authentication is required")?,
        )
        .await
    };
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
    if conversational && let Err(error) = conversation.record_response(&request, &response) {
        let event = AgentDesktopEvent {
            event: AgentDesktopEventKind::Error,
            request_id: request.request_id,
            request_type: request.request_type,
            model: request.model,
            payload: json!({"error":{"code":"invalid_conversation_response","message":error.to_string()}}),
        };
        print_agent_event(&event, parsed.json_output)?;
        return Ok(event);
    }
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

fn parse_agent_args(
    args: &[String],
    require_prompt: bool,
) -> Result<ParsedAgentChatArgs, Box<dyn std::error::Error>> {
    let mut options = AgentPrepareOptions::default();
    let mut server_url = default_llm_server_url().to_string();
    let mut uses_legacy_server = false;
    let mut json_output = false;
    let mut api_key = None;
    let mut provider = None;
    let mut prompt = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--image" => {
                let value = required_value(args, index, "--image")?;
                options.image_paths.push(PathBuf::from(value));
                index += 2;
            }
            "--request-type" => {
                let value = required_value(args, index, "--request-type")?;
                options.request_type = Some(
                    AgentRequestType::parse(value)
                        .ok_or_else(|| format!("invalid request type `{value}`"))?,
                );
                index += 2;
            }
            "--provider" => {
                let value = required_value(args, index, "--provider")?;
                if !provider_exists(value) {
                    return Err(format!("unknown agent provider `{value}`").into());
                }
                provider = Some(value.to_string());
                index += 2;
            }
            "--model" => {
                options.model = Some(required_value(args, index, "--model")?.to_string());
                index += 2;
            }
            "--api-key" => {
                api_key = Some(required_value(args, index, "--api-key")?.to_string());
                index += 2;
            }
            "--server" => {
                server_url = required_value(args, index, "--server")?.to_string();
                uses_legacy_server = true;
                index += 2;
            }
            "--json" => {
                json_output = true;
                index += 1;
            }
            "--stream" => {
                options.stream = true;
                index += 1;
            }
            value if value.starts_with("--") => return Err(USAGE.into()),
            value => {
                prompt.push(value.to_string());
                index += 1;
            }
        }
    }

    let prompt = prompt.join(" ");
    if require_prompt && prompt.trim().is_empty() {
        return Err(USAGE.into());
    }

    options.provider = provider.clone();
    if !uses_legacy_server && options.request_type.is_none() {
        options.request_type = Some(AgentRequestType::Conversation);
    }
    let explicit_model = options.model.is_some();
    Ok(ParsedAgentChatArgs {
        explicit_model,
        prompt,
        provider,
        api_key,
        server_url,
        uses_legacy_server,
        json_output,
        options,
    })
}

fn required_value<'a>(
    args: &'a [String],
    index: usize,
    name: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    args.get(index + 1)
        .map(String::as_str)
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| format!("{name} requires a value").into())
}

fn open_preferences() -> Result<AgentPreferencesStore, Box<dyn std::error::Error>> {
    let preferences = AgentPreferencesStore::from_default_path()?;
    if let Some(model) = preferences.migrate_retired_model()? {
        eprintln!(
            "Updated saved Codex model: gpt-5.3-codex is no longer supported with ChatGPT accounts; using {model}. Change it with /model."
        );
    }
    Ok(preferences)
}

fn resolve_request_provider(
    parsed: &ParsedAgentChatArgs,
    store: &AgentAuthStore,
    preferences: &AgentPreferencesStore,
) -> Result<String, Box<dyn std::error::Error>> {
    preferred_request_provider(parsed, store, preferences)?
        .ok_or_else(|| {
            "no agent provider is configured; run `dowe agent` interactively or pass --provider and --api-key".into()
        })
}

fn preferred_request_provider(
    parsed: &ParsedAgentChatArgs,
    store: &AgentAuthStore,
    preferences: &AgentPreferencesStore,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if let Some(provider) = parsed.provider.as_deref() {
        return Ok(Some(provider.to_string()));
    }
    if let Some(provider) = parsed
        .options
        .model
        .as_deref()
        .and_then(infer_provider_from_model)
    {
        return Ok(Some(provider));
    }
    if let Some(provider) = preferences.provider()? {
        return Ok(Some(provider));
    }
    first_configured_provider(store)
}

fn restore_selection(
    provider: Option<&str>,
    explicit_model: Option<&str>,
    preferences: &AgentPreferencesStore,
) -> Result<(Option<String>, Option<ThinkingLevel>), Box<dyn std::error::Error>> {
    let saved = preferences.read()?;
    let same_provider = saved.provider.as_deref() == provider;
    let model = explicit_model.map(str::to_string).or_else(|| {
        if same_provider {
            saved.model.clone()
        } else {
            None
        }
    });
    let thinking = if let Some(provider) = provider.filter(|_| same_provider) {
        let actual = model
            .as_deref()
            .unwrap_or(provider_default_model(provider)?);
        let saved_model = saved
            .model
            .as_deref()
            .unwrap_or(provider_default_model(provider)?);
        if normalize_model_id(provider, actual) == normalize_model_id(provider, saved_model) {
            saved.thinking_level
        } else {
            None
        }
    } else {
        None
    };
    Ok((model, thinking))
}

fn first_configured_provider(
    store: &AgentAuthStore,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let providers = builtin_provider_info(store)?;
    Ok(providers
        .iter()
        .find(|provider| {
            provider.configured && provider.source.as_deref() == Some("stored credential")
        })
        .or_else(|| providers.iter().find(|provider| provider.configured))
        .map(|provider| provider.id.clone()))
}

pub(super) async fn ensure_provider_auth(
    store: &AgentAuthStore,
    provider: &str,
    explicit_api_key: Option<&str>,
    allow_interactive: bool,
) -> Result<Option<ResolvedProviderAuth>, Box<dyn std::error::Error>> {
    if provider == "openai-codex"
        && explicit_api_key.is_none()
        && let Some(credential) = store.read(provider)?
        && token_needs_refresh(&credential)
    {
        let refreshed = refresh_openai_codex_credential(&credential).await?;
        store.save(provider, &refreshed)?;
    }
    let definition = provider_definition(provider)
        .ok_or_else(|| format!("unknown agent provider `{provider}`"))?;
    match resolve_provider_auth(&definition, store, explicit_api_key, None)? {
        Some(auth) => Ok(Some(auth)),
        None if allow_interactive && is_interactive_terminal() => {
            let Some(configured) = configure_provider(store, Some(provider)).await? else {
                return Ok(None);
            };
            if configured.provider != provider {
                return Err(format!(
                    "configured `{}` instead of `{provider}`",
                    configured.provider
                )
                .into());
            }
            resolve_provider_auth(&definition, store, explicit_api_key, None)?
                .map(Some)
                .ok_or_else(|| format!("provider `{provider}` is still not configured").into())
        }
        None => Err(format!(
            "provider `{provider}` is not configured; set {} or use `dowe agent` interactively",
            definition.env_keys.join(" or ")
        )
        .into()),
    }
}

struct ConfiguredProvider {
    provider: String,
    api_key: Option<String>,
}

async fn configure_provider(
    store: &AgentAuthStore,
    requested: Option<&str>,
) -> Result<Option<ConfiguredProvider>, Box<dyn std::error::Error>> {
    let mut previous_method = 0;
    let (account, provider) = loop {
        let Some(method) = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select authentication method")
            .items(["Sign in with an account", "Sign in with an API key"])
            .default(previous_method)
            .interact_opt()?
        else {
            return Ok(None);
        };
        previous_method = method;
        let account = method == 0;
        let provider = match requested {
            Some(provider) => provider.to_string(),
            None => match select_provider(store, account)? {
                Some(provider) => provider,
                None => continue,
            },
        };
        break (account, provider);
    };
    let definition =
        provider_definition(&provider).ok_or_else(|| format!("unknown provider `{provider}`"))?;
    if account && !definition.supports_account {
        return Err(format!(
            "provider `{}` does not support account sign-in",
            definition.name
        )
        .into());
    }
    if !account && !definition.supports_api_key {
        return Err(format!("provider `{}` requires account sign-in", definition.name).into());
    }

    let credential = if account && provider == "openai-codex" {
        let credential = login_openai_codex(|url| {
            eprintln!("Opening browser for OpenAI Codex login.");
            eprintln!("{url}");
        })
        .await?;
        eprintln!("OpenAI Codex account connected.");
        credential
    } else if account {
        let access = Password::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Paste {} account access token", definition.name))
            .allow_empty_password(false)
            .interact()?;
        AgentCredential::OAuth {
            access,
            refresh: None,
            expires: None,
            env: Default::default(),
        }
    } else {
        prompt_api_key(&definition)?
    };
    let display_key = match credential {
        AgentCredential::ApiKey { .. } => credential.secret().map(str::to_string),
        AgentCredential::OAuth { .. } => None,
    };
    store.save(&provider, &credential)?;
    AgentPreferencesStore::from_default_path()?.select_provider(&provider)?;
    eprintln!("Configured {}.", definition.name);
    Ok(Some(ConfiguredProvider {
        provider,
        api_key: display_key,
    }))
}

fn prompt_api_key(
    definition: &dowe_agent::AgentProviderDefinition,
) -> Result<AgentCredential, Box<dyn std::error::Error>> {
    let key = Password::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Enter {} API key", definition.name))
        .allow_empty_password(false)
        .interact()?;
    let mut scoped_env = std::collections::BTreeMap::new();
    if definition.id == "cloudflare-ai-gateway" || definition.id == "cloudflare-workers-ai" {
        let account = Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Cloudflare account ID")
            .allow_empty(false)
            .interact_text()?;
        scoped_env.insert("CLOUDFLARE_ACCOUNT_ID".to_string(), account);
        if definition.id == "cloudflare-ai-gateway" {
            let gateway = Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Cloudflare AI Gateway ID")
                .allow_empty(false)
                .interact_text()?;
            scoped_env.insert("CLOUDFLARE_GATEWAY_ID".to_string(), gateway);
        }
    }
    Ok(AgentCredential::ApiKey {
        key: Some(key),
        env: scoped_env,
    })
}

pub(super) fn select_model(
    provider: &str,
    current: Option<&str>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let models = provider_models(provider);
    let default = current
        .map(str::to_string)
        .unwrap_or(provider_default_model(provider)?.to_string());
    if models.is_empty() {
        return Ok(Some(
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Model")
                .default(default)
                .allow_empty(false)
                .interact_text()?,
        ));
    }
    let mut items = models
        .iter()
        .map(|model| format!("{} • {}", model.name, model.id))
        .collect::<Vec<_>>();
    let custom_index = items.len();
    items.push("Enter another model id".to_string());
    let default_index = models
        .iter()
        .position(|model| model.id == default)
        .unwrap_or(custom_index);
    let Some(index) = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Select {provider} model"))
        .items(&items)
        .default(default_index)
        .interact_opt()?
    else {
        return Ok(None);
    };
    if index == custom_index {
        return Ok(Some(
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Model")
                .default(default)
                .allow_empty(false)
                .interact_text()?,
        ));
    }
    Ok(Some(models[index].id.to_string()))
}

pub(super) fn select_provider(
    store: &AgentAuthStore,
    account: bool,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let providers = builtin_provider_info(store)?
        .into_iter()
        .filter(|provider| !account || provider.supports_account)
        .collect::<Vec<_>>();
    if providers.is_empty() {
        return Err("no providers support the selected authentication method".into());
    }
    let items = providers.iter().map(provider_label).collect::<Vec<_>>();
    let index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select provider to configure")
        .items(&items)
        .default(0)
        .interact_opt()?;
    Ok(index.map(|index| providers[index].id.clone()))
}

fn provider_label(provider: &AgentProviderInfo) -> String {
    let status = provider
        .source
        .as_deref()
        .map(|source| format!("stored: {source}"))
        .unwrap_or_else(|| "unconfigured".to_string());
    format!("{} • {}", provider.name, status)
}

fn infer_provider_from_model(model: &str) -> Option<String> {
    let provider = model.split_once('/')?.0;
    provider_exists(provider).then(|| provider.to_string())
}

fn print_agent_event(
    event: &AgentDesktopEvent,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if json_output {
        println!("{}", serde_json::to_string(event)?);
        return Ok(());
    }

    match event.event {
        AgentDesktopEventKind::RequestPrepared => {}
        AgentDesktopEventKind::ResponseReceived => {
            print_agent_payload(&event.payload, event.request_type)?
        }
        AgentDesktopEventKind::Error => {
            let message = event
                .payload
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("Agent request failed.");
            eprintln!("Error: {}", super::markdown::terminal_text(message));
        }
    }
    Ok(())
}

fn print_agent_payload(
    payload: &Value,
    request_type: AgentRequestType,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = match agent_response_text(payload) {
        Ok(text) => text,
        Err(error) if request_type == AgentRequestType::Conversation => return Err(error.into()),
        Err(_) => serde_json::to_string_pretty(payload)?,
    };
    let color = is_interactive_terminal() && dialoguer::console::colors_enabled();
    println!("\n{}\n", super::markdown::render_markdown(&content, color));
    Ok(())
}

#[derive(Clone)]
struct ParsedAgentChatArgs {
    explicit_model: bool,
    prompt: String,
    provider: Option<String>,
    api_key: Option<String>,
    server_url: String,
    uses_legacy_server: bool,
    json_output: bool,
    options: AgentPrepareOptions,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_cli_defaults_to_conversation_without_changing_structured_modes() {
        for args in [
            vec![],
            vec!["hola".into()],
            vec!["--json".into(), "hola".into()],
        ] {
            let parsed = parse_agent_args(&args, false).unwrap();
            assert_eq!(
                parsed.options.request_type,
                Some(AgentRequestType::Conversation)
            );
        }
        let structured = parse_agent_args(
            &["--request-type".into(), "clarify".into(), "hola".into()],
            true,
        )
        .unwrap();
        assert_eq!(
            structured.options.request_type,
            Some(AgentRequestType::Clarify)
        );
        let legacy = parse_agent_args(
            &[
                "--server".into(),
                "http://localhost:1234".into(),
                "hola".into(),
            ],
            true,
        )
        .unwrap();
        assert!(legacy.uses_legacy_server);
        assert_eq!(legacy.options.request_type, None);
    }

    #[test]
    fn only_exit_is_an_interactive_exit_command() {
        assert!(is_exit_command("/exit"));
        assert!(!is_exit_command("/quit"));
        assert!(!is_exit_command(":q"));
    }

    #[test]
    fn agent_provider_precedence_keeps_explicit_overrides_temporary() {
        let home = tempfile::tempdir().unwrap();
        let auth = AgentAuthStore::new(home.path().join("auth.json"));
        let preferences = AgentPreferencesStore::new(home.path().join("preferences.json"));
        auth.save("anthropic", &AgentCredential::api_key("test-key"))
            .unwrap();
        let parsed = parse_agent_args(&[], false).unwrap();
        assert_eq!(
            preferred_request_provider(&parsed, &auth, &preferences)
                .unwrap()
                .as_deref(),
            Some("anthropic")
        );
        preferences.select_provider("openai-codex").unwrap();
        assert_eq!(
            preferred_request_provider(&parsed, &auth, &preferences)
                .unwrap()
                .as_deref(),
            Some("openai-codex")
        );
        for (args, expected) in [
            (vec!["--provider", "google"], "google"),
            (vec!["--model", "anthropic/claude-test"], "anthropic"),
            (
                vec!["--provider", "google", "--model", "anthropic/claude-test"],
                "google",
            ),
        ] {
            let args = args.into_iter().map(str::to_string).collect::<Vec<_>>();
            let parsed = parse_agent_args(&args, false).unwrap();
            assert_eq!(
                preferred_request_provider(&parsed, &auth, &preferences)
                    .unwrap()
                    .as_deref(),
                Some(expected)
            );
            assert_eq!(
                preferences.provider().unwrap().as_deref(),
                Some("openai-codex")
            );
        }
    }
}

fn is_exit_command(command: &str) -> bool {
    command == "/exit"
}

fn read_agent_prompt(
    footer: &super::footer::Footer<'_>,
    json_output: bool,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if !json_output && is_interactive_terminal() {
        return Ok(super::prompt::read_interactive_prompt(footer)?);
    }
    if !json_output {
        print!("\n> ");
        io::stdout().flush()?;
    }
    let mut prompt = String::new();
    if io::stdin().read_line(&mut prompt)? == 0 {
        return Ok(None);
    }
    Ok(Some(prompt.trim_end_matches(['\r', '\n']).to_string()))
}
