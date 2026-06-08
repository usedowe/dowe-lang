use crate::error::{AgentError, AgentResult};
use crate::model::AgentImageInput;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::fs;
use std::path::{Path, PathBuf};

pub fn encode_image_paths(paths: &[PathBuf]) -> AgentResult<Vec<AgentImageInput>> {
    paths
        .iter()
        .map(|path| encode_image(path))
        .collect::<AgentResult<Vec<_>>>()
}

pub fn encode_image(path: impl AsRef<Path>) -> AgentResult<AgentImageInput> {
    let path = path.as_ref();
    let mime_type = mime_type_for(path)?;
    let bytes = fs::read(path).map_err(|error| AgentError::at_path(path, error.to_string()))?;
    let encoded = STANDARD.encode(bytes);
    let data_url = format!("data:{mime_type};base64,{encoded}");

    Ok(AgentImageInput {
        path: path.to_string_lossy().to_string(),
        mime_type,
        data_url,
    })
}

fn mime_type_for(path: &Path) -> AgentResult<String> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| AgentError::at_path(path, "image path must have an extension"))?;

    match extension.as_str() {
        "png" => Ok("image/png".to_string()),
        "jpg" | "jpeg" => Ok("image/jpeg".to_string()),
        "webp" => Ok("image/webp".to_string()),
        "gif" => Ok("image/gif".to_string()),
        _ => Err(AgentError::at_path(path, "unsupported image extension")),
    }
}
