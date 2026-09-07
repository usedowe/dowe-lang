use super::{HarnessConfig, HarnessRole, ModelSelection};
use crate::{AgentError, AgentResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCapabilities {
    pub tools: bool,
    pub images: bool,
}

pub fn builtin_model_capabilities(provider: &str, model: &str) -> Option<ModelCapabilities> {
    super::builtin_capability_evidence(provider, model).map(|entry| entry.capabilities)
}

impl HarnessConfig {
    pub fn require_capabilities(
        &self,
        selection: &ModelSelection,
        role: HarnessRole,
        images: bool,
    ) -> AgentResult<()> {
        if role != HarnessRole::Execute && !images {
            return Ok(());
        }
        let key = format!(
            "{}/{}",
            selection.provider,
            crate::normalize_model_id(&selection.provider, &selection.model)
        );
        let capabilities = self.capabilities.get(&key).copied().or_else(|| builtin_model_capabilities(&selection.provider, &selection.model))
            .ok_or_else(|| AgentError::new(format!("model capabilities are unknown for {key}; explicitly declare /capabilities {key} <tools:true|false> <images:true|false> or select a supported model")))?;
        if role == HarnessRole::Execute && !capabilities.tools {
            return Err(AgentError::new(
                "selected model does not support harness tools",
            ));
        }
        if images && !capabilities.images {
            return Err(AgentError::new(
                "selected model does not support images; no attachment sent",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_capabilities_require_explicit_declaration_and_text_models_reject_images() {
        let mut config = HarnessConfig::default();
        let custom = ModelSelection::new("azure-openai-responses", "deployment");
        assert!(
            config
                .require_capabilities(&custom, HarnessRole::Execute, false)
                .is_err()
        );
        config.capabilities.insert(
            "azure-openai-responses/deployment".into(),
            ModelCapabilities {
                tools: true,
                images: false,
            },
        );
        assert!(
            config
                .require_capabilities(&custom, HarnessRole::Execute, false)
                .is_ok()
        );
        assert!(
            config
                .require_capabilities(&custom, HarnessRole::Execute, true)
                .is_err()
        );
        assert!(
            config
                .require_capabilities(
                    &ModelSelection::new("openai-codex", "gpt-5.3-codex-spark"),
                    HarnessRole::Execute,
                    true
                )
                .is_err()
        );
        assert!(
            config
                .require_capabilities(
                    &ModelSelection::new("openai", "gpt-5.5"),
                    HarnessRole::Execute,
                    true
                )
                .is_ok()
        );
    }
}
