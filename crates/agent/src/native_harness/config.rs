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
}

impl HarnessRole {
    pub fn parse(value: &str) -> AgentResult<Self> {
        serde_json::from_value(serde_json::Value::String(value.into()))
            .map_err(|_| AgentError::new("role must be plan, execute, compact or review"))
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
        if !crate::provider_exists(&self.provider)
            || self.model.trim().is_empty()
            || self.model.chars().any(char::is_control)
        {
            return Err(AgentError::new("invalid harness provider/model selection"));
        }
        validate_agent_model(&self.provider, &self.model)?;
        if let Some(level) = self.thinking {
            validate_thinking(&self.provider, &self.model, level)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HarnessConfig {
    pub roles: BTreeMap<HarnessRole, ModelSelection>,
    pub capabilities: BTreeMap<String, super::ModelCapabilities>,
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
            shell: None,
            max_rounds: 16,
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
    pub fn validate(&self) -> AgentResult<()> {
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
            ModelSelection::new(provider, model).validate()?;
        }
        for selection in self.roles.values() {
            selection.validate()?;
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
        selected.validate()?;
        Ok(selected.clone())
    }
}
