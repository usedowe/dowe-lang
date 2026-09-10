
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

