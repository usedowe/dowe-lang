fn env_completions(root: &Path) -> Vec<LanguageCompletion> {
    environment_config(root)
        .map(|environment| {
            environment
                .variables
                .into_iter()
                .map(|variable| {
                    completion(
                        &variable.name,
                        LanguageCompletionKind::Variable,
                        "env variable",
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn i18n_completions(root: &Path) -> Vec<LanguageCompletion> {
    crate::parser::parse_translation_catalog(root)
        .map(|catalog| {
            catalog
                .locales
                .iter()
                .flat_map(|locale| locale.values.iter().map(|value| value.key.as_str()))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .map(|value| {
                    completion(
                        &format!("\"{value}\""),
                        LanguageCompletionKind::Value,
                        "translation key",
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn action_completions(root: &Path, document: &LanguageDocument) -> Vec<LanguageCompletion> {
    parse_source_file(root, &document.path, document.source.clone())
        .map(|file| {
            file.nodes
                .iter()
                .flat_map(collect_actions)
                .map(|name| completion(&name, LanguageCompletionKind::Function, "view fn"))
                .collect()
        })
        .unwrap_or_else(|_| {
            collect_line_declarations(&document.source, "fn")
                .into_iter()
                .map(|name| completion(&name, LanguageCompletionKind::Function, "view fn"))
                .collect()
        })
}

fn signal_completions(root: &Path, document: &LanguageDocument) -> Vec<LanguageCompletion> {
    parse_source_file(root, &document.path, document.source.clone())
        .map(|file| {
            let types = crate::parser::TypeRegistry::parse_file(root, &file).unwrap_or_default();
            let mut names = file
                .nodes
                .iter()
                .flat_map(|node| collect_signals(node, &types))
                .collect::<Vec<_>>();
            names.extend(imported_view_store_paths(root, &file));
            names
                .into_iter()
                .map(|name| completion(&name, LanguageCompletionKind::Variable, "reactive path"))
                .collect()
        })
        .unwrap_or_else(|_| {
            collect_line_signals(&document.source)
                .into_iter()
                .map(|name| completion(&name, LanguageCompletionKind::Variable, "signal path"))
                .collect()
        })
}

fn imported_view_store_paths(root: &Path, file: &crate::parser::SourceFile) -> Vec<String> {
    let mut output = Vec::new();
    for import in &file.imports {
        let Ok(path) = crate::parser::resolve_import(root, &file.path, import) else {
            continue;
        };
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(target) = parse_source_file(root, &path, source) else {
            continue;
        };
        let Some(store) = target.nodes.iter().find(|node| {
            node.name == "store"
                && node
                    .args
                    .first()
                    .and_then(SourceValue::as_required_string)
                    .is_some_and(|name| name == import.local)
        }) else {
            continue;
        };
        output.push(import.local.clone());
        let types = crate::parser::TypeRegistry::parse_file(root, &target).unwrap_or_default();
        let fields = signal_type_fields(store, &types)
            .or_else(|| store.prop("value").map(|prop| signal_fields(&prop.value)));
        output.extend(
            fields
                .unwrap_or_default()
                .into_iter()
                .map(|field| format!("{}.{field}", import.local)),
        );
    }
    output
}

fn middleware_context(prefix: &str) -> bool {
    prefix
        .split_whitespace()
        .last()
        .is_some_and(|value| value.starts_with("middleware:"))
}

fn middleware_completions(root: &Path, document: &LanguageDocument) -> Vec<LanguageCompletion> {
    parse_source_file(root, &document.path, document.source.clone())
        .map(|file| {
            file.imports
                .iter()
                .filter_map(|import| {
                    let path = crate::parser::resolve_import(root, &file.path, import).ok()?;
                    let source = fs::read_to_string(&path).ok()?;
                    let target = parse_source_file(root, &path, source).ok()?;
                    target
                        .nodes
                        .iter()
                        .any(|node| {
                            node.name == "middleware"
                                && node
                                    .args
                                    .first()
                                    .and_then(SourceValue::as_required_string)
                                    .is_some_and(|name| name == import.local)
                        })
                        .then(|| {
                            completion(
                                &import.local,
                                LanguageCompletionKind::Function,
                                "server middleware",
                            )
                        })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn collect_actions(node: &SourceNode) -> Vec<String> {
    let mut output = Vec::new();
    if node.name == "fn"
        && let Some(name) = node.args.first().and_then(SourceValue::as_required_string)
    {
        output.push(name);
    }
    for child in &node.children {
        output.extend(collect_actions(child));
    }
    output
}

fn collect_signals(node: &SourceNode, types: &crate::parser::TypeRegistry) -> Vec<String> {
    let mut output = Vec::new();
    if matches!(node.name.as_str(), "signal" | "const")
        && let Some(name) = node.args.first().and_then(SourceValue::as_required_string)
    {
        output.push(name.clone());
        if let Some(schema) = signal_type_fields(node, types) {
            output.extend(schema.into_iter().map(|field| format!("{name}.{field}")));
        } else if let Some(value) = node.prop("value") {
            output.extend(
                signal_fields(&value.value)
                    .into_iter()
                    .map(|field| format!("{name}.{field}")),
            );
        }
    }
    for child in &node.children {
        output.extend(collect_signals(child, types));
    }
    output
}

fn signal_type_fields(
    node: &SourceNode,
    types: &crate::parser::TypeRegistry,
) -> Option<Vec<String>> {
    let type_name = node.prop("type")?.value.as_required_string()?;
    let schema = types.resolve(node, &type_name).ok()?;
    Some(crate::parser::reference_fields_for_type(&schema))
}

fn collect_line_declarations(source: &str, keyword: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            if parts.next()? == keyword {
                parts.next().map(str::to_string)
            } else {
                None
            }
        })
        .collect()
}

fn collect_line_signals(source: &str) -> Vec<String> {
    let mut output = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        let mut parts = trimmed.split_whitespace();
        if !matches!(parts.next(), Some("signal" | "const")) {
            continue;
        }
        let Some(name) = parts.next() else {
            continue;
        };
        output.push(name.to_string());
        if let Some(body) = trimmed
            .split("value:{")
            .nth(1)
            .and_then(|value| value.split('}').next())
        {
            for token in body.split_whitespace() {
                if let Some((field, _)) = token.split_once(':')
                    && !field.is_empty()
                {
                    output.push(format!("{name}.{field}"));
                }
            }
        }
    }
    output
}

fn prop_completions(component: &str) -> Vec<LanguageCompletion> {
    props_for_component(component)
        .into_iter()
        .map(|label| {
            documented_completion(
                label,
                LanguageCompletionKind::Property,
                "component prop",
                component_prop_documentation(component, label),
            )
        })
        .collect()
}

fn server_prop_completions(owner: &str) -> Vec<LanguageCompletion> {
    server_props(owner)
        .into_iter()
        .map(|label| {
            documented_completion(
                label,
                LanguageCompletionKind::Property,
                "server prop",
                server_owner_prop_documentation(owner, label),
            )
        })
        .collect()
}

