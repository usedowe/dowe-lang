use crate::language::analysis::document_workspace_root;
use crate::language::model::{
    LanguageDocument, LanguageDocumentSymbol, LanguageRange, LanguageSymbolKind,
};
use crate::parser::{SourceNode, SourceValue, parse_source_file};
use std::path::Path;

pub fn document_symbols(root: &Path, document: &LanguageDocument) -> Vec<LanguageDocumentSymbol> {
    let root = document_workspace_root(root, &document.path);
    parse_source_file(&root, &document.path, document.source.clone())
        .map(|file| file.nodes.iter().filter_map(symbol_for_node).collect())
        .unwrap_or_default()
}

fn symbol_for_node(node: &SourceNode) -> Option<LanguageDocumentSymbol> {
    let kind = symbol_kind(node)?;
    let name = symbol_name(node);
    let range = node_range(node);
    Some(LanguageDocumentSymbol {
        name,
        kind,
        range,
        selection_range: range,
        children: node.children.iter().filter_map(symbol_for_node).collect(),
    })
}

fn symbol_kind(node: &SourceNode) -> Option<LanguageSymbolKind> {
    match node.name.as_str() {
        "config" | "main" | "views" | "desktop" | "server" | "route" | "group" => {
            Some(LanguageSymbolKind::Module)
        }
        "layout" | "page" | "component" | "type" | "entity" => Some(LanguageSymbolKind::Class),
        "fn" | "handler" | "middleware" | "seeder" | "init" | "go" | "cron" | "test" => {
            Some(LanguageSymbolKind::Function)
        }
        "method" | "get" | "post" | "put" | "patch" | "delete" | "websocket" => {
            Some(LanguageSymbolKind::Method)
        }
        "const" | "signal" | "store" | "database" | "cache" | "vector" | "query" | "kv" | "emb"
        | "request" | "ws" | "agent" | "str" | "math" | "parse" | "url" | "csv" | "sort"
        | "list" | "json" | "date" | "id" | "let" => Some(LanguageSymbolKind::Variable),
        "fonts" | "env" | "variable" | "cors" | "design" | "theme" => {
            Some(LanguageSymbolKind::Property)
        }
        _ => None,
    }
}

fn symbol_name(node: &SourceNode) -> String {
    if let Some(arg) = node.args.first().and_then(SourceValue::as_string_like) {
        format!("{} {arg}", node.name)
    } else if let Some(path) = node
        .prop("path")
        .and_then(|prop| prop.value.as_string_like())
    {
        format!("{} {path}", node.name)
    } else if let Some(name) = node
        .prop("name")
        .and_then(|prop| prop.value.as_string_like())
    {
        format!("{} {name}", node.name)
    } else {
        node.name.clone()
    }
}

fn node_range(node: &SourceNode) -> LanguageRange {
    LanguageRange::single_line(node.location.line, node.location.column, node.name.len())
}
