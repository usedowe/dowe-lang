use crate::error::DoweResult;
use crate::model::{DoweType, DoweTypeField, StoreLiteral};
use crate::parser::source_ast::{SourceNode, SourceObjectEntry, SourceValue};
use std::collections::HashMap;

use super::node_error;

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
