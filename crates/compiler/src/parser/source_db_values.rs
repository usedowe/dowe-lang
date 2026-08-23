fn required_string_prop(node: &SourceNode, name: &str) -> DoweResult<String> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("store operation must declare `{name}`")))?;
    match &prop.value {
        SourceValue::String(value) if !value.is_empty() => Ok(value.clone()),
        _ => Err(node_error(
            node,
            format!("`{name}` must be a quoted static string literal"),
        )),
    }
}

fn required_literal_prop(node: &SourceNode, name: &str) -> DoweResult<StoreLiteral> {
    let value = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("store operation must declare `{name}`")))?;
    store_literal(&value.value)
}

pub fn store_literal(value: &SourceValue) -> DoweResult<StoreLiteral> {
    Ok(match value {
        SourceValue::String(value) => StoreLiteral::String(value.clone()),
        SourceValue::Number(value) => StoreLiteral::Number(value.clone()),
        SourceValue::Boolean(value) => StoreLiteral::Bool(*value),
        SourceValue::Null => StoreLiteral::Null,
        SourceValue::Bareword(value) => StoreLiteral::Reference(value.clone()),
        SourceValue::Array(values) => StoreLiteral::Array(
            values
                .iter()
                .map(store_literal)
                .collect::<DoweResult<Vec<_>>>()?,
        ),
        SourceValue::Object(entries) => {
            let mut values = Vec::new();
            for entry in entries {
                match entry {
                    SourceObjectEntry::KeyValue { key, value } => {
                        values.push((key.clone(), store_literal(value)?));
                    }
                    SourceObjectEntry::Spread(value) => {
                        return Err(DoweError::new(format!(
                            "store literals do not support spread `{value}`"
                        )));
                    }
                }
            }
            StoreLiteral::Object(values)
        }
    })
}

fn required_filter_prop(node: &SourceNode, name: &str) -> DoweResult<StoreFilter> {
    let value = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("store operation must declare `{name}`")))?;
    let SourceValue::Object(entries) = &value.value else {
        return Err(node_error(node, format!("`{name}` must be an object")));
    };
    if entries.is_empty() {
        return Err(node_error(
            node,
            format!("`{name}` must declare at least one equality field"),
        ));
    }
    let SourceObjectEntry::KeyValue { key, value } = &entries[0] else {
        return Err(node_error(node, format!("`{name}` cannot use spread")));
    };
    validate_database_name(node, key, "field")?;
    let mut additional = Vec::new();
    for entry in &entries[1..] {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(node_error(node, format!("`{name}` cannot use spread")));
        };
        validate_database_name(node, key, "field")?;
        additional.push(StoreMatchField {
            field: key.clone(),
            value: store_literal(value)?,
        });
    }
    Ok(StoreFilter {
        field: key.clone(),
        value: store_literal(value)?,
        additional,
    })
}

fn optional_match_fields_prop(node: &SourceNode, name: &str) -> DoweResult<Vec<StoreMatchField>> {
    let Some(prop) = node.prop(name) else {
        return Ok(Vec::new());
    };
    let SourceValue::Object(entries) = &prop.value else {
        return Err(node_error(node, format!("`{name}` must be an object")));
    };
    let mut output = Vec::new();
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(node_error(node, format!("`{name}` cannot use spread")));
        };
        validate_database_name(node, key, "field")?;
        output.push(StoreMatchField {
            field: key.clone(),
            value: store_literal(value)?,
        });
    }
    Ok(output)
}

fn optional_string_array_prop(node: &SourceNode, name: &str) -> DoweResult<Vec<String>> {
    let Some(prop) = node.prop(name) else {
        return Ok(Vec::new());
    };
    let SourceValue::Array(values) = &prop.value else {
        return Err(node_error(node, format!("`{name}` must be an array")));
    };
    let mut output = Vec::new();
    for value in values {
        let SourceValue::String(value) = value else {
            return Err(node_error(
                node,
                format!("`{name}` values must be quoted static string literals"),
            ));
        };
        validate_database_name(node, &value, "field")?;
        output.push(value.clone());
    }
    Ok(output)
}

fn optional_bool_prop(node: &SourceNode, name: &str) -> DoweResult<Option<bool>> {
    node.prop(name)
        .map(|prop| match &prop.value {
            SourceValue::Boolean(value) => Ok(*value),
            _ => Err(node_error(node, format!("`{name}` must be boolean"))),
        })
        .transpose()
}

fn optional_query_params_prop(node: &SourceNode) -> DoweResult<Vec<StoreLiteral>> {
    let Some(prop) = node.prop("params") else {
        return Ok(Vec::new());
    };
    let SourceValue::Array(values) = &prop.value else {
        return Err(node_error(node, "`params` must be an array"));
    };
    values.iter().map(store_literal).collect()
}

