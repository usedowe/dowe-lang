use crate::auth::{AuthFileLock, default_auth_path, write_private_json};
use crate::{
    AgentError, AgentResult, ThinkingLevel, normalize_model_id, provider_default_model,
    provider_exists, validate_thinking,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPreferences {
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<ThinkingLevel>,
}

#[derive(Debug, Clone)]
pub struct AgentPreferencesStore {
    path: PathBuf,
}

impl AgentPreferencesStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn from_default_path() -> AgentResult<Self> {
        Ok(Self::new(
            default_auth_path()?.with_file_name("preferences.json"),
        ))
    }

    pub fn provider(&self) -> AgentResult<Option<String>> {
        Ok(self.read()?.provider)
    }

    pub fn read(&self) -> AgentResult<AgentPreferences> {
        let preferences = self.load()?;
        self.validate(&preferences)?;
        Ok(preferences)
    }

    fn load(&self) -> AgentResult<AgentPreferences> {
        let data = match fs::read(&self.path) {
            Ok(data) => data,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(AgentPreferences::default());
            }
            Err(error) => return Err(AgentError::at_path(&self.path, error.to_string())),
        };
        let preferences: AgentPreferences = serde_json::from_slice(&data)
            .map_err(|_| AgentError::at_path(&self.path, "invalid agent preferences file"))?;
        Ok(preferences)
    }

    pub fn migrate_retired_model(&self) -> AgentResult<Option<String>> {
        if replacement(&self.load()?).is_none() {
            return Ok(None);
        }
        let _lock = AuthFileLock::acquire(&self.path.with_extension("json.lock"))?;
        let mut preferences = self.load()?;
        let Some(model) = replacement(&preferences) else {
            return Ok(None);
        };
        set_model(&mut preferences, "openai-codex", model);
        self.validate(&preferences)?;
        write_private_json(&self.path, &preferences)?;
        Ok(Some(model.to_string()))
    }

    pub fn select_provider(&self, provider: &str) -> AgentResult<()> {
        self.update(|selection| {
            if selection.provider.as_deref() != Some(provider) {
                *selection = AgentPreferences {
                    provider: Some(provider.to_string()),
                    ..Default::default()
                };
            }
        })
    }

    pub fn select_model(&self, provider: &str, model: &str) -> AgentResult<()> {
        self.update(|selection| set_model(selection, provider, model))
    }

    pub fn select_thinking(
        &self,
        provider: &str,
        model: &str,
        level: ThinkingLevel,
    ) -> AgentResult<()> {
        self.update(|selection| {
            set_model(selection, provider, model);
            selection.thinking_level = Some(level);
        })
    }

    fn update(&self, update: impl FnOnce(&mut AgentPreferences)) -> AgentResult<()> {
        let _lock = AuthFileLock::acquire(&self.path.with_extension("json.lock"))?;
        let mut selection = self.read()?;
        update(&mut selection);
        self.validate(&selection)?;
        write_private_json(&self.path, &selection)
    }

    fn validate(&self, selection: &AgentPreferences) -> AgentResult<()> {
        if let Some(provider) = &selection.provider {
            self.validate_provider(provider)?;
            if selection
                .model
                .as_deref()
                .is_some_and(|m| m.trim().is_empty() || m.chars().any(char::is_control))
            {
                return Err(AgentError::new("invalid model in agent preferences"));
            }
            if let Some(model) = &selection.model {
                crate::validate_agent_model(provider, model)?;
            }
            if let Some(level) = selection.thinking_level {
                validate_thinking(
                    provider,
                    selection
                        .model
                        .as_deref()
                        .unwrap_or(provider_default_model(provider)?),
                    level,
                )?;
            }
        } else if selection.model.is_some() || selection.thinking_level.is_some() {
            return Err(AgentError::new("agent preferences require a provider"));
        }
        Ok(())
    }

    fn validate_provider(&self, provider: &str) -> AgentResult<()> {
        if !provider_exists(provider) {
            return Err(AgentError::at_path(
                &self.path,
                "unknown provider in agent preferences",
            ));
        }
        Ok(())
    }
}

fn replacement(selection: &AgentPreferences) -> Option<&'static str> {
    crate::provider::retired_model_replacement(
        selection.provider.as_deref()?,
        selection.model.as_deref()?,
    )
}

fn set_model(selection: &mut AgentPreferences, provider: &str, model: &str) {
    if selection.provider.as_deref() != Some(provider) {
        selection.thinking_level = None;
    }
    let model = normalize_model_id(provider, model);
    selection.provider = Some(provider.to_string());
    selection.model = Some(model.to_string());
    if selection
        .thinking_level
        .is_some_and(|level| validate_thinking(provider, model, level).is_err())
    {
        selection.thinking_level = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentAuthStore, AgentCredential};

    #[test]
    fn preferences_are_optional_and_persist_independently_of_credentials() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("agent/preferences.json");
        let store = AgentPreferencesStore::new(&path);
        assert_eq!(store.provider().unwrap(), None);
        assert!(!root.path().join("agent").exists());
        let auth = AgentAuthStore::new(path.with_file_name("auth.json"));
        auth.save("anthropic", &AgentCredential::api_key("test-secret"))
            .unwrap();
        let credentials = fs::read(auth.path()).unwrap();
        store.select_provider("openai-codex").unwrap();
        assert_eq!(
            AgentPreferencesStore::new(&path)
                .provider()
                .unwrap()
                .as_deref(),
            Some("openai-codex")
        );
        store.select_provider("anthropic").unwrap();
        assert_eq!(store.provider().unwrap().as_deref(), Some("anthropic"));
        assert_eq!(fs::read(auth.path()).unwrap(), credentials);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&fs::read(&path).unwrap()).unwrap(),
            serde_json::json!({"provider":"anthropic"})
        );
        assert!(!path.with_extension("json.lock").exists());
        assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 2);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn model_and_thinking_survive_restart_and_reset_when_incompatible() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("preferences.json");
        let store = AgentPreferencesStore::new(&path);
        store
            .select_thinking("openai-codex", "gpt-6-astra", ThinkingLevel::Max)
            .unwrap();
        let restored = AgentPreferencesStore::new(&path).read().unwrap();
        assert_eq!(restored.model.as_deref(), Some("gpt-6-astra"));
        assert_eq!(restored.thinking_level, Some(ThinkingLevel::Max));
        store.select_provider("openai-codex").unwrap();
        assert_eq!(store.read().unwrap(), restored);
        store.select_model("openai-codex", "gpt-5.5").unwrap();
        assert_eq!(store.read().unwrap().thinking_level, None);
        let before = fs::read(&path).unwrap();
        assert!(
            store
                .select_thinking("openai-codex", "gpt-5.5", ThinkingLevel::Max)
                .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), before);
        store.select_provider("anthropic").unwrap();
        assert_eq!(store.read().unwrap().model, None);
        assert_eq!(store.read().unwrap().thinking_level, None);
    }

    #[test]
    fn retired_codex_preferences_migrate_once_without_touching_credentials() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("preferences.json");
        let store = AgentPreferencesStore::new(&path);
        assert_eq!(store.migrate_retired_model().unwrap(), None);
        assert!(!path.exists());
        let auth = AgentAuthStore::new(path.with_file_name("auth.json"));
        auth.save("openai-codex", &AgentCredential::api_key("test-secret"))
            .unwrap();
        let credentials = fs::read(auth.path()).unwrap();
        for model in ["gpt-5.3-codex", "openai-codex/gpt-5.3-codex"] {
            let old = serde_json::json!({"provider":"openai-codex","model":model,"thinkingLevel":"medium"}).to_string();
            fs::write(&path, &old).unwrap();
            assert!(store.read().is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), old);
            assert_eq!(
                store.migrate_retired_model().unwrap().as_deref(),
                Some("gpt-5.5")
            );
            let saved = store.read().unwrap();
            assert_eq!(saved.model.as_deref(), Some("gpt-5.5"));
            assert_eq!(saved.thinking_level, Some(ThinkingLevel::Medium));
            assert_eq!(fs::read(auth.path()).unwrap(), credentials);
            assert_eq!(store.migrate_retired_model().unwrap(), None);
        }
        store
            .select_model("openai-codex", "gpt-5.3-codex-spark")
            .unwrap();
        assert_eq!(store.migrate_retired_model().unwrap(), None);
        assert!(store.select_model("openai-codex", "gpt-5.3-codex").is_err());
        store.select_model("openai", "gpt-5.3-codex").unwrap();
        assert_eq!(store.migrate_retired_model().unwrap(), None);
    }

    #[test]
    fn invalid_preferences_fail_without_exposing_file_contents() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("preferences.json");
        let store = AgentPreferencesStore::new(&path);
        assert!(store.select_provider("secret-unknown-provider").is_err());
        assert!(!path.exists());
        for contents in [
            "secret-invalid-json",
            "{\"provider\":\"secret-unknown-provider\"}",
        ] {
            fs::write(&path, contents).unwrap();
            let error = store.provider().unwrap_err().to_string();
            assert!(!error.contains("secret"));
        }
    }
}
