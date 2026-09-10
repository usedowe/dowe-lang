use crate::auth::default_auth_path;
use crate::provider::{provider_exists, AgentModelDefinition};
use crate::{AgentError, AgentResult};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;

#[path = "../catalog/generated.rs"]
mod generated;

/// The checked-in, generated snapshot is used at runtime. It contains no network or parsing path.
pub fn builtin_models(provider: &str) -> &'static [AgentModelDefinition] {
    match provider {
        "amazon-bedrock" => generated::AMAZON_BEDROCK,
        "anthropic" => generated::ANTHROPIC,
        "github-copilot" => generated::GITHUB_COPILOT,
        "azure-openai-responses" => generated::AZURE_OPENAI_RESPONSES,
        "cloudflare-ai-gateway" => generated::CLOUDFLARE_AI_GATEWAY,
        "cloudflare-workers-ai" => generated::CLOUDFLARE_WORKERS_AI,
        "deepseek" => generated::DEEPSEEK,
        "fireworks" => generated::FIREWORKS,
        "google" => generated::GOOGLE,
        "google-vertex" => generated::GOOGLE_VERTEX,
        "groq" => generated::GROQ,
        "kimi-coding" => generated::KIMI_CODING,
        "minimax" => generated::MINIMAX,
        "minimax-cn" => generated::MINIMAX_CN,
        "mistral" => generated::MISTRAL,
        "nvidia" => generated::NVIDIA,
        "openai" => generated::OPENAI,
        "openai-codex" => generated::OPENAI_CODEX,
        "opencode" => generated::OPENCODE,
        "opencode-go" => generated::OPENCODE_GO,
        "openrouter" => generated::OPENROUTER,
        "qwen-token-plan" => generated::QWEN_TOKEN_PLAN,
        "qwen-token-plan-cn" => generated::QWEN_TOKEN_PLAN_CN,
        "qwen-token-plan-individual" => generated::QWEN_TOKEN_PLAN_INDIVIDUAL,
        "vercel-ai-gateway" => generated::VERCEL_AI_GATEWAY,
        "xai" => generated::XAI,
        "zai" => generated::ZAI,
        "zai-coding-cn" => generated::ZAI_CODING_CN,
        _ => &[],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCatalogModel {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OverrideFile {
    schema_version: u32,
    providers: std::collections::BTreeMap<String, Vec<OverrideModel>>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OverrideModel {
    id: String,
    name: Option<String>,
}

/// Load optional user models without leaking override strings into the process lifetime.
pub fn models_with_local_overrides(provider: &str) -> AgentResult<Vec<AgentCatalogModel>> {
    if !provider_exists(provider) {
        return Err(AgentError::new(format!("unknown agent provider `{provider}`")));
    }
    let mut result = builtin_models(provider)
        .iter()
        .map(|model| AgentCatalogModel { id: model.id.to_string(), name: model.name.to_string() })
        .collect::<Vec<_>>();
    let path = default_auth_path()?.with_file_name("models.json");
    if !path.is_file() { return Ok(result); }
    let contents = fs::read_to_string(&path).map_err(|e| AgentError::at_path(&path, e.to_string()))?;
    let file: OverrideFile = serde_json::from_str(&contents)
        .map_err(|e| AgentError::at_path(&path, format!("invalid model catalog override: {e}")))?;
    if file.schema_version != 1 {
        return Err(AgentError::at_path(&path, format!("unsupported model catalog override schema version: {}", file.schema_version)));
    }
    for unknown in file.providers.keys().filter(|id| !provider_exists(id)) {
        return Err(AgentError::at_path(&path, format!("unknown provider in model catalog override: `{unknown}`")));
    }
    let Some(entries) = file.providers.get(provider) else { return Ok(result); };
    let mut ids = result.iter().map(|model| model.id.as_str()).collect::<BTreeSet<_>>();
    let mut overrides = Vec::with_capacity(entries.len());
    for entry in entries {
        validate_override_id(&entry.id).map_err(|message| AgentError::at_path(&path, message))?;
        if !ids.insert(entry.id.as_str()) {
            return Err(AgentError::at_path(&path, format!("duplicate model id in model catalog override: `{}`", entry.id)));
        }
        let name = entry.name.as_deref().unwrap_or(&entry.id);
        if name.trim().is_empty() || name.chars().any(char::is_control) {
            return Err(AgentError::at_path(&path, "invalid model name in model catalog override"));
        }
        overrides.push(AgentCatalogModel { id: entry.id.clone(), name: name.to_string() });
    }
    overrides.sort_by(|a, b| a.id.cmp(&b.id));
    result.extend(overrides);
    Ok(result)
}

fn validate_override_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 256 || id.chars().any(|c| c.is_control() || c.is_whitespace()) || id.contains("://") || id.contains('$') || id.contains('\\') {
        return Err(format!("invalid model id in model catalog override: `{id}`"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_covers_every_provider() {
        for provider in crate::provider::builtin_provider_ids() { assert!(!builtin_models(provider).is_empty(), "{provider}"); }
    }
    #[test]
    fn snapshot_is_deterministically_sorted() {
        let ids = builtin_models("openrouter").iter().map(|m| m.id).collect::<Vec<_>>();
        let mut sorted = ids.clone(); sorted.sort_unstable();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn catalog_carries_context_metadata_for_openrouter_deepseek() {
        assert_eq!(
            builtin_models("openrouter")
                .iter()
                .find(|model| model.id == "deepseek/deepseek-v4-flash")
                .and_then(|model| model.context_window),
            Some(1_000_000)
        );
    }

    #[test]
    fn generated_catalog_contains_only_mistrals_documented_default_model() {
        let models = builtin_models("mistral");
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "mistral-large-latest");
        assert_eq!(models[0].name, "Mistral Large");
    }

    #[test]
    fn override_schema_and_ids_are_validated() {
        let file: OverrideFile = serde_json::from_str(r#"{"schemaVersion":1,"providers":{"openai":[{"id":"custom/model"}]}}"#).unwrap();
        assert_eq!(file.schema_version, 1);
        assert!(serde_json::from_str::<OverrideFile>(r#"{"schemaVersion":2,"providers":{}}"#).is_ok());
        assert!(validate_override_id("custom/model").is_ok());
        assert!(validate_override_id("custom model").is_err());
        assert!(validate_override_id("https://example.invalid/model").is_err());
    }
}
