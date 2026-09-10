use crate::{AgentError, AgentResult, ThinkingLevel, validate_agent_model, validate_thinking};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessRole {
    Plan,
    #[default]
    Execute,
    Compact,
    Review,
    Research,
        ImageGeneration,
    Codegraph,
}

impl HarnessRole {
    pub fn parse(value: &str) -> AgentResult<Self> {
        serde_json::from_value(serde_json::Value::String(value.into()))
            .map_err(|_| AgentError::new("role must be plan, execute, compact, review, research, image_generation or codegraph"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codegraph_role_parses_and_serializes_as_snake_case() {
        assert_eq!(HarnessRole::parse("codegraph").unwrap(), HarnessRole::Codegraph);
        assert_eq!(serde_json::to_string(&HarnessRole::Codegraph).unwrap(), "\"codegraph\"");
        assert_eq!(HarnessRole::parse("research").unwrap(), HarnessRole::Research);
        assert_eq!(HarnessRole::parse("image_generation").unwrap(), HarnessRole::ImageGeneration);
    }

    #[test]
    fn codegraph_resolves_configured_selection_then_active_fallback() {
        let active = ModelSelection::new("openai", "gpt-5.5");
        let configured = ModelSelection::new("openai-codex", "gpt-5.3-codex-spark");
        let mut config = HarnessConfig::default();
        config.roles.insert(HarnessRole::Codegraph, configured.clone());

        assert_eq!(config.resolve(HarnessRole::Codegraph, None, &active).unwrap(), configured);

        config.roles.remove(&HarnessRole::Codegraph);
        assert_eq!(config.resolve(HarnessRole::Codegraph, None, &active).unwrap(), active);
    }

    #[test]
    fn recommended_roles_keep_expensive_work_explicit_and_read_roles_cheap() {
        let roles = HarnessConfig::recommended_roles();
        assert_eq!(
            roles[&HarnessRole::Plan].model,
            "gpt-6-astra"
        );
        assert_eq!(
            roles[&HarnessRole::Execute].model,
            "gpt-5.6-luna"
        );
        assert_eq!(
            roles[&HarnessRole::Compact].model,
            "deepseek/deepseek-v4-flash"
        );
        assert_eq!(roles[&HarnessRole::Research].provider, "openrouter");
        assert_eq!(roles[&HarnessRole::Codegraph].provider, "openrouter");
    }

    #[test]
    fn default_round_limit_is_pi_like_and_hard_limit_remains_validated() {
        let config = HarnessConfig::default();
        assert_eq!(config.max_rounds, 64);
        assert!(config.validate().is_ok());

        let mut maximum = config.clone();
        maximum.max_rounds = 128;
        assert!(maximum.validate().is_ok());

        maximum.max_rounds = 129;
        assert!(maximum.validate().is_err());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelSelection {
    pub provider: String,
    pub model: String,
    #[serde(default)]
    pub thinking: Option<ThinkingLevel>,
}

impl ModelSelection {
    pub fn new(provider: &str, model: &str) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            thinking: None,
        }
    }

    pub fn validate(&self) -> AgentResult<()> {
        let registry = crate::provider::ProviderRegistry::default();
            if (!crate::provider_exists(&self.provider) && registry.definition(&self.provider).is_none())
            || self.model.trim().is_empty()
            || self.model.chars().any(char::is_control)
        {
            return Err(AgentError::new("invalid harness provider/model selection"));
        }
        if registry.definition(&self.provider).is_some() && !registry.contains_model(&self.provider, &self.model) { return Err(AgentError::new("model is not declared by the dynamic provider")); }
        validate_agent_model(&self.provider, &self.model)?;
        if let Some(level) = self.thinking {
            validate_thinking(&self.provider, &self.model, level)?;
        }
        Ok(())
    }
}

impl ModelSelection {
    pub fn validate_with_registry(&self, registry: &crate::provider::ProviderRegistry) -> AgentResult<()> {
        if (!crate::provider_exists(&self.provider) && registry.definition(&self.provider).is_none()) || self.model.trim().is_empty() || self.model.chars().any(char::is_control) { return Err(AgentError::new("invalid harness provider/model selection")); }
        if registry.definition(&self.provider).is_some() && !registry.contains_model(&self.provider, &self.model) { return Err(AgentError::new("model is not declared by the dynamic provider")); }
        validate_agent_model(&self.provider, &self.model)?;
        if let Some(level) = self.thinking { validate_thinking(&self.provider, &self.model, level)?; }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HarnessConfig {
    pub roles: BTreeMap<HarnessRole, ModelSelection>,
    pub capabilities: BTreeMap<String, super::ModelCapabilities>,
    pub providers: crate::provider::ProviderRegistry,
    pub shell: Option<String>,
    pub max_rounds: usize,
    pub max_output_bytes: usize,
    pub context_limit: Option<u64>,
    pub token_budget: u64,
    pub cost_budget_usd: Option<f64>,
    pub duration_seconds: u64,
    pub shell_timeout_ms: u64,
    pub memory_enabled: bool,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            roles: BTreeMap::new(),
            capabilities: BTreeMap::new(),
                providers: crate::provider::ProviderRegistry::default(),
            shell: None,
            max_rounds: 64,
            max_output_bytes: 32768,
            context_limit: None,
            token_budget: 200000,
            cost_budget_usd: None,
            duration_seconds: 900,
            shell_timeout_ms: 120000,
            memory_enabled: true,
        }
    }
}

impl HarnessConfig {
    /// Model assignments tuned for the native task pipeline. Applying this
    /// profile is explicit; an unavailable provider remains an actionable
    /// authentication error and is never replaced silently.
    pub fn recommended_roles() -> BTreeMap<HarnessRole, ModelSelection> {
        let mut plan = ModelSelection::new("openai-codex", "gpt-6-astra");
        plan.thinking = Some(ThinkingLevel::Medium);
        let mut execute = ModelSelection::new("openai-codex", "gpt-5.6-luna");
        execute.thinking = Some(ThinkingLevel::Medium);
        let compact = ModelSelection::new("openrouter", "deepseek/deepseek-v4-flash");
        BTreeMap::from([
            (HarnessRole::Plan, plan.clone()),
            (HarnessRole::Execute, execute),
            (HarnessRole::Compact, compact.clone()),
            (HarnessRole::Review, plan),
            (HarnessRole::Research, compact.clone()),
            (HarnessRole::Codegraph, compact),
        ])
    }

    pub fn validate(&self) -> AgentResult<()> {
        self.providers.validate()?;
        if !(1..=128).contains(&self.max_rounds)
            || !(1024..=131072).contains(&self.max_output_bytes)
            || self.token_budget == 0
            || self.duration_seconds == 0
            || !(1..=900000).contains(&self.shell_timeout_ms)
            || self.context_limit.is_some_and(|limit| limit < 8192)
            || self
                .cost_budget_usd
                .is_some_and(|cost| !cost.is_finite() || cost <= 0.0)
        {
            return Err(AgentError::new(
                "invalid harness budget; context must be at least 8192 tokens",
            ));
        }
        if let Some(shell) = &self.shell {
            let path = std::path::Path::new(shell);
            if !path.is_absolute() || !path.is_file() || shell.chars().any(char::is_control) {
                return Err(AgentError::new(
                    "configure an absolute installed shell executable",
                ));
            }
        }
        if self.capabilities.len() > 256 {
            return Err(AgentError::new("too many model capability declarations"));
        }
        for key in self.capabilities.keys() {
            let (provider, model) = key
                .split_once('/')
                .ok_or_else(|| AgentError::new("capability keys must be provider/model"))?;
            ModelSelection::new(provider, model).validate_with_registry(&self.providers)?;
        }
        for selection in self.roles.values() {
            selection.validate_with_registry(&self.providers)?
        }
        Ok(())
    }

    pub fn resolve(
        &self,
        role: HarnessRole,
        explicit: Option<&ModelSelection>,
        active: &ModelSelection,
    ) -> AgentResult<ModelSelection> {
        let selected = explicit.or_else(|| self.roles.get(&role)).unwrap_or(active);
        selected.validate_with_registry(&self.providers)?;
        Ok(selected.clone())
    }
}
