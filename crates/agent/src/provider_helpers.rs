pub fn protocol_for_model(
    definition: &AgentProviderDefinition,
    model: &str,
) -> AgentProviderProtocol {
    let lower = normalize_model_id(definition.id, model).to_ascii_lowercase();
    match definition.id {
        "openrouter" => {
            if lower.starts_with("anthropic/") {
                AgentProviderProtocol::AnthropicMessages
            } else {
                AgentProviderProtocol::OpenAiCompletions
            }
        }
        "cloudflare-ai-gateway" => {
            if lower.starts_with("claude") {
                AgentProviderProtocol::AnthropicMessages
            } else if lower.starts_with("gpt-")
                || lower.starts_with("o1")
                || lower.starts_with("o3")
                || lower.starts_with("o4")
            {
                AgentProviderProtocol::OpenAiResponses
            } else {
                AgentProviderProtocol::OpenAiCompletions
            }
        }
        "fireworks" => {
            let name = lower.rsplit('/').next().unwrap_or_default();
            if name.starts_with("glm-") || name.starts_with("kimi-k3") {
                AgentProviderProtocol::OpenAiCompletions
            } else {
                definition.protocol
            }
        }
        "vercel-ai-gateway" => AgentProviderProtocol::AnthropicMessages,
        "github-copilot" => {
            if lower.starts_with("claude") {
                AgentProviderProtocol::AnthropicMessages
            } else if lower.starts_with("gpt-5")
                || lower.starts_with("grok-")
                || lower.starts_with("mai-")
            {
                AgentProviderProtocol::OpenAiResponses
            } else {
                AgentProviderProtocol::OpenAiCompletions
            }
        }
        "opencode" | "opencode-go" => {
            if lower.starts_with("claude")
                || (definition.id == "opencode"
                    && matches!(lower.as_str(), "qwen3.5-plus" | "qwen3.6-plus"))
                || (definition.id == "opencode-go"
                    && matches!(lower.as_str(), "minimax-m3" | "qwen3.8-flash"))
            {
                AgentProviderProtocol::AnthropicMessages
            } else if lower.starts_with("gpt-")
                || lower.starts_with("grok-")
                || lower.starts_with("muse-")
            {
                AgentProviderProtocol::OpenAiResponses
            } else if definition.id == "opencode" && lower.starts_with("gemini-") {
                AgentProviderProtocol::GoogleGenerativeAi
            } else {
                AgentProviderProtocol::OpenAiCompletions
            }
        }
        _ => definition.protocol,
    }
}

pub fn normalize_model_id<'a>(provider: &str, model: &'a str) -> &'a str {
    if provider == "openrouter" || provider == "vercel-ai-gateway" {
        return model;
    }
    model
        .split_once('/')
        .filter(|(prefix, _)| *prefix == provider)
        .map(|(_, value)| value)
        .unwrap_or(model)
}

#[allow(clippy::too_many_arguments)]
fn definition(
    id: &'static str,
    name: &'static str,
    env_keys: &'static [&'static str],
    required_env: &'static [&'static str],
    base_url: Option<&'static str>,
    default_model: &'static str,
    protocol: AgentProviderProtocol,
    supports_api_key: bool,
    supports_account: bool,
) -> AgentProviderDefinition {
    AgentProviderDefinition {
        id,
        name,
        env_keys,
        required_env,
        base_url,
        default_model,
        protocol,
        supports_api_key,
        supports_account,
    }
}

fn validate_required_env(
    definition: &AgentProviderDefinition,
    scoped_env: &BTreeMap<String, String>,
) -> AgentResult<()> {
    for name in definition.required_env {
        if env_value(scoped_env, name).is_none()
            && env::var(name)
                .ok()
                .filter(|value| !value.is_empty())
                .is_none()
        {
            return Err(AgentError::new(format!(
                "provider `{}` requires {name}",
                definition.id
            )));
        }
    }
    Ok(())
}

fn env_value(scoped_env: &BTreeMap<String, String>, name: &str) -> Option<String> {
    scoped_env
        .get(name)
        .and_then(|value| expand_credential_value(value, scoped_env))
        .or_else(|| env::var(name).ok().filter(|value| !value.is_empty()))
}

fn unknown_provider(id: &str) -> AgentError {
    AgentError::new(format!(
        "unknown agent provider `{id}`; use `dowe agent providers` to list providers"
    ))
}

pub fn provider_is_configured(id: &str, auth_store: &AgentAuthStore) -> AgentResult<bool> {
    Ok(provider_info(id, auth_store)?.configured)
}

pub fn provider_default_model(id: &str) -> AgentResult<&'static str> {
    provider_definition(id)
        .map(|definition| definition.default_model)
        .ok_or_else(|| unknown_provider(id))
}

pub fn provider_exists(id: &str) -> bool {
    provider_definition(id).is_some()
}

pub(crate) fn retired_model_replacement(provider: &str, model: &str) -> Option<&'static str> {
    if provider == "openai-codex" && normalize_model_id(provider, model) == "gpt-5.3-codex" {
        Some("gpt-5.5")
    } else {
        None
    }
}

pub fn validate_agent_model(provider: &str, model: &str) -> AgentResult<()> {
    if let Some(replacement) = retired_model_replacement(provider, model) {
        return Err(AgentError::new(format!(
            "model `{model}` is no longer supported by Codex with a ChatGPT account; select `{replacement}` with /model"
        )));
    }
    Ok(())
}

pub fn provider_models(id: &str) -> &'static [AgentModelDefinition] {
    crate::catalog::builtin_models(id)
}

pub fn auth_file_has_provider(path: &Path, provider: &str) -> AgentResult<bool> {
    Ok(AgentAuthStore::new(path).read(provider)?.is_some())
}


