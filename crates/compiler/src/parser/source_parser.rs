use crate::error::{DoweError, DoweResult};
use crate::parser::source_ast::{SourceFile, SourceImport, SourceLocation};
use std::collections::HashSet;
use std::path::Path;

mod assembly;
mod imports;
mod lines;
mod nodes;

use assembly::FlatNode;
use imports::parse_imports;
use lines::logical_lines;
use nodes::{multiline_header, parse_continuation_prop, parse_node};

pub fn parse_source_file(root: &Path, path: &Path, source: String) -> DoweResult<SourceFile> {
    let relative_path = path.strip_prefix(root).unwrap_or(path).to_path_buf();
    let mut imports = Vec::<SourceImport>::new();
    let mut flat_nodes = Vec::new();
    let logical_lines = logical_lines(path, &source)?;
    let mut line_index = 0usize;

    while line_index < logical_lines.len() {
        let logical = &logical_lines[line_index];
        let line_number = logical.line;
        let indent_spaces = logical.indent_spaces;
        let trimmed = logical.source.as_str();
        let column = indent_spaces + 1;
        let location = SourceLocation { path: path.to_path_buf(), relative_path: relative_path.clone(), line: line_number, column, indent: indent_spaces / 2 };

        if trimmed.starts_with("import ") {
            if indent_spaces != 0 {
                return Err(DoweError::at_path(path, format!("{line_number}:{column}: imports must be top-level")));
            }
            imports.extend(parse_imports(path, location, trimmed)?);
            line_index += 1;
        } else {
            let (node_source, opens_prop_suite) = multiline_header(trimmed);
            let mut node = parse_node(path, &relative_path, line_number, column, node_source)?;
            if opens_prop_suite {
                if node.name == "type" {
                    return Err(DoweError::at_path(path, format!("{line_number}:{column}: `type` declarations do not accept property suites")));
                }
                if !node.props.is_empty() {
                    return Err(DoweError::at_path(path, format!("{line_number}:{column}: property suite headers cannot contain inline props")));
                }
                let child_level = indent_spaces / 2 + 1;
                let mut next = line_index + 1;
                let mut seen_props = HashSet::new();
                while next < logical_lines.len() && logical_lines[next].indent_spaces / 2 == child_level {
                    let Some(prop) = parse_continuation_prop(path, &relative_path, &logical_lines[next])? else { break; };
                    if !seen_props.insert(prop.name.clone()) {
                        return Err(DoweError::at_path(path, format!("{}:{}: duplicate prop `{}`", prop.location.line, prop.location.column, prop.name)));
                    }
                    node.props.push(prop);
                    next += 1;
                }
                let mut remaining = next;
                while remaining < logical_lines.len() && logical_lines[remaining].indent_spaces > indent_spaces {
                    if logical_lines[remaining].indent_spaces / 2 == child_level
                        && parse_continuation_prop(path, &relative_path, &logical_lines[remaining])?.is_some()
                    {
                        return Err(DoweError::at_path(path, format!("{}:{}: property suite props must appear before child nodes", logical_lines[remaining].line, logical_lines[remaining].indent_spaces + 1)));
                    }
                    remaining += 1;
                }
                line_index = next;
            } else {
                line_index += 1;
            }
            flat_nodes.push(FlatNode { level: indent_spaces / 2, node });
        }
    }

    let mut index = 0usize;
    let nodes = assembly::parse_block(&flat_nodes, &mut index, 0)?;
    if index < flat_nodes.len() {
        let node = &flat_nodes[index].node;
        return Err(DoweError::at_path(&node.location.path, format!("{}:{}: block is not nested under a parent", node.location.line, node.location.column)));
    }
    Ok(SourceFile { path: path.to_path_buf(), relative_path, imports, nodes, source })
}
