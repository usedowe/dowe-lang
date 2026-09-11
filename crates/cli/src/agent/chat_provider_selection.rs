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
    let provider = match requested {
        Some(provider) => provider.to_string(),
        None => match select_login_provider(store)? {
            Some(provider) => provider,
            None => return Ok(None),
        },
    };
    let definition =
        provider_definition(&provider).ok_or_else(|| format!("unknown provider `{provider}`"))?;
    if provider != "openai-codex" && provider != "openrouter" && !definition.supports_api_key {
        return Err(format!(
            "provider `{}` cannot be configured through interactive login; use its environment or ambient credential",
            definition.name
        )
        .into());
    }

    let credential = if provider == "openai-codex" {
        let credential = login_openai_codex(|url| {
            eprintln!("Opening browser for OpenAI Codex login.");
            eprintln!("{url}");
        })
        .await?;
        eprintln!("OpenAI Codex account connected.");
        credential
    } else if provider == "openrouter" {
        let credential = login_openrouter(|url| {
            eprintln!("Opening browser for OpenRouter login.");
            eprintln!("{url}");
        })
        .await?;
        eprintln!("OpenRouter account connected.");
        credential
    } else {
        prompt_api_key(&definition)?
    };
    let display_key = match credential {
        AgentCredential::ApiKey { .. } => credential.secret().map(str::to_string),
        AgentCredential::OAuth { .. } => None,
    };
    store.save(&provider, &credential)?;
    eprintln!("Configured {}.", definition.name);
    Ok(Some(ConfiguredProvider {
        provider,
        api_key: display_key,
    }))
}

const LOGIN_PROVIDER_IDS: &[&str] = &["openai-codex", "openrouter"];

fn select_login_provider(
    store: &AgentAuthStore,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let providers = builtin_provider_info(store)?
        .into_iter()
        .filter(|provider| LOGIN_PROVIDER_IDS.contains(&provider.id.as_str()))
        .collect::<Vec<_>>();
    let items = aligned_provider_labels(&providers, menu_width());
    let Some(index) = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select provider to configure")
        .items(&items)
        .default(0)
        .interact_opt()?
    else {
        return Ok(None);
    };
    Ok(Some(providers[index].id.clone()))
}
