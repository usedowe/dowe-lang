use crate::error::{DoweError, DoweResult};
use crate::model::{
    CompileEnvironment, EnvironmentConfig, EnvironmentValueSource, EnvironmentVariable,
    EnvironmentVisibility,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub(crate) fn parse_environment_files(root: &Path) -> DoweResult<EnvironmentConfig> {
    parse_environment_files_for(root, CompileEnvironment::Development)
}

pub(crate) fn parse_environment_files_for(
    root: &Path,
    environment: CompileEnvironment,
) -> DoweResult<EnvironmentConfig> {
    reject_environment_source(root, "env.dowe")?;
    reject_environment_source(root, "src/env.dowe")?;

    let example_path = root.join(".env.example");
    let local_path = root.join(environment.file_name());
    let examples = parse_optional_file(&example_path)?;
    let locals = parse_optional_file(&local_path)?;
    let names = examples
        .keys()
        .chain(locals.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let variables = names
        .into_iter()
        .map(|name| {
            let (resolved_source, resolved_value) = match std::env::var(&name) {
                Ok(value) => (EnvironmentValueSource::Os, Some(value)),
                Err(_) => match locals.get(&name) {
                    Some(value) => (EnvironmentValueSource::DotEnv, Some(value.clone())),
                    None => (EnvironmentValueSource::Missing, None),
                },
            };
            EnvironmentVariable {
                name,
                visibility: EnvironmentVisibility::Server,
                resolved_source,
                resolved_value,
            }
        })
        .collect();
    Ok(EnvironmentConfig { variables })
}

fn reject_environment_source(root: &Path, relative: &str) -> DoweResult<()> {
    let path = root.join(relative);
    if path.is_file() {
        return Err(DoweError::at_path(
            &path,
            format!(
                "`{relative}` is no longer supported; declare names in `.env.example`, development values in `.env`, deploy values in `.env.live`, `.env.stage`, or `.env.uat`, and keep using `env.NAME` in Dowe source"
            ),
        ));
    }
    Ok(())
}

fn parse_optional_file(path: &Path) -> DoweResult<BTreeMap<String, String>> {
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let source =
        fs::read_to_string(path).map_err(|error| DoweError::at_path(path, error.to_string()))?;
    parse_dotenv(path, &source)
}

fn parse_dotenv(path: &Path, source: &str) -> DoweResult<BTreeMap<String, String>> {
    let mut values = BTreeMap::new();
    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, raw_value)) = line.split_once('=') else {
            return Err(dotenv_error(path, line_number, "expected `NAME=value`"));
        };
        let name = name.trim();
        if !is_valid_environment_name(name) {
            return Err(dotenv_error(
                path,
                line_number,
                "environment variable names must use uppercase letters, numbers, and underscores",
            ));
        }
        if values.contains_key(name) {
            return Err(dotenv_error(
                path,
                line_number,
                format!("duplicate environment variable `{name}`"),
            ));
        }
        let value = parse_value(path, line_number, raw_value.trim())?;
        values.insert(name.to_string(), value);
    }
    Ok(values)
}

fn parse_value(path: &Path, line: usize, value: &str) -> DoweResult<String> {
    let Some(quote) = value
        .chars()
        .next()
        .filter(|value| matches!(value, '\'' | '"'))
    else {
        return Ok(value.to_string());
    };
    if value.len() < 2 || !value.ends_with(quote) {
        return Err(dotenv_error(path, line, "unterminated quoted value"));
    }
    let inner = &value[quote.len_utf8()..value.len() - quote.len_utf8()];
    if quote == '\'' {
        return Ok(inner.to_string());
    }
    decode_double_quoted(path, line, inner)
}

fn decode_double_quoted(path: &Path, line: usize, value: &str) -> DoweResult<String> {
    let mut output = String::new();
    let mut chars = value.chars();
    while let Some(value) = chars.next() {
        if value != '\\' {
            output.push(value);
            continue;
        }
        let escaped = chars
            .next()
            .ok_or_else(|| dotenv_error(path, line, "unterminated escape sequence"))?;
        output.push(match escaped {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '\\' => '\\',
            '"' => '"',
            _ => {
                return Err(dotenv_error(
                    path,
                    line,
                    format!("unsupported escape sequence `\\{escaped}`"),
                ));
            }
        });
    }
    Ok(output)
}

fn is_valid_environment_name(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_uppercase())
        && chars.all(|value| value.is_ascii_uppercase() || value.is_ascii_digit() || value == '_')
}

fn dotenv_error(path: &Path, line: usize, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(path, format!("{line}: {}", message.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::parse_dotenv;
    use std::path::Path;

    #[test]
    fn parses_empty_plain_and_quoted_values() {
        let values = parse_dotenv(
            Path::new("/project/.env"),
            "EMPTY=\nPLAIN=https://api.example.com\nSINGLE='one two'\nDOUBLE=\"line\\nvalue\"\n",
        )
        .expect("dotenv");

        assert_eq!(values.get("EMPTY").map(String::as_str), Some(""));
        assert_eq!(
            values.get("PLAIN").map(String::as_str),
            Some("https://api.example.com")
        );
        assert_eq!(values.get("SINGLE").map(String::as_str), Some("one two"));
        assert_eq!(
            values.get("DOUBLE").map(String::as_str),
            Some("line\nvalue")
        );
    }

    #[test]
    fn rejects_invalid_names_lines_and_quotes() {
        for source in ["lower=value\n", "MISSING\n", "BROKEN=\"value\n"] {
            let error = parse_dotenv(Path::new("/project/.env"), source).expect_err("invalid");

            assert!(error.to_string().contains("/project/.env"));
            assert!(error.to_string().contains("1:"));
        }
    }
}
