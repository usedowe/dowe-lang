mod bundle;
mod host;

use crate::{RuntimeError, RuntimeResult};
use std::env;
use std::fs;

pub async fn run_embedded() -> RuntimeResult<bool> {
    let executable = env::current_exe()?;
    let bytes = fs::read(&executable)?;
    let Some(bundle) = bundle::EmbeddedBundle::read(&bytes)? else {
        return Ok(false);
    };
    let resources = bundle.extract()?;
    let manifest = bundle::read_manifest(resources.path())?;
    let entry = resources.path().join(&manifest.entry);
    if !entry.is_file() {
        return Err(RuntimeError::new(format!(
            "embedded desktop entry is missing: {}",
            manifest.entry
        )));
    }
    host::run(&manifest.name, &entry)?;
    Ok(true)
}

pub fn run_development_host_from_env() -> RuntimeResult<bool> {
    let Ok(uri) = env::var("DOWE_INTERNAL_DESKTOP_URL") else {
        return Ok(false);
    };
    if !(uri.starts_with("http://") || uri.starts_with("https://")) {
        return Err(RuntimeError::new("invalid internal desktop URL"));
    }
    let name = env::var("DOWE_INTERNAL_DESKTOP_NAME").unwrap_or_else(|_| "Dowe Dev".to_string());
    host::run_uri(&name, &uri)?;
    Ok(true)
}
