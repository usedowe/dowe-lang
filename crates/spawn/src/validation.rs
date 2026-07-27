use crate::config::{EnvMode, SpawnConfig, StreamMode};
use crate::error::{SpawnError, SpawnPhase, SpawnResult};
use std::collections::BTreeMap;
use std::path::Path;

pub fn validate_config(config: &SpawnConfig) -> SpawnResult<()> {
    if config.command.trim().is_empty() {
        return Err(error(
            config,
            SpawnPhase::Validation,
            "command cannot be empty",
        ));
    }

    if let Some(cwd) = &config.options.cwd {
        validate_cwd(config, cwd)?;
    }

    if config.options.pty.is_some() && config.options.stderr == StreamMode::Pipe {
        return Err(error(
            config,
            SpawnPhase::Validation,
            "pty mode cannot provide a separate stderr pipe",
        ));
    }

    validate_environment_map(config, &config.options.env)?;
    for key in &config.options.env_remove {
        validate_environment_name(config, key)?;
    }

    #[cfg(not(unix))]
    if config.options.uid.is_some() || config.options.gid.is_some() {
        return Err(error(
            config,
            SpawnPhase::Validation,
            "uid and gid are only supported on Unix platforms",
        ));
    }

    Ok(())
}

pub fn apply_environment(
    command: &mut std::process::Command,
    config: &SpawnConfig,
) -> SpawnResult<()> {
    match config.options.env_mode {
        EnvMode::Inherit => {}
        EnvMode::Clean | EnvMode::Replace => {
            command.env_clear();
        }
    }

    for key in &config.options.env_remove {
        command.env_remove(key);
    }

    for (key, value) in &config.options.env {
        command.env(key, value);
    }

    Ok(())
}

pub fn apply_pty_environment(
    command: &mut portable_pty::CommandBuilder,
    config: &SpawnConfig,
) -> SpawnResult<()> {
    match config.options.env_mode {
        EnvMode::Inherit => {}
        EnvMode::Clean | EnvMode::Replace => {
            command.env_clear();
        }
    }

    for key in &config.options.env_remove {
        command.env_remove(key);
    }

    for (key, value) in &config.options.env {
        command.env(key, value);
    }

    Ok(())
}

fn validate_cwd(config: &SpawnConfig, cwd: &Path) -> SpawnResult<()> {
    if !cwd.exists() {
        return Err(error(
            config,
            SpawnPhase::Cwd,
            format!("cwd does not exist: {}", cwd.display()),
        ));
    }

    if !cwd.is_dir() {
        return Err(error(
            config,
            SpawnPhase::Cwd,
            format!("cwd is not a directory: {}", cwd.display()),
        ));
    }

    Ok(())
}

fn validate_environment_map(
    config: &SpawnConfig,
    env: &BTreeMap<String, String>,
) -> SpawnResult<()> {
    for (key, value) in env {
        validate_environment_name(config, key)?;
        if value.contains('\0') {
            return Err(error(
                config,
                SpawnPhase::Environment,
                format!("environment variable `{key}` contains a nul byte"),
            ));
        }
    }
    Ok(())
}

fn validate_environment_name(config: &SpawnConfig, key: &str) -> SpawnResult<()> {
    if key.is_empty() {
        return Err(error(
            config,
            SpawnPhase::Environment,
            "environment variable names cannot be empty",
        ));
    }

    if key.contains('=') || key.contains('\0') {
        return Err(error(
            config,
            SpawnPhase::Environment,
            format!("environment variable `{key}` has an invalid name"),
        ));
    }

    Ok(())
}

fn error(config: &SpawnConfig, phase: SpawnPhase, message: impl Into<String>) -> SpawnError {
    SpawnError::new(config.command.clone(), phase, message)
}
