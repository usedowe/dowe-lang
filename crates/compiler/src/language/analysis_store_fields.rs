fn imported_view_store_fields(
    root: &Path,
    file: &SourceFile,
    reference_root: &str,
) -> Option<Vec<String>> {
    let import = file
        .imports
        .iter()
        .find(|import| import.local == reference_root)?;
    let path = resolve_import(root, &file.path, import).ok()?;
    let target = read_source_file(root, &path).ok()?;
    let store = target.nodes.iter().find(|node| {
        node.name == "store"
            && node
                .args
                .first()
                .and_then(SourceValue::as_required_string)
                .is_some_and(|name| name == reference_root)
    })?;
    let types = crate::parser::TypeRegistry::parse_file(root, &target).ok()?;
    store
        .prop("type")
        .and_then(|prop| prop.value.as_required_string())
        .and_then(|name| types.resolve(store, &name).ok())
        .map(|schema| reference_fields_for_type(&schema))
        .or_else(|| store.prop("value").map(|prop| signal_fields(&prop.value)))
}

fn collect_store_table_fields(nodes: &[SourceNode], tables: &mut HashMap<String, DoweType>) {
    for node in nodes {
        if let Some((_, fields)) = store_binding_fields(node, tables)
            && store_binding_expression(node)
                .is_some_and(|(_, expression)| expression.ends_with(".insert"))
            && let Some(table) = prop_string(node, "table")
        {
            tables.insert(table, fields);
        }
        collect_store_table_fields(&node.children, tables);
    }
}

fn find_reference_fields(
    nodes: &[SourceNode],
    tables: &HashMap<String, DoweType>,
    types: &crate::parser::TypeRegistry,
    reference_root: &str,
) -> Option<Vec<String>> {
    for node in nodes {
        if matches!(node.name.as_str(), "signal" | "const")
            && node
                .args
                .first()
                .and_then(SourceValue::as_required_string)
                .is_some_and(|name| name == reference_root)
        {
            return signal_type(node, types)
                .map(|value| reference_fields_for_type(&value))
                .or_else(|| node.prop("value").map(|prop| signal_fields(&prop.value)));
        }
        if let Some((binding, fields)) = request_json_binding_fields(node, types)
            && binding == reference_root
        {
            return Some(reference_fields_for_type(&fields));
        }
        if let Some((binding, fields)) = store_binding_fields(node, tables)
            && binding == reference_root
        {
            return Some(reference_fields_for_type(&fields));
        }
        if let Some((binding, fields)) = kv_binding_fields(node)
            && binding == reference_root
        {
            return Some(reference_fields_for_type(&fields));
        }
        if let Some((binding, fields)) = vector_binding_fields(node)
            && binding == reference_root
        {
            return Some(reference_fields_for_type(&fields));
        }
        if let Some((binding, fields)) = queue_binding_fields(node)
            && binding == reference_root
        {
            return Some(reference_fields_for_type(&fields));
        }
        if let Some(fields) = find_reference_fields(&node.children, tables, types, reference_root) {
            return Some(fields);
        }
    }
    None
}

fn store_binding_fields(
    node: &SourceNode,
    tables: &HashMap<String, DoweType>,
) -> Option<(String, DoweType)> {
    let (binding, expression) = store_binding_expression(node)?;
    if expression.ends_with(".insert") {
        let value = node.prop("value")?;
        let mut schema = type_from_source_value(&value.value);
        if let DoweType::Object(fields) = &mut schema
            && !fields.iter().any(|field| field.name == "id")
        {
            fields.push(DoweTypeField {
                name: "id".to_string(),
                value: DoweType::String,
                optional: false,
            });
        }
        return Some((binding, schema));
    }
    if expression.ends_with(".read") {
        let table = prop_string(node, "table")?;
        return tables.get(&table).cloned().map(|fields| (binding, fields));
    }
    if expression.ends_with(".update") || expression.ends_with(".delete") {
        return Some((
            binding,
            DoweType::Object(vec![DoweTypeField {
                name: "changed".to_string(),
                value: DoweType::Number,
                optional: false,
            }]),
        ));
    }
    None
}

fn store_binding_expression(node: &SourceNode) -> Option<(String, String)> {
    if node.name == "query" {
        let binding = node.args.first()?.as_required_string()?;
        let expression = node.prop("conn")?.value.as_required_string()?;
        return Some((binding, expression));
    }
    assignment_expression(node)
}

fn kv_binding_fields(node: &SourceNode) -> Option<(String, DoweType)> {
    let (binding, expression) = kv_binding_expression(node)?;
    if expression.ends_with(".set") {
        return Some((
            binding,
            DoweType::Object(vec![
                DoweTypeField {
                    name: "ok".to_string(),
                    value: DoweType::Bool,
                    optional: false,
                },
                DoweTypeField {
                    name: "key".to_string(),
                    value: DoweType::String,
                    optional: false,
                },
            ]),
        ));
    }
    if expression.ends_with(".delete") {
        return Some((
            binding,
            DoweType::Object(vec![DoweTypeField {
                name: "deleted".to_string(),
                value: DoweType::Bool,
                optional: false,
            }]),
        ));
    }
    if expression.ends_with(".clear") {
        return Some((
            binding,
            DoweType::Object(vec![DoweTypeField {
                name: "cleared".to_string(),
                value: DoweType::Number,
                optional: false,
            }]),
        ));
    }
    None
}

fn kv_binding_expression(node: &SourceNode) -> Option<(String, String)> {
    if node.name != "kv" {
        return None;
    }
    let binding = node.args.first()?.as_required_string()?;
    let expression = node.prop("conn")?.value.as_required_string()?;
    Some((binding, expression))
}

fn vector_binding_fields(node: &SourceNode) -> Option<(String, DoweType)> {
    if node.name != "emb" {
        return None;
    }
    let binding = node.args.first()?.as_required_string()?;
    let expression = node.prop("conn")?.value.as_required_string()?;
    let fields = if expression.ends_with(".upsert") {
        vec![
            DoweTypeField {
                name: "id".to_string(),
                value: DoweType::String,
                optional: false,
            },
            DoweTypeField {
                name: "dimensions".to_string(),
                value: DoweType::Number,
                optional: false,
            },
            DoweTypeField {
                name: "created".to_string(),
                value: DoweType::Bool,
                optional: false,
            },
        ]
    } else if expression.ends_with(".delete") {
        vec![DoweTypeField {
            name: "deleted".to_string(),
            value: DoweType::Bool,
            optional: false,
        }]
    } else if expression.ends_with(".read") {
        vec![
            DoweTypeField {
                name: "id".to_string(),
                value: DoweType::String,
                optional: false,
            },
            DoweTypeField {
                name: "vector".to_string(),
                value: DoweType::Array(Box::new(DoweType::Number)),
                optional: false,
            },
            DoweTypeField {
                name: "metadata".to_string(),
                value: DoweType::Unknown,
                optional: false,
            },
        ]
    } else {
        return None;
    };
    Some((binding, DoweType::Object(fields)))
}

fn queue_binding_fields(node: &SourceNode) -> Option<(String, DoweType)> {
    if node.name != "msg"
        || !node
            .prop("conn")?
            .value
            .as_required_string()?
            .ends_with(".publish")
    {
        return None;
    }
    let binding = node.args.first()?.as_required_string()?;
    Some((binding, queue_publish_result_type()))
}

fn request_json_binding_fields(
    node: &SourceNode,
    types: &crate::parser::TypeRegistry,
) -> Option<(String, DoweType)> {
    if node.name != "const" || node.args.len() != 1 {
        return None;
    }
    let binding = node.args[0].as_string_like()?;
    let (_, type_name) = binding.split_once(':')?;
    if node.prop("value")?.value.as_string_like()?.as_str() != "req.json" {
        return None;
    }
    let (binding, _) = binding.split_once(':')?;
    types
        .resolve(node, type_name)
        .ok()
        .map(|value| (binding.to_string(), value))
}


