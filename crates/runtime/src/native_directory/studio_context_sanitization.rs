pub(super) async fn run_studio_blocking<T, F>(work: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        std::thread::Builder::new()
            .name("dowe-studio-context".to_string())
            .stack_size(64 * 1024 * 1024)
            .spawn(work)
            .map_err(|error| error.to_string())?
            .join()
            .map_err(|_| "Studio context worker panicked".to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

pub(super) fn sanitize_studio_text(value: &str, root: &Path) -> String {
    let root = root.to_string_lossy().replace('\\', "/");
    value
        .replace(&root, "<workspace>")
        .replace(&root.replace('/', "\\"), "<workspace>")
}

pub(super) fn visible_studio_path(path: &str) -> bool {
    let path = Path::new(path);
    !path.components().any(|component| {
        let Component::Normal(name) = component else {
            return true;
        };
        matches!(
            name.to_string_lossy().as_ref(),
            ".env" | ".agents" | ".dowe" | ".git" | "AGENTS.md" | "CLAUDE.md"
        )
    })
}

pub(super) fn studio_workspace_id(root: &Path) -> String {
    let mut digest = Sha256::new();
    digest.update(b"dowe-studio-workspace:");
    digest.update(root.to_string_lossy().as_bytes());
    hex_digest(&digest.finalize())
}

pub(super) fn hex_digest(digest: &[u8]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
