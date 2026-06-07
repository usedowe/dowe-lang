use crate::language::analysis::{document_workspace_root, environment_config, reference_fields};
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
    if let Some(env_name) = token.strip_prefix("env.") {
        if environment_config(root)
            .ok()?
            .variables
            .iter()
            .any(|variable| variable.name == env_name)
        {
            return Some(format!("Dowe environment variable `{env_name}`"));
        }
    }
    if let Some((reference_root, field)) = token.split_once('.')
        && reference_fields(root, document, reference_root)
            .iter()
            .any(|known| known == field)
    {
        return Some(format!("Dowe inferred field `{token}`"));
    }
    if component_names().contains(&token.as_str()) {
        return Some(format!("Dowe component `{token}`"));
    }
    if prop_names().contains(&token.as_str()) {
        return Some(format!("Dowe component prop `{token}`"));
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
    let path = root.join("src/config.dowe");
    let source = fs::read_to_string(&path).ok()?;
    for (index, line) in source.lines().enumerate() {
        if line.contains(&format!("name:{name}")) || line.contains(&format!("name:\"{name}\"")) {
            let column = line.find("variable").unwrap_or(0) + 1;
            return Some(LanguageLocation {
                path,
                range: LanguageRange::single_line(index + 1, column, 8),
            });
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
        let matches = node
            .args
            .first()
            .and_then(SourceValue::as_required_string)
            .is_some_and(|value| value == name);
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
            "action" | "signal" | "handler" | "middleware" | "type"
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

fn component_names() -> Vec<&'static str> {
    vec![
        "Box",
        "Flex",
        "Grid",
        "Card",
        "Table",
        "AppBar",
        "Footer",
        "BottomBar",
        "SideNav",
        "Sidebar",
        "NavMenu",
        "Scaffold",
        "Tabs",
        "tab",
        "Drawer",
        "Input",
        "Select",
        "Option",
        "Code",
        "Video",
        "Candlestick",
        "Divider",
        "Button",
        "Alert",
        "Svg",
        "Path",
        "Title",
        "Text",
    ]
}

fn prop_names() -> Vec<&'static str> {
    vec![
        "size",
        "variant",
        "color",
        "scheme",
        "orientation",
        "show",
        "p",
        "gap",
        "columns",
        "bind",
        "onClick",
        "href",
        "message",
        "visible",
        "open",
        "onClose",
        "middleware",
        "base",
        "body",
        "update",
        "reset",
        "data",
        "stream",
        "upColor",
        "downColor",
        "emptyLabel",
        "maxPoints",
        "viewBox",
        "d",
        "fill",
        "w",
        "h",
        "label",
        "placeholder",
        "labelFloating",
        "value",
        "description",
        "type",
        "bordered",
        "blurred",
        "boxed",
        "floating",
        "position",
        "disableOverlayClose",
        "hideCloseButton",
        "platform",
        "animation",
        "striped",
        "dividers",
        "emptyTitle",
        "emptyDescription",
        "field",
        "align",
        "width",
    ]
}
