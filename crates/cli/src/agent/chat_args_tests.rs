#[derive(Clone)]
struct ParsedAgentChatArgs {
    explicit_model: bool,
    prompt: String,
    provider: Option<String>,
    api_key: Option<String>,
    server_url: String,
    uses_legacy_server: bool,
    json_output: bool,
    options: AgentPrepareOptions,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_cli_defaults_to_conversation_without_changing_structured_modes() {
        for args in [
            vec![],
            vec!["hola".into()],
            vec!["--json".into(), "hola".into()],
        ] {
            let parsed = parse_agent_args(&args, false).unwrap();
            assert_eq!(
                parsed.options.request_type,
                Some(AgentRequestType::Conversation)
            );
        }
        let structured = parse_agent_args(
            &["--request-type".into(), "clarify".into(), "hola".into()],
            true,
        )
        .unwrap();
        assert_eq!(
            structured.options.request_type,
            Some(AgentRequestType::Clarify)
        );
        let legacy = parse_agent_args(
            &[
                "--server".into(),
                "http://localhost:1234".into(),
                "hola".into(),
            ],
            true,
        )
        .unwrap();
        assert!(legacy.uses_legacy_server);
        assert_eq!(legacy.options.request_type, None);
    }

    #[test]
    fn menu_labels_align_and_truncate_without_controls() {
        let labels = aligned_columns(
            &["Short".into(), "A very long provider name".into()],
            &["stored: env".into(), "unconfigured".into()],
            32,
        );
        assert_eq!(labels.len(), 2);
        assert!(labels.iter().all(|label| measure_text_width(label) <= 32));
        assert_eq!(
            measure_text_width(&labels[0]),
            measure_text_width(&labels[1])
        );
        assert!(!labels.iter().any(|label| label.contains('\n')));
    }

    #[test]
    fn menu_items_preserve_values_while_padding_display_labels() {
        let labels = aligned_menu_items(&["low", "very long\tlevel"], 20);
        assert_eq!(labels[0].trim(), "low");
        assert_eq!(labels[1].trim(), "very long level");
        assert_eq!(
            measure_text_width(&labels[0]),
            measure_text_width(&labels[1])
        );
    }

    #[test]
    fn only_exit_is_an_interactive_exit_command() {
        assert!(is_exit_command("/exit"));
        assert!(!is_exit_command("/quit"));
        assert!(!is_exit_command(":q"));
    }

    #[test]
    fn empty_auth_store_does_not_expose_ambient_only_providers() {
        let root = tempfile::tempdir().unwrap();
        let auth = AgentAuthStore::new(root.path().join("auth.json"));

        let providers = configured_model_providers(&auth).unwrap();
        assert!(
            !providers
                .iter()
                .any(|provider| provider == "amazon-bedrock")
        );
        assert!(!providers.iter().any(|provider| provider == "google-vertex"));

        let entries = model_menu_entries(&providers).unwrap();
        assert!(
            !entries
                .iter()
                .any(|entry| entry.provider == "amazon-bedrock")
        );
        assert!(
            !entries
                .iter()
                .any(|entry| entry.provider == "google-vertex")
        );
    }

    #[test]
    fn interactive_provider_lists_only_codex_and_openrouter() {
        let root = tempfile::tempdir().unwrap();
        let auth = AgentAuthStore::new(root.path().join("auth.json"));
        auth.save("anthropic", &AgentCredential::api_key("anthropic-secret"))
            .unwrap();
        auth.save("openai-codex", &AgentCredential::api_key("codex-secret"))
            .unwrap();
        auth.save("openrouter", &AgentCredential::api_key("router-secret"))
            .unwrap();

        let providers = interactive_provider_info(&auth).unwrap();
        assert_eq!(
            providers
                .iter()
                .map(|provider| provider.id.as_str())
                .collect::<Vec<_>>(),
            ["openai-codex", "openrouter"]
        );
        assert_eq!(
            configured_model_providers(&auth).unwrap(),
            ["openai-codex", "openrouter"]
        );
    }

    #[test]
    fn model_menu_aggregates_configured_providers_without_credentials() {
        let root = tempfile::tempdir().unwrap();
        let auth = AgentAuthStore::new(root.path().join("auth.json"));
        auth.save("openai-codex", &AgentCredential::api_key("codex-secret"))
            .unwrap();
        auth.save("openrouter", &AgentCredential::api_key("router-secret"))
            .unwrap();

        let providers = configured_model_providers(&auth).unwrap();
        assert!(providers.iter().any(|provider| provider == "openai-codex"));
        assert!(providers.iter().any(|provider| provider == "openrouter"));
        let entries = model_menu_entries(&providers).unwrap();
        assert!(entries.iter().any(|entry| {
            entry.provider == "openai-codex"
                && entry.model.as_deref() == Some("gpt-5.5")
                && entry.label.contains("openai-codex")
                && entry.label.contains("gpt-5.5")
        }));
        assert!(entries.iter().any(|entry| {
            entry.provider == "openrouter"
                && entry.model.as_deref() == Some("openai/gpt-5.5")
                && entry.label.contains("openrouter")
                && entry.label.contains("openai/gpt-5.5")
        }));
        assert!(
            entries
                .iter()
                .any(|entry| entry.provider == "openai-codex" && entry.model.is_none())
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.provider == "openrouter" && entry.model.is_none())
        );
        assert!(!entries.iter().any(|entry| entry.label.contains("secret")));
    }

    #[test]
    fn selecting_aggregated_entry_persists_provider_and_model_together() {
        let root = tempfile::tempdir().unwrap();
        let preferences = AgentPreferencesStore::new(root.path().join("preferences.json"));
        preferences.select_model("openai-codex", "gpt-5.5").unwrap();

        let selected = SelectedModel {
            provider: "openrouter".to_string(),
            model: "anthropic/claude-sonnet-4".to_string(),
        };
        preferences
            .select_model(&selected.provider, &selected.model)
            .unwrap();
        let saved = preferences.read().unwrap();
        assert_eq!(saved.provider.as_deref(), Some("openrouter"));
        assert_eq!(saved.model.as_deref(), Some("anthropic/claude-sonnet-4"));
    }

    #[test]
    fn agent_provider_precedence_keeps_explicit_overrides_temporary() {
        let home = tempfile::tempdir().unwrap();
        let auth = AgentAuthStore::new(home.path().join("auth.json"));
        let preferences = AgentPreferencesStore::new(home.path().join("preferences.json"));
        auth.save("anthropic", &AgentCredential::api_key("test-key"))
            .unwrap();
        let parsed = parse_agent_args(&[], false).unwrap();
        assert_eq!(
            preferred_request_provider(&parsed, &auth, &preferences)
                .unwrap()
                .as_deref(),
            Some("anthropic")
        );
        preferences.select_provider("openai-codex").unwrap();
        assert_eq!(
            preferred_request_provider(&parsed, &auth, &preferences)
                .unwrap()
                .as_deref(),
            Some("openai-codex")
        );
        for (args, expected) in [
            (vec!["--provider", "google"], "google"),
            (vec!["--model", "anthropic/claude-test"], "anthropic"),
            (
                vec!["--provider", "google", "--model", "anthropic/claude-test"],
                "google",
            ),
        ] {
            let args = args.into_iter().map(str::to_string).collect::<Vec<_>>();
            let parsed = parse_agent_args(&args, false).unwrap();
            assert_eq!(
                preferred_request_provider(&parsed, &auth, &preferences)
                    .unwrap()
                    .as_deref(),
                Some(expected)
            );
            assert_eq!(
                preferences.provider().unwrap().as_deref(),
                Some("openai-codex")
            );
        }
    }
}
