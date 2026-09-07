use crate::error::{AgentError, AgentResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentCredential {
    ApiKey {
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        env: BTreeMap<String, String>,
    },
    OAuth {
        access: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        refresh: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expires: Option<u64>,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        env: BTreeMap<String, String>,
    },
}

impl AgentCredential {
    pub fn api_key(key: impl Into<String>) -> Self {
        Self::ApiKey {
            key: Some(key.into()),
            env: BTreeMap::new(),
        }
    }

    pub fn ambient(env: BTreeMap<String, String>) -> Self {
        Self::ApiKey { key: None, env }
    }

    pub fn secret(&self) -> Option<&str> {
        match self {
            Self::ApiKey { key, .. } => key.as_deref(),
            Self::OAuth { access, .. } => Some(access),
        }
    }

    pub fn env(&self) -> &BTreeMap<String, String> {
        match self {
            Self::ApiKey { env, .. } | Self::OAuth { env, .. } => env,
        }
    }

    pub fn auth_type(&self) -> &'static str {
        match self {
            Self::ApiKey { .. } => "api_key",
            Self::OAuth { .. } => "oauth",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCredentialStatus {
    pub provider: String,
    pub auth_type: String,
}

#[derive(Debug, Clone)]
pub struct AgentAuthStore {
    path: PathBuf,
}

impl AgentAuthStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn from_default_path() -> AgentResult<Self> {
        Ok(Self::new(default_auth_path()?))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn read(&self, provider: &str) -> AgentResult<Option<AgentCredential>> {
        Ok(self.load()?.remove(provider))
    }

    pub fn list(&self) -> AgentResult<Vec<AgentCredentialStatus>> {
        Ok(self
            .load()?
            .into_iter()
            .map(|(provider, credential)| AgentCredentialStatus {
                provider,
                auth_type: credential.auth_type().to_string(),
            })
            .collect())
    }

    pub fn save(&self, provider: &str, credential: &AgentCredential) -> AgentResult<()> {
        if provider.trim().is_empty() {
            return Err(AgentError::new("provider cannot be empty"));
        }
        self.with_lock(|mut credentials| {
            credentials.insert(provider.to_string(), credential.clone());
            Ok(credentials)
        })?;
        Ok(())
    }

    pub fn delete(&self, provider: &str) -> AgentResult<()> {
        self.with_lock(|mut credentials| {
            credentials.remove(provider);
            Ok(credentials)
        })?;
        Ok(())
    }

    fn load(&self) -> AgentResult<BTreeMap<String, AgentCredential>> {
        if !self.path.is_file() {
            return Ok(BTreeMap::new());
        }
        let contents = fs::read_to_string(&self.path)
            .map_err(|error| AgentError::at_path(&self.path, error.to_string()))?;
        if contents.trim().is_empty() {
            return Ok(BTreeMap::new());
        }
        serde_json::from_str(&contents)
            .map_err(|error| AgentError::at_path(&self.path, format!("invalid auth file: {error}")))
    }

    fn with_lock<F>(&self, operation: F) -> AgentResult<()>
    where
        F: FnOnce(
            BTreeMap<String, AgentCredential>,
        ) -> AgentResult<BTreeMap<String, AgentCredential>>,
    {
        let lock_path = self.path.with_extension("json.lock");
        let _lock = AuthFileLock::acquire(&lock_path)?;
        let next = operation(self.load()?)?;
        self.write(next)
    }

    fn write(&self, credentials: BTreeMap<String, AgentCredential>) -> AgentResult<()> {
        write_private_json(&self.path, &credentials)
    }
}

pub(super) fn write_private_json(path: &Path, value: &impl Serialize) -> AgentResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| AgentError::new("agent file path has no parent directory"))?;
    fs::create_dir_all(parent).map_err(|error| AgentError::at_path(parent, error.to_string()))?;
    set_private_directory(parent)?;
    let data = serde_json::to_vec_pretty(value)
        .map_err(|error| AgentError::new(format!("could not serialize agent file: {error}")))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let temporary = parent.join(format!(".agent-{}-{timestamp}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| AgentError::at_path(&temporary, error.to_string()))?;
    set_private_file(&file)?;
    file.write_all(&data)
        .and_then(|_| file.sync_all())
        .map_err(|error| AgentError::at_path(&temporary, error.to_string()))?;
    drop(file);
    fs::rename(&temporary, path).map_err(|error| AgentError::at_path(path, error.to_string()))?;
    set_private_path(path)?;
    Ok(())
}

pub fn default_auth_path() -> AgentResult<PathBuf> {
    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .ok_or_else(|| AgentError::new("could not determine the user home directory"))?;
    Ok(PathBuf::from(home)
        .join(".dowe")
        .join("agent")
        .join("auth.json"))
}

pub fn expand_credential_value(
    value: &str,
    scoped_env: &BTreeMap<String, String>,
) -> Option<String> {
    if value.starts_with("!") {
        return None;
    }
    let mut output = String::new();
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        if character != '$' {
            output.push(character);
            continue;
        }
        if characters.peek() == Some(&'$') {
            characters.next();
            output.push('$');
            continue;
        }
        let name = if characters.peek() == Some(&'{') {
            characters.next();
            let mut name = String::new();
            for next in characters.by_ref() {
                if next == '}' {
                    break;
                }
                name.push(next);
            }
            name
        } else {
            let mut name = String::new();
            while let Some(next) = characters.peek().copied() {
                if next.is_ascii_alphanumeric() || next == '_' {
                    name.push(next);
                    characters.next();
                } else {
                    break;
                }
            }
            name
        };
        if name.is_empty() {
            output.push('$');
            continue;
        }
        let resolved = scoped_env
            .get(&name)
            .cloned()
            .or_else(|| env::var(&name).ok())?;
        output.push_str(&resolved);
    }
    Some(output)
}

pub(super) struct AuthFileLock {
    path: PathBuf,
}

impl AuthFileLock {
    pub(super) fn acquire(path: &Path) -> AgentResult<Self> {
        let parent = path
            .parent()
            .ok_or_else(|| AgentError::new("auth lock path has no parent directory"))?;
        fs::create_dir_all(parent)
            .map_err(|error| AgentError::at_path(parent, error.to_string()))?;
        for _ in 0..200 {
            match OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(file) => {
                    set_private_file(&file)?;
                    drop(file);
                    return Ok(Self {
                        path: path.to_path_buf(),
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => return Err(AgentError::at_path(path, error.to_string())),
            }
        }
        Err(AgentError::at_path(
            path,
            "timed out waiting for the auth file lock",
        ))
    }
}

impl Drop for AuthFileLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn set_private_directory(path: &Path) -> AgentResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|error| AgentError::at_path(path, error.to_string()))?;
    }
    Ok(())
}

fn set_private_file(file: &std::fs::File) -> AgentResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|error| AgentError::new(error.to_string()))?;
    }
    Ok(())
}

fn set_private_path(path: &Path) -> AgentResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|error| AgentError::at_path(path, error.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn saves_typed_credentials_atomically_without_exposing_status_values() {
        let root = tempfile::tempdir().expect("root");
        let path = root.path().join("auth.json");
        let store = AgentAuthStore::new(&path);
        store
            .save("openai", &AgentCredential::api_key("secret"))
            .expect("save");

        assert_eq!(
            store.read("openai").expect("read").unwrap().secret(),
            Some("secret")
        );
        assert_eq!(store.list().expect("list")[0].auth_type, "api_key");
        assert_eq!(
            fs::metadata(&path).expect("metadata").permissions().mode() & 0o777,
            0o600
        );
        assert!(
            !serde_json::to_string(&store.list().expect("status"))
                .expect("status")
                .contains("secret")
        );
    }

    #[test]
    fn expands_scoped_environment_values_without_executing_commands() {
        let mut scoped = BTreeMap::new();
        scoped.insert("TOKEN".to_string(), "value".to_string());
        assert_eq!(
            expand_credential_value("prefix-${TOKEN}", &scoped),
            Some("prefix-value".to_string())
        );
        assert_eq!(expand_credential_value("!security-store", &scoped), None);
    }
}
