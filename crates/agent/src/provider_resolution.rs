pub fn provider_info(id: &str, auth_store: &AgentAuthStore) -> AgentResult<AgentProviderInfo> {
    let definition = provider_definition(id).ok_or_else(|| unknown_provider(id))?;
    let resolved = resolve_provider_auth(&definition, auth_store, None, None).unwrap_or_default();
    Ok(AgentProviderInfo {
        id: definition.id.to_string(),
        name: definition.name.to_string(),
        configured: resolved.is_some(),
        source: resolved.map(|auth| auth.source),
        default_model: definition.default_model.to_string(),
        protocol: definition.protocol.as_str().to_string(),
        api_key_env: definition
            .env_keys
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        supports_account: definition.supports_account,
    })
}

pub fn builtin_provider_info(auth_store: &AgentAuthStore) -> AgentResult<Vec<AgentProviderInfo>> {
    builtin_provider_ids()
        .iter()
        .map(|id| provider_info(id, auth_store))
        .collect()
}

pub fn resolve_provider_auth(
    definition: &AgentProviderDefinition,
    auth_store: &AgentAuthStore,
    explicit_api_key: Option<&str>,
    preferred_credential: Option<&AgentCredential>,
) -> AgentResult<Option<ResolvedProviderAuth>> {
    let stored = match preferred_credential {
        Some(credential) => Some(credential.clone()),
        None => auth_store.read(definition.id)?,
    };
    let mut scoped_env = stored
        .as_ref()
        .map(|credential| credential.env().clone())
        .unwrap_or_default();
    let optional_env: &[&str] = match definition.id {
        "amazon-bedrock" => &["AWS_REGION", "AWS_DEFAULT_REGION"],
        "google-vertex" => &["GOOGLE_CLOUD_PROJECT", "GOOGLE_CLOUD_LOCATION"],
        _ => &[],
    };
    for name in definition.required_env.iter().chain(optional_env) {
        if let Some(value) = env::var(name).ok().filter(|value| !value.is_empty()) {
            scoped_env.entry((*name).to_string()).or_insert(value);
        }
    }

    if let Some(key) = explicit_api_key.filter(|value| !value.is_empty()) {
        validate_required_env(definition, &scoped_env)?;
        return Ok(Some(ResolvedProviderAuth {
            kind: AgentAuthKind::ApiKey,
            secret: Some(key.to_string()),
            env: scoped_env,
            source: "command line".to_string(),
        }));
    }

    if let Some(credential) = stored.as_ref() {
        let secret = credential
            .secret()
            .and_then(|value| expand_credential_value(value, &scoped_env));
        if secret.is_some() || credential.secret().is_none() {
            validate_required_env(definition, &scoped_env)?;
            return Ok(Some(ResolvedProviderAuth {
                kind: match credential {
                    AgentCredential::ApiKey { .. } => AgentAuthKind::ApiKey,
                    AgentCredential::OAuth { .. } => AgentAuthKind::OAuth,
                },
                secret,
                env: scoped_env,
                source: "stored credential".to_string(),
            }));
        }
        return Err(AgentError::new(format!(
            "stored credential for `{}` could not be resolved; sign in again",
            definition.id
        )));
    }

    for name in definition.env_keys {
        if let Ok(value) = env::var(name)
            && !value.is_empty()
        {
            validate_required_env(definition, &scoped_env)?;
            let kind = if definition.id == "anthropic" && *name == "ANTHROPIC_AUTH_TOKEN" {
                AgentAuthKind::OAuth
            } else {
                AgentAuthKind::ApiKey
            };
            return Ok(Some(ResolvedProviderAuth {
                kind,
                secret: Some(value),
                env: scoped_env,
                source: (*name).to_string(),
            }));
        }
    }

    Ok(None)
}

pub fn provider_base_url(
    definition: &AgentProviderDefinition,
    auth: &ResolvedProviderAuth,
) -> AgentResult<String> {
    if definition.id == "azure-openai-responses" {
        return env_value(&auth.env, "AZURE_OPENAI_BASE_URL")
            .or_else(|| env::var("AZURE_OPENAI_BASE_URL").ok())
            .ok_or_else(|| AgentError::new("Azure OpenAI requires AZURE_OPENAI_BASE_URL"))
            .and_then(|base| {
                let mut url = reqwest::Url::parse(&base)
                    .map_err(|_| AgentError::new("invalid Azure OpenAI base URL"))?;
                let host = url.host_str().unwrap_or_default();
                if [
                    ".openai.azure.com",
                    ".cognitiveservices.azure.com",
                    ".ai.azure.com",
                ]
                .iter()
                .any(|suffix| host.ends_with(suffix))
                    && matches!(
                        url.path().trim_end_matches('/'),
                        "" | "/openai" | "/openai/v1/responses"
                    )
                {
                    url.set_path("/openai/v1");
                    url.set_query(None);
                }
                Ok(url.to_string().trim_end_matches('/').to_string())
            });
    }
    if definition.id == "google-vertex" {
        if auth.kind == AgentAuthKind::ApiKey && auth.secret.is_some() {
            return Ok("https://aiplatform.googleapis.com".to_string());
        }
        if auth.kind == AgentAuthKind::Ambient {
            return Err(AgentError::new(
                "Google Vertex ADC is not implemented; configure GOOGLE_CLOUD_API_KEY for Vertex Express",
            ));
        }
    }
    let base = definition
        .base_url
        .ok_or_else(|| AgentError::new(format!("provider `{}` has no base URL", definition.id)))?;
    let mut value = base.to_string();
    if definition.id == "amazon-bedrock"
        && let Some(region) = env_value(&auth.env, "AWS_REGION")
            .or_else(|| env_value(&auth.env, "AWS_DEFAULT_REGION"))
    {
        if region.is_empty()
            || !region
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        {
            return Err(AgentError::new("invalid AWS region"));
        }
        value = value.replace("us-east-1", &region);
    }
    for (name, replacement) in &auth.env {
        value = value.replace(&format!("{{{name}}}"), replacement);
    }
    if let Some(location) = env_value(&auth.env, "GOOGLE_CLOUD_LOCATION") {
        value = value.replace("{location}", &location);
    }
    value = value.replace(
        "global-aiplatform.googleapis.com",
        "aiplatform.googleapis.com",
    );
    if value.contains("{CLOUDFLARE_") || value.contains("{location}") {
        return Err(AgentError::new(format!(
            "provider `{}` is missing required endpoint configuration",
            definition.id
        )));
    }
    Ok(value)
}


