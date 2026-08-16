use crate::cloud;
use crate::error::{DeployError, DeployResult};
use crate::files::write_file;
use crate::model::DeployEnvironment;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub(crate) const SSH_TRAILER_MAGIC: &[u8; 8] = b"DOWESSH1";
pub(crate) const DOCKER_TRAILER_MAGIC: &[u8; 8] = b"DOWESRV1";
const TRAILER_VERSION: u64 = 1;
const TRAILER_SIZE: usize = 112;

pub(crate) struct EmbeddedPayload {
    pub application: Vec<u8>,
    pub metadata: Vec<u8>,
}

pub(crate) fn encode_embedded_payload(
    runtime: &[u8],
    application: &[u8],
    metadata: &[u8],
    magic: &[u8; 8],
) -> Vec<u8> {
    let application_offset = runtime.len() as u64;
    let metadata_offset = application_offset + application.len() as u64;
    let mut output =
        Vec::with_capacity(runtime.len() + application.len() + metadata.len() + TRAILER_SIZE);
    output.extend_from_slice(runtime);
    output.extend_from_slice(application);
    output.extend_from_slice(metadata);
    output.extend_from_slice(magic);
    output.extend_from_slice(&TRAILER_VERSION.to_le_bytes());
    output.extend_from_slice(&application_offset.to_le_bytes());
    output.extend_from_slice(&(application.len() as u64).to_le_bytes());
    output.extend_from_slice(&metadata_offset.to_le_bytes());
    output.extend_from_slice(&(metadata.len() as u64).to_le_bytes());
    output.extend_from_slice(&Sha256::digest(application));
    output.extend_from_slice(&Sha256::digest(metadata));
    output
}

pub(crate) fn read_embedded_payload(
    executable: &Path,
    magic: &[u8; 8],
    kind: &str,
) -> DeployResult<Option<EmbeddedPayload>> {
    let mut file = fs::File::open(executable)?;
    let length = usize::try_from(file.metadata()?.len())
        .map_err(|_| DeployError::new(format!("embedded {kind} executable is too large")))?;
    if length < TRAILER_SIZE {
        return Ok(None);
    }
    file.seek(SeekFrom::End(-(TRAILER_SIZE as i64)))?;
    let mut trailer = [0u8; TRAILER_SIZE];
    file.read_exact(&mut trailer)?;
    if &trailer[..8] != magic {
        return Ok(None);
    }
    file.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::with_capacity(length);
    file.read_to_end(&mut bytes)?;
    decode_embedded_payload(&bytes, magic, kind)
}

pub(crate) fn decode_embedded_payload(
    bytes: &[u8],
    magic: &[u8; 8],
    kind: &str,
) -> DeployResult<Option<EmbeddedPayload>> {
    if bytes.len() < TRAILER_SIZE {
        return Ok(None);
    }
    let trailer = &bytes[bytes.len() - TRAILER_SIZE..];
    if &trailer[..8] != magic {
        return Ok(None);
    }
    if read_u64(trailer, 8, kind)? != TRAILER_VERSION {
        return Err(DeployError::new(format!(
            "unsupported embedded {kind} executable version"
        )));
    }
    let application_offset = usize_value(read_u64(trailer, 16, kind)?, kind)?;
    let application_length = usize_value(read_u64(trailer, 24, kind)?, kind)?;
    let metadata_offset = usize_value(read_u64(trailer, 32, kind)?, kind)?;
    let metadata_length = usize_value(read_u64(trailer, 40, kind)?, kind)?;
    let payload_end = bytes.len() - TRAILER_SIZE;
    let application_end = checked_end(application_offset, application_length, payload_end, kind)?;
    let metadata_end = checked_end(metadata_offset, metadata_length, payload_end, kind)?;
    if application_end != metadata_offset || metadata_end != payload_end {
        return Err(DeployError::new(format!(
            "invalid embedded {kind} executable layout"
        )));
    }
    let application = &bytes[application_offset..application_end];
    let metadata = &bytes[metadata_offset..metadata_end];
    if Sha256::digest(application).as_slice() != &trailer[48..80] {
        return Err(DeployError::new(format!(
            "embedded {kind} application checksum mismatch"
        )));
    }
    if Sha256::digest(metadata).as_slice() != &trailer[80..112] {
        return Err(DeployError::new(format!(
            "embedded {kind} metadata checksum mismatch"
        )));
    }
    Ok(Some(EmbeddedPayload {
        application: application.to_vec(),
        metadata: metadata.to_vec(),
    }))
}

pub(crate) fn materialize_application(
    output: &Path,
    application: &[u8],
    client_environment: &[(String, String)],
    environment: DeployEnvironment,
    kind: &str,
) -> DeployResult<()> {
    reset_runtime_root(output)?;
    let artifact = output.join("app.dowebin");
    write_file(&artifact, application)?;
    cloud::materialize_cloud_artifact(&artifact, output)?;
    fs::remove_file(artifact)?;
    write_client_environment(output, client_environment, environment, kind)
}

pub(crate) fn validate_access_metadata(
    environment: DeployEnvironment,
    access_hash: Option<&str>,
    kind: &str,
) -> DeployResult<()> {
    match (environment, access_hash) {
        (DeployEnvironment::Live, None) => Ok(()),
        (DeployEnvironment::Stage | DeployEnvironment::Uat, Some(hash)) if is_sha256_hex(hash) => {
            Ok(())
        }
        _ => Err(DeployError::new(format!(
            "invalid embedded {kind} access metadata"
        ))),
    }
}

pub(crate) fn validate_client_environment(
    values: &[(String, String)],
    kind: &str,
) -> DeployResult<()> {
    if values.iter().any(|(name, _)| !is_environment_name(name)) {
        return Err(DeployError::new(format!(
            "invalid embedded {kind} environment name"
        )));
    }
    Ok(())
}

pub(crate) fn reset_runtime_root(output: &Path) -> DeployResult<()> {
    fs::create_dir_all(output)?;
    for entry in fs::read_dir(output)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if entry.file_name() == ".dowe" && file_type.is_dir() && !file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() && !file_type.is_symlink() {
            fs::remove_dir_all(entry.path())?;
        } else {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

pub(crate) fn set_executable(path: &Path) -> DeployResult<()> {
    set_executable_permissions(path)
}

fn write_client_environment(
    output: &Path,
    values: &[(String, String)],
    environment: DeployEnvironment,
    kind: &str,
) -> DeployResult<()> {
    validate_client_environment(values, kind)?;
    let mut content = String::new();
    for (name, value) in values {
        let escaped = value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        content.push_str(name);
        content.push_str("=\"");
        content.push_str(&escaped);
        content.push_str("\"\n");
    }
    if !content.is_empty() {
        write_file(&output.join(".env"), &content)?;
        write_file(
            &output.join(environment.compile_environment().file_name()),
            content,
        )?;
    }
    Ok(())
}

fn read_u64(bytes: &[u8], offset: usize, kind: &str) -> DeployResult<u64> {
    let end = offset
        .checked_add(8)
        .filter(|end| *end <= bytes.len())
        .ok_or_else(|| DeployError::new(format!("truncated embedded {kind} executable")))?;
    Ok(u64::from_le_bytes(
        bytes[offset..end].try_into().expect("u64 bytes"),
    ))
}

fn usize_value(value: u64, kind: &str) -> DeployResult<usize> {
    usize::try_from(value)
        .map_err(|_| DeployError::new(format!("embedded {kind} offset is too large")))
}

fn checked_end(offset: usize, length: usize, limit: usize, kind: &str) -> DeployResult<usize> {
    offset
        .checked_add(length)
        .filter(|end| *end <= limit)
        .ok_or_else(|| DeployError::new(format!("invalid embedded {kind} executable bounds")))
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_environment_name(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some(first) if first.is_ascii_uppercase())
        && characters.all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        })
}

#[cfg(unix)]
fn set_executable_permissions(path: &Path) -> DeployResult<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable_permissions(_path: &Path) -> DeployResult<()> {
    Ok(())
}
