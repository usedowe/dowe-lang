use dowe_agent::native_harness::{
    HarnessConfig, HarnessRole, ModelCapabilities, ModelSelection, builtin_model_capabilities,
};

#[test]
fn official_provider_snapshots_cover_tools_and_native_image_inputs() {
    for (provider, models, images) in [
        (
            "anthropic",
            &[
                "claude-fable-5-1",
                "claude-opus-5",
                "claude-sonnet-5",
                "claude-haiku-4-5-20251001",
                "claude-haiku-4-5",
                "claude-sonnet-4-5-20250929",
                "claude-sonnet-4-5",
            ][..],
            true,
        ),
        (
            "google",
            &["gemini-2.5-flash", "gemini-3.8-flash"][..],
            true,
        ),
        ("minimax", &["MiniMax-M3"][..], true),
        (
            "minimax",
            &[
                "MiniMax-M2.7",
                "MiniMax-M2.7-highspeed",
                "MiniMax-M2.5",
                "MiniMax-M2.5-highspeed",
                "MiniMax-M2.1",
                "MiniMax-M2.1-highspeed",
                "MiniMax-M2",
            ][..],
            false,
        ),
    ] {
        for model in models {
            assert_eq!(
                builtin_model_capabilities(provider, model),
                Some(ModelCapabilities {
                    tools: true,
                    images
                }),
                "{provider}/{model}"
            );
            assert_eq!(
                builtin_model_capabilities(provider, &format!("{provider}/{model}")),
                builtin_model_capabilities(provider, model)
            );
        }
    }
}

#[test]
fn explicit_negative_evidence_blocks_tools_and_images_without_fallback() {
    let config = HarnessConfig::default();
    assert_eq!(
        builtin_model_capabilities("google", "gemini-3.1-flash-tts-preview"),
        Some(ModelCapabilities {
            tools: false,
            images: false
        })
    );
    assert!(
        config
            .require_capabilities(
                &ModelSelection::new("google", "gemini-3.1-flash-tts-preview"),
                HarnessRole::Execute,
                false
            )
            .unwrap_err()
            .to_string()
            .contains("does not support harness tools")
    );
    assert!(
        config
            .require_capabilities(
                &ModelSelection::new("minimax", "MiniMax-M2.7"),
                HarnessRole::Review,
                true
            )
            .unwrap_err()
            .to_string()
            .contains("no attachment sent")
    );
    assert!(
        config
            .require_capabilities(
                &ModelSelection::new("minimax", "MiniMax-M2.7"),
                HarnessRole::Execute,
                false
            )
            .is_ok()
    );
}

#[test]
fn similar_names_and_other_provider_routes_do_not_inherit_evidence() {
    for (provider, model) in [
        ("anthropic", "claude-sonnet-4-5-20259999"),
        ("google", "gemini-3.8-flash-latest"),
        ("minimax", "MiniMax-M3-unverified"),
        ("minimax", "minimax-m3"),
        ("minimax-cn", "MiniMax-M3"),
        ("google-vertex", "gemini-2.5-flash"),
        ("openrouter", "anthropic/claude-sonnet-4-5"),
        (
            "amazon-bedrock",
            "anthropic.claude-sonnet-4-5-20250929-v1:0",
        ),
    ] {
        assert_eq!(
            builtin_model_capabilities(provider, model),
            None,
            "{provider}/{model}"
        );
        assert!(
            HarnessConfig::default()
                .require_capabilities(
                    &ModelSelection::new(provider, model),
                    HarnessRole::Execute,
                    false
                )
                .is_err()
        );
    }
}

#[test]
fn local_declarations_override_verified_builtins_until_explicitly_removed() {
    let mut config = HarnessConfig::default();
    let selected = ModelSelection::new("anthropic", "claude-sonnet-4-5");
    config.capabilities.insert(
        "anthropic/claude-sonnet-4-5".into(),
        ModelCapabilities {
            tools: false,
            images: false,
        },
    );
    assert!(
        config
            .require_capabilities(&selected, HarnessRole::Execute, false)
            .is_err()
    );
    config.capabilities.remove("anthropic/claude-sonnet-4-5");
    assert!(
        config
            .require_capabilities(&selected, HarnessRole::Execute, true)
            .is_ok()
    );
}
