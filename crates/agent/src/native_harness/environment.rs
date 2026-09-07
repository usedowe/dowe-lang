use crate::{AgentError, AgentResult};
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn set_local_environment_value(
    root: impl AsRef<Path>,
    profile: &str,
    name: &str,
    value: &str,
) -> AgentResult<()> {
    if !matches!(profile, ".env" | ".env.live" | ".env.stage" | ".env.uat") {
        return Err(AgentError::new(
            "environment profile must be .env, .env.live, .env.stage or .env.uat",
        ));
    }
    if name.is_empty()
        || name.len() > 128
        || !name
            .bytes()
            .enumerate()
            .all(|(i, b)| b == b'_' || b.is_ascii_alphabetic() || i > 0 && b.is_ascii_digit())
        || value.chars().any(char::is_control)
        || value.len() > 16384
        || value.contains(['"', '\\'])
    {
        return Err(AgentError::new(
            "invalid environment name/value; multiline or escaped values require local editor",
        ));
    }
    let root = fs::canonicalize(root)?;
    let path = root.join(profile);
    let ignored = root.join(".gitignore");
    for candidate in [&path, &ignored] {
        if fs::symlink_metadata(candidate).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err(AgentError::new(
                "local environment editing does not traverse symlinks",
            ));
        }
    }
    let ignores = fs::read_to_string(ignored).unwrap_or_default();
    if !ignores
        .lines()
        .any(|line| line.trim().trim_start_matches('/') == profile)
    {
        return Err(AgentError::new(
            "add an explicit environment profile entry to .gitignore before storing secrets; review Git tracking separately",
        ));
    }
    if ignores.lines().any(|line| {
        line.trim().starts_with('!')
            && line.trim().trim_start_matches('!').trim_start_matches('/') != ".env.example"
    }) {
        return Err(AgentError::new(
            "negative gitignore rules require manual review before protected environment editing",
        ));
    }
    let _lock = crate::auth::AuthFileLock::acquire(&root.join(format!("{profile}.agent.lock")))?;
    let before = match fs::read_to_string(&path) {
        Ok(text) if text.len() <= 1048576 => text,
        Ok(_) => return Err(AgentError::new("environment file exceeds edit limit")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };
    let matches = before
        .lines()
        .filter(|line| {
            line.split_once('=')
                .is_some_and(|(key, _)| key.trim() == name)
        })
        .count();
    if matches > 1 {
        return Err(AgentError::new(
            "duplicate environment key requires local editor",
        ));
    }
    let replacement = format!("{name}=\"{value}\"\n");
    let mut next = String::new();
    for line in before.split_inclusive('\n') {
        if line
            .split_once('=')
            .is_some_and(|(key, _)| key.trim() == name)
        {
            next.push_str(&replacement);
        } else {
            next.push_str(line);
        }
    }
    if matches == 0 {
        if !next.is_empty() && !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str(&replacement);
    }
    if !ignores
        .lines()
        .any(|line| matches!(line.trim().trim_matches('/'), ".dowe"))
    {
        return Err(AgentError::new(
            "ignore .dowe generated artifacts before staging environment values",
        ));
    }
    let staging = root.join(".dowe").join("agent-env");
    for candidate in [root.join(".dowe"), staging.clone()] {
        if fs::symlink_metadata(&candidate).is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err(AgentError::new(
                "environment staging cannot traverse symlinks",
            ));
        }
    }
    fs::create_dir_all(&staging)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&staging, fs::Permissions::from_mode(0o700))?;
    }
    let temporary = staging.join(format!("{}.tmp", super::identifier()));
    let mut options = fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let cleanup = TemporaryEnvironmentFile(temporary.clone());
    let mut file = options.open(&temporary)?;
    file.write_all(next.as_bytes())?;
    file.sync_all()?;
    if fs::read_to_string(&path).unwrap_or_default() != before {
        fs::remove_file(temporary)?;
        return Err(AgentError::new(
            "environment file changed during local edit",
        ));
    }
    fs::rename(temporary, path)?;
    drop(cleanup);
    Ok(())
}

struct TemporaryEnvironmentFile(std::path::PathBuf);
impl Drop for TemporaryEnvironmentFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
