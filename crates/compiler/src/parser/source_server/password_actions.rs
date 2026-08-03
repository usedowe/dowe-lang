fn parse_password_declaration(node: &SourceNode) -> DoweResult<ServerPasswordStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "password uses `password <binding> source:\"hash|verify\" value:<password> [hash:<phc>]`",
        ));
    }
    let binding = node.args[0]
        .as_string_like()
        .ok_or_else(|| node_error(node, "password requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    let source = required_source_selector(node, "password")?;
    let value = required_store_literal_prop(node, "value")?;
    match source.as_str() {
        "hash" => {
            reject_unknown_props(node, &["source", "value"])?;
            Ok(ServerPasswordStatement::Hash { binding, value })
        }
        "verify" => {
            reject_unknown_props(node, &["source", "value", "hash", "required"])?;
            Ok(ServerPasswordStatement::Verify {
                binding,
                value,
                hash: required_store_literal_prop(node, "hash")?,
                required: optional_bool_prop(node, "required")?.unwrap_or(false),
            })
        }
        _ => Err(node_error(
            node,
            "password `source` must be `hash` or `verify`",
        )),
    }
}

fn validate_password_statement_references(
    node: &SourceNode,
    statement: &ServerPasswordStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    match statement {
        ServerPasswordStatement::Hash { value, .. } => {
            validate_store_literal_references(node, value, bindings)
        }
        ServerPasswordStatement::Verify { value, hash, .. } => {
            validate_store_literal_references(node, value, bindings)?;
            validate_store_literal_references(node, hash, bindings)
        }
    }
}

fn infer_password_statement(
    statement: &ServerPasswordStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    match statement {
        ServerPasswordStatement::Hash { binding, .. } => {
            bindings.insert(binding.clone(), DoweType::String);
        }
        ServerPasswordStatement::Verify { binding, .. } => {
            bindings.insert(
                binding.clone(),
                DoweType::Object(vec![DoweTypeField {
                    name: "valid".to_string(),
                    value: DoweType::Bool,
                    optional: false,
                }]),
            );
        }
    }
}
