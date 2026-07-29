use crate::{RuntimeError, RuntimeResult};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path};
use tempfile::TempDir;

const MAGIC: &[u8; 8] = b"DOWEAPP1";
const TRAILER_SIZE: usize = 64;
const FORMAT_VERSION: u32 = 1;

pub(super) struct EmbeddedBundle<'a> {
    payload: &'a [u8],
}

#[derive(Deserialize)]
pub(super) struct AppManifest {
    pub name: String,
    pub entry: String,
}

impl<'a> EmbeddedBundle<'a> {
    pub(super) fn read(executable: &'a [u8]) -> RuntimeResult<Option<Self>> {
        if executable.len() < TRAILER_SIZE {
            return Ok(None);
        }
        let trailer = &executable[executable.len() - TRAILER_SIZE..];
        if &trailer[..8] != MAGIC {
            return Ok(None);
        }
        let version = read_u32(&trailer[8..12])?;
        if version != FORMAT_VERSION {
            return Err(RuntimeError::new(format!(
                "unsupported embedded desktop format version: {version}"
            )));
        }
        let offset = usize::try_from(read_u64(&trailer[16..24])?)
            .map_err(|_| RuntimeError::new("embedded desktop offset exceeds this platform"))?;
        let length = usize::try_from(read_u64(&trailer[24..32])?)
            .map_err(|_| RuntimeError::new("embedded desktop length exceeds this platform"))?;
        let end = offset
            .checked_add(length)
            .ok_or_else(|| RuntimeError::new("embedded desktop payload range overflowed"))?;
        if end != executable.len() - TRAILER_SIZE {
            return Err(RuntimeError::new(
                "embedded desktop payload range is invalid",
            ));
        }
        let payload = executable
            .get(offset..end)
            .ok_or_else(|| RuntimeError::new("embedded desktop payload is outside executable"))?;
        if Sha256::digest(payload).as_slice() != &trailer[32..64] {
            return Err(RuntimeError::new(
                "embedded desktop payload checksum is invalid",
            ));
        }
        Ok(Some(Self { payload }))
    }

    pub(super) fn extract(&self) -> RuntimeResult<TempDir> {
        let directory = tempfile::tempdir()?;
        let mut cursor = 0;
        let count = read_payload_u32(self.payload, &mut cursor)?;
        for _ in 0..count {
            let path_length = read_payload_u32(self.payload, &mut cursor)? as usize;
            let content_length = usize::try_from(read_payload_u64(self.payload, &mut cursor)?)
                .map_err(|_| RuntimeError::new("embedded desktop file is too large"))?;
            let path_bytes = take(self.payload, &mut cursor, path_length)?;
            let path = std::str::from_utf8(path_bytes)
                .map_err(|_| RuntimeError::new("embedded desktop path is not UTF-8"))?;
            validate_path(path)?;
            let content = take(self.payload, &mut cursor, content_length)?;
            let output = directory.path().join(path);
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&output, content)?;
            set_readonly(&output)?;
        }
        if cursor != self.payload.len() {
            return Err(RuntimeError::new(
                "embedded desktop payload has trailing data",
            ));
        }
        Ok(directory)
    }
}

pub(super) fn read_manifest(root: &Path) -> RuntimeResult<AppManifest> {
    let bytes = fs::read(root.join("manifest.json"))?;
    let manifest: AppManifest = serde_json::from_slice(&bytes)
        .map_err(|error| RuntimeError::new(format!("invalid desktop manifest: {error}")))?;
    validate_path(&manifest.entry)?;
    if !manifest.entry.starts_with("web/") {
        return Err(RuntimeError::new(
            "embedded desktop entry must be under web/",
        ));
    }
    Ok(manifest)
}

fn validate_path(value: &str) -> RuntimeResult<()> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(RuntimeError::new(format!(
            "invalid embedded desktop path: {value}"
        )));
    }
    Ok(())
}

#[cfg(unix)]
fn set_readonly(path: &Path) -> RuntimeResult<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_readonly(_path: &Path) -> RuntimeResult<()> {
    Ok(())
}

fn read_u32(bytes: &[u8]) -> RuntimeResult<u32> {
    bytes
        .try_into()
        .map(u32::from_le_bytes)
        .map_err(|_| RuntimeError::new("invalid embedded desktop integer"))
}

fn read_u64(bytes: &[u8]) -> RuntimeResult<u64> {
    bytes
        .try_into()
        .map(u64::from_le_bytes)
        .map_err(|_| RuntimeError::new("invalid embedded desktop integer"))
}

fn read_payload_u32(payload: &[u8], cursor: &mut usize) -> RuntimeResult<u32> {
    read_u32(take(payload, cursor, 4)?)
}

fn read_payload_u64(payload: &[u8], cursor: &mut usize) -> RuntimeResult<u64> {
    read_u64(take(payload, cursor, 8)?)
}

fn take<'a>(payload: &'a [u8], cursor: &mut usize, length: usize) -> RuntimeResult<&'a [u8]> {
    let end = cursor
        .checked_add(length)
        .ok_or_else(|| RuntimeError::new("embedded desktop payload overflowed"))?;
    let value = payload
        .get(*cursor..end)
        .ok_or_else(|| RuntimeError::new("embedded desktop payload is truncated"))?;
    *cursor = end;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{EmbeddedBundle, MAGIC, TRAILER_SIZE, read_manifest};
    use sha2::{Digest, Sha256};

    #[test]
    fn reads_and_extracts_verified_payload() {
        let manifest = br#"{"name":"Example","entry":"web/index.html"}"#;
        let payload = payload(&[("manifest.json", manifest), ("web/index.html", b"Dowe")]);
        let executable = executable(&payload);
        let bundle = EmbeddedBundle::read(&executable)
            .expect("read")
            .expect("bundle");
        let directory = bundle.extract().expect("extract");

        assert_eq!(
            std::fs::read(directory.path().join("web/index.html")).expect("entry"),
            b"Dowe"
        );
        assert_eq!(
            read_manifest(directory.path()).expect("manifest").name,
            "Example"
        );
    }

    #[test]
    fn rejects_traversal_and_changed_payload() {
        let traversal = payload(&[("../outside", b"bad")]);
        let traversal_executable = executable(&traversal);
        let bundle = EmbeddedBundle::read(&traversal_executable)
            .expect("read")
            .expect("bundle");
        assert!(bundle.extract().is_err());

        let mut changed = executable(&payload(&[("manifest.json", b"{}")]));
        changed[b"runtime".len()] ^= 1;
        assert!(EmbeddedBundle::read(&changed).is_err());

        let manifest = br#"{"name":"Example","entry":"../outside.html"}"#;
        let executable = executable(&payload(&[("manifest.json", manifest)]));
        let bundle = EmbeddedBundle::read(&executable)
            .expect("read")
            .expect("bundle");
        let directory = bundle.extract().expect("extract");
        assert!(read_manifest(directory.path()).is_err());
    }

    fn payload(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(entries.len() as u32).to_le_bytes());
        for (path, content) in entries {
            bytes.extend_from_slice(&(path.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&(content.len() as u64).to_le_bytes());
            bytes.extend_from_slice(path.as_bytes());
            bytes.extend_from_slice(content);
        }
        bytes
    }

    fn executable(payload: &[u8]) -> Vec<u8> {
        let mut bytes = b"runtime".to_vec();
        let offset = bytes.len();
        bytes.extend_from_slice(payload);
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&(offset as u64).to_le_bytes());
        bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&Sha256::digest(payload));
        assert_eq!(bytes.len(), offset + payload.len() + TRAILER_SIZE);
        bytes
    }
}
