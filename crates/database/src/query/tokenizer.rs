use crate::error::{StoreError, StoreResult};
use serde_json::Value;

pub fn bind_query_params(sql: &str, params: &[Value]) -> StoreResult<String> {
    let mut output = String::with_capacity(sql.len());
    let chars = sql.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    while index < chars.len() {
        let value = chars[index];
        if in_string {
            output.push(value);
            if escaped {
                escaped = false;
            } else if value == '\\' {
                escaped = true;
            } else if value == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if value == '"' {
            in_string = true;
            output.push(value);
            index += 1;
            continue;
        }
        if value != '?' {
            output.push(value);
            index += 1;
            continue;
        }
        let start = index + 1;
        let mut end = start;
        while end < chars.len() && chars[end].is_ascii_digit() {
            end += 1;
        }
        if end == start {
            output.push(value);
            index += 1;
            continue;
        }
        let parameter = chars[start..end]
            .iter()
            .collect::<String>()
            .parse::<usize>()
            .map_err(|_| StoreError::InvalidQuery("query parameter is invalid".to_string()))?;
        let value = params.get(parameter.saturating_sub(1)).ok_or_else(|| {
            StoreError::InvalidQuery(format!("query parameter `?{parameter}` is missing"))
        })?;
        output.push_str(&serde_json::to_string(value)?);
        index = end;
    }
    Ok(output)
}

pub(super) fn tokenize(sql: &str) -> StoreResult<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut brace_depth = 0usize;

    for value in sql.chars() {
        if in_string {
            current.push(value);
            if escaped {
                escaped = false;
            } else if value == '\\' {
                escaped = true;
            } else if value == '"' {
                in_string = false;
            }
            continue;
        }

        match value {
            '"' => {
                in_string = true;
                current.push(value);
            }
            '{' => {
                brace_depth += 1;
                current.push(value);
            }
            '}' => {
                brace_depth = brace_depth.saturating_sub(1);
                current.push(value);
            }
            ',' if brace_depth == 0 => {
                current.push(value);
            }
            '=' if brace_depth == 0 => {
                push_current(&mut tokens, &mut current);
                tokens.push("=".to_string());
            }
            value if value.is_whitespace() && brace_depth == 0 => {
                push_current(&mut tokens, &mut current);
            }
            _ => current.push(value),
        }
    }

    if in_string {
        return Err(StoreError::InvalidQuery(
            "query has an unterminated string".to_string(),
        ));
    }
    push_current(&mut tokens, &mut current);
    Ok(tokens)
}

fn push_current(tokens: &mut Vec<String>, current: &mut String) {
    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }
    current.clear();
}
