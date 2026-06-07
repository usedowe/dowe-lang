use crate::error::DoweResult;
use crate::language::analysis::document_workspace_root;
use crate::parser::{
    SourceFile, SourceImport, SourceNode, SourceProp, SourceValue, parse_source_file,
};
use std::path::Path;

pub fn format_document(root: &Path, path: &Path, source: &str) -> DoweResult<String> {
    let root = document_workspace_root(root, path);
    let file = parse_source_file(&root, path, source.to_string())?;
    Ok(format_file(&file))
}

fn format_file(file: &SourceFile) -> String {
    let mut lines = Vec::new();
    for import in &file.imports {
        lines.push(format_import(import));
    }
    for node in &file.nodes {
        format_node(node, &mut lines);
    }
    let mut output = lines.join("\n");
    output.push('\n');
    output
}

fn format_import(import: &SourceImport) -> String {
    format!("import {} from \"{}\"", import.local, import.path)
}

fn format_node(node: &SourceNode, lines: &mut Vec<String>) {
    let indent = "  ".repeat(node.location.indent);
    let mut parts = vec![node.name.clone()];
    parts.extend(node.args.iter().map(SourceValue::to_source));
    parts.extend(node.props.iter().map(format_prop));
    lines.push(format!("{indent}{}", parts.join(" ")));
    for child in &node.children {
        format_node(child, lines);
    }
}

fn format_prop(prop: &SourceProp) -> String {
    format!("{}:{}", prop.name, prop.value.to_source())
}
