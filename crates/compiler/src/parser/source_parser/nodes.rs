use crate::error::{DoweError, DoweResult};
use crate::parser::source_ast::{SourceLocation, SourceNode, SourceProp};
use crate::parser::source_values::{parse_value, split_top_level_whitespace};
use super::lines::LogicalLine;
use std::collections::HashSet;
use std::path::Path;

pub(super) fn multiline_header(source: &str) -> (&str, bool) {
    let Some(header) = source.strip_suffix(':') else { return (source, false); };
    (header.trim_end(), true)
}

pub(super) fn parse_continuation_prop(path: &Path, relative_path: &Path, line: &LogicalLine) -> DoweResult<Option<SourceProp>> {
    let tokens = split_top_level_whitespace(&line.source, line.indent_spaces)?;
    let Some(first) = tokens.first() else { return Ok(None); };
    if matches!(first.text.chars().next(), Some('"' | '{' | '[')) { return Ok(None); }
    let Some((name, value)) = first.text.split_once(':') else { return Ok(None); };
    if tokens.len() != 1 {
        return Err(DoweError::at_path(path, format!("{}:{}: property suites require one prop per line", line.line, first.column)));
    }
    if value.is_empty() { return Ok(None); }
    if !is_prop_name(name) {
        return Err(DoweError::at_path(path, format!("{}:{}: invalid property suite prop", line.line, first.column)));
    }
    Ok(Some(SourceProp {
        name: name.to_string(),
        value: parse_value(path, line.line, first.column + name.len() + 1, value)?,
        location: SourceLocation { path: path.to_path_buf(), relative_path: relative_path.to_path_buf(), line: line.line, column: first.column, indent: line.indent_spaces / 2 },
    }))
}

fn is_prop_name(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic() || first == '_')
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
}

pub(super) fn parse_node(path: &Path, relative_path: &Path, line: usize, column: usize, source: &str) -> DoweResult<SourceNode> {
    let tokens = split_top_level_whitespace(source, column.saturating_sub(1))?;
    let Some(first) = tokens.first() else { return Err(DoweError::at_path(path, format!("{line}:{column}: missing node"))); };
    let mut args = Vec::new();
    let mut props = Vec::new();
    let mut seen_props = HashSet::new();
    let mut node_name = first.text.clone();

    if (first.text.starts_with("views:") || first.text.starts_with("endpoints:") || first.text.starts_with("databases:"))
        && !first.text.starts_with('"') && !first.text.starts_with('{') && !first.text.starts_with('[')
        && let Some((name, value)) = first.text.split_once(':')
    {
        if name.is_empty() || value.is_empty() {
            return Err(DoweError::at_path(path, format!("{line}:{}: prop `{name}` must have a value", first.column)));
        }
        node_name = name.to_string();
        seen_props.insert(name.to_string());
        props.push(SourceProp {
            name: name.to_string(), value: parse_value(path, line, first.column + name.len() + 1, value)?,
            location: SourceLocation { path: path.to_path_buf(), relative_path: relative_path.to_path_buf(), line, column: first.column, indent: column.saturating_sub(1) / 2 },
        });
    }

    for (token_index, token) in tokens.iter().enumerate().skip(1) {
        let typed_binding = matches!(node_name.as_str(), "let" | "const") && token_index == 1;
        if !token.text.starts_with('"') && !token.text.starts_with('{') && !token.text.starts_with('[') && !typed_binding
            && let Some((name, value)) = token.text.split_once(':')
        {
            if name.is_empty() || value.is_empty() {
                return Err(DoweError::at_path(path, format!("{line}:{}: prop `{name}` must have a value", token.column)));
            }
            if !seen_props.insert(name.to_string()) {
                return Err(DoweError::at_path(path, format!("{line}:{}: duplicate prop `{name}`", token.column)));
            }
            props.push(SourceProp {
                name: name.to_string(), value: parse_value(path, line, token.column + name.len() + 1, value)?,
                location: SourceLocation { path: path.to_path_buf(), relative_path: relative_path.to_path_buf(), line, column: token.column, indent: column.saturating_sub(1) / 2 },
            });
        } else {
            args.push(parse_value(path, line, token.column, &token.text)?);
        }
    }
    Ok(SourceNode {
        location: SourceLocation { path: path.to_path_buf(), relative_path: relative_path.to_path_buf(), line, column, indent: column.saturating_sub(1) / 2 },
        name: node_name, args, props, children: Vec::new(),
    })
}
