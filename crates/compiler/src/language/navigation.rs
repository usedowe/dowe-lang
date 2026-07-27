use crate::language::analysis::{document_workspace_root, environment_config, reference_fields};
use crate::language::documentation::{
    component_documentation, component_prop_documentation, server_documentation,
    server_prop_documentation, stdlib_documentation, theme_documentation,
};
use crate::language::model::{LanguageDocument, LanguageLocation, LanguageRange};
use crate::parser::{SourceFile, SourceNode, SourceValue, parse_source_file, resolve_import};
use std::fs;
use std::path::Path;

pub fn definition_at(
    root: &Path,
    document: &LanguageDocument,
    line: usize,
    column: usize,
) -> Option<LanguageLocation> {
    let root = document_workspace_root(root, &document.path);
    let root = root.as_path();
    let token = token_at(&document.source, line, column)?;
    if let Some(location) = import_definition(root, document, &token) {
        return Some(location);
    }
    if let Some(env_name) = token.strip_prefix("env.") {
        return env_definition(root, env_name);
    }
    symbol_definition(root, document, &token)
}

pub fn hover_at(
    root: &Path,
    document: &LanguageDocument,
    line: usize,
    column: usize,
) -> Option<String> {
    let root = document_workspace_root(root, &document.path);
    let root = root.as_path();
    let token = token_at(&document.source, line, column)?;
    let source_line = document
        .source
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or_default();
    let owner = source_line.split_whitespace().next().unwrap_or_default();
    if document.path.ends_with("theme.dowe")
        && let Some(value) = theme_documentation(
            owner,
            &token,
            source_line.trim_start().len() == source_line.len(),
        )
    {
        return Some(value);
    }
    if matches!(
        (owner, token.as_str()),
        ("group", "path" | "layout" | "platform") | ("route", "path" | "page" | "platform")
    ) {
        return Some(format!(
            "Dowe view route prop `{owner}.{token}`, validated by the shared compiler"
        ));
    }
    if let Some(component) = source_line.split_whitespace().next()
        && let Some(value) = if component == token {
            component_documentation(component)
        } else {
            component_prop_documentation(component, &token)
        }
    {
        return Some(value);
    }
    if let Some(value) = component_documentation(&token) {
        return Some(value);
    }
    if let Some(value) = server_documentation(&token) {
        return Some(value);
    }
    if let Some(value) = stdlib_documentation(&token) {
        return Some(value);
    }
    if let Some(value) = server_prop_documentation(source_line, &token) {
        return Some(value);
    }
    if let Some(env_name) = token.strip_prefix("env.")
        && environment_config(root)
            .ok()?
            .variables
            .iter()
            .any(|variable| variable.name == env_name)
    {
        return Some(format!("Dowe environment variable `{env_name}`"));
    }
    if let Some((reference_root, field)) = token.split_once('.')
        && reference_fields(root, document, reference_root)
            .iter()
            .any(|known| known == field)
    {
        return Some(format!("Dowe inferred field `{token}`"));
    }
    if token == "go" {
        return Some(
            "Dowe server go job: starts an imported server fn in an isolated process".to_string(),
        );
    }
    if token == "cron" {
        return Some(
            "Dowe server cron job: schedules isolated UTC executions from server init".to_string(),
        );
    }
    if token == "store" {
        return Some("Dowe View Store: imported application state".to_string());
    }
    if token == "database" {
        return Some(
            "Dowe Database declaration: server-only data access with local development persistence"
                .to_string(),
        );
    }
    if token == "vector" {
        return Some(
            "Dowe Vector declaration: server-only embedding storage with local or authenticated WebSocket execution"
                .to_string(),
        );
    }
    if token == "emb" {
        return Some(
            "Dowe embedding operation: upsert, search, read, delete, or list through a Vector connection"
                .to_string(),
        );
    }
    if token == "db" {
        return Some("Dowe Database operation namespace used by `query`".to_string());
    }
    if token == "const" {
        return Some(
            "Dowe view constant: immutable serializable data outside reactive Signal state"
                .to_string(),
        );
    }
    if token.starts_with('/') {
        return Some("Dowe route path".to_string());
    }
    None
}

fn import_definition(
    root: &Path,
    document: &LanguageDocument,
    token: &str,
) -> Option<LanguageLocation> {
    let file = parse_source_file(root, &document.path, document.source.clone()).ok()?;
    for import in &file.imports {
        if import.local == token || import.path == token {
            let path = resolve_import(root, &file.path, import).ok()?;
            let target = read_source_file(root, &path);
            let range = target
                .as_ref()
                .and_then(|file| exported_range(file, &import.local))
                .unwrap_or_else(|| LanguageRange::single_line(1, 1, 1));
            return Some(LanguageLocation { path, range });
        }
    }
    None
}

fn symbol_definition(
    root: &Path,
    document: &LanguageDocument,
    token: &str,
) -> Option<LanguageLocation> {
    let file = parse_source_file(root, &document.path, document.source.clone()).ok()?;
    find_symbol(&file.nodes, token).map(|range| LanguageLocation {
        path: document.path.clone(),
        range,
    })
}

fn env_definition(root: &Path, name: &str) -> Option<LanguageLocation> {
    for file_name in [".env", ".env.example"] {
        let path = root.join(file_name);
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        for (index, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            let Some((candidate, _)) = trimmed.split_once('=') else {
                continue;
            };
            if candidate.trim() == name {
                let column = line.len() - trimmed.len() + 1;
                return Some(LanguageLocation {
                    path,
                    range: LanguageRange::single_line(index + 1, column, name.len()),
                });
            }
        }
    }
    None
}

fn read_source_file(root: &Path, path: &Path) -> Option<SourceFile> {
    let source = fs::read_to_string(path).ok()?;
    parse_source_file(root, path, source).ok()
}

fn exported_range(file: &SourceFile, name: &str) -> Option<LanguageRange> {
    file.nodes.iter().find_map(|node| {
        let matches = if node.name == "let" {
            node.args
                .first()
                .and_then(SourceValue::as_required_string)
                .is_some_and(|value| value == name)
        } else {
            node.args
                .first()
                .and_then(SourceValue::as_required_string)
                .is_some_and(|value| value == name)
        };
        if matches {
            Some(LanguageRange::single_line(
                node.location.line,
                node.location.column,
                node.name.len(),
            ))
        } else {
            None
        }
    })
}

fn find_symbol(nodes: &[SourceNode], token: &str) -> Option<LanguageRange> {
    for node in nodes {
        if matches!(
            node.name.as_str(),
            "fn" | "signal"
                | "const"
                | "database"
                | "cache"
                | "vector"
                | "query"
                | "kv"
                | "emb"
                | "request"
                | "ws"
                | "agent"
                | "str"
                | "math"
                | "parse"
                | "url"
                | "csv"
                | "sort"
                | "list"
                | "json"
                | "date"
                | "id"
                | "entity"
                | "seeder"
                | "store"
                | "handler"
                | "middleware"
                | "type"
        ) && node
            .args
            .first()
            .and_then(SourceValue::as_required_string)
            .is_some_and(|value| value == token)
        {
            return Some(LanguageRange::single_line(
                node.location.line,
                node.location.column,
                node.name.len(),
            ));
        }
        if node.name == "let"
            && node
                .args
                .first()
                .and_then(SourceValue::as_required_string)
                .is_some_and(|value| value == token)
        {
            return Some(LanguageRange::single_line(
                node.location.line,
                node.location.column,
                node.name.len(),
            ));
        }
        if let Some(range) = find_symbol(&node.children, token) {
            return Some(range);
        }
    }
    None
}

fn token_at(source: &str, line: usize, column: usize) -> Option<String> {
    let value = source.lines().nth(line.saturating_sub(1))?;
    let chars = value.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        return None;
    }
    let mut index = column.saturating_sub(1).min(chars.len().saturating_sub(1));
    if !is_token_char(chars[index]) && index > 0 {
        index -= 1;
    }
    if !is_token_char(chars[index]) {
        return None;
    }
    let mut start = index;
    while start > 0 && is_token_char(chars[start - 1]) {
        start -= 1;
    }
    let mut end = index + 1;
    while end < chars.len() && is_token_char(chars[end]) {
        end += 1;
    }
    let token = chars[start..end].iter().collect::<String>();
    Some(token.trim_matches('"').to_string())
}

fn is_token_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '_' | '-' | '.' | '/' | '"')
}
