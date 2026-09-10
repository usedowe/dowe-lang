fn validate_inline_task_value(
    node: &SourceNode,
    value: &SourceValue,
    bindings: &HashSet<String>,
) -> DoweResult<()> {
    match value {
        SourceValue::Bareword(reference) => {
            let binding = reference.split('.').next().unwrap_or(reference);
            if !bindings.contains(binding) {
                return Err(node_error(
                    node,
                    format!(
                        "inline task body cannot capture outer binding `{binding}`; pass it through `args`"
                    ),
                ));
            }
        }
        SourceValue::Array(values) => {
            for value in values {
                validate_inline_task_value(node, value, bindings)?;
            }
        }
        SourceValue::Object(entries) => {
            for entry in entries {
                match entry {
                    SourceObjectEntry::KeyValue { value, .. } => {
                        validate_inline_task_value(node, value, bindings)?;
                    }
                    SourceObjectEntry::Spread(_) => {
                        return Err(node_error(
                            node,
                            "inline task values do not support object spread",
                        ));
                    }
                }
            }
        }
        SourceValue::String(_)
        | SourceValue::Number(_)
        | SourceValue::Boolean(_)
        | SourceValue::Null => {}
    }
    Ok(())
}

fn inline_task_binding(node: &SourceNode, imports: &ServerImports) -> Option<String> {
    let creates_binding = matches!(
        node.name.as_str(),
        "database"
            | "db"
            | "cache"
            | "kv"
            | "query"
            | "vector"
            | "emb"
            | "spawn"
            | "file"
            | "password"
            | "http"
            | "crypto"
            | "jwt"
            | "agent"
    ) || dowe_stdlib::is_stdlib_namespace(&node.name)
        || imports.callables.contains_key(&node.name);
    creates_binding
        .then(|| node.args.first().and_then(SourceValue::as_required_string))
        .flatten()
        .map(|binding| binding.split(':').next().unwrap_or(&binding).to_string())
}

fn background_job_id(node: &SourceNode, kind: &str, target: &str) -> String {
    format!(
        "{}:{}:{kind}:{target}",
        node.location.relative_path.display(),
        node.location.line,
    )
}

fn legacy_task_error(node: &SourceNode) -> DoweError {
    let mut repair = "task".to_string();
    if let Some(target) = node.args.first() {
        repair.push_str(" fn:");
        repair.push_str(&target.to_source());
        for argument in node.args.iter().skip(1) {
            repair.push(' ');
            repair.push_str(&argument.to_source());
        }
    }
    for prop in &node.props {
        repair.push(' ');
        repair.push_str(&prop.name);
        repair.push(':');
        repair.push_str(&prop.value.to_source());
    }
    node_error(node, format!("`go` was renamed to `task`; use `{repair}`"))
}

fn reject_background_references(node: &SourceNode, value: &StoreLiteral) -> DoweResult<()> {
    match value {
        StoreLiteral::Reference(reference) => Err(node_error(
            node,
            format!("background args must be static JSON; found reference `{reference}`"),
        )),
        StoreLiteral::Array(values) => {
            for value in values {
                reject_background_references(node, value)?;
            }
            Ok(())
        }
        StoreLiteral::Object(entries) => {
            for (_, value) in entries {
                reject_background_references(node, value)?;
            }
            Ok(())
        }
        StoreLiteral::Null
        | StoreLiteral::Bool(_)
        | StoreLiteral::Number(_)
        | StoreLiteral::String(_) => Ok(()),
    }
}

fn parse_server_function_params(
    node: &SourceNode,
    types: &TypeRegistry,
) -> DoweResult<Vec<ServerFunctionParameter>> {
    let Some(prop) = node.prop("params") else {
        return Ok(Vec::new());
    };
    let SourceValue::Object(entries) = &prop.value else {
        return Err(prop_error(prop, "fn params must be an object"));
    };
    if entries.is_empty() {
        return Err(prop_error(prop, "fn params must be a non-empty object"));
    }
    let mut params = Vec::new();
    let mut names = HashSet::new();
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(prop, "fn params does not support object spread"));
        };
        if !names.insert(key.clone()) {
            return Err(prop_error(prop, format!("duplicate fn parameter `{key}`")));
        }
        validate_binding_name(node, key)?;
        let type_name = value
            .as_required_string()
            .ok_or_else(|| prop_error(prop, "fn params values must be type names"))?;
        let schema = types.resolve(node, &type_name)?;
        params.push(ServerFunctionParameter {
            name: key.clone(),
            type_name,
            schema,
        });
    }
    Ok(params)
}

