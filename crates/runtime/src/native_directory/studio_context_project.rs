pub(super) async fn inspect_dowe_project(args: &Value) -> RuntimeResult<Value> {
    let path = PathBuf::from(required_string(args, "path")?);
    let metadata = tokio::fs::symlink_metadata(&path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project path: {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Ok(json!("invalid"));
    }
    let main = path.join("main.dowe");
    if tokio::fs::symlink_metadata(&main)
        .await
        .map(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Ok(json!(if is_valid_dowe_project(&path).await {
            "dowe"
        } else {
            "invalid"
        }));
    }
    Ok(json!(if is_empty_directory(&path).await? {
        "empty"
    } else {
        "invalid"
    }))
}

pub(super) const STUDIO_CONTEXT_PROTOCOL_VERSION: u64 = 1;
pub(super) const STUDIO_CONTEXT_MAX_FILES: usize = 24;
pub(super) const STUDIO_CONTEXT_MAX_FILE_BYTES: usize = 64 * 1024;
pub(super) const STUDIO_CONTEXT_MAX_TOTAL_BYTES: usize = 256 * 1024;
pub(super) const STUDIO_CONTEXT_MAX_SOURCE_READ_BYTES: usize = 2 * 1024 * 1024;
pub(super) const STUDIO_CONTEXT_MAX_QUERY_BYTES: usize = 4096;
pub(super) const STUDIO_CONTEXT_MAX_IMAGE_BYTES: usize = 512 * 1024;
pub(super) const STUDIO_CONTEXT_MAX_IMPORTS: usize = 64;
pub(super) const STUDIO_CONTEXT_MAX_DECLARATIONS: usize = 128;
pub(super) const STUDIO_CONTEXT_MAX_GRAPH_NODES: usize = 24;
pub(super) const STUDIO_CONTEXT_MAX_DIRECTORIES: usize = 4096;
pub(super) const STUDIO_CONTEXT_MAX_SOURCE_FILES: usize = 4096;
pub(super) const STUDIO_STAGE_MAX_FILES: usize = 24;
pub(super) const STUDIO_STAGE_MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
pub(super) const STUDIO_STAGE_MAX_TOTAL_BYTES: usize = 512 * 1024;
pub(super) const STUDIO_STAGE_MAX_PLAN_BYTES: usize = 768 * 1024;
pub(super) const STUDIO_STAGE_MAX_DIFF_BYTES: usize = 256 * 1024;
pub(super) const STUDIO_STAGE_MAX_PATCH_BYTES: usize = 16 * 1024;
pub(super) const STUDIO_STAGE_MAX_TEST_OUTPUT_BYTES: usize = 128 * 1024;
pub(super) const STUDIO_STAGE_DIRECTORY: &str = "studio-staging";

#[derive(Clone)]
pub(super) struct StudioSourceEntry {
    pub(super) relative: String,
    path: PathBuf,
    size: u64,
}

pub(super) struct StudioSourceContent {
    content: String,
    pub(super) digest: String,
}

pub(super) async fn full_studio_source_fingerprint(root: &Path) -> RuntimeResult<String> {
    let mut entries = Vec::new();
    collect_studio_sources(root, root, &mut entries).await?;
    entries.sort_by(|left, right| left.relative.cmp(&right.relative));
    let mut digest = Sha256::new();
    for entry in entries {
        let source = read_studio_source(&entry).await?;
        digest.update(entry.relative.as_bytes());
        digest.update([0]);
        digest.update(source.digest.as_bytes());
    }
    Ok(hex_digest(&digest.finalize()))
}

