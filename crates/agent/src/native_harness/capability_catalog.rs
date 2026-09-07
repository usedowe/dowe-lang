use super::ModelCapabilities;
use serde::Serialize;

pub const CAPABILITY_CATALOG_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CapabilityEvidence {
    pub reference: &'static str,
    pub sha256: &'static str,
    pub retrieved_at: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct BuiltinCapabilityRecord {
    pub provider: &'static str,
    pub models: &'static [&'static str],
    pub capabilities: ModelCapabilities,
    pub tools_evidence: CapabilityEvidence,
    pub images_evidence: CapabilityEvidence,
}

const OPENAI: CapabilityEvidence = CapabilityEvidence {
    reference: "npm:@earendil-works/pi-ai@0.85.1/dist/providers/data/openai.json",
    sha256: "775d3b7d95d14b741d0b287b1153d40f051a9ea03035a82b487889acc7fa69e2",
    retrieved_at: None,
};
const CODEX: CapabilityEvidence = CapabilityEvidence {
    reference: "npm:@earendil-works/pi-ai@0.85.1/dist/providers/data/openai-codex.json",
    sha256: "a10bfcfd34db6bcb98d8ee46175e154cab30530a137dbf31ac1434020ed3ffdd",
    retrieved_at: None,
};
const CLAUDE: CapabilityEvidence = official(
    "https://platform.claude.com/docs/en/about-claude/models/overview.md",
    "b2ae292320b5d068b20c426e6e4e0333d1a11e25c8457748ca39de25de283c27",
);
const SONNET: CapabilityEvidence = official(
    "https://platform.claude.com/docs/en/models/sonnet-4-5/overview.md",
    "5dbd3ea4fe536c31cebfdb336087e15ff20cf2e2ae4a222b3669ce0ba1a3bf27",
);
const CLAUDE_TOOLS: CapabilityEvidence = official(
    "https://platform.claude.com/docs/en/agents-and-tools/tool-use/overview.md",
    "87e737a50fa73fe6b55fad499a2b23f16e6c4e1fffad818197e0c112ecb5aae7",
);
const GOOGLE_FLASH: CapabilityEvidence = official(
    "https://ai.google.dev/gemini-api/docs/models/gemini-3.8-flash",
    "a35b8d4eb8b5215b59ababf44c971c910f32e078d93a7ad9e243716729f346de",
);
const GOOGLE_DEFAULT: CapabilityEvidence = official(
    "https://ai.google.dev/gemini-api/docs/models/gemini-2.5-flash",
    "261612ab278d7a59a7160fe4a209c566930456a41f5994538f2f25f4d914b944",
);
const GOOGLE_TTS: CapabilityEvidence = official(
    "https://ai.google.dev/gemini-api/docs/models/gemini-3.1-flash-tts-preview",
    "3387f82aedcce8ed34a0dad29d8b7e26dc97686e7bb6b3b8cbead677b8fb4cae",
);
const MINIMAX: CapabilityEvidence = official(
    "https://platform.minimax.io/docs/api-reference/text-anthropic-api",
    "21d38affed0e20d9125e247cf88d358ca3d34268f16b2bbdfb52d8cf8a184657",
);

const fn official(reference: &'static str, sha256: &'static str) -> CapabilityEvidence {
    CapabilityEvidence {
        reference,
        sha256,
        retrieved_at: Some("2026-09-07"),
    }
}

const fn record(
    provider: &'static str,
    models: &'static [&'static str],
    tools: bool,
    images: bool,
    tools_evidence: CapabilityEvidence,
    images_evidence: CapabilityEvidence,
) -> BuiltinCapabilityRecord {
    BuiltinCapabilityRecord {
        provider,
        models,
        capabilities: ModelCapabilities { tools, images },
        tools_evidence,
        images_evidence,
    }
}

const CATALOG: &[BuiltinCapabilityRecord] = &[
    record(
        "openai",
        &["gpt-5.4", "gpt-5.4-mini", "gpt-5.5"],
        true,
        true,
        OPENAI,
        OPENAI,
    ),
    record(
        "openai-codex",
        &["gpt-5.3-codex-spark"],
        true,
        false,
        CODEX,
        CODEX,
    ),
    record(
        "openai-codex",
        &[
            "gpt-5.4",
            "gpt-5.4-mini",
            "gpt-5.5",
            "gpt-5.6-luna",
            "gpt-5.6-sol",
            "gpt-5.6-terra",
            "gpt-6-astra",
        ],
        true,
        true,
        CODEX,
        CODEX,
    ),
    record(
        "anthropic",
        &[
            "claude-fable-5-1",
            "claude-opus-5",
            "claude-sonnet-5",
            "claude-haiku-4-5-20251001",
            "claude-haiku-4-5",
        ],
        true,
        true,
        CLAUDE,
        CLAUDE,
    ),
    record(
        "anthropic",
        &["claude-sonnet-4-5-20250929", "claude-sonnet-4-5"],
        true,
        true,
        CLAUDE_TOOLS,
        SONNET,
    ),
    record(
        "google",
        &["gemini-2.5-flash"],
        true,
        true,
        GOOGLE_DEFAULT,
        GOOGLE_DEFAULT,
    ),
    record(
        "google",
        &["gemini-3.8-flash"],
        true,
        true,
        GOOGLE_FLASH,
        GOOGLE_FLASH,
    ),
    record(
        "google",
        &["gemini-3.1-flash-tts-preview"],
        false,
        false,
        GOOGLE_TTS,
        GOOGLE_TTS,
    ),
    record("minimax", &["MiniMax-M3"], true, true, MINIMAX, MINIMAX),
    record(
        "minimax",
        &[
            "MiniMax-M2.7",
            "MiniMax-M2.7-highspeed",
            "MiniMax-M2.5",
            "MiniMax-M2.5-highspeed",
            "MiniMax-M2.1",
            "MiniMax-M2.1-highspeed",
            "MiniMax-M2",
        ],
        true,
        false,
        MINIMAX,
        MINIMAX,
    ),
];

pub fn builtin_capability_catalog() -> &'static [BuiltinCapabilityRecord] {
    CATALOG
}

pub fn builtin_capability_evidence(
    provider: &str,
    model: &str,
) -> Option<&'static BuiltinCapabilityRecord> {
    let model = crate::normalize_model_id(provider, model);
    CATALOG
        .iter()
        .find(|entry| entry.provider == provider && entry.models.contains(&model))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn snapshot_is_unique_exact_and_has_per_property_provenance() {
        let mut models = BTreeSet::new();
        for record in CATALOG {
            assert!(crate::provider_exists(record.provider));
            assert!(!record.models.is_empty());
            for model in record.models {
                assert!(!model.is_empty() && !model.chars().any(char::is_whitespace));
                assert_eq!(*model, crate::normalize_model_id(record.provider, model));
                assert!(models.insert((record.provider, model)));
                assert_eq!(
                    super::super::builtin_model_capabilities(record.provider, model),
                    Some(record.capabilities)
                );
            }
            for source in [record.tools_evidence, record.images_evidence] {
                assert_eq!(source.sha256.len(), 64);
                assert!(source.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()));
                if source.reference.starts_with("https://") {
                    assert_eq!(source.retrieved_at, Some("2026-09-07"));
                } else {
                    assert!(
                        source
                            .reference
                            .starts_with("npm:@earendil-works/pi-ai@0.85.1/")
                    );
                }
            }
        }
        assert_eq!(models.len(), 29);
        let sonnet = builtin_capability_evidence("anthropic", "claude-sonnet-4-5").unwrap();
        assert_ne!(sonnet.tools_evidence, sonnet.images_evidence);
    }
}
