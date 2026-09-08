use super::{node_error, validate_identifier};
use crate::error::DoweResult;
use crate::model::{DoweType, DoweTypeField};
use crate::parser::source_ast::SourceNode;
use std::collections::{HashMap, HashSet};

pub(super) fn resolve_type(
    name: &str,
    declarations: &HashMap<String, SourceNode>,
    names: &HashSet<String>,
    definitions: &mut HashMap<String, DoweType>,
    stack: &mut Vec<String>,
) -> DoweResult<DoweType> {
    if let Some(value) = definitions.get(name) {
        return Ok(value.clone());
    }
    if stack.iter().any(|value| value == name) {
        return Err(node_error(
            declarations.get(name).expect("declared type"),
            format!("recursive type `{name}` is not supported"),
        ));
    }
    stack.push(name.to_string());
    let node = declarations.get(name).expect("declared type");
    let mut fields = Vec::new();
    let mut seen = HashSet::new();
    for child in &node.children {
        let field = parse_type_field(child, declarations, names, definitions, stack)?;
        if !seen.insert(field.name.clone()) {
            return Err(node_error(
                child,
                format!("duplicate field `{}`", field.name),
            ));
        }
        fields.push(field);
    }
    stack.pop();
    let value = DoweType::Object(fields);
    definitions.insert(name.to_string(), value.clone());
    Ok(value)
}

fn parse_type_field(
    node: &SourceNode,
    declarations: &HashMap<String, SourceNode>,
    names: &HashSet<String>,
    definitions: &mut HashMap<String, DoweType>,
    stack: &mut Vec<String>,
) -> DoweResult<DoweTypeField> {
    if !node.args.is_empty() || !node.props.is_empty() || !node.children.is_empty() {
        return Err(node_error(node, "type fields use `name:type`"));
    }
    let Some((raw_name, raw_type)) = node.name.split_once(':') else {
        return Err(node_error(node, "type fields use `name:type`"));
    };
    if raw_name.is_empty() || raw_type.is_empty() {
        return Err(node_error(node, "type fields use `name:type`"));
    }
    let (name, optional) = raw_name
        .strip_suffix('?')
        .map(|value| (value, true))
        .unwrap_or((raw_name, false));
    validate_identifier(node, name, "field")?;
    let value = parse_type_reference_with_declarations(
        node,
        raw_type,
        declarations,
        names,
        definitions,
        stack,
    )?;
    Ok(DoweTypeField {
        name: name.to_string(),
        value,
        optional,
    })
}

fn parse_type_reference_with_declarations(
    node: &SourceNode,
    value: &str,
    declarations: &HashMap<String, SourceNode>,
    names: &HashSet<String>,
    definitions: &mut HashMap<String, DoweType>,
    stack: &mut Vec<String>,
) -> DoweResult<DoweType> {
    if let Some(inner) = value.strip_suffix("[]") {
        if inner.is_empty() {
            return Err(node_error(node, "array type must declare an item type"));
        }
        return parse_type_reference_with_declarations(
            node,
            inner,
            declarations,
            names,
            definitions,
            stack,
        )
        .map(|value| DoweType::Array(Box::new(value)));
    }
    if let Some(value) = scalar_type(value) {
        return Ok(value);
    }
    validate_identifier(node, value, "type")?;
    if let Some(value) = definitions.get(value) {
        return Ok(value.clone());
    }
    if names.contains(value) {
        return resolve_type(value, declarations, names, definitions, stack);
    }
    Err(node_error(node, format!("unknown type `{value}`")))
}

pub(super) fn parse_type_reference(
    node: &SourceNode,
    value: &str,
    definitions: &HashMap<String, DoweType>,
) -> DoweResult<DoweType> {
    if let Some(inner) = value.strip_suffix("[]") {
        if inner.is_empty() {
            return Err(node_error(node, "array type must declare an item type"));
        }
        return parse_type_reference(node, inner, definitions)
            .map(|value| DoweType::Array(Box::new(value)));
    }
    if let Some(value) = scalar_type(value) {
        return Ok(value);
    }
    validate_identifier(node, value, "type")?;
    definitions
        .get(value)
        .cloned()
        .ok_or_else(|| node_error(node, format!("unknown type `{value}`")))
}

fn scalar_type(value: &str) -> Option<DoweType> {
    match value {
        "string" => Some(DoweType::String),
        "number" => Some(DoweType::Number),
        "bool" | "boolean" => Some(DoweType::Bool),
        "null" => Some(DoweType::Null),
        "unknown" => Some(DoweType::Unknown),
        _ => None,
    }
}
