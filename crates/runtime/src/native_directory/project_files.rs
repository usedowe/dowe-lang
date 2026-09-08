use super::studio_context::*;
use super::*;
pub(super) async fn studio_sketch(root: &Path, args: &Value) -> RuntimeResult<Value> {
    let image = required_string(args, "image")?;
    let encoded = image
        .strip_prefix("data:image/png;base64,")
        .ok_or_else(|| RuntimeError::new("sketch must be a PNG data URL"))?;
    if image.len() > STUDIO_CONTEXT_MAX_IMAGE_BYTES {
        return Err(RuntimeError::new("sketch image exceeds its size limit"));
    }
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
        .map_err(|error| RuntimeError::new(format!("invalid sketch image: {error}")))?;
    if bytes.len() > STUDIO_CONTEXT_MAX_IMAGE_BYTES {
        return Err(RuntimeError::new("sketch image exceeds its size limit"));
    }
    let directory = root.join(".dowe");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    tokio::fs::write(directory.join("latest-sketch.png"), bytes)
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    Ok(json!(true))
}

pub(super) async fn valid_dowe_project(args: &Value) -> RuntimeResult<Value> {
    let path = PathBuf::from(required_string(args, "path")?);
    Ok(json!(is_valid_dowe_project(&path).await))
}

pub(super) async fn is_valid_dowe_project(path: &Path) -> bool {
    let Ok(metadata) = tokio::fs::symlink_metadata(path).await else {
        return false;
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return false;
    }
    let main = path.join("main.dowe");
    let Ok(main_metadata) = tokio::fs::symlink_metadata(&main).await else {
        return false;
    };
    if main_metadata.file_type().is_symlink() || !main_metadata.is_file() {
        return false;
    }
    let Ok(root) = tokio::fs::canonicalize(path).await else {
        return false;
    };
    run_studio_blocking(move || dowe_compiler::inspect_project_capabilities(&root).is_ok())
        .await
        .unwrap_or(false)
}

pub(super) async fn validated_dowe_project_root(value: &str) -> RuntimeResult<PathBuf> {
    let path = PathBuf::from(value);
    let metadata = tokio::fs::symlink_metadata(&path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(RuntimeError::new(
            "selected application path must be a regular directory",
        ));
    }
    let root = tokio::fs::canonicalize(&path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    if !is_valid_dowe_project(&root).await {
        return Err(RuntimeError::new(
            "selected folder does not contain a valid Dowe project",
        ));
    }
    Ok(root)
}

pub(super) async fn empty_dowe_folder(args: &Value) -> RuntimeResult<Value> {
    let path = PathBuf::from(required_string(args, "path")?);
    Ok(json!(is_empty_directory(&path).await?))
}

pub(super) async fn is_empty_directory(path: &Path) -> RuntimeResult<bool> {
    let metadata = tokio::fs::symlink_metadata(path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(RuntimeError::new(
            "project path must be a regular directory",
        ));
    }
    let mut entries = tokio::fs::read_dir(path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    Ok(entries
        .next_entry()
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))?
        .is_none())
}

pub(super) async fn initialize_dowe_project(args: &Value) -> RuntimeResult<Value> {
    let path = PathBuf::from(required_string(args, "path")?);
    let template_name = args
        .get("template")
        .and_then(Value::as_str)
        .unwrap_or("blank");
    let template = ProjectTemplate::from_str(template_name)
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    match tokio::fs::symlink_metadata(&path).await {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(RuntimeError::new(
                "project path must not be a symbolic link",
            ));
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err(RuntimeError::new("project path must be a directory"));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            tokio::fs::create_dir_all(&path)
                .await
                .map_err(|error| RuntimeError::new(error.to_string()))?;
        }
        Err(error) => return Err(RuntimeError::new(error.to_string())),
    }
    let root = tokio::fs::canonicalize(&path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    if !is_empty_directory(&root).await? {
        return Err(RuntimeError::new(
            "new Dowe projects must use an empty folder",
        ));
    }
    let initialized_root = root.clone();
    run_studio_blocking(move || init_project(&initialized_root, InitProjectOptions::new(template)))
        .await
        .map_err(|error| RuntimeError::new(format!("could not initialize Dowe project: {error}")))?
        .map_err(|error| {
            RuntimeError::new(format!("could not initialize Dowe project: {error}"))
        })?;
    Ok(json!(root.to_string_lossy().into_owned()))
}

pub(super) async fn clone_dowe_repository(args: &Value) -> RuntimeResult<Value> {
    let url = normalized_github_repository_url(&required_string(args, "url")?)?;
    let requested_destination = PathBuf::from(required_string(args, "destination")?);
    if !is_empty_directory(&requested_destination).await? {
        return Err(RuntimeError::new(
            "repository imports require an empty destination folder",
        ));
    }
    let destination = tokio::fs::canonicalize(&requested_destination)
        .await
        .map_err(|error| {
            RuntimeError::new(format!("could not inspect repository folder: {error}"))
        })?;
    let destination_arg = destination.to_string_lossy().into_owned();
    let output = tokio::time::timeout(
        Duration::from_secs(120),
        tokio::process::Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--no-tags",
                "--single-branch",
                &url,
                &destination_arg,
            ])
            .env("GIT_TERMINAL_PROMPT", "0")
            .output(),
    )
    .await
    .map_err(|_| RuntimeError::new("GitHub repository clone timed out"))?
    .map_err(|error| RuntimeError::new(format!("could not start git clone: {error}")))?;
    if !output.status.success() {
        let details = if output.stderr.is_empty() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };
        let details =
            truncate_studio_text(&sanitize_studio_text(details.trim(), &destination), 512);
        let details = if details.is_empty() {
            "git clone returned a failure".to_string()
        } else {
            details
        };
        return Err(RuntimeError::new(format!(
            "could not clone GitHub repository: {details}"
        )));
    }
    if !is_valid_dowe_project(&destination).await {
        return Err(RuntimeError::new(
            "the GitHub repository does not contain a valid Dowe project",
        ));
    }
    Ok(json!(destination.to_string_lossy().into_owned()))
}

pub(super) fn normalized_github_repository_url(value: &str) -> RuntimeResult<String> {
    let parsed = reqwest::Url::parse(value.trim())
        .map_err(|_| RuntimeError::new("GitHub repository URL is invalid"))?;
    let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
    if parsed.scheme() != "https"
        || !matches!(host.as_str(), "github.com" | "www.github.com")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(RuntimeError::new(
            "repository imports accept public HTTPS GitHub URLs only",
        ));
    }
    let parts = parsed
        .path_segments()
        .ok_or_else(|| RuntimeError::new("GitHub repository URL has no repository path"))?
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err(RuntimeError::new(
            "GitHub repository URL must contain an owner and repository",
        ));
    }
    let repository = parts[1].strip_suffix(".git").unwrap_or(parts[1]);
    if !valid_github_segment(parts[0]) || !valid_github_segment(repository) {
        return Err(RuntimeError::new(
            "GitHub repository URL contains invalid names",
        ));
    }
    Ok(format!(
        "https://github.com/{}/{}.git",
        parts[0], repository
    ))
}

pub(super) fn valid_github_segment(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value.len() <= 100
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

#[derive(Clone)]
pub(super) struct StudioChangeSpec {
    pub(super) path: String,
    pub(super) operation: String,
    pub(super) owner: String,
    pub(super) content: String,
}

pub(super) async fn create_folder(args: &Value) -> RuntimeResult<Value> {
    let parent = required_string(args, "parent")?;
    let name = required_string(args, "name")?;
    validate_folder_name(&name)?;
    let parent = PathBuf::from(parent);
    let destination = parent.join(&name);
    tokio::fs::create_dir(&destination)
        .await
        .map_err(|error| RuntimeError::new(format!("could not create folder: {error}")))?;
    Ok(json!(destination.to_string_lossy().into_owned()))
}

pub(super) async fn list_project_files(args: &Value) -> RuntimeResult<Value> {
    let root = PathBuf::from(required_string(args, "root")?);
    let root = tokio::fs::canonicalize(&root)
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    collect_project_tree(&root, &root).await
}

pub(super) async fn collect_project_tree(root: &Path, directory: &Path) -> RuntimeResult<Value> {
    let mut entries = tokio::fs::read_dir(directory)
        .await
        .map_err(|error| RuntimeError::new(format!("could not list project files: {error}")))?;
    let mut files = Vec::new();
    let mut folders = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| RuntimeError::new(format!("could not read project entry: {error}")))?
    {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.')
            || matches!(name.as_str(), "AGENTS.md" | "CLAUDE.md")
            || name == ".dowe"
        {
            continue;
        }
        let path = entry.path();
        let metadata = tokio::fs::symlink_metadata(&path).await.map_err(|error| {
            RuntimeError::new(format!("could not read project metadata: {error}"))
        })?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if metadata.is_dir() {
            if should_skip_project_directory(&name) {
                continue;
            }
            let tree = Box::pin(collect_project_tree(root, &path)).await?;
            folders.push(json!({
                "id": relative,
                "name": name,
                "path": relative,
                "files": tree.get("files").cloned().unwrap_or_else(|| json!([])),
                "folders": tree.get("folders").cloned().unwrap_or_else(|| json!([])),
            }));
        } else if metadata.is_file() {
            files.push(json!({
                "id": relative,
                "name": name,
                "path": relative,
                "icon": project_file_icon(&name),
            }));
        }
    }
    files.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    folders.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    Ok(json!({ "files": files, "folders": folders }))
}

pub(super) fn should_skip_project_directory(name: &str) -> bool {
    matches!(
        name,
        "build" | "coverage" | "dist" | "node_modules" | "out" | "target"
    )
}

pub(super) fn project_file_icon(name: &str) -> &'static str {
    match Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("dowe") | Some("rs") | Some("ts") | Some("js") | Some("swift") | Some("kt") => {
            "code-file"
        }
        Some("json") | Some("toml") | Some("yaml") | Some("yml") | Some("md") | Some("txt") => {
            "file-text"
        }
        Some("png") | Some("jpg") | Some("jpeg") | Some("webp") | Some("svg") => "gallery",
        _ => "file",
    }
}

pub(super) async fn read_project_file(args: &Value) -> RuntimeResult<Value> {
    let root = tokio::fs::canonicalize(PathBuf::from(required_string(args, "root")?))
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    let relative = required_string(args, "path")?;
    validate_project_relative_path(&relative)?;
    let path = root.join(&relative);
    let mut current = root.clone();
    for component in Path::new(&relative).components() {
        let Component::Normal(name) = component else {
            return Err(RuntimeError::new("project file path must be relative"));
        };
        current.push(name);
        let metadata = tokio::fs::symlink_metadata(&current)
            .await
            .map_err(|error| RuntimeError::new(format!("could not read project file: {error}")))?;
        if metadata.file_type().is_symlink() {
            return Err(RuntimeError::new(
                "project file path cannot contain symlinks",
            ));
        }
    }
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not read project file: {error}")))?;
    if !metadata.is_file() {
        return Err(RuntimeError::new("project path is not a file"));
    }
    if metadata.len() > MAX_PROJECT_FILE_BYTES {
        return Err(RuntimeError::new("project file exceeds the 2 MiB limit"));
    }
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not read project file: {error}")))?;
    Ok(json!(content))
}

const MAX_PROJECT_FILE_BYTES: u64 = 2 * 1024 * 1024;

pub(super) fn validate_project_relative_path(value: &str) -> RuntimeResult<()> {
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::CurDir
                    | Component::ParentDir
                    | Component::RootDir
                    | Component::Prefix(_)
            )
        })
    {
        return Err(RuntimeError::new("project file path must be relative"));
    }
    Ok(())
}

pub(super) async fn write_project_file(args: &Value) -> RuntimeResult<Value> {
    let root = tokio::fs::canonicalize(PathBuf::from(required_string(args, "root")?))
        .await
        .map_err(|error| RuntimeError::new(format!("could not inspect project folder: {error}")))?;
    let relative = required_string(args, "path")?;
    validate_project_relative_path(&relative)?;
    let content = args
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| RuntimeError::new("native directory argument `content` is required"))?;
    if content.len() as u64 > MAX_PROJECT_FILE_BYTES {
        return Err(RuntimeError::new("project file exceeds the 2 MiB limit"));
    }
    let path = root.join(&relative);
    let mut current = root.clone();
    for component in Path::new(&relative).components() {
        let Component::Normal(name) = component else {
            return Err(RuntimeError::new("project file path must be relative"));
        };
        current.push(name);
        let metadata = tokio::fs::symlink_metadata(&current)
            .await
            .map_err(|error| RuntimeError::new(format!("could not write project file: {error}")))?;
        if metadata.file_type().is_symlink() {
            return Err(RuntimeError::new(
                "project file path cannot contain symlinks",
            ));
        }
    }
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|error| RuntimeError::new(format!("could not write project file: {error}")))?;
    if !metadata.is_file() {
        return Err(RuntimeError::new("project path is not a file"));
    }
    tokio::fs::write(path, content)
        .await
        .map_err(|error| RuntimeError::new(format!("could not write project file: {error}")))?;
    Ok(json!(true))
}

pub(super) async fn list_folders(args: &Value) -> RuntimeResult<Value> {
    let parent = PathBuf::from(required_string(args, "parent")?);
    let mut entries = tokio::fs::read_dir(&parent)
        .await
        .map_err(|error| RuntimeError::new(format!("could not list folders: {error}")))?;
    let mut folders = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| RuntimeError::new(format!("could not read folder entry: {error}")))?
    {
        let metadata = entry.metadata().await.map_err(|error| {
            RuntimeError::new(format!("could not read folder metadata: {error}"))
        })?;
        if metadata.is_dir() {
            folders.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    folders.sort_unstable();
    Ok(json!(folders))
}
