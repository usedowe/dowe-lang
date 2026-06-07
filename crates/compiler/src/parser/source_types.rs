use crate::error::{DoweError, DoweResult};
use crate::model::{DoweType, DoweTypeField, StoreLiteral};
use crate::parser::source_ast::{SourceNode, SourceObjectEntry, SourceValue};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct TypeRegistry {
    definitions: HashMap<String, DoweType>,
}

impl TypeRegistry {
    pub fn parse(path: &Path, nodes: &[SourceNode]) -> DoweResult<Self> {
        let mut declarations = HashMap::<String, SourceNode>::new();
        for node in nodes.iter().filter(|node| node.name == "type") {
            let name = node
                .args
                .first()
                .and_then(SourceValue::as_required_string)
                .ok_or_else(|| node_error(node, "type must declare a name"))?;
            validate_identifier(node, &name, "type")?;
            if node.args.len() != 1 || !node.props.is_empty() {
                return Err(node_error(node, "type only accepts one name"));
            }
            if node.children.is_empty() {
                return Err(node_error(
                    node,
                    format!("type `{name}` must declare fields"),
                ));
            }
            if declarations.insert(name.clone(), node.clone()).is_some() {
                return Err(node_error(node, format!("duplicate type `{name}`")));
            }
        }

        let mut definitions = HashMap::new();
        let names = declarations.keys().cloned().collect::<HashSet<_>>();
        for name in names.iter() {
            resolve_type(
                name,
                &declarations,
                &names,
                &mut definitions,
                &mut Vec::new(),
            )?;
        }

        for node in nodes {
            if node.name == "type" || matches!(node.name.as_str(), "main" | "page" | "layout") {
                continue;
            }
            if declarations.is_empty() {
                continue;
            }
            if node.location.indent == 0
                && !matches!(node.name.as_str(), "handler" | "middleware")
                && !node.name.starts_with("type")
            {
                return Err(DoweError::at_path(
                    path,
                    format!(
                        "{}:{}: unsupported top-level block `{}`",
                        node.location.line, node.location.column, node.name
                    ),
                ));
            }
        }

        Ok(Self { definitions })
    }

    pub fn empty() -> Self {
        Self {
            definitions: HashMap::new(),
        }
    }

    pub fn resolve(&self, node: &SourceNode, name: &str) -> DoweResult<DoweType> {
        parse_type_reference(node, name, &self.definitions)
    }
}

fn resolve_type(
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
    if !names.contains(value) {
        return Err(node_error(node, format!("unknown type `{value}`")));
    }
    resolve_type(value, declarations, names, definitions, stack)
}

fn parse_type_reference(
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

pub fn reference_fields_for_type(value: &DoweType) -> Vec<String> {
    match value {
        DoweType::Object(fields) => fields.iter().map(|field| field.name.clone()).collect(),
        DoweType::Array(item) => reference_fields_for_type(item),
        _ => Vec::new(),
    }
}

pub fn validate_reference_path(
    node: &SourceNode,
    reference: &str,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    let Some((binding, path)) = reference.split_once('.') else {
        return Ok(());
    };
    let Some(value) = bindings.get(binding) else {
        return Ok(());
    };
    match resolve_path(value, path) {
        PathResolution::Known | PathResolution::Unknown => Ok(()),
        PathResolution::Missing => Err(node_error(
            node,
            format!("unknown field `{reference}` on typed variable `{binding}`"),
        )),
    }
}

pub fn type_from_source_value(value: &SourceValue) -> DoweType {
    match value {
        SourceValue::Null => DoweType::Null,
        SourceValue::Boolean(_) => DoweType::Bool,
        SourceValue::Number(_) => DoweType::Number,
        SourceValue::String(_) => DoweType::String,
        SourceValue::Bareword(_) => DoweType::Unknown,
        SourceValue::Array(values) => values
            .first()
            .map(type_from_source_value)
            .map(|value| DoweType::Array(Box::new(value)))
            .unwrap_or_else(|| DoweType::Array(Box::new(DoweType::Unknown))),
        SourceValue::Object(entries) => DoweType::Object(
            entries
                .iter()
                .filter_map(|entry| match entry {
                    SourceObjectEntry::KeyValue { key, value } => Some(DoweTypeField {
                        name: key.clone(),
                        value: type_from_source_value(value),
                        optional: false,
                    }),
                    SourceObjectEntry::Spread(_) => None,
                })
                .collect(),
        ),
    }
}

pub fn type_from_store_literal(value: &StoreLiteral) -> DoweType {
    match value {
        StoreLiteral::Null => DoweType::Null,
        StoreLiteral::Bool(_) => DoweType::Bool,
        StoreLiteral::Number(_) => DoweType::Number,
        StoreLiteral::String(_) => DoweType::String,
        StoreLiteral::Reference(_) => DoweType::Unknown,
        StoreLiteral::Array(values) => values
            .first()
            .map(type_from_store_literal)
            .map(|value| DoweType::Array(Box::new(value)))
            .unwrap_or_else(|| DoweType::Array(Box::new(DoweType::Unknown))),
        StoreLiteral::Object(entries) => DoweType::Object(
            entries
                .iter()
                .map(|(name, value)| DoweTypeField {
                    name: name.clone(),
                    value: type_from_store_literal(value),
                    optional: false,
                })
                .collect(),
        ),
    }
}

pub fn validate_source_value_type(
    node: &SourceNode,
    value: &SourceValue,
    schema: &DoweType,
    label: &str,
) -> DoweResult<()> {
    match (value, schema) {
        (_, DoweType::Unknown) => Ok(()),
        (SourceValue::String(_), DoweType::String)
        | (SourceValue::Number(_), DoweType::Number)
        | (SourceValue::Boolean(_), DoweType::Bool)
        | (SourceValue::Null, DoweType::Null) => Ok(()),
        (SourceValue::Array(values), DoweType::Array(item)) => {
            for value in values {
                validate_source_value_type(node, value, item, label)?;
            }
            Ok(())
        }
        (SourceValue::Object(entries), DoweType::Object(fields)) => {
            let entries = entries
                .iter()
                .filter_map(|entry| match entry {
                    SourceObjectEntry::KeyValue { key, value } => Some((key, value)),
                    SourceObjectEntry::Spread(_) => None,
                })
                .collect::<HashMap<_, _>>();
            for field in fields {
                match entries.get(&field.name) {
                    Some(SourceValue::Null) if field.optional => {}
                    Some(value) => validate_source_value_type(node, value, &field.value, label)?,
                    None if field.optional => {}
                    None => {
                        return Err(node_error(
                            node,
                            format!("`{label}` is missing required field `{}`", field.name),
                        ));
                    }
                }
            }
            for key in entries.keys() {
                if !fields.iter().any(|field| &field.name == *key) {
                    return Err(node_error(
                        node,
                        format!("`{label}` declares unknown field `{key}`"),
                    ));
                }
            }
            Ok(())
        }
        (SourceValue::Null, _) => Err(node_error(
            node,
            format!("`{label}` does not match declared type"),
        )),
        _ => Err(node_error(
            node,
            format!("`{label}` does not match declared type"),
        )),
    }
}

enum PathResolution {
    Known,
    Unknown,
    Missing,
}

fn resolve_path(value: &DoweType, path: &str) -> PathResolution {
    let mut current = value.clone();
    for segment in path.split('.') {
        match current {
            DoweType::Unknown => return PathResolution::Unknown,
            DoweType::Object(fields) => {
                let Some(field) = fields.into_iter().find(|field| field.name == segment) else {
                    return PathResolution::Missing;
                };
                current = field.value;
            }
            _ => return PathResolution::Missing,
        }
    }
    PathResolution::Known
}

fn validate_identifier(node: &SourceNode, value: &str, label: &str) -> DoweResult<()> {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(node_error(node, format!("{label} name must not be empty")));
    };
    if !(first.is_ascii_alphabetic() || first == '_')
        || !chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
    {
        return Err(node_error(
            node,
            format!("{label} `{value}` must be an ASCII identifier"),
        ));
    }
    Ok(())
}

fn node_error(node: &SourceNode, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &node.location.path,
        format!(
            "{}:{}: {}",
            node.location.line,
            node.location.column,
            message.as_ref()
        ),
    )
}
