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

