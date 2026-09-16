use super::{CleanEdge, CleanGraph, Evidence};
use crate::{CodeGraphError, CodeGraphResult};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn declarations(graph: &CleanGraph) -> Vec<super::CleanNode> {
    graph
        .nodes()
        .filter(|node| {
            matches!(
                node.kind.as_str(),
                "layout"
                    | "page"
                    | "component"
                    | "entity"
                    | "handler"
                    | "route"
                    | "middleware"
                    | "store"
                    | "schema"
                    | "type"
            )
        })
        .cloned()
        .collect()
}

fn files(graph: &CleanGraph) -> Vec<super::CleanNode> {
    graph
        .nodes()
        .filter(|node| {
            node.kind == "file"
                && node
                    .path
                    .as_deref()
                    .is_some_and(|path| path.ends_with(".dowe"))
        })
        .cloned()
        .collect()
}

pub(super) fn add_dowe_reference_edges(root: &Path, graph: &mut CleanGraph) -> CodeGraphResult<()> {
    let declarations = declarations(graph);
    for source in files(graph) {
        let Some(path) = source.path.as_deref() else {
            continue;
        };
        let file_path = root.join(path);
        let content = fs::read_to_string(&file_path)
            .map_err(|error| CodeGraphError::at_path(&file_path, error.to_string()))?;
        let content = strip_line_comments(&content);
        let mut tokens = content.split_whitespace();
        while let Some(token) = tokens.next() {
            let Some((key, value)) = token.split_once(':') else {
                continue;
            };
            let relation = match key {
                "layout" => "uses_layout",
                "page" => "uses_page",
                "handler" => "handled_by",
                "entity" => "uses_entity",
                "middleware" => "uses_middleware",
                "component" => "uses_component",
                "store" => "uses_store",
                "schema" => "uses_schema",
                "type" => "uses_type",
                _ => continue,
            };
            let mut value = value.to_owned();
            if value.starts_with('[') && !value.ends_with(']') {
                for part in &mut tokens {
                    value.push(' ');
                    value.push_str(part);
                    if part.ends_with(']') {
                        break;
                    }
                }
            }
            for name in value.split_whitespace().map(|name| {
                name.trim_matches(|character: char| {
                    character == '"' || character == '[' || character == ']'
                })
            }) {
                let Some(target) = declarations.iter().find(|target| target.name == name) else {
                    continue;
                };
                let _ = graph.insert_edge(CleanEdge {
                    source: source.id.clone(),
                    relation: relation.into(),
                    target: target.id.clone(),
                    evidence: Evidence::Inferred,
                });
            }
        }
    }
    Ok(())
}

pub(super) fn add_dowe_symbol_usage_edges(
    root: &Path,
    graph: &mut CleanGraph,
) -> CodeGraphResult<()> {
    let declarations = declarations(graph);
    for source in files(graph) {
        let Some(path) = source.path.as_deref() else {
            continue;
        };
        let file_path = root.join(path);
        let content = fs::read_to_string(&file_path)
            .map_err(|error| CodeGraphError::at_path(&file_path, error.to_string()))?;
        let semantic_content = mask_strings(&strip_line_comments(&content));
        let names = semantic_content
            .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .filter(|name| !name.is_empty())
            .collect::<BTreeSet<_>>();
        for target in declarations.iter().filter(|target| {
            target.path.as_deref() != Some(path) && names.contains(target.name.as_str())
        }) {
            let _ = graph.insert_edge(CleanEdge {
                source: source.id.clone(),
                relation: "uses".into(),
                target: target.id.clone(),
                evidence: Evidence::Inferred,
            });
        }
    }
    Ok(())
}

fn strip_line_comments(content: &str) -> String {
    content
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .collect::<Vec<_>>()
        .join("\n")
}

fn mask_strings(content: &str) -> String {
    let mut masked = String::with_capacity(content.len());
    let mut quoted = false;
    let mut escaped = false;
    for character in content.chars() {
        if quoted {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
            masked.push(if character == '\n' { '\n' } else { ' ' });
        } else if character == '"' {
            quoted = true;
            masked.push(' ');
        } else {
            masked.push(character);
        }
    }
    masked
}
