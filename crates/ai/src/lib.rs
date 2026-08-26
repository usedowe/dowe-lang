use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::{Display, Formatter};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const DEFAULT_REGISTRY_URL: &str = "https://cdn.dowe.dev/models/manifest.json";
pub const DEFAULT_MODEL: &str = "gemma-4-e2b";
pub const ADVANCED_MODEL: &str = "gemma-4-e4b";
pub const MAX_CONTEXT_FILES: usize = 32;
pub const MAX_CONTEXT_BYTES: usize = 512 * 1024;

#[derive(Debug)]
pub enum AiError {
    Io(std::io::Error),
    Http(reqwest::Error),
    Json(serde_json::Error),
    InvalidManifest(String),
    ChecksumMismatch {
        path: String,
        expected: String,
        actual: String,
    },
}

impl Display for AiError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "AI storage error: {error}"),
            Self::Http(error) => write!(formatter, "AI registry request failed: {error}"),
            Self::Json(error) => write!(formatter, "AI manifest is invalid JSON: {error}"),
            Self::InvalidManifest(error) => write!(formatter, "AI manifest is invalid: {error}"),
            Self::ChecksumMismatch {
                path,
                expected,
                actual,
            } => write!(
                formatter,
                "AI model checksum mismatch for {path}: expected {expected}, got {actual}"
            ),
        }
    }
}

impl std::error::Error for AiError {}
impl From<std::io::Error> for AiError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<reqwest::Error> for AiError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}
impl From<serde_json::Error> for AiError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiDevice {
    Cpu,
    Metal,
    Cuda,
}

impl AiDevice {
    pub fn detect() -> Self {
        if let Ok(value) = std::env::var("DOWE_AI_DEVICE") {
            return match value.to_ascii_lowercase().as_str() {
                "metal" => Self::Metal,
                "cuda" => Self::Cuda,
                _ => Self::Cpu,
            };
        }
        if cfg!(target_os = "macos") {
            Self::Metal
        } else {
            Self::Cpu
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Metal => "metal",
            Self::Cuda => "cuda",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiManifest {
    pub schema: u32,
    pub registry: String,
    pub models: Vec<AiModel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiModel {
    pub id: String,
    pub version: String,
    pub family: String,
    pub parameters: String,
    pub format: String,
    pub quantization: Option<String>,
    pub devices: Vec<AiDevice>,
    pub size: u64,
    pub files: Vec<AiModelFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiModelFile {
    pub path: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
}

impl AiManifest {
    pub fn validate(&self) -> Result<(), AiError> {
        if self.schema != 1 {
            return Err(AiError::InvalidManifest(format!(
                "unsupported schema {}",
                self.schema
            )));
        }
        if self.models.is_empty() {
            return Err(AiError::InvalidManifest("models cannot be empty".into()));
        }
        for model in &self.models {
            if model.id.is_empty() || model.version.is_empty() || model.files.is_empty() {
                return Err(AiError::InvalidManifest(format!(
                    "model `{}` is incomplete",
                    model.id
                )));
            }
            for file in &model.files {
                if file.path.is_empty()
                    || Path::new(&file.path).is_absolute()
                    || file.path.split('/').any(|part| part == "..")
                {
                    return Err(AiError::InvalidManifest(format!(
                        "invalid file path in `{}`",
                        model.id
                    )));
                }
                if file.sha256.len() != 64
                    || !file
                        .sha256
                        .chars()
                        .all(|character| character.is_ascii_hexdigit())
                {
                    return Err(AiError::InvalidManifest(format!(
                        "invalid SHA-256 for `{}`",
                        file.path
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn model_for(
        &self,
        device: AiDevice,
        preferred: Option<&str>,
    ) -> Result<&AiModel, AiError> {
        if let Some(id) = preferred {
            let model = self
                .models
                .iter()
                .find(|model| model.id == id)
                .ok_or_else(|| {
                    AiError::InvalidManifest(format!("model `{id}` is not registered"))
                })?;
            if !model.devices.contains(&device) {
                return Err(AiError::InvalidManifest(format!(
                    "model `{id}` does not support {}",
                    device.as_str()
                )));
            }
            return Ok(model);
        }
        let preferred = if device == AiDevice::Cpu {
            DEFAULT_MODEL
        } else {
            ADVANCED_MODEL
        };
        self.models
            .iter()
            .find(|model| model.id == preferred && model.devices.contains(&device))
            .or_else(|| {
                self.models
                    .iter()
                    .find(|model| model.devices.contains(&device))
            })
            .ok_or_else(|| {
                AiError::InvalidManifest(format!("no model supports {}", device.as_str()))
            })
    }
}

pub fn build_file_context(
    root: impl AsRef<Path>,
    files: &serde_json::Value,
) -> Result<String, AiError> {
    let root = root.as_ref().canonicalize()?;
    let entries = files
        .as_array()
        .ok_or_else(|| AiError::InvalidManifest("AI files must be an array".into()))?;
    if entries.len() > MAX_CONTEXT_FILES {
        return Err(AiError::InvalidManifest(
            "AI file context exceeds the file limit".into(),
        ));
    }
    let mut total = 0;
    let mut context = String::new();
    for entry in entries {
        let relative = entry
            .as_str()
            .ok_or_else(|| AiError::InvalidManifest("AI file paths must be strings".into()))?;
        let path = root.join(relative);
        let canonical = path.canonicalize()?;
        if !canonical.starts_with(&root) {
            return Err(AiError::InvalidManifest(format!(
                "AI file escapes project root: {relative}"
            )));
        }
        let content = fs::read_to_string(&canonical)?;
        total += content.len();
        if total > MAX_CONTEXT_BYTES {
            return Err(AiError::InvalidManifest(
                "AI file context exceeds the byte limit".into(),
            ));
        }
        context.push_str("\n<file path=\"");
        context.push_str(relative);
        context.push_str("\">\n");
        context.push_str(&content);
        context.push_str("\n</file>\n");
    }
    Ok(context)
}

pub struct AiRegistry {
    root: PathBuf,
    client: Client,
}

impl AiRegistry {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, AiError> {
        Ok(Self {
            root: root.into(),
            client: Client::builder().timeout(Duration::from_secs(60)).build()?,
        })
    }

    pub fn fetch_manifest(&self, url: Option<&str>) -> Result<AiManifest, AiError> {
        let manifest = self
            .client
            .get(url.unwrap_or(DEFAULT_REGISTRY_URL))
            .send()?
            .error_for_status()?
            .json::<AiManifest>()?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn install(&self, manifest: &AiManifest, model: &AiModel) -> Result<PathBuf, AiError> {
        manifest.validate()?;
        let destination = self.root.join(&model.id).join(&model.version);
        let staging = self.root.join(format!(".{}.staging", model.id));
        if staging.exists() {
            fs::remove_dir_all(&staging)?;
        }
        fs::create_dir_all(&staging)?;
        for entry in &model.files {
            let path = staging.join(&entry.path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut response = self.client.get(&entry.url).send()?.error_for_status()?;
            let mut file = File::create(&path)?;
            response.copy_to(&mut file)?;
            file.flush()?;
            verify_sha256(&path, &entry.sha256, &entry.path)?;
        }
        if destination.exists() {
            fs::remove_dir_all(&destination)?;
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(staging, &destination)?;
        Ok(destination)
    }
}

fn verify_sha256(path: &Path, expected: &str, display_path: &str) -> Result<(), AiError> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    let actual = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if actual != expected.to_ascii_lowercase() {
        return Err(AiError::ChecksumMismatch {
            path: display_path.into(),
            expected: expected.into(),
            actual,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(id: &str, devices: Vec<AiDevice>) -> AiModel {
        AiModel {
            id: id.into(),
            version: "1.0.0".into(),
            family: "gemma".into(),
            parameters: "2B".into(),
            format: "safetensors".into(),
            quantization: None,
            devices,
            size: 1,
            files: vec![AiModelFile {
                path: "model.safetensors".into(),
                url: "https://cdn.dowe.dev/model.safetensors".into(),
                size: 1,
                sha256: "0".repeat(64),
            }],
        }
    }

    #[test]
    fn selects_e2b_for_cpu_and_e4b_for_gpu() {
        let manifest = AiManifest {
            schema: 1,
            registry: DEFAULT_REGISTRY_URL.into(),
            models: vec![
                model(DEFAULT_MODEL, vec![AiDevice::Cpu]),
                model(ADVANCED_MODEL, vec![AiDevice::Metal]),
            ],
        };
        assert_eq!(
            manifest.model_for(AiDevice::Cpu, None).expect("cpu").id,
            DEFAULT_MODEL
        );
        assert_eq!(
            manifest.model_for(AiDevice::Metal, None).expect("metal").id,
            ADVANCED_MODEL
        );
    }

    #[test]
    fn builds_bounded_file_context() {
        let root = tempfile::tempdir().expect("root");
        std::fs::write(root.path().join("main.dowe"), "main").expect("file");
        let context =
            build_file_context(root.path(), &serde_json::json!(["main.dowe"])).expect("context");
        assert!(context.contains("<file path=\"main.dowe\">"));
        assert!(context.contains("main"));
    }

    #[test]
    fn rejects_traversal_in_manifest_files() {
        let mut entry = model(DEFAULT_MODEL, vec![AiDevice::Cpu]);
        entry.files[0].path = "../model.safetensors".into();
        let manifest = AiManifest {
            schema: 1,
            registry: DEFAULT_REGISTRY_URL.into(),
            models: vec![entry],
        };
        assert!(manifest.validate().is_err());
    }
}
