fn parse_inline_on_click_action(
    prop: &SourceProp,
    scope_kind: &str,
    scope_name: &str,
    node_line: usize,
    node_column: usize,
) -> DoweResult<ViewAction> {
    let SourceValue::Object(entries) = &prop.value else {
        return Err(prop_error(
            prop,
            "`onClick` must be a fn or inline set object",
        ));
    };
    let mut target = None;
    let mut operation = None;
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(prop, "`onClick` does not support object spread"));
        };
        match key.as_str() {
            "set" => {
                if target.is_some() {
                    return Err(prop_error(prop, "`onClick` must declare one `set` target"));
                }
                let SourceValue::Bareword(value) = value else {
                    return Err(prop_error(
                        prop,
                        "`onClick set` must be a Signal or View Store path",
                    ));
                };
                target = Some(value.clone());
            }
            "value" => {
                if operation.is_some() {
                    return Err(prop_error(
                        prop,
                        "`onClick` accepts one of `value`, `add`, or `append`",
                    ));
                }
                operation = Some((inline_set_value(prop, value)?, None));
            }
            "add" => {
                if operation.is_some() {
                    return Err(prop_error(
                        prop,
                        "`onClick` accepts one of `value`, `add`, or `append`",
                    ));
                }
                let SourceValue::Number(value) = value else {
                    return Err(prop_error(prop, "`onClick add` must be a number"));
                };
                operation = Some((
                    "$dowe:onClick:add".to_string(),
                    Some(StdlibCall {
                        namespace: "math".to_string(),
                        function: "add".to_string(),
                        args: vec![
                            StdlibArgument {
                                name: "left".to_string(),
                                value: StdlibValue::Reference(String::new()),
                            },
                            StdlibArgument {
                                name: "right".to_string(),
                                value: StdlibValue::Number(value.clone()),
                            },
                        ],
                    }),
                ));
            }
            "append" => {
                if operation.is_some() {
                    return Err(prop_error(
                        prop,
                        "`onClick` accepts one of `value`, `add`, or `append`",
                    ));
                }
                let SourceValue::String(value) = value else {
                    return Err(prop_error(prop, "`onClick append` must be a quoted string"));
                };
                operation = Some((
                    "$dowe:onClick:append".to_string(),
                    Some(StdlibCall {
                        namespace: "str".to_string(),
                        function: "join".to_string(),
                        args: vec![
                            StdlibArgument {
                                name: "values".to_string(),
                                value: StdlibValue::Array(vec![
                                    StdlibValue::Reference(String::new()),
                                    StdlibValue::String(value.clone()),
                                ]),
                            },
                            StdlibArgument {
                                name: "delimiter".to_string(),
                                value: StdlibValue::String(String::new()),
                            },
                        ],
                    }),
                ));
            }
            _ => {
                return Err(prop_error(
                    prop,
                    "`onClick` only supports `set` with `value`, `add`, or `append`",
                ));
            }
        }
    }
    let target = target.ok_or_else(|| prop_error(prop, "`onClick` requires a `set` target"))?;
    let (source, mut call) = operation
        .ok_or_else(|| prop_error(prop, "`onClick` requires `value`, `add`, or `append`"))?;
    if let Some(call) = &mut call {
        match &mut call.args[0].value {
            StdlibValue::Reference(value) => *value = target.clone(),
            StdlibValue::Array(values) => {
                if let Some(StdlibValue::Reference(value)) = values.first_mut() {
                    *value = target.clone();
                }
            }
            _ => {}
        }
    }
    let name = format!("__dowe_on_click_{node_line}_{}", prop.location.column);
    Ok(ViewAction {
        id: synthetic_reactive_id(
            "on-click",
            scope_kind,
            scope_name,
            node_line,
            node_column,
            &name,
        ),
        name,
        params: Vec::new(),
        return_type: None,
        kind: ViewActionKind::Assign(ViewAssignAction {
            target,
            source,
            literal: None,
            call,
        }),
    })
}

fn inline_set_value(prop: &SourceProp, value: &SourceValue) -> DoweResult<String> {
    match value {
        SourceValue::Bareword(value) if value.starts_with('!') && value.len() > 1 => {
            Ok(value.clone())
        }
        SourceValue::Bareword(value) => Ok(value.clone()),
        SourceValue::Boolean(value) => Ok(format!("$dowe:bool:{value}")),
        SourceValue::String(value) => Ok(format!("$dowe:string:{value}")),
        _ => Err(prop_error(
            prop,
            "`onClick value` must be a reference, string, `!reference`, `true`, or `false`",
        )),
    }
}

fn parse_constant(
    node: &SourceNode,
    scope_kind: &str,
    scope_name: &str,
) -> DoweResult<ViewConstant> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "`const` must declare one name and no children",
        ));
    }
    for prop in &node.props {
        if prop.name != "value" {
            return Err(prop_error(
                prop,
                format!("unknown prop `{}` on `const`", prop.name),
            ));
        }
    }
    let name = node.args[0]
        .as_required_string()
        .ok_or_else(|| node_error(node, "`const` must declare a name"))?;
    let value = node
        .prop("value")
        .ok_or_else(|| node_error(node, "`const` requires `value`"))?;
    Ok(ViewConstant {
        id: reactive_id("const", scope_kind, scope_name, node, &name),
        name,
        value: signal_value(&value.value, node)?,
    })
}

fn parse_signal(
    node: &SourceNode,
    scope_kind: &str,
    scope_name: &str,
    types: &TypeRegistry,
) -> DoweResult<ViewSignal> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "`signal` must declare one name and no children",
        ));
    }
    let name = node.args[0]
        .as_required_string()
        .ok_or_else(|| node_error(node, "`signal` must declare a name"))?;
    let value = node
        .prop("value")
        .ok_or_else(|| node_error(node, "`signal` requires `value`"))?;
    let initial = signal_value(&value.value, node)?;
    let scope = signal_scope(node)?;
    let storage = signal_storage(node)?;
    if storage == ViewSignalStorage::Local && scope != ViewSignalScope::Global {
        return Err(node_error(
            node,
            "`signal storage:\"local\"` requires `scope:\"global\"`",
        ));
    }
    let schema = optional_prop_string(node, "type")?
        .map(|name| {
            let schema = types.resolve(node, &name)?;
            validate_source_value_type(node, &value.value, &schema, "signal value")?;
            Ok::<ViewSignalValue, DoweError>(view_schema_value(&schema))
        })
        .transpose()?;
    Ok(ViewSignal {
        id: reactive_id("signal", scope_kind, scope_name, node, &name),
        storage_key: name.clone(),
        name,
        scope,
        storage,
        initial,
        schema,
    })
}

fn signal_scope(node: &SourceNode) -> DoweResult<ViewSignalScope> {
    match optional_static_string_prop(node, "scope")?.as_deref() {
        None | Some("page") => Ok(ViewSignalScope::Page),
        Some("global") => Ok(ViewSignalScope::Global),
        Some(_) => Err(node_error(
            node,
            "`signal scope` must be \"page\" or \"global\"",
        )),
    }
}

fn signal_storage(node: &SourceNode) -> DoweResult<ViewSignalStorage> {
    match optional_static_string_prop(node, "storage")?.as_deref() {
        None | Some("none") => Ok(ViewSignalStorage::None),
        Some("local") => Ok(ViewSignalStorage::Local),
        Some(_) => Err(node_error(
            node,
            "`signal storage` must be \"none\" or \"local\"",
        )),
    }
}

fn view_schema_value(value: &DoweType) -> ViewSignalValue {
    match value {
        DoweType::Unknown | DoweType::Null => ViewSignalValue::Null,
        DoweType::Bool => ViewSignalValue::Bool(false),
        DoweType::Number => ViewSignalValue::Number("0".to_string()),
        DoweType::String => ViewSignalValue::String(String::new()),
        DoweType::Array(item) => ViewSignalValue::Array(vec![view_schema_value(item)]),
        DoweType::Object(fields) => ViewSignalValue::Object(
            fields
                .iter()
                .map(|field| (field.name.clone(), view_schema_field_value(field)))
                .collect(),
        ),
    }
}

fn view_schema_field_value(field: &DoweTypeField) -> ViewSignalValue {
    view_schema_value(&field.value)
}

fn signal_value(value: &SourceValue, node: &SourceNode) -> DoweResult<ViewSignalValue> {
    match value {
        SourceValue::Null => Ok(ViewSignalValue::Null),
        SourceValue::Boolean(value) => Ok(ViewSignalValue::Bool(*value)),
        SourceValue::Number(value) => Ok(ViewSignalValue::Number(value.clone())),
        SourceValue::String(value) => Ok(ViewSignalValue::String(value.clone())),
        SourceValue::Bareword(_) => Err(node_error(
            node,
            "`signal value` string literals must use double quotes",
        )),
        SourceValue::Array(values) => values
            .iter()
            .map(|value| signal_value(value, node))
            .collect::<DoweResult<Vec<_>>>()
            .map(ViewSignalValue::Array),
        SourceValue::Object(entries) => {
            let mut values = Vec::new();
            for entry in entries {
                match entry {
                    SourceObjectEntry::KeyValue { key, value } => {
                        values.push((key.clone(), signal_value(value, node)?));
                    }
                    SourceObjectEntry::Spread(_) => {
                        return Err(node_error(node, "`signal` value cannot use object spread"));
                    }
                }
            }
            Ok(ViewSignalValue::Object(values))
        }
    }
}

