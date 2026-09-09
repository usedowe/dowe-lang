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

pub fn builtin_image_generation_capability(provider: &str, model: &str) -> bool {
    provider == "openai" && crate::normalize_model_id(provider, model) == "gpt-image-1"
}

impl HarnessConfig {
    pub fn require_image_generation_capability(
        &self,
        selection: &ModelSelection,
        role: HarnessRole,
    ) -> AgentResult<()> {
        if !matches!(role, HarnessRole::Execute | HarnessRole::ImageGeneration) {
            return Err(AgentError::new(
                "image generation is available only to the Execute role or image_generation model selection",
            ));
        }
        if !builtin_image_generation_capability(&selection.provider, &selection.model) {
            return Err(AgentError::new(format!(
                "image generation capability is unknown or unsupported for {}/{}; select openai/gpt-image-1",
                selection.provider, selection.model
            )));
        }
        Ok(())
    }

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
    fn image_generation_is_distinct_from_vision_and_tools() {
        let config = HarnessConfig::default();
        let generator = ModelSelection::new("openai", "gpt-image-1");
        assert!(
            config
                .require_image_generation_capability(&generator, HarnessRole::Execute)
                .is_ok()
        );
        let vision = ModelSelection::new("openai", "gpt-5.5");
        assert!(
            config
                .require_image_generation_capability(&vision, HarnessRole::Execute)
                .is_err()
        );
        let unknown = ModelSelection::new("openai", "custom-image-model");
        assert!(
            config
                .require_image_generation_capability(&unknown, HarnessRole::Execute)
                .is_err()
        );
    }

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
