use crate::language::analysis::{document_workspace_root, exports_symbol, normalize_path};
use crate::language::completion::{dowe_files, importable_project_path, project_root_import_path};
use crate::language::documentation::{
    component_documentation, server_documentation, stdlib_documentation,
};
use crate::language::model::{
    LanguageCodeAction, LanguageDocument, LanguagePosition, LanguageRange, LanguageTextEdit,
};
use crate::parser::{SourceNode, SourceValue, parse_source_file, resolve_import};
use std::fs;
use std::path::Path;

pub fn code_actions_at(
    root: &Path,
    document: &LanguageDocument,
    line: usize,
    column: usize,
) -> Vec<LanguageCodeAction> {
    let root = document_workspace_root(root, &document.path);
    let Some(symbol) = token_at(&document.source, line, column) else {
        return Vec::new();
    };
    let Ok(file) = parse_source_file(&root, &document.path, document.source.clone()) else {
        return Vec::new();
    };
    if file.imports.iter().any(|import| import.local == symbol) {
        return Vec::new();
    }
    if known_document_symbol(&file.nodes, &symbol)
        || component_documentation(&symbol).is_some()
        || server_documentation(&symbol).is_some()
        || stdlib_documentation(&symbol).is_some()
    {
        return Vec::new();
    }

    let source_root = normalize_path(root.clone());
    let current = normalize_path(document.path.clone());
    let mut actions = dowe_files(&source_root)
        .into_iter()
        .map(normalize_path)
        .filter(|path| path != &current)
        .filter(|path| importable_project_path(&source_root, path))
        .filter_map(|path| {
            let source = fs::read_to_string(&path).ok()?;
            let target = parse_source_file(&root, &path, source).ok()?;
            exports_symbol(&target, &symbol).then_some(path)
        })
        .filter_map(|path| {
            let import_path = project_root_import_path(&source_root, &path)?;
            let edit = import_edit(&root, &document.source, &file, &symbol, &import_path, &path);
            Some(LanguageCodeAction {
                title: format!("Import {symbol} from \"{import_path}\""),
                edit,
            })
        })
        .collect::<Vec<_>>();
    actions.sort_by(|left, right| left.title.cmp(&right.title));
    actions
}

fn import_insertion_range(source: &str, file: &crate::parser::SourceFile) -> LanguageRange {
    let position = if let Some(import) = file.imports.last() {
        let line = source
            .lines()
            .nth(import.location.line.saturating_sub(1))
            .unwrap_or_default();
        LanguagePosition {
            line: import.location.line,
            column: line.chars().count() + 1,
        }
    } else {
        LanguagePosition { line: 1, column: 1 }
    };
    LanguageRange {
        start: position,
        end: position,
    }
}

fn import_edit(
    root: &Path,
    source: &str,
    file: &crate::parser::SourceFile,
    symbol: &str,
    path: &str,
    target: &Path,
) -> LanguageTextEdit {
    if let Some(existing) = file.imports.iter().find(|import| {
        resolve_import(root, &file.path, import)
            .map(normalize_path)
            .is_ok_and(|resolved| resolved == target)
    }) {
        let mut names = file
            .imports
            .iter()
            .filter(|import| {
                import.location.line == existing.location.line && import.path == existing.path
            })
            .map(|import| import.local.as_str())
            .collect::<Vec<_>>();
        names.push(symbol);
        let line = source
            .lines()
            .nth(existing.location.line.saturating_sub(1))
            .unwrap_or_default();
        let braced = line.trim_start().starts_with("import {");
        let names = names.join(", ");
        let new_text = if braced {
            format!("import {{ {names} }} from \"{}\"", existing.path)
        } else {
            format!("import {names} from \"{}\"", existing.path)
        };
        return LanguageTextEdit {
            range: LanguageRange {
                start: LanguagePosition {
                    line: existing.location.line,
                    column: 1,
                },
                end: LanguagePosition {
                    line: existing.location.line,
                    column: line.chars().count() + 1,
                },
            },
            new_text,
        };
    }
    LanguageTextEdit {
        range: import_insertion_range(source, file),
        new_text: import_text(source, file, symbol, path),
    }
}

fn import_text(source: &str, file: &crate::parser::SourceFile, symbol: &str, path: &str) -> String {
    let import = format!("import {symbol} from \"{path}\"");
    if file.imports.is_empty() {
        if source.is_empty() {
            format!("{import}\n")
        } else {
            format!("{import}\n\n")
        }
    } else {
        format!("\n{import}")
    }
}

fn token_at(source: &str, line: usize, column: usize) -> Option<String> {
    let line = source.lines().nth(line.saturating_sub(1))?;
    let chars = line.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        return None;
    }
    let mut index = column.saturating_sub(1).min(chars.len().saturating_sub(1));
    if !is_identifier_char(chars[index]) && index > 0 && is_identifier_char(chars[index - 1]) {
        index -= 1;
    }
    if !is_identifier_char(chars[index]) {
        return None;
    }
    let mut start = index;
    while start > 0 && is_identifier_char(chars[start - 1]) {
        start -= 1;
    }
    let mut end = index + 1;
    while end < chars.len() && is_identifier_char(chars[end]) {
        end += 1;
    }
    Some(chars[start..end].iter().collect())
}

fn is_identifier_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || value == '_'
}

fn known_document_symbol(nodes: &[SourceNode], symbol: &str) -> bool {
    nodes.iter().any(|node| {
        let declares_symbol = matches!(
            node.name.as_str(),
            "layout"
                | "page"
                | "component"
                | "fn"
                | "signal"
                | "const"
                | "handler"
                | "middleware"
                | "views"
                | "endpoints"
                | "type"
                | "database"
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
                | "let"
        ) && node
            .args
            .first()
            .and_then(SourceValue::as_required_string)
            .is_some_and(|value| value == symbol);
        declares_symbol || known_document_symbol(&node.children, symbol)
    })
}
