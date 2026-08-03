fn parse_file_declaration(node: &SourceNode) -> DoweResult<ServerFileStatement> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "file uses `file <binding> source:\"write|read|exists|delete\" root:<path> path:<path> [data:<bytes>]`",
        ));
    }
    let binding = node.args[0]
        .as_string_like()
        .ok_or_else(|| node_error(node, "file requires a result binding"))?;
    validate_binding_name(node, &binding)?;
    let source = required_source_selector(node, "file")?;
    let root = required_store_literal_prop(node, "root")?;
    let path = required_store_literal_prop(node, "path")?;
    match source.as_str() {
        "write" => {
            reject_unknown_props(node, &["source", "root", "path", "data", "sha256"])?;
            Ok(ServerFileStatement::Write {
                binding,
                root,
                path,
                data: required_reference_prop(node, "data")?,
                sha256: node
                    .prop("sha256")
                    .map(|prop| store_literal(&prop.value))
                    .transpose()?,
            })
        }
        "read" => {
            reject_unknown_props(node, &["source", "root", "path"])?;
            Ok(ServerFileStatement::Read {
                binding,
                root,
                path,
            })
        }
        "exists" => {
            reject_unknown_props(node, &["source", "root", "path"])?;
            Ok(ServerFileStatement::Exists {
                binding,
                root,
                path,
            })
        }
        "delete" => {
            reject_unknown_props(node, &["source", "root", "path"])?;
            Ok(ServerFileStatement::Delete {
                binding,
                root,
                path,
            })
        }
        _ => Err(node_error(
            node,
            "file `source` must be `write`, `read`, `exists`, or `delete`",
        )),
    }
}

fn validate_file_statement_references(
    node: &SourceNode,
    statement: &ServerFileStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    let (root, path) = match statement {
        ServerFileStatement::Write {
            root,
            path,
            data,
            sha256,
            ..
        } => {
            validate_reference_path(node, data, bindings)?;
            if let Some(sha256) = sha256 {
                validate_store_literal_references(node, sha256, bindings)?;
            }
            (root, path)
        }
        ServerFileStatement::Read { root, path, .. }
        | ServerFileStatement::Exists { root, path, .. }
        | ServerFileStatement::Delete { root, path, .. } => (root, path),
    };
    validate_store_literal_references(node, root, bindings)?;
    validate_store_literal_references(node, path, bindings)
}

fn infer_file_statement(statement: &ServerFileStatement, bindings: &mut HashMap<String, DoweType>) {
    let binding = match statement {
        ServerFileStatement::Write { binding, .. }
        | ServerFileStatement::Read { binding, .. }
        | ServerFileStatement::Exists { binding, .. }
        | ServerFileStatement::Delete { binding, .. } => binding,
    };
    bindings.insert(binding.clone(), DoweType::Unknown);
}
