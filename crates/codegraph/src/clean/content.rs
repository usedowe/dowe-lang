use super::{CleanEdge, CleanGraph, CleanNode, Evidence, Namespace};
use crate::{CodeGraphError, CodeGraphResult};
use std::path::Path;

pub(super) fn add_i18n_keys(
    graph: &mut CleanGraph,
    file_id: &str,
    path: &str,
    bytes: &[u8],
) -> CodeGraphResult<()> {
    let locale = Path::new(path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("default")
        .to_string();
    let locale_id = format!("dowe:i18n-locale:{locale}");
    graph.insert_node(CleanNode {
        id: locale_id.clone(),
        namespace: Namespace::I18n,
        kind: "locale".into(),
        name: locale,
        path: Some(path.into()),
        start_line: None,
        end_line: None,
        evidence: Evidence::Compiler,
    });
    graph
        .insert_edge(CleanEdge {
            source: file_id.into(),
            relation: "declares_locale".into(),
            target: locale_id.clone(),
            evidence: Evidence::Compiler,
        })
        .map_err(CodeGraphError::new)?;
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|error| {
        CodeGraphError::at_path(Path::new(path), format!("invalid i18n JSON: {error}"))
    })?;
    let mut keys = Vec::new();
    collect_i18n_keys(&value, String::new(), 0, &mut keys);
    for key in keys.into_iter().take(512) {
        let id = format!("dowe:i18n:{path}:{key}");
        graph.insert_node(CleanNode {
            id: id.clone(),
            namespace: Namespace::I18n,
            kind: "translation_key".into(),
            name: key,
            path: Some(path.into()),
            start_line: None,
            end_line: None,
            evidence: Evidence::Compiler,
        });
        graph
            .insert_edge(CleanEdge {
                source: file_id.into(),
                relation: "defines_translation".into(),
                target: id.clone(),
                evidence: Evidence::Compiler,
            })
            .map_err(CodeGraphError::new)?;
        graph
            .insert_edge(CleanEdge {
                source: locale_id.clone(),
                relation: "translates".into(),
                target: id.clone(),
                evidence: Evidence::Compiler,
            })
            .map_err(CodeGraphError::new)?;
    }
    Ok(())
}

fn collect_i18n_keys(
    value: &serde_json::Value,
    prefix: String,
    depth: usize,
    keys: &mut Vec<String>,
) {
    if depth > 8 {
        return;
    }
    let Some(object) = value.as_object() else {
        if !prefix.is_empty() {
            keys.push(prefix);
        }
        return;
    };
    for (key, value) in object {
        let full = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        collect_i18n_keys(value, full, depth + 1, keys);
    }
}

pub(super) fn add_product_sections(
    graph: &mut CleanGraph,
    file_id: &str,
    path: &str,
    bytes: &[u8],
) -> CodeGraphResult<()> {
    let mut parents: Vec<(usize, String)> = Vec::new();
    for (line_index, line) in String::from_utf8_lossy(bytes).lines().enumerate() {
        let level = line
            .chars()
            .take_while(|character| *character == '#')
            .count();
        let heading = line[level..].trim();
        if level == 0 || heading.is_empty() {
            continue;
        }
        let lower = heading.to_lowercase();
        let kind = if lower.contains("use case") || lower.contains("caso de uso") {
            "use_case"
        } else if lower.contains("capability") || lower.contains("capacidad") {
            "capability"
        } else if lower.contains("requirement") || lower.contains("requisito") {
            "requirement"
        } else {
            "documentation_section"
        };
        let id = format!("dowe:product:{path}:{}", line_index + 1);
        while parents
            .last()
            .is_some_and(|(parent_level, _)| *parent_level >= level)
        {
            parents.pop();
        }
        let parent = parents.last().cloned();
        graph.insert_node(CleanNode {
            id: id.clone(),
            namespace: Namespace::Product,
            kind: kind.into(),
            name: heading.into(),
            path: Some(path.into()),
            start_line: Some(line_index as u32 + 1),
            end_line: Some(line_index as u32 + 1),
            evidence: Evidence::Compiler,
        });
        graph
            .insert_edge(CleanEdge {
                source: file_id.into(),
                relation: "documents".into(),
                target: id.clone(),
                evidence: Evidence::Compiler,
            })
            .map_err(CodeGraphError::new)?;
        if let Some((_, parent_id)) = parent {
            graph
                .insert_edge(CleanEdge {
                    source: parent_id.clone(),
                    relation: "contains".into(),
                    target: id.clone(),
                    evidence: Evidence::Compiler,
                })
                .map_err(CodeGraphError::new)?;
            if kind == "use_case"
                && graph
                    .node(&parent_id)
                    .is_some_and(|node| node.kind == "capability")
            {
                graph
                    .insert_edge(CleanEdge {
                        source: parent_id,
                        relation: "supports_use_case".into(),
                        target: id.clone(),
                        evidence: Evidence::Compiler,
                    })
                    .map_err(CodeGraphError::new)?;
            }
        }
        parents.push((level, id));
    }
    Ok(())
}
