use crate::error::{DoweError, DoweResult};
use crate::parser::source_ast::{SourceObjectEntry, SourceValue};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceToken {
    pub column: usize,
    pub text: String,
}

pub fn split_top_level_whitespace(
    source: &str,
    base_column: usize,
) -> DoweResult<Vec<SourceToken>> {
    let mut tokens = Vec::new();
    let mut start = None;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut string_delimiter = None;
    let mut escaped = false;
    let mut multiline_string = false;

    for (index, value) in source.char_indices() {
        if source[index..].starts_with("\"\"\"") {
            start.get_or_insert(index);
            multiline_string = !multiline_string;
            continue;
        }
        if multiline_string {
            continue;
        }
        if let Some(delimiter) = string_delimiter {
            if escaped {
                escaped = false;
            } else if value == '\\' {
                escaped = true;
            } else if value == delimiter {
                string_delimiter = None;
            }
            continue;
        }

        match value {
            '"' => {
                start.get_or_insert(index);
                string_delimiter = Some(value);
            }
            '{' => {
                start.get_or_insert(index);
                brace_depth += 1;
            }
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '[' => {
                start.get_or_insert(index);
                bracket_depth += 1;
            }
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            value if value.is_whitespace() && brace_depth == 0 && bracket_depth == 0 => {
                if let Some(open) = start.take() {
                    tokens.push(SourceToken {
                        column: base_column + open + 1,
                        text: source[open..index].to_string(),
                    });
                }
            }
            _ => {
                start.get_or_insert(index);
            }
        }
    }

    if string_delimiter.is_some() || multiline_string {
        return Err(DoweError::new("missing string closing quote"));
    }

    if let Some(open) = start {
        tokens.push(SourceToken {
            column: base_column + open + 1,
            text: source[open..].to_string(),
        });
    }

    Ok(tokens)
}

pub fn parse_value(
    path: &Path,
    line: usize,
    column: usize,
    source: &str,
) -> DoweResult<SourceValue> {
    let value = source.trim();
    if value.is_empty() {
        return Err(DoweError::at_path(
            path,
            format!("{line}:{column}: missing value"),
        ));
    }
    if value.starts_with("\"\"\"") {
        return parse_multiline_string(path, line, column, value).map(SourceValue::String);
    }
    if value.starts_with('"') {
        return parse_string(path, line, column, value).map(SourceValue::String);
    }
    if value == "true" {
        return Ok(SourceValue::Boolean(true));
    }
    if value == "false" {
        return Ok(SourceValue::Boolean(false));
    }
    if value == "null" {
        return Ok(SourceValue::Null);
    }
    if value.starts_with('[') {
        return parse_array(path, line, column, value);
    }
    if value.starts_with('{') {
        return parse_object(path, line, column, value);
    }
    if is_numeric_literal(value) {
        return Ok(SourceValue::Number(value.to_string()));
    }
    Ok(SourceValue::Bareword(value.to_string()))
}

fn parse_multiline_string(
    path: &Path,
    line: usize,
    column: usize,
    source: &str,
) -> DoweResult<String> {
    if !source.ends_with("\"\"\"") || source.matches("\"\"\"").count() != 2 {
        return Err(DoweError::at_path(
            path,
            format!("{line}:{column}: invalid multiline string delimiter"),
        ));
    }
    let body = &source[3..source.len() - 3];
    let body = body.strip_prefix('\n').unwrap_or(body);
    let body = body.strip_suffix('\n').unwrap_or(body);
    let lines = body.lines().collect::<Vec<_>>();
    let common_indent = lines
        .iter()
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.chars().take_while(|value| *value == ' ').count())
        .min()
        .unwrap_or(0);
    let normalized = lines
        .iter()
        .map(|value| {
            if value.trim().is_empty() {
                String::new()
            } else {
                value.chars().skip(common_indent).collect()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .replace("\\\"\\\"\\\"", "\"\"\"");
    if normalized.is_empty() {
        return Err(DoweError::at_path(
            path,
            format!("{line}:{column}: multiline string cannot be empty"),
        ));
    }
    Ok(normalized)
}

fn parse_string(path: &Path, line: usize, column: usize, source: &str) -> DoweResult<String> {
    if !source.ends_with('"') || source.len() < 2 {
        return Err(DoweError::at_path(
            path,
            format!("{line}:{column}: missing string closing quote"),
        ));
    }

    let mut output = String::new();
    let mut escaped = false;
    for value in source[1..source.len() - 1].chars() {
        if escaped {
            output.push(value);
            escaped = false;
        } else if value == '\\' {
            escaped = true;
        } else {
            output.push(value);
        }
    }

    if escaped {
        return Err(DoweError::at_path(
            path,
            format!("{line}:{column}: missing escaped value"),
        ));
    }

    Ok(output)
}

fn parse_array(path: &Path, line: usize, column: usize, source: &str) -> DoweResult<SourceValue> {
    if !source.starts_with('[') || !source.ends_with(']') {
        return Err(DoweError::at_path(
            path,
            format!("{line}:{column}: invalid array value"),
        ));
    }

    let body = &source[1..source.len() - 1];
    let values = split_top_level_array_items(body, column + 1)?
        .into_iter()
        .map(|token| parse_value(path, line, token.column, &token.text))
        .collect::<DoweResult<Vec<_>>>()?;
    Ok(SourceValue::Array(values))
}

fn parse_object(path: &Path, line: usize, column: usize, source: &str) -> DoweResult<SourceValue> {
    if !source.starts_with('{') || !source.ends_with('}') {
        return Err(DoweError::at_path(
            path,
            format!("{line}:{column}: invalid object value"),
        ));
    }

    let body = source[1..source.len() - 1].trim();
    if body.is_empty() {
        return Ok(SourceValue::Object(Vec::new()));
    }

    let mut entries = Vec::new();
    for token in split_top_level_whitespace(body, column + 1)? {
        if let Some(spread) = token.text.strip_prefix("...") {
            if spread.is_empty() {
                return Err(DoweError::at_path(
                    path,
                    format!("{line}:{}: invalid spread value", token.column),
                ));
            }
            entries.push(SourceObjectEntry::Spread(spread.to_string()));
            continue;
        }
        let Some((key, value)) = token.text.split_once(':') else {
            return Err(DoweError::at_path(
                path,
                format!("{line}:{}: invalid object entry", token.column),
            ));
        };
        if key.is_empty() || value.is_empty() {
            return Err(DoweError::at_path(
                path,
                format!("{line}:{}: invalid object entry", token.column),
            ));
        }
        entries.push(SourceObjectEntry::KeyValue {
            key: key.to_string(),
            value: parse_value(path, line, token.column + key.len() + 1, value)?,
        });
    }

    Ok(SourceValue::Object(entries))
}

fn split_top_level_array_items(source: &str, base_column: usize) -> DoweResult<Vec<SourceToken>> {
    let mut tokens = Vec::new();
    let mut start = None;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut string_delimiter = None;
    let mut escaped = false;
    let mut multiline_string = false;

    let mut push_token = |end: usize, start: &mut Option<usize>| {
        if let Some(open) = start.take() {
            let text = source[open..end].trim();
            if !text.is_empty() {
                let offset = source[open..].find(text).unwrap_or(0);
                tokens.push(SourceToken {
                    column: base_column + open + offset + 1,
                    text: text.to_string(),
                });
            }
        }
    };

    for (index, value) in source.char_indices() {
        if source[index..].starts_with("\"\"\"") {
            start.get_or_insert(index);
            multiline_string = !multiline_string;
            continue;
        }
        if multiline_string {
            continue;
        }
        if let Some(delimiter) = string_delimiter {
            if escaped {
                escaped = false;
            } else if value == '\\' {
                escaped = true;
            } else if value == delimiter {
                string_delimiter = None;
            }
            continue;
        }

        match value {
            '"' => {
                start.get_or_insert(index);
                string_delimiter = Some(value);
            }
            '{' => {
                start.get_or_insert(index);
                brace_depth += 1;
            }
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '[' => {
                start.get_or_insert(index);
                bracket_depth += 1;
            }
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if brace_depth == 0 && bracket_depth == 0 => push_token(index, &mut start),
            value if value.is_whitespace() && brace_depth == 0 && bracket_depth == 0 => {
                push_token(index, &mut start);
            }
            _ => {
                start.get_or_insert(index);
            }
        }
    }

    if string_delimiter.is_some() || multiline_string {
        return Err(DoweError::new("missing string closing quote"));
    }

    push_token(source.len(), &mut start);
    Ok(tokens)
}

fn is_numeric_literal(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }

    let mut dot_seen = false;
    let mut digit_seen = false;
    for (index, value) in value.chars().enumerate() {
        if value.is_ascii_digit() {
            digit_seen = true;
        } else if index == 0 && matches!(value, '-' | '+') {
            continue;
        } else if value == '.' && !dot_seen {
            dot_seen = true;
        } else {
            return false;
        }
    }
    digit_seen
}

#[cfg(test)]
mod tests {
    use super::parse_value;
    use crate::parser::source_ast::{SourceObjectEntry, SourceValue};
    use std::path::Path;

    #[test]
    fn parses_whitespace_separated_arrays() {
        let value = parse_value(
            Path::new("/project/main.dowe"),
            1,
            1,
            "[\"GET\" \"POST\" PATCH]",
        )
        .expect("array");

        assert_eq!(
            value,
            SourceValue::Array(vec![
                SourceValue::String("GET".to_string()),
                SourceValue::String("POST".to_string()),
                SourceValue::Bareword("PATCH".to_string()),
            ])
        );
        assert_eq!(value.to_source(), "[\"GET\" \"POST\" PATCH]");
    }

    #[test]
    fn preserves_nested_array_and_object_boundaries() {
        let value = parse_value(
            Path::new("/project/main.dowe"),
            1,
            1,
            "[{ name:\"one, two\" values:[1 2] } { name:\"two\" values:[3 4] }]",
        )
        .expect("nested array");

        let SourceValue::Array(items) = value else {
            panic!("expected array");
        };
        assert_eq!(items.len(), 2);
        assert!(matches!(
            items[0],
            SourceValue::Object(ref entries)
                if entries.iter().any(|entry| matches!(
                    entry,
                    SourceObjectEntry::KeyValue { key, value }
                        if key == "values"
                            && matches!(value, SourceValue::Array(values) if values.len() == 2)
                ))
        ));
    }

    #[test]
    fn accepts_comma_separated_migration_arrays() {
        let value =
            parse_value(Path::new("/project/main.dowe"), 1, 1, "[1, 2,]").expect("legacy array");

        assert_eq!(value.to_source(), "[1 2]");
    }
}
