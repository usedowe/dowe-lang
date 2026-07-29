use crate::error::{DeployError, DeployResult};
use crate::files::write_file;
use crate::model::{BuildReport, BuildTarget};
use dowe_compiler::CompiledProject;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

const TRAILER_MAGIC: &[u8; 8] = b"DOWEAPP1";
const TRAILER_SIZE: usize = 64;
const FORMAT_VERSION: u32 = 1;

pub(crate) fn build(
    project: &CompiledProject,
    target: BuildTarget,
    output: &Path,
    dry_run: bool,
) -> DeployResult<BuildReport> {
    let web_root = project.root.join(".dowe/apps/desktop/web");
    let artifact = output.join(format!(
        "{}{}",
        safe_name(&project.app_config.name),
        if target == BuildTarget::Windows {
            ".exe"
        } else {
            ""
        }
    ));
    let payload = desktop_payload(project, &web_root)?;
    let payload_path = output.join("app.dowe-bundle");
    write_file(&payload_path, &payload)?;
    if !dry_run {
        let runtime = std::env::current_exe()?;
        let runtime_bytes = fs::read(&runtime)?;
        validate_runtime(target, &runtime_bytes)?;
        let executable = append_payload(&runtime_bytes, &payload);
        write_file(&artifact, &executable)?;
        validate_executable(target, &executable)?;
        set_executable(&artifact)?;
    }
    Ok(BuildReport {
        target,
        output_dir: output.to_path_buf(),
        artifact,
        commands: Vec::new(),
        built: !dry_run,
    })
}

fn desktop_payload(project: &CompiledProject, web_root: &Path) -> DeployResult<Vec<u8>> {
    if !web_root.join("index.html").is_file() {
        return Err(DeployError::new(format!(
            "missing generated desktop entry: {}",
            web_root.join("index.html").display()
        )));
    }
    let manifest = serde_json::to_vec(&json!({
        "bundle": project.app_config.bundle,
        "entry": "web/index.html",
        "name": project.app_config.name,
        "version": FORMAT_VERSION,
    }))?;
    let mut entries = vec![("manifest.json".to_string(), manifest)];
    let mut web_entries = Vec::new();
    collect_entries(web_root, web_root, &mut web_entries)?;
    entries.extend(
        web_entries
            .into_iter()
            .map(|(path, bytes)| (format!("web/{path}"), bytes)),
    );
    entries.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));

    let mut payload = Vec::new();
    payload.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    for (path, bytes) in entries {
        payload.extend_from_slice(&(path.len() as u32).to_le_bytes());
        payload.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        payload.extend_from_slice(path.as_bytes());
        payload.extend_from_slice(&bytes);
    }
    Ok(payload)
}

fn collect_entries(
    root: &Path,
    current: &Path,
    entries: &mut Vec<(String, Vec<u8>)>,
) -> DeployResult<()> {
    let mut children = fs::read_dir(current)?.collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(|entry| entry.file_name());
    for entry in children {
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            return Err(DeployError::new(format!(
                "desktop bundle rejects symbolic link: {}",
                entry.path().display()
            )));
        }
        if file_type.is_dir() {
            collect_entries(root, &entry.path(), entries)?;
        } else if file_type.is_file() {
            let relative = entry
                .path()
                .strip_prefix(root)
                .map_err(|_| DeployError::new("desktop asset escaped its root"))?
                .to_string_lossy()
                .replace('\\', "/");
            entries.push((relative, fs::read(entry.path())?));
        }
    }
    Ok(())
}

fn append_payload(runtime: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut executable = Vec::with_capacity(runtime.len() + payload.len() + TRAILER_SIZE);
    executable.extend_from_slice(runtime);
    executable.extend_from_slice(payload);
    executable.extend_from_slice(TRAILER_MAGIC);
    executable.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    executable.extend_from_slice(&0u32.to_le_bytes());
    executable.extend_from_slice(&(runtime.len() as u64).to_le_bytes());
    executable.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    executable.extend_from_slice(&Sha256::digest(payload));
    executable
}

fn validate_executable(target: BuildTarget, bytes: &[u8]) -> DeployResult<()> {
    validate_runtime(target, bytes)?;
    if bytes.len() < TRAILER_SIZE {
        return Err(DeployError::new(
            "Dowe desktop executable is missing its trailer",
        ));
    }
    let trailer = &bytes[bytes.len() - TRAILER_SIZE..];
    if &trailer[..8] != TRAILER_MAGIC {
        return Err(DeployError::new(
            "Dowe desktop executable has an invalid trailer",
        ));
    }
    let offset = u64::from_le_bytes(trailer[16..24].try_into().unwrap()) as usize;
    let length = u64::from_le_bytes(trailer[24..32].try_into().unwrap()) as usize;
    let end = offset
        .checked_add(length)
        .ok_or_else(|| DeployError::new("Dowe desktop payload range overflowed"))?;
    if end != bytes.len() - TRAILER_SIZE {
        return Err(DeployError::new("Dowe desktop payload range is invalid"));
    }
    if Sha256::digest(&bytes[offset..end]).as_slice() != &trailer[32..64] {
        return Err(DeployError::new("Dowe desktop payload checksum is invalid"));
    }
    Ok(())
}

fn validate_runtime(target: BuildTarget, bytes: &[u8]) -> DeployResult<()> {
    let valid = match target {
        BuildTarget::Windows => bytes.starts_with(b"MZ"),
        BuildTarget::Linux => bytes.starts_with(b"\x7fELF"),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(DeployError::new(format!(
            "current Dowe {} runtime has an invalid executable format",
            target.as_str()
        )))
    }
}

fn safe_name(value: &str) -> String {
    let value = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .collect::<String>();
    if value.is_empty() {
        "DoweApp".to_string()
    } else {
        value
    }
}

#[cfg(unix)]
fn set_executable(path: &Path) -> DeployResult<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> DeployResult<()> {
    Ok(())
}

#[cfg(test)]
mod tests;
