use crate::auth::{AgentAuthStore, AgentCredential, expand_credential_value};
use crate::error::{AgentError, AgentResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::path::Path;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentProviderProtocol {
    OpenAiCompletions,
    OpenAiResponses,
    AnthropicMessages,
    GoogleGenerativeAi,
    GoogleVertex,
    MistralConversations,
    BedrockConverse,
    PiMessages,
}

impl AgentProviderProtocol {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAiCompletions => "openai-completions",
            Self::OpenAiResponses => "openai-responses",
            Self::AnthropicMessages => "anthropic-messages",
            Self::GoogleGenerativeAi => "google-generative-ai",
            Self::GoogleVertex => "google-vertex",
            Self::MistralConversations => "mistral-conversations",
            Self::BedrockConverse => "bedrock-converse-stream",
            Self::PiMessages => "pi-messages",
        }
    }
}

impl std::str::FromStr for AgentProviderProtocol {
    type Err = AgentError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "openai-completions" => Ok(Self::OpenAiCompletions), "openai-responses" => Ok(Self::OpenAiResponses),
            "anthropic-messages" => Ok(Self::AnthropicMessages), "google-generative-ai" => Ok(Self::GoogleGenerativeAi),
            "google-vertex" => Ok(Self::GoogleVertex), "mistral-conversations" => Ok(Self::MistralConversations),
            "bedrock-converse-stream" => Ok(Self::BedrockConverse), "pi-messages" => Ok(Self::PiMessages),
            _ => Err(AgentError::new("unsupported dynamic provider protocol")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentAuthKind {
    ApiKey,
    OAuth,
    Ambient,
}

impl AgentAuthKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ApiKey => "api_key",
            Self::OAuth => "oauth",
            Self::Ambient => "ambient",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AgentProviderDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub env_keys: &'static [&'static str],
    pub required_env: &'static [&'static str],
    pub base_url: Option<&'static str>,
    pub default_model: &'static str,
    pub protocol: AgentProviderProtocol,
    pub supports_api_key: bool,
    pub supports_account: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProviderInfo {
    pub id: String,
    pub name: String,
    pub configured: bool,
    pub source: Option<String>,
    pub default_model: String,
    pub protocol: String,
    pub api_key_env: Vec<String>,
    pub supports_account: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentModelDefinition {
    pub id: &'static str,
    pub name: &'static str,
    /// Capability metadata sourced from the provider catalog when available.
    pub tools: Option<bool>,
    pub images: Option<bool>,
    /// Context window metadata sourced from the provider catalog when available.
    pub context_window: Option<u64>,
}

const MAX_DYNAMIC_PROVIDER_NAME: usize = 64;
const MAX_DYNAMIC_MODEL_ID: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DynamicProviderDefinition {
    pub name: String,
    pub base_url: String,
    pub protocol: AgentProviderProtocol,
    pub models: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProviderRegistry { pub providers: BTreeMap<String, DynamicProviderDefinition> }

impl ProviderRegistry {
    pub fn validate(&self) -> AgentResult<()> {
        if self.providers.len() > 64 { return Err(AgentError::new("too many dynamic providers")); }
        for (id, definition) in &self.providers {
            let name = id.strip_prefix("custom/").filter(|v| !v.is_empty()).ok_or_else(|| AgentError::new("dynamic provider ids must use custom/<name>"))?;
            validate_dynamic_name(name)?;
            if provider_exists(id) { return Err(AgentError::new(format!("dynamic provider collides with built-in `{id}`"))); }
            validate_dynamic_name(&definition.name)?;
            validate_dynamic_url(&definition.base_url)?;
            if !matches!(definition.protocol, AgentProviderProtocol::OpenAiCompletions | AgentProviderProtocol::OpenAiResponses | AgentProviderProtocol::AnthropicMessages | AgentProviderProtocol::GoogleGenerativeAi | AgentProviderProtocol::MistralConversations | AgentProviderProtocol::PiMessages) { return Err(AgentError::new("unsupported dynamic provider protocol")); }
                if definition.models.is_empty() || definition.models.len() > 256 { return Err(AgentError::new("dynamic provider must declare 1..256 models")); }
            let mut models = BTreeSet::new();
            for model in &definition.models { validate_dynamic_model(model)?; if !models.insert(model) { return Err(AgentError::new("duplicate dynamic provider model")); } }
            if let Some(name) = &definition.api_key_env { validate_env_name(name)?; }
        }
        Ok(())
    }
    pub fn definition(&self, id: &str) -> Option<&DynamicProviderDefinition> { self.providers.get(id) }
    pub fn contains_model(&self, provider: &str, model: &str) -> bool { self.definition(provider).is_some_and(|p| p.models.iter().any(|m| m == model)) }
    pub fn canonical_material(&self) -> AgentResult<String> { self.validate()?; serde_json::to_string(self).map_err(Into::into) }
    pub fn resolve_auth(&self, provider: &str, explicit_api_key: Option<&str>) -> AgentResult<Option<ResolvedProviderAuth>> {
        let definition = self.definition(provider).ok_or_else(|| unknown_provider(provider))?;
        if let Some(key) = explicit_api_key.filter(|v| !v.is_empty()) { return Ok(Some(ResolvedProviderAuth { kind: AgentAuthKind::ApiKey, secret: Some(key.into()), env: BTreeMap::new(), source: "explicit request".into() })); }
        let Some(name) = definition.api_key_env.as_deref() else { return Ok(None); };
        let Some(value) = env::var(name).ok().filter(|v| !v.is_empty()) else { return Ok(None); };
        Ok(Some(ResolvedProviderAuth { kind: AgentAuthKind::ApiKey, secret: Some(value), env: BTreeMap::new(), source: name.into() }))
    }
}
fn validate_dynamic_name(value: &str) -> AgentResult<()> { if value.is_empty() || value.len() > MAX_DYNAMIC_PROVIDER_NAME || !value.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' || c == b'_') { return Err(AgentError::new("dynamic provider names must be lowercase ASCII and bounded")); } Ok(()) }
fn validate_dynamic_model(value: &str) -> AgentResult<()> { if value.is_empty() || value.len() > MAX_DYNAMIC_MODEL_ID || value.chars().any(|c| c.is_control() || c.is_whitespace()) || value.contains("://") || value.contains('\\') { return Err(AgentError::new("invalid dynamic provider model")); } Ok(()) }
fn validate_dynamic_url(value: &str) -> AgentResult<()> { if value.chars().any(char::is_control) || value.contains('@') || value.contains('?') || value.contains('#') { return Err(AgentError::new("dynamic provider URL cannot contain credentials, query, fragment, or controls")); } let url = reqwest::Url::parse(value).map_err(|_| AgentError::new("invalid dynamic provider URL"))?; let allowed = url.scheme() == "https" || (url.scheme() == "http" && url.host_str().is_some_and(|h| matches!(h, "localhost" | "127.0.0.1" | "::1"))); if !allowed || url.host_str().is_none() { return Err(AgentError::new("dynamic provider URL must use HTTPS or loopback HTTP")); } Ok(()) }
fn validate_env_name(name: &str) -> AgentResult<()> { if name.is_empty() || !name.bytes().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_') { return Err(AgentError::new("invalid dynamic provider environment name")); } Ok(()) }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProviderAuth {
    pub kind: AgentAuthKind,
    pub secret: Option<String>,
    pub env: BTreeMap<String, String>,
    pub source: String,
}

#[allow(dead_code)]
const OPENAI_CODEX_MODELS: &[AgentModelDefinition] = &[
    AgentModelDefinition {
        id: "gpt-5.3-codex-spark",
        name: "GPT-5.3 Codex Spark",
            tools: None,
            images: None,
            context_window: None,
    },
    AgentModelDefinition {
        id: "gpt-5.4",
        name: "GPT-5.4",
            tools: None,
            images: None,
            context_window: None,
    },
    AgentModelDefinition {
        id: "gpt-5.4-mini",
        name: "GPT-5.4 mini",
            tools: None,
            images: None,
            context_window: None,
    },
    AgentModelDefinition {
        id: "gpt-5.5",
        name: "GPT-5.5",
            tools: None,
            images: None,
            context_window: None,
    },
    AgentModelDefinition {
        id: "gpt-5.6-luna",
        name: "GPT-5.6 Luna",
            tools: None,
            images: None,
            context_window: None
    },
    AgentModelDefinition {
        id: "gpt-5.6-sol",
        name: "GPT-5.6 Sol",
            tools: None,
            images: None,
            context_window: None
    },
    AgentModelDefinition {
        id: "gpt-5.6-terra",
        name: "GPT-5.6 Terra",
            tools: None,
            images: None,
            context_window: None
    },
    AgentModelDefinition {
        id: "gpt-6-astra",
        name: "GPT-6 Astra",
            tools: None,
            images: None,
            context_window: None
    },
];

const PROVIDER_IDS: &[&str] = &[
    "amazon-bedrock",
    "anthropic",
    "azure-openai-responses",
    "cloudflare-ai-gateway",
    "cloudflare-workers-ai",
    "deepseek",
    "fireworks",
    "github-copilot",
    "google",
    "google-vertex",
    "groq",
    "kimi-coding",
    "minimax",
    "minimax-cn",
    "mistral",
    "nvidia",
    "openai",
    "openai-codex",
    "opencode",
    "opencode-go",
    "openrouter",
    "qwen-token-plan",
    "qwen-token-plan-cn",
    "qwen-token-plan-individual",
    "vercel-ai-gateway",
    "xai",
    "zai",
    "zai-coding-cn",
];

pub fn builtin_provider_ids() -> &'static [&'static str] {
    PROVIDER_IDS
}

pub fn provider_definition(id: &str) -> Option<AgentProviderDefinition> {
    let definition = match id {
        "amazon-bedrock" => definition(
            "amazon-bedrock",
            "Amazon Bedrock",
            &["AWS_BEARER_TOKEN_BEDROCK"],
            &[],
            Some("https://bedrock-runtime.us-east-1.amazonaws.com"),
            "anthropic.claude-3-5-sonnet-20241022-v2:0",
            AgentProviderProtocol::BedrockConverse,
            true,
            false,
        ),
        "anthropic" => definition(
            "anthropic",
            "Anthropic",
            &[
                "ANTHROPIC_AUTH_TOKEN",
                "ANTHROPIC_OAUTH_TOKEN",
                "ANTHROPIC_API_KEY",
            ],
            &[],
            Some("https://api.anthropic.com"),
            "claude-sonnet-4-5",
            AgentProviderProtocol::AnthropicMessages,
            true,
            true,
        ),
        "azure-openai-responses" => definition(
            "azure-openai-responses",
            "Azure OpenAI",
            &["AZURE_OPENAI_API_KEY"],
            &["AZURE_OPENAI_BASE_URL"],
            None,
            "gpt-4o",
            AgentProviderProtocol::OpenAiResponses,
            true,
            false,
        ),
        "cloudflare-ai-gateway" => definition(
            "cloudflare-ai-gateway",
            "Cloudflare AI Gateway",
            &["CLOUDFLARE_API_KEY"],
            &["CLOUDFLARE_ACCOUNT_ID", "CLOUDFLARE_GATEWAY_ID"],
            Some(
                "https://gateway.ai.cloudflare.com/v1/{CLOUDFLARE_ACCOUNT_ID}/{CLOUDFLARE_GATEWAY_ID}",
            ),
            "gpt-4o",
            AgentProviderProtocol::OpenAiResponses,
            true,
            false,
        ),
        "cloudflare-workers-ai" => definition(
            "cloudflare-workers-ai",
            "Cloudflare Workers AI",
            &["CLOUDFLARE_API_KEY"],
            &["CLOUDFLARE_ACCOUNT_ID"],
            Some("https://api.cloudflare.com/client/v4/accounts/{CLOUDFLARE_ACCOUNT_ID}/ai/v1"),
            "@cf/deepseek-ai/deepseek-v4-flash-0731",
            AgentProviderProtocol::OpenAiCompletions,
            true,
            false,
        ),
        "deepseek" => definition(
            "deepseek",
            "DeepSeek",
            &["DEEPSEEK_API_KEY"],
            &[],
            Some("https://api.deepseek.com"),
            "deepseek-v4-flash",
            AgentProviderProtocol::OpenAiCompletions,
            true,
            false,
        ),
        "fireworks" => definition(
            "fireworks",
            "Fireworks",
            &["FIREWORKS_API_KEY"],
            &[],
            Some("https://api.fireworks.ai/inference"),
            "accounts/fireworks/models/deepseek-v4-flash-0731",
            AgentProviderProtocol::AnthropicMessages,
            true,
            false,
        ),
        "github-copilot" => definition(
            "github-copilot",
            "GitHub Copilot",
            &["COPILOT_GITHUB_TOKEN"],
            &[],
            Some("https://api.individual.githubcopilot.com"),
            "claude-sonnet-4-5",
            AgentProviderProtocol::AnthropicMessages,
            true,
            true,
        ),
        "google" => definition(
            "google",
            "Google",
            &["GEMINI_API_KEY"],
            &[],
            Some("https://generativelanguage.googleapis.com/v1beta"),
            "gemini-2.5-flash",
            AgentProviderProtocol::GoogleGenerativeAi,
            true,
            false,
        ),
        "google-vertex" => definition(
            "google-vertex",
            "Google Vertex AI",
            &["GOOGLE_CLOUD_API_KEY"],
            &[],
            Some("https://{location}-aiplatform.googleapis.com"),
            "gemini-2.5-flash",
            AgentProviderProtocol::GoogleVertex,
            true,
            false,
        ),
        "groq" => definition(
            "groq",
            "Groq",
            &["GROQ_API_KEY"],
            &[],
            Some("https://api.groq.com/openai/v1"),
            "llama-3.1-8b-instant",
            AgentProviderProtocol::OpenAiCompletions,
            true,
            false,
        ),
        "kimi-coding" => definition(
            "kimi-coding",
            "Kimi For Coding",
            &["KIMI_API_KEY"],
            &[],
            Some("https://api.kimi.com/coding"),
            "k3",
            AgentProviderProtocol::AnthropicMessages,
            true,
            true,
        ),
        "minimax" => definition(
            "minimax",
            "MiniMax",
            &["MINIMAX_API_KEY"],
            &[],
            Some("https://api.minimax.io/anthropic"),
            "MiniMax-M2.7",
            AgentProviderProtocol::AnthropicMessages,
            true,
            false,
        ),
        "minimax-cn" => definition(
            "minimax-cn",
            "MiniMax CN",
            &["MINIMAX_CN_API_KEY"],
            &[],
            Some("https://api.minimaxi.com/anthropic"),
            "MiniMax-M2.7",
            AgentProviderProtocol::AnthropicMessages,
            true,
            false,
        ),
        "mistral" => definition(
            "mistral",
            "Mistral",
            &["MISTRAL_API_KEY"],
            &[],
            Some("https://api.mistral.ai/v1"),
            "mistral-large-latest",
            AgentProviderProtocol::MistralConversations,
            true,
            false,
        ),
        "nvidia" => definition(
            "nvidia",
            "NVIDIA",
            &["NVIDIA_API_KEY"],
            &[],
            Some("https://integrate.api.nvidia.com/v1"),
            "deepseek-ai/deepseek-v4-flash-0731",
            AgentProviderProtocol::OpenAiCompletions,
            true,
            false,
        ),
        "openai" => definition(
            "openai",
            "OpenAI",
            &["OPENAI_API_KEY"],
            &[],
            Some("https://api.openai.com/v1"),
            "gpt-5.5",
            AgentProviderProtocol::OpenAiResponses,
            true,
            false,
        ),
        "openai-codex" => definition(
            "openai-codex",
            "OpenAI Codex",
            &[],
            &[],
            Some("https://chatgpt.com/backend-api"),
            "gpt-5.5",
            AgentProviderProtocol::OpenAiResponses,
            false,
            true,
        ),
        "opencode" => definition(
            "opencode",
            "OpenCode Zen",
            &["OPENCODE_API_KEY"],
            &[],
            Some("https://opencode.ai/zen"),
            "claude-sonnet-4-5",
            AgentProviderProtocol::AnthropicMessages,
            true,
            false,
        ),
        "opencode-go" => definition(
            "opencode-go",
            "OpenCode Go",
            &["OPENCODE_API_KEY"],
            &[],
            Some("https://opencode.ai/zen/go"),
            "minimax-m3",
            AgentProviderProtocol::AnthropicMessages,
            true,
            false,
        ),
        "openrouter" => definition(
            "openrouter",
            "OpenRouter",
            &["OPENROUTER_API_KEY"],
            &[],
            Some("https://openrouter.ai/api/v1"),
            "openai/gpt-5.5",
            AgentProviderProtocol::OpenAiCompletions,
            true,
            true,
        ),
        "qwen-token-plan" => definition(
            "qwen-token-plan",
            "Qwen Token Plan",
            &["QWEN_TOKEN_PLAN_API_KEY"],
            &[],
            Some("https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1"),
            "MiniMax-M2.5",
            AgentProviderProtocol::OpenAiCompletions,
            true,
            false,
        ),
        "qwen-token-plan-cn" => definition(
            "qwen-token-plan-cn",
            "Qwen Token Plan CN",
            &["QWEN_TOKEN_PLAN_CN_API_KEY"],
            &[],
            Some("https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1"),
            "MiniMax-M2.5",
            AgentProviderProtocol::OpenAiCompletions,
            true,
            false,
        ),
        "qwen-token-plan-individual" => definition(
            "qwen-token-plan-individual",
            "Qwen Token Plan Individual",
            &["QWEN_TOKEN_PLAN_API_KEY"],
            &[],
            Some("https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1"),
            "deepseek-v4-flash-0731",
            AgentProviderProtocol::OpenAiCompletions,
            true,
            false,
        ),
        "vercel-ai-gateway" => definition(
            "vercel-ai-gateway",
            "Vercel AI Gateway",
            &["AI_GATEWAY_API_KEY"],
            &[],
            Some("https://ai-gateway.vercel.sh"),
            "anthropic/claude-sonnet-4-5",
            AgentProviderProtocol::AnthropicMessages,
            true,
            false,
        ),
        "xai" => definition(
            "xai",
            "xAI",
            &["XAI_API_KEY"],
            &[],
            Some("https://api.x.ai/v1"),
            "grok-4.3",
            AgentProviderProtocol::OpenAiResponses,
            true,
            true,
        ),
        "zai" => definition(
            "zai",
            "Z.AI",
            &["ZAI_API_KEY"],
            &[],
            Some("https://api.z.ai/api/coding/paas/v4"),
            "glm-4.7",
            AgentProviderProtocol::OpenAiCompletions,
            true,
            false,
        ),
        "zai-coding-cn" => definition(
            "zai-coding-cn",
            "Z.AI Coding CN",
            &["ZAI_CODING_CN_API_KEY"],
            &[],
            Some("https://open.bigmodel.cn/api/coding/paas/v4"),
            "glm-4.6v",
            AgentProviderProtocol::OpenAiCompletions,
            true,
            false,
        ),
        _ => return None,
    };
    Some(definition)
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AgentCredential;
    use std::fs;

    #[test]
    fn exposes_pi_provider_order_and_all_catalog_entries() {
        assert_eq!(builtin_provider_ids().len(), 28);
        assert_eq!(builtin_provider_ids()[0], "amazon-bedrock");
        assert_eq!(builtin_provider_ids()[27], "zai-coding-cn");
        for retired in [
            "ant-ling", "baseten", "cerebras", "huggingface", "moonshotai",
            "moonshotai-cn", "radius", "together", "xiaomi", "xiaomi-token-plan-ams",
            "xiaomi-token-plan-cn", "xiaomi-token-plan-sgp",
        ] {
            assert!(!builtin_provider_ids().contains(&retired));
            assert!(provider_definition(retired).is_none(), "{retired}");
        }
        assert_eq!(provider_models("openai-codex").len(), 8);
        assert_eq!(provider_models("openai-codex")[0].id, "gpt-5.3-codex-spark");
        assert_eq!(provider_default_model("openai-codex").unwrap(), "gpt-5.5");
        assert!(
            !provider_models("openai-codex")
                .iter()
                .any(|model| model.id == "gpt-5.3-codex")
        );
    }

    #[test]
    fn mistral_definition_uses_api_key_and_conversations_transport() {
        let definition = provider_definition("mistral").expect("provider");
        assert_eq!(definition.name, "Mistral");
        assert_eq!(definition.env_keys, &["MISTRAL_API_KEY"]);
        assert_eq!(definition.base_url, Some("https://api.mistral.ai/v1"));
        assert_eq!(definition.default_model, "mistral-large-latest");
        assert_eq!(definition.protocol, AgentProviderProtocol::MistralConversations);
        assert!(definition.supports_api_key);
        assert!(!definition.supports_account);

        let root = tempfile::tempdir().expect("root");
        let store = AgentAuthStore::new(root.path().join("auth.json"));
        let resolved = resolve_provider_auth(&definition, &store, Some("mistral-test-key"), None)
            .expect("resolve")
            .expect("auth");
        assert_eq!(resolved.kind, AgentAuthKind::ApiKey);
        assert_eq!(resolved.secret.as_deref(), Some("mistral-test-key"));
    }

    #[test]
    fn stored_credentials_win_over_environment_values() {
        let root = tempfile::tempdir().expect("root");
        let path = root.path().join("auth.json");
        let store = AgentAuthStore::new(&path);
        store
            .save("openai", &AgentCredential::api_key("stored"))
            .expect("save");
        let definition = provider_definition("openai").expect("provider");
        let resolved = resolve_provider_auth(&definition, &store, None, None)
            .expect("resolve")
            .expect("auth");
        assert_eq!(resolved.secret.as_deref(), Some("stored"));
        assert!(!fs::read_to_string(path).expect("auth").is_empty());
    }

    #[test]
    fn explicit_model_selects_openrouter_anthropic_transport() {
        let definition = provider_definition("openrouter").expect("provider");
        assert_eq!(
            protocol_for_model(&definition, "anthropic/claude-sonnet-4"),
            AgentProviderProtocol::AnthropicMessages
        );
        assert_eq!(
            protocol_for_model(&definition, "openai/gpt-5.5"),
            AgentProviderProtocol::OpenAiCompletions
        );
        assert_eq!(
            normalize_model_id("fireworks", "accounts/fireworks/models/model"),
            "accounts/fireworks/models/model"
        );
        assert_eq!(normalize_model_id("openai", "openai/gpt-5.5"), "gpt-5.5");
    }
}
