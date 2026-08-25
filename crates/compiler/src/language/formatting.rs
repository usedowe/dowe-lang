use crate::error::DoweResult;
use crate::language::analysis::document_workspace_root;
use crate::parser::{
    SourceFile, SourceNode, SourceObjectEntry, SourceProp, SourceValue, parse_source_file,
};
use dowe_components::ColorFamily;
use std::path::Path;

const MAX_LINE_WIDTH: usize = 100;

pub fn format_document(root: &Path, path: &Path, source: &str) -> DoweResult<String> {
    let root = document_workspace_root(root, path);
    let file = parse_source_file(&root, path, source.to_string())?;
    Ok(format_file(&file))
}

fn format_file(file: &SourceFile) -> String {
    let mut lines = Vec::new();
    let mut imports = file.imports.iter().peekable();
    while let Some(import) = imports.next() {
        let mut names = vec![import.local.as_str()];
        while imports.peek().is_some_and(|next| {
            next.location.line == import.location.line && next.path == import.path
        }) {
            names.push(imports.next().expect("peeked import").local.as_str());
        }
        lines.push(format_import(&names, &import.path));
    }
    for node in &file.nodes {
        format_node(node, &mut lines);
    }
    let mut output = lines.join("\n");
    output.push('\n');
    output
}

fn format_import(names: &[&str], path: &str) -> String {
    format!("import {} from \"{}\"", names.join(", "), path)
}

fn format_node(node: &SourceNode, lines: &mut Vec<String>) {
    let indent = "  ".repeat(node.location.indent);
    if matches!(node.name.as_str(), "views" | "endpoints")
        && node.args.is_empty()
        && node
            .props
            .first()
            .is_some_and(|prop| prop.name == node.name)
    {
        format_self_named_node(node, &indent, lines);
        for child in &node.children {
            format_node(child, lines);
        }
        return;
    }
    let mut header = vec![node.name.clone()];
    header.extend(node.args.iter().map(SourceValue::to_source));
    let mut inline = header.clone();
    inline.extend(node.props.iter().map(format_prop));
    let inline = format!("{indent}{}", inline.join(" "));
    let inline = indent_multiline_lines(&inline, &indent);
    let has_multiline_string = node
        .props
        .iter()
        .any(|prop| matches!(&prop.value, SourceValue::String(value) if value.contains('\n')));
    let grouped_colors = node.name == "colors"
        && node.args.is_empty()
        && node.props.is_empty()
        && !node.children.is_empty()
        && node.children.iter().all(|child| {
            ColorFamily::from_theme_name(&child.name).is_some()
                || matches!(
                    child.name.as_str(),
                    "softPrimary"
                        | "softSecondary"
                        | "softAccent"
                        | "softMuted"
                        | "softSuccess"
                        | "softInfo"
                        | "softWarning"
                        | "softDanger"
                )
        });
    if grouped_colors {
        lines.push(format!("{indent}colors:"));
    } else if !node.props.is_empty()
        && (inline.chars().count() > MAX_LINE_WIDTH || has_multiline_string)
    {
        lines.push(format!("{indent}{}:", header.join(" ")));
        for prop in &node.props {
            format_multiline_prop(prop, node.location.indent + 1, lines);
        }
    } else {
        lines.push(inline);
    }
    for child in &node.children {
        format_node(child, lines);
    }
}

fn format_self_named_node(node: &SourceNode, indent: &str, lines: &mut Vec<String>) {
    if let [prop] = node.props.as_slice()
        && matches!(prop.value, SourceValue::Array(_) | SourceValue::Object(_))
        && indent.chars().count() + prop.name.len() + prop.value.to_source().chars().count() + 1
            > MAX_LINE_WIDTH
    {
        format_value(
            &prop.value,
            node.location.indent,
            format!("{indent}{}:", prop.name),
            true,
            lines,
        );
        return;
    }
    lines.push(format!(
        "{indent}{}",
        node.props
            .iter()
            .map(format_prop)
            .collect::<Vec<_>>()
            .join(" ")
    ));
}

fn format_prop(prop: &SourceProp) -> String {
    format!("{}:{}", prop.name, prop.value.to_source())
}

fn indent_multiline_lines(value: &str, indent: &str) -> String {
    value.replace("\n\"\"\"", &format!("\n{indent}\"\"\""))
}

fn format_multiline_prop(prop: &SourceProp, indent: usize, lines: &mut Vec<String>) {
    let prefix = format!("{}{}:", "  ".repeat(indent), prop.name);
    if let SourceValue::String(value) = &prop.value
        && value.contains('\n')
    {
        lines.push(format!("{prefix}\"\"\""));
        let content_indent = "  ".repeat(indent + 1);
        lines.extend(value.split('\n').map(|line| {
            let line = line.replace("\"\"\"", "\\\"\\\"\\\"");
            if line.is_empty() {
                String::new()
            } else {
                format!("{content_indent}{line}")
            }
        }));
        lines.push(format!("{}\"\"\"", "  ".repeat(indent)));
        return;
    }
    let inline = format!("{prefix}{}", prop.value.to_source());
    let expand = inline.chars().count() > MAX_LINE_WIDTH
        && matches!(prop.value, SourceValue::Array(_) | SourceValue::Object(_));
    format_value(&prop.value, indent, prefix, expand, lines);
}

fn format_value(
    value: &SourceValue,
    indent: usize,
    prefix: String,
    expand: bool,
    lines: &mut Vec<String>,
) {
    match value {
        SourceValue::String(value) if value.contains('\n') => {
            lines.push(format!("{prefix}\"\"\""));
            let content_indent = "  ".repeat(indent + 1);
            lines.extend(value.split('\n').map(|line| {
                let line = line.replace("\"\"\"", "\\\"\\\"\\\"");
                if line.is_empty() {
                    String::new()
                } else {
                    format!("{content_indent}{line}")
                }
            }));
            lines.push(format!("{}\"\"\"", "  ".repeat(indent)));
        }
        SourceValue::Array(values) if expand => {
            lines.push(format!("{prefix}["));
            for value in values {
                let item_prefix = "  ".repeat(indent + 1);
                let item_inline = format!("{item_prefix}{}", value.to_source());
                let item_expand = matches!(value, SourceValue::Array(_) | SourceValue::Object(_))
                    || item_inline.chars().count() > MAX_LINE_WIDTH;
                format_value(value, indent + 1, item_prefix, item_expand, lines);
            }
            lines.push(format!("{}]", "  ".repeat(indent)));
        }
        SourceValue::Object(entries) if expand => {
            lines.push(format!("{prefix}{{"));
            for entry in entries {
                match entry {
                    SourceObjectEntry::KeyValue { key, value } => {
                        let entry_prefix = format!("{}{key}:", "  ".repeat(indent + 1));
                        let entry_inline = format!("{entry_prefix}{}", value.to_source());
                        let entry_expand = entry_inline.chars().count() > MAX_LINE_WIDTH
                            && matches!(value, SourceValue::Array(_) | SourceValue::Object(_));
                        format_value(value, indent + 1, entry_prefix, entry_expand, lines);
                    }
                    SourceObjectEntry::Spread(value) => {
                        lines.push(format!("{}...{value}", "  ".repeat(indent + 1)));
                    }
                }
            }
            lines.push(format!("{}}}", "  ".repeat(indent)));
        }
        _ => lines.push(format!("{prefix}{}", value.to_source())),
    }
}
