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

#[derive(Clone)]
struct LogicalLine {
    line: usize,
    indent_spaces: usize,
    source: String,
}

#[derive(Default)]
struct DelimiterState {
    brace_depth: usize,
    bracket_depth: usize,
}

pub fn parse_source_file(root: &Path, path: &Path, source: String) -> DoweResult<SourceFile> {
    let relative_path = path.strip_prefix(root).unwrap_or(path).to_path_buf();
    let mut imports = Vec::new();
    let mut flat_nodes = Vec::new();
    let logical_lines = logical_lines(path, &source)?;
    let mut line_index = 0usize;

    while line_index < logical_lines.len() {
        let logical = &logical_lines[line_index];
        let line_number = logical.line;
        let indent_spaces = logical.indent_spaces;
        let trimmed = logical.source.as_str();
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
            imports.extend(parse_imports(path, location, trimmed)?);
            line_index += 1;
        } else {
            let (node_source, opens_prop_suite) = multiline_header(trimmed);
            let mut node = parse_node(path, &relative_path, line_number, column, node_source)?;
            if opens_prop_suite {
                if node.name == "type" {
                    return Err(DoweError::at_path(
                        path,
                        format!(
                            "{line_number}:{column}: `type` declarations do not accept property suites"
                        ),
                    ));
                }
                if !node.props.is_empty() {
                    return Err(DoweError::at_path(
                        path,
                        format!(
                            "{line_number}:{column}: property suite headers cannot contain inline props"
                        ),
                    ));
                }
                let child_level = indent_spaces / 2 + 1;
                let mut next = line_index + 1;
                let mut seen_props = HashSet::new();
                while next < logical_lines.len()
                    && logical_lines[next].indent_spaces / 2 == child_level
                {
                    let Some(prop) =
                        parse_continuation_prop(path, &relative_path, &logical_lines[next])?
                    else {
                        break;
                    };
                    if !seen_props.insert(prop.name.clone()) {
                        return Err(DoweError::at_path(
                            path,
                            format!(
                                "{}:{}: duplicate prop `{}`",
                                prop.location.line, prop.location.column, prop.name
                            ),
                        ));
                    }
                    node.props.push(prop);
                    next += 1;
                }
                let mut remaining = next;
                while remaining < logical_lines.len()
                    && logical_lines[remaining].indent_spaces > indent_spaces
                {
                    if logical_lines[remaining].indent_spaces / 2 == child_level
                        && parse_continuation_prop(path, &relative_path, &logical_lines[remaining])?
                            .is_some()
                    {
                        return Err(DoweError::at_path(
                            path,
                            format!(
                                "{}:{}: property suite props must appear before child nodes",
                                logical_lines[remaining].line,
                                logical_lines[remaining].indent_spaces + 1
                            ),
                        ));
                    }
                    remaining += 1;
                }
                line_index = next;
            } else {
                line_index += 1;
            }
            flat_nodes.push(FlatNode {
                level: indent_spaces / 2,
                node,
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

fn logical_lines(path: &Path, source: &str) -> DoweResult<Vec<LogicalLine>> {
    let physical_lines = source.lines().collect::<Vec<_>>();
    let mut logical_lines = Vec::new();
    let mut index = 0usize;

    while index < physical_lines.len() {
        let physical = physical_lines[index];
        if physical.trim().is_empty() {
            index += 1;
            continue;
        }
        let line = index + 1;
        let indent_spaces = leading_indent(path, line, physical)?;
        let mut value = physical[indent_spaces..].trim_end().to_string();
        if value.contains("\"\"\"") {
            let opening_count = value.matches("\"\"\"").count();
            if opening_count != 1 || !value.ends_with("\"\"\"") {
                return Err(DoweError::at_path(
                    path,
                    format!(
                        "{line}:{}: multiline strings must open after a prop value",
                        indent_spaces + 1
                    ),
                ));
            }
            loop {
                index += 1;
                let Some(next) = physical_lines.get(index).copied() else {
                    return Err(DoweError::at_path(
                        path,
                        format!(
                            "{line}:{}: missing multiline string closing delimiter",
                            indent_spaces + 1
                        ),
                    ));
                };
                let next_indent = leading_indent(path, index + 1, next)?;
                if next.trim() == "\"\"\"" {
                    if next_indent != indent_spaces {
                        return Err(DoweError::at_path(
                            path,
                            format!(
                                "{}:{}: multiline string closing delimiter must align with its prop",
                                index + 1,
                                next_indent + 1
                            ),
                        ));
                    }
                    value.push('\n');
                    value.push_str("\"\"\"");
                    break;
                }
                value.push('\n');
                value.push_str(next);
            }
            logical_lines.push(LogicalLine {
                line,
                indent_spaces,
                source: value,
            });
            index += 1;
            continue;
        }
        let mut delimiters = DelimiterState::default();
        delimiters.scan(path, line, &value)?;

        while delimiters.is_open() {
            index += 1;
            let Some(physical) = physical_lines.get(index).copied() else {
                return Err(DoweError::at_path(
                    path,
                    format!("{line}:{}: unclosed structured value", indent_spaces + 1),
                ));
            };
            if !physical.trim().is_empty() {
                leading_indent(path, index + 1, physical)?;
            }
            value.push('\n');
            value.push_str(physical.trim());
            delimiters.scan(path, index + 1, physical.trim())?;
        }

        logical_lines.push(LogicalLine {
            line,
            indent_spaces,
            source: value,
        });
        index += 1;
    }

    Ok(logical_lines)
}

impl DelimiterState {
    fn is_open(&self) -> bool {
        self.brace_depth > 0 || self.bracket_depth > 0
    }

    fn scan(&mut self, path: &Path, line: usize, source: &str) -> DoweResult<()> {
        let mut string_delimiter = None;
        let mut escaped = false;

        for (column, value) in source.char_indices() {
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
                '"' => string_delimiter = Some(value),
                '{' => self.brace_depth += 1,
                '}' if self.brace_depth == 0 => {
                    return Err(DoweError::at_path(
                        path,
                        format!("{line}:{}: unexpected `}}`", column + 1),
                    ));
                }
                '}' => self.brace_depth -= 1,
                '[' => self.bracket_depth += 1,
                ']' if self.bracket_depth == 0 => {
                    return Err(DoweError::at_path(
                        path,
                        format!("{line}:{}: unexpected `]`", column + 1),
                    ));
                }
                ']' => self.bracket_depth -= 1,
                _ => {}
            }
        }

        if string_delimiter.is_some() {
            return Err(DoweError::at_path(
                path,
                format!("{line}:1: strings cannot continue across lines"),
            ));
        }

        Ok(())
    }
}

fn multiline_header(source: &str) -> (&str, bool) {
    let Some(header) = source.strip_suffix(':') else {
        return (source, false);
    };
    (header.trim_end(), true)
}

fn parse_continuation_prop(
    path: &Path,
    relative_path: &Path,
    line: &LogicalLine,
) -> DoweResult<Option<SourceProp>> {
    let tokens = split_top_level_whitespace(&line.source, line.indent_spaces)?;
    let Some(first) = tokens.first() else {
        return Ok(None);
    };
    if matches!(first.text.chars().next(), Some('"' | '{' | '[')) {
        return Ok(None);
    }
    let Some((name, value)) = first.text.split_once(':') else {
        return Ok(None);
    };
    if tokens.len() != 1 {
        return Err(DoweError::at_path(
            path,
            format!(
                "{}:{}: property suites require one prop per line",
                line.line, first.column
            ),
        ));
    }
    if value.is_empty() {
        return Ok(None);
    }
    if !is_prop_name(name) {
        return Err(DoweError::at_path(
            path,
            format!(
                "{}:{}: invalid property suite prop",
                line.line, first.column
            ),
        ));
    }
    Ok(Some(SourceProp {
        name: name.to_string(),
        value: parse_value(path, line.line, first.column + name.len() + 1, value)?,
        location: SourceLocation {
            path: path.to_path_buf(),
            relative_path: relative_path.to_path_buf(),
            line: line.line,
            column: first.column,
            indent: line.indent_spaces / 2,
        },
    }))
}

fn is_prop_name(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic() || first == '_')
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
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

fn parse_imports(
    path: &Path,
    location: SourceLocation,
    source: &str,
) -> DoweResult<Vec<SourceImport>> {
    let tokens = split_top_level_whitespace(source, location.column.saturating_sub(1))?;
    let Some(from_index) = tokens.iter().position(|token| token.text == "from") else {
        return Err(DoweError::at_path(
            path,
            format!(
                "{}:{}: invalid import syntax",
                location.line, location.column
            ),
        ));
    };
    if tokens.first().is_none_or(|token| token.text != "import")
        || from_index < 2
        || tokens.len() != from_index + 2
    {
        return Err(DoweError::at_path(
            path,
            format!(
                "{}:{}: invalid import syntax",
                location.line, location.column
            ),
        ));
    }
    let names = tokens[1..from_index]
        .iter()
        .map(|token| token.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let names = parse_import_names(path, &location, &names)?;
    let SourceValue::String(import_path) = parse_value(
        path,
        location.line,
        tokens[from_index + 1].column,
        &tokens[from_index + 1].text,
    )?
    else {
        return Err(DoweError::at_path(
            path,
            format!(
                "{}:{}: import path must be a string",
                location.line,
                tokens[from_index + 1].column
            ),
        ));
    };
    Ok(names
        .into_iter()
        .map(|local| SourceImport {
            local,
            path: import_path.clone(),
            location: location.clone(),
        })
        .collect())
}

fn parse_import_names(
    path: &Path,
    location: &SourceLocation,
    source: &str,
) -> DoweResult<Vec<String>> {
    let source = source.trim();
    let source = if source.starts_with('{') {
        source
            .strip_prefix('{')
            .and_then(|value| value.strip_suffix('}'))
    } else if source.contains('{') || source.contains('}') {
        None
    } else {
        Some(source)
    }
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .ok_or_else(|| {
        DoweError::at_path(
            path,
            format!(
                "{}:{}: invalid import syntax",
                location.line, location.column
            ),
        )
    })?;
    let mut names = Vec::new();
    for name in source.split(',').map(str::trim) {
        if !is_import_name(name) || names.iter().any(|existing| existing == name) {
            return Err(DoweError::at_path(
                path,
                format!(
                    "{}:{}: invalid import syntax",
                    location.line, location.column
                ),
            ));
        }
        names.push(name.to_string());
    }
    Ok(names)
}

fn is_import_name(name: &str) -> bool {
    let mut characters = name.chars();
    matches!(characters.next(), Some(value) if value.is_ascii_alphabetic() || value == '_')
        && characters.all(|value| value.is_ascii_alphanumeric() || value == '_')
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
    let mut node_name = first.text.clone();

    if (first.text.starts_with("views:") || first.text.starts_with("endpoints:"))
        && !first.text.starts_with('"')
        && !first.text.starts_with('{')
        && !first.text.starts_with('[')
        && let Some((name, value)) = first.text.split_once(':')
    {
        if name.is_empty() || value.is_empty() {
            return Err(DoweError::at_path(
                path,
                format!("{line}:{}: prop `{name}` must have a value", first.column),
            ));
        }
        node_name = name.to_string();
        seen_props.insert(name.to_string());
        props.push(SourceProp {
            name: name.to_string(),
            value: parse_value(path, line, first.column + name.len() + 1, value)?,
            location: SourceLocation {
                path: path.to_path_buf(),
                relative_path: relative_path.to_path_buf(),
                line,
                column: first.column,
                indent: column.saturating_sub(1) / 2,
            },
        });
    }

    for (token_index, token) in tokens.iter().enumerate().skip(1) {
        let typed_binding = matches!(node_name.as_str(), "let" | "const") && token_index == 1;
        if !token.text.starts_with('"')
            && !token.text.starts_with('{')
            && !token.text.starts_with('[')
            && !typed_binding
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
        name: node_name,
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
