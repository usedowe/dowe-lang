use crate::error::{DoweError, DoweResult};
use crate::parser::source_ast::{
    SourceFile, SourceImport, SourceLocation, SourceNode, SourceProp, SourceValue,
};
use crate::parser::source_values::{parse_value, split_top_level_whitespace};
use std::collections::HashSet;
use std::path::Path;

#[derive(Clone)]
struct FlatNode {
    level: usize,
    node: SourceNode,
}

pub fn parse_source_file(root: &Path, path: &Path, source: String) -> DoweResult<SourceFile> {
    let relative_path = path.strip_prefix(root).unwrap_or(path).to_path_buf();
    let mut imports = Vec::new();
    let mut flat_nodes = Vec::new();

    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        if line.trim().is_empty() {
            continue;
        }
        let indent_spaces = leading_indent(path, line_number, line)?;
        let trimmed = &line[indent_spaces..];
        let column = indent_spaces + 1;
        let location = SourceLocation {
            path: path.to_path_buf(),
            relative_path: relative_path.clone(),
            line: line_number,
            column,
            indent: indent_spaces / 2,
        };

        if trimmed.starts_with("import ") {
            if indent_spaces != 0 {
                return Err(DoweError::at_path(
                    path,
                    format!("{line_number}:{column}: imports must be top-level"),
                ));
            }
            imports.push(parse_import(path, location, trimmed)?);
        } else {
            flat_nodes.push(FlatNode {
                level: indent_spaces / 2,
                node: parse_node(path, &relative_path, line_number, column, trimmed)?,
            });
        }
    }

    let mut index = 0usize;
    let nodes = parse_block(&flat_nodes, &mut index, 0)?;
    if index < flat_nodes.len() {
        let node = &flat_nodes[index].node;
        return Err(DoweError::at_path(
            &node.location.path,
            format!(
                "{}:{}: block is not nested under a parent",
                node.location.line, node.location.column
            ),
        ));
    }

    Ok(SourceFile {
        path: path.to_path_buf(),
        relative_path,
        imports,
        nodes,
        source,
    })
}

fn leading_indent(path: &Path, line: usize, source: &str) -> DoweResult<usize> {
    let mut count = 0usize;
    for value in source.chars() {
        match value {
            ' ' => count += 1,
            '\t' => {
                return Err(DoweError::at_path(
                    path,
                    format!("{line}:1: tabs are not valid indentation in Dowe Source Format"),
                ));
            }
            _ => break,
        }
    }

    if count % 2 != 0 {
        return Err(DoweError::at_path(
            path,
            format!("{line}:1: indentation must use two spaces per level"),
        ));
    }

    Ok(count)
}

fn parse_import(path: &Path, location: SourceLocation, source: &str) -> DoweResult<SourceImport> {
    let tokens = split_top_level_whitespace(source, location.column.saturating_sub(1))?;
    if tokens.len() != 4 || tokens[0].text != "import" || tokens[2].text != "from" {
        return Err(DoweError::at_path(
            path,
            format!(
                "{}:{}: invalid import syntax",
                location.line, location.column
            ),
        ));
    }
    let SourceValue::String(import_path) =
        parse_value(path, location.line, tokens[3].column, &tokens[3].text)?
    else {
        return Err(DoweError::at_path(
            path,
            format!(
                "{}:{}: import path must be a string",
                location.line, tokens[3].column
            ),
        ));
    };
    Ok(SourceImport {
        local: tokens[1].text.clone(),
        path: import_path,
        location,
    })
}

fn parse_node(
    path: &Path,
    relative_path: &Path,
    line: usize,
    column: usize,
    source: &str,
) -> DoweResult<SourceNode> {
    let tokens = split_top_level_whitespace(source, column.saturating_sub(1))?;
    let Some(first) = tokens.first() else {
        return Err(DoweError::at_path(
            path,
            format!("{line}:{column}: missing node"),
        ));
    };
    let mut args = Vec::new();
    let mut props = Vec::new();
    let mut seen_props = HashSet::new();

    for (token_index, token) in tokens.iter().enumerate().skip(1) {
        let typed_let_binding = first.text == "let" && token_index == 1;
        if !token.text.starts_with('"')
            && !token.text.starts_with('{')
            && !token.text.starts_with('[')
            && !typed_let_binding
            && let Some((name, value)) = token.text.split_once(':')
        {
            if name.is_empty() || value.is_empty() {
                return Err(DoweError::at_path(
                    path,
                    format!("{line}:{}: prop `{name}` must have a value", token.column),
                ));
            }
            if !seen_props.insert(name.to_string()) {
                return Err(DoweError::at_path(
                    path,
                    format!("{line}:{}: duplicate prop `{name}`", token.column),
                ));
            }
            props.push(SourceProp {
                name: name.to_string(),
                value: parse_value(path, line, token.column + name.len() + 1, value)?,
                location: SourceLocation {
                    path: path.to_path_buf(),
                    relative_path: relative_path.to_path_buf(),
                    line,
                    column: token.column,
                    indent: column.saturating_sub(1) / 2,
                },
            });
        } else {
            args.push(parse_value(path, line, token.column, &token.text)?);
        }
    }

    Ok(SourceNode {
        location: SourceLocation {
            path: path.to_path_buf(),
            relative_path: relative_path.to_path_buf(),
            line,
            column,
            indent: column.saturating_sub(1) / 2,
        },
        name: first.text.clone(),
        args,
        props,
        children: Vec::new(),
    })
}

fn parse_block(
    flat_nodes: &[FlatNode],
    index: &mut usize,
    level: usize,
) -> DoweResult<Vec<SourceNode>> {
    let mut nodes = Vec::new();

    while *index < flat_nodes.len() {
        let current = &flat_nodes[*index];
        if current.level < level {
            break;
        }
        if current.level > level {
            return Err(DoweError::at_path(
                &current.node.location.path,
                format!(
                    "{}:{}: block is not nested under a parent",
                    current.node.location.line, current.node.location.column
                ),
            ));
        }

        let mut node = current.node.clone();
        *index += 1;
        if *index < flat_nodes.len() {
            let next = &flat_nodes[*index];
            if next.level > level + 1 {
                return Err(DoweError::at_path(
                    &next.node.location.path,
                    format!(
                        "{}:{}: indentation can only increase one level at a time",
                        next.node.location.line, next.node.location.column
                    ),
                ));
            }
            if next.level == level + 1 {
                node.children = parse_block(flat_nodes, index, level + 1)?;
            }
        }
        nodes.push(node);
    }

    Ok(nodes)
}
