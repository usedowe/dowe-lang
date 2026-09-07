use crate::{EnvMode, SpawnConfig, SpawnError, SpawnPhase, SpawnResult};
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;

pub(super) struct Prepared {
    pub executable: Vec<u16>,
    pub command_line: Vec<u16>,
    pub environment: Vec<u16>,
    pub cwd: Vec<u16>,
}

pub(super) fn prepare(config: &SpawnConfig) -> SpawnResult<Prepared> {
    let mut environment = std::process::Command::new(&config.command);
    environment.env_clear();
    if config.options.env_mode == EnvMode::Inherit {
        environment.envs(std::env::vars_os());
    }
    crate::validation::apply_environment(&mut environment, config)?;
    let cwd = config
        .options
        .cwd
        .clone()
        .map(Ok)
        .unwrap_or_else(std::env::current_dir)
        .map_err(|error| failure(config, error.to_string()))?;
    let cwd = std::path::absolute(cwd).map_err(|error| failure(config, error.to_string()))?;
    let executable = PathBuf::from(&config.command);
    let paths = if executable.is_absolute() {
        vec![executable]
    } else if executable.components().count() > 1 {
        vec![cwd.join(executable)]
    } else {
        let mut paths = vec![cwd.clone()];
        if let Some((_, Some(path))) = environment
            .get_envs()
            .find(|(key, _)| key.to_string_lossy().eq_ignore_ascii_case("PATH"))
        {
            paths.extend(std::env::split_paths(path).map(|path| cwd.join(path)));
        }
        paths
            .into_iter()
            .map(|path| path.join(&executable))
            .collect()
    };
    let executable = paths
        .into_iter()
        .flat_map(|path| {
            if path.extension().is_none() {
                vec![path.clone(), path.with_extension("exe")]
            } else {
                vec![path]
            }
        })
        .find(|path| path.is_file())
        .ok_or_else(|| failure(config, "executable not found in cwd or effective PATH"))?;
    if executable.extension().is_some_and(|value| {
        ["bat", "cmd"]
            .iter()
            .any(|extension| value.to_string_lossy().eq_ignore_ascii_case(extension))
    }) {
        return Err(failure(
            config,
            "ConPTY Group requires an executable, not implicit batch shell parsing",
        ));
    }
    let executable = wide(executable.as_os_str());
    let mut command_line = quote(&executable[..executable.len() - 1]);
    for arg in &config.args {
        command_line.push(b' ' as u16);
        command_line.extend(quote(&arg.encode_utf16().collect::<Vec<_>>()));
    }
    if command_line.contains(&0) {
        return Err(failure(config, "command arguments cannot contain NUL"));
    }
    command_line.push(0);
    if command_line.len() > 32767 {
        return Err(failure(
            config,
            "Windows command line exceeds 32767 UTF-16 units",
        ));
    }
    let mut block = Vec::new();
    for (key, value) in environment.get_envs() {
        if let Some(value) = value {
            block.extend(key.encode_wide());
            block.push(b'=' as u16);
            block.extend(value.encode_wide());
            block.push(0);
        }
    }
    if block.is_empty() {
        block.push(0);
    }
    block.push(0);
    Ok(Prepared {
        executable,
        command_line,
        environment: block,
        cwd: wide(cwd.as_os_str()),
    })
}

fn failure(config: &SpawnConfig, message: impl Into<String>) -> SpawnError {
    SpawnError::new(&config.command, SpawnPhase::CommandResolution, message)
}
fn wide(value: &std::ffi::OsStr) -> Vec<u16> {
    value.encode_wide().chain(Some(0)).collect()
}
fn quote(value: &[u16]) -> Vec<u16> {
    let mut output = vec![34];
    let mut slashes = 0;
    for unit in value {
        if *unit == 92 {
            slashes += 1;
            continue;
        }
        output.extend(std::iter::repeat_n(
            92,
            if *unit == 34 {
                slashes * 2 + 1
            } else {
                slashes
            },
        ));
        output.push(*unit);
        slashes = 0;
    }
    output.extend(std::iter::repeat_n(92, slashes * 2));
    output.push(34);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crt_arguments_preserve_quotes_backslashes_empty_and_unicode() {
        for (input, expected) in [
            ("", "\"\""),
            ("a b", "\"a b\""),
            ("x\\", "\"x\\\\\""),
            ("a\"b", "\"a\\\"b\""),
            ("日本語", "\"日本語\""),
        ] {
            assert_eq!(
                String::from_utf16(&quote(&input.encode_utf16().collect::<Vec<_>>())).unwrap(),
                expected
            );
        }
    }
    #[test]
    fn native_environment_uses_clean_replace_remove_and_case_insensitive_overrides() {
        let mut config = SpawnConfig::new(
            std::env::current_exe().unwrap().to_str().unwrap(),
            [] as [&str; 0],
        );
        config.options.env_mode = EnvMode::Replace;
        config.options.env.insert("Path".into(), "one".into());
        config.options.env.insert("PATH".into(), "two".into());
        config.options.env.insert("VALUE".into(), "日本語".into());
        let prepared = prepare(&config).unwrap();
        let values: Vec<_> = prepared
            .environment
            .split(|unit| *unit == 0)
            .filter(|value| !value.is_empty())
            .map(|value| String::from_utf16(value).unwrap())
            .collect();
        assert_eq!(values.len(), 2);
        assert_eq!(
            values
                .iter()
                .filter(|value| value.to_ascii_lowercase().starts_with("path="))
                .count(),
            1
        );
        assert!(values.iter().any(|value| value == "VALUE=日本語"));
        assert!(prepared.environment.ends_with(&[0, 0]));
        assert!(
            values
                .iter()
                .any(|value| value.eq_ignore_ascii_case("PATH=one"))
        );
        config.options.env.clear();
        config.options.env_mode = EnvMode::Inherit;
        config.options.env_remove.push("PATH".into());
        let removed = prepare(&config).unwrap();
        assert!(
            !removed
                .environment
                .split(|unit| *unit == 0)
                .filter_map(|value| String::from_utf16(value).ok())
                .any(|value| value.to_ascii_lowercase().starts_with("path="))
        );
    }
}
