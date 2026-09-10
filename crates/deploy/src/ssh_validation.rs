fn project_slug(root: &Path) -> DeployResult<String> {
    let name = root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| DeployError::new("SSH project name is missing"))?;
    let slug = name
        .chars()
        .map(|value| {
            if value.is_ascii_alphanumeric() {
                value.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    let slug = slug.trim_matches('-');
    if slug.is_empty() || slug.len() > 63 {
        return Err(DeployError::new("SSH project name is invalid"));
    }
    Ok(slug.to_string())
}

fn validate_host(value: &str) -> DeployResult<()> {
    if value.len() > 253
        || value.starts_with('-')
        || value.chars().any(|character| {
            !(character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | ':' | '[' | ']'))
        })
    {
        return Err(DeployError::new("SSH host is invalid"));
    }
    Ok(())
}

fn validate_user(value: &str) -> DeployResult<()> {
    if value.len() > 64
        || value.starts_with('-')
        || value.chars().any(|character| {
            !(character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.'))
        })
    {
        return Err(DeployError::new("SSH user is invalid"));
    }
    Ok(())
}

fn shell_word(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn write_environment_file(file: &mut impl Write, values: &[(String, String)]) -> DeployResult<()> {
    for (name, value) in values {
        let escaped = value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        writeln!(file, "{name}=\"{escaped}\"")?;
    }
    Ok(())
}

fn run_inherited(program: &str, args: &[String]) -> DeployResult<()> {
    let status = Command::new(program)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| DeployError::new(format!("failed to start {program}: {error}")))?;
    if !status.success() {
        return Err(DeployError::new(format!(
            "{program} failed with status {status}"
        )));
    }
    Ok(())
}


