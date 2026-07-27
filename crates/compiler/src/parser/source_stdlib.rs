use crate::error::{DoweError, DoweResult};
use crate::model::{DoweType, DoweTypeField};
use crate::parser::source_ast::{SourceNode, SourceObjectEntry, SourceProp, SourceValue};
use dowe_stdlib::{
    StdlibArgument, StdlibCall, StdlibReturnKind, StdlibSurface, StdlibValue, validate_call,
};

pub fn parse_stdlib_call(
    node: &SourceNode,
    expression: &str,
    surface: StdlibSurface,
    excluded_props: &[&str],
) -> DoweResult<Option<StdlibCall>> {
    let Some((namespace, function)) = expression.split_once('.') else {
        return Ok(None);
    };
    if !dowe_stdlib::is_stdlib_namespace(namespace) {
        return Ok(None);
    }
    if dowe_stdlib::signature(namespace, function).is_none() {
        return Err(node_error(
            node,
            format!("unsupported stdlib function `{expression}`"),
        ));
    }
    let call = StdlibCall {
        namespace: namespace.to_string(),
        function: function.to_string(),
        args: node
            .props
            .iter()
            .filter(|prop| !excluded_props.iter().any(|name| *name == prop.name))
            .map(|prop| {
                Ok(StdlibArgument {
                    name: prop.name.clone(),
                    value: stdlib_value(prop)?,
                })
            })
            .collect::<DoweResult<Vec<_>>>()?,
    };
    validate_call(&call, surface).map_err(|error| node_error(node, error.to_string()))?;
    Ok(Some(call))
}

pub fn stdlib_value(prop: &SourceProp) -> DoweResult<StdlibValue> {
    source_value_to_stdlib(&prop.value)
        .map_err(|message| prop_error(prop, format!("invalid stdlib argument: {message}")))
}

pub fn dowe_type_from_stdlib_return(kind: StdlibReturnKind) -> DoweType {
    match kind {
        StdlibReturnKind::Unknown => DoweType::Unknown,
        StdlibReturnKind::Null => DoweType::Null,
        StdlibReturnKind::Bool => DoweType::Bool,
        StdlibReturnKind::Number => DoweType::Number,
        StdlibReturnKind::String => DoweType::String,
        StdlibReturnKind::Array => DoweType::Array(Box::new(DoweType::Unknown)),
        StdlibReturnKind::Object => DoweType::Object(vec![DoweTypeField {
            name: "value".to_string(),
            value: DoweType::Unknown,
            optional: true,
        }]),
    }
}

fn source_value_to_stdlib(value: &SourceValue) -> Result<StdlibValue, String> {
    Ok(match value {
        SourceValue::String(value) => StdlibValue::String(value.clone()),
        SourceValue::Number(value) => StdlibValue::Number(value.clone()),
        SourceValue::Boolean(value) => StdlibValue::Bool(*value),
        SourceValue::Null => StdlibValue::Null,
        SourceValue::Bareword(value) => StdlibValue::Reference(value.clone()),
        SourceValue::Array(values) => StdlibValue::Array(
            values
                .iter()
                .map(source_value_to_stdlib)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        SourceValue::Object(entries) => {
            let mut values = Vec::new();
            for entry in entries {
                match entry {
                    SourceObjectEntry::KeyValue { key, value } => {
                        values.push((key.clone(), source_value_to_stdlib(value)?));
                    }
                    SourceObjectEntry::Spread(value) => {
                        return Err(format!("spread `{value}` is not supported"));
                    }
                }
            }
            StdlibValue::Object(values)
        }
    })
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

fn prop_error(prop: &SourceProp, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &prop.location.path,
        format!(
            "{}:{}: {}",
            prop.location.line,
            prop.location.column,
            message.as_ref()
        ),
    )
}
