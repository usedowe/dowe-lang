fn build_view_inspector_map(
    root_node: &SourceNode,
    tree: &ViewNode,
    usages_by_path: &HashMap<PathBuf, Vec<dowe_generator_web::ViewInspectorLocation>>,
) -> dowe_generator_web::ViewInspectorMap {
    let (signals, actions) = inspector_runtime_metadata(tree);
    let mut sources = Vec::new();
    collect_inspector_sources(
        &root_node.children,
        usages_by_path,
        &signals,
        &actions,
        &mut sources,
    );
    let expected = count_inspector_elements(tree);
    if sources.len() > expected {
        sources.truncate(expected);
    }
    while sources.len() < expected {
        sources.push(InspectorSource {
            kind: "View".to_string(),
            path: root_node
                .location
                .relative_path
                .to_string_lossy()
                .to_string(),
            start_line: root_node.location.line,
            end_line: root_node.location.line,
            usages: Vec::new(),
            props: Vec::new(),
            signals: signals.clone(),
            actions: actions.clone(),
        });
    }
    let nodes = sources
        .into_iter()
        .enumerate()
        .map(|(index, source)| {
            let id = inspector_id(index, &source);
            dowe_generator_web::ViewInspectorNode {
                id,
                kind: source.kind,
                source_path: source.path,
                start_line: source.start_line,
                end_line: source.end_line,
                usages: source.usages,
                props: source.props,
                signals: source.signals,
                actions: source.actions,
            }
        })
        .collect();
    dowe_generator_web::ViewInspectorMap { nodes }
}

#[derive(Clone)]
struct InspectorSource {
    kind: String,
    path: String,
    start_line: usize,
    end_line: usize,
    usages: Vec<dowe_generator_web::ViewInspectorLocation>,
    props: Vec<dowe_generator_web::ViewInspectorProp>,
    signals: Vec<dowe_generator_web::ViewInspectorSignal>,
    actions: Vec<dowe_generator_web::ViewInspectorAction>,
}

fn collect_inspector_sources(
    nodes: &[SourceNode],
    usages_by_path: &HashMap<PathBuf, Vec<dowe_generator_web::ViewInspectorLocation>>,
    signals: &[dowe_generator_web::ViewInspectorSignal],
    actions: &[dowe_generator_web::ViewInspectorAction],
    output: &mut Vec<InspectorSource>,
) {
    for node in nodes {
        let structural = matches!(
            node.name.as_str(),
            "meta"
                | "const"
                | "signal"
                | "fn"
                | "init"
                | "action"
                | "if"
                | "else"
                | "each"
                | "children"
                | "Splash"
                | "top"
                | "start"
                | "center"
                | "end"
                | "bottom"
                | "header"
                | "body"
                | "footer"
                | "overlays"
                | "appBar"
                | "main"
                | "content"
                | "actions"
                | "tab"
                | "item"
                | "option"
                | "column"
                | "group"
                | "step"
                | "slide"
        );
        let is_component = !structural && node.name.chars().next().is_some_and(char::is_uppercase);
        if is_component {
            let end_line = source_end_line(node);
            let source = InspectorSource {
                kind: node.name.clone(),
                path: node.location.relative_path.to_string_lossy().to_string(),
                start_line: node.location.line,
                end_line,
                usages: usages_by_path
                    .get(&node.location.path)
                    .cloned()
                    .unwrap_or_default(),
                props: inspector_props(node),
                signals: signals.to_vec(),
                actions: actions.to_vec(),
            };
            output.push(source.clone());
            if !matches!(node.name.as_str(), "Text" | "Title") {
                output.extend(
                    node.children
                        .iter()
                        .filter(|child| child.name.starts_with('"'))
                        .map(|_| source.clone()),
                );
            }
        }
        for child in &node.children {
            if !child.name.starts_with('"') {
                collect_inspector_sources(
                    std::slice::from_ref(child),
                    usages_by_path,
                    signals,
                    actions,
                    output,
                );
            }
        }
    }
}

fn inspector_props(node: &SourceNode) -> Vec<dowe_generator_web::ViewInspectorProp> {
    node.props
        .iter()
        .map(|prop| dowe_generator_web::ViewInspectorProp {
            name: prop.name.clone(),
            value: bounded_inspector_value(&prop.value.to_source()),
        })
        .collect()
}

fn bounded_inspector_value(value: &str) -> String {
    const MAX_VALUE_LENGTH: usize = 160;
    let value = value.replace('\n', "\\n");
    if value.chars().count() <= MAX_VALUE_LENGTH {
        return value;
    }
    let truncated = value.chars().take(MAX_VALUE_LENGTH).collect::<String>();
    format!("{truncated}…")
}

fn inspector_runtime_metadata(
    tree: &ViewNode,
) -> (
    Vec<dowe_generator_web::ViewInspectorSignal>,
    Vec<dowe_generator_web::ViewInspectorAction>,
) {
    let mut signals = Vec::new();
    let mut actions = Vec::new();
    collect_inspector_runtime_metadata(tree, &mut signals, &mut actions);
    (signals, actions)
}

fn collect_inspector_runtime_metadata(
    node: &ViewNode,
    signals: &mut Vec<dowe_generator_web::ViewInspectorSignal>,
    actions: &mut Vec<dowe_generator_web::ViewInspectorAction>,
) {
    match node {
        ViewNode::Scope {
            signals: scope_signals,
            actions: scope_actions,
            children,
            ..
        } => {
            signals.extend(scope_signals.iter().map(inspector_signal));
            actions.extend(scope_actions.iter().map(inspector_action));
            for child in children {
                collect_inspector_runtime_metadata(child, signals, actions);
            }
        }
        ViewNode::Splash {
            content, children, ..
        } => {
            for child in content.iter().chain(children) {
                collect_inspector_runtime_metadata(child, signals, actions);
            }
        }
        _ => {}
    }
}

fn inspector_signal(
    signal: &dowe_components::ViewSignal,
) -> dowe_generator_web::ViewInspectorSignal {
    dowe_generator_web::ViewInspectorSignal {
        id: signal.id.clone(),
        name: signal.name.clone(),
        scope: signal.scope.as_str().to_string(),
        storage: signal.storage.as_str().to_string(),
        initial_json: inspector_signal_value_json(&signal.initial),
    }
}

fn inspector_signal_value_json(value: &dowe_components::ViewSignalValue) -> String {
    match value {
        dowe_components::ViewSignalValue::Null => "null".to_string(),
        dowe_components::ViewSignalValue::Bool(value) => value.to_string(),
        dowe_components::ViewSignalValue::Number(value) => value.clone(),
        dowe_components::ViewSignalValue::String(value) => {
            format!(r#""{}""#, escape_inspector_json(value))
        }
        dowe_components::ViewSignalValue::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(inspector_signal_value_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        dowe_components::ViewSignalValue::Object(entries) => format!(
            "{{{}}}",
            entries
                .iter()
                .map(|(key, value)| {
                    format!(
                        r#""{}":{}"#,
                        escape_inspector_json(key),
                        inspector_signal_value_json(value)
                    )
                })
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn inspector_action(
    action: &dowe_components::ViewAction,
) -> dowe_generator_web::ViewInspectorAction {
    let (kind, detail) = match &action.kind {
        dowe_components::ViewActionKind::Sequence(_) => ("sequence", String::new()),
        dowe_components::ViewActionKind::Request(request) => (
            "request",
            format!("{} {}", request.method.as_str(), request.path),
        ),
        dowe_components::ViewActionKind::Assign(assign) => {
            ("assign", format!("{} ← {}", assign.target, assign.source))
        }
        dowe_components::ViewActionKind::Reset(reset) => ("reset", reset.target.clone()),
    };
    dowe_generator_web::ViewInspectorAction {
        id: action.id.clone(),
        name: action.name.clone(),
        kind: kind.to_string(),
        detail,
    }
}

fn escape_inspector_json(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped
}

fn source_end_line(node: &SourceNode) -> usize {
    node.children
        .iter()
        .map(source_end_line)
        .chain(node.props.iter().map(|prop| prop.location.line))
        .fold(node.location.line, usize::max)
}

fn count_inspector_elements(node: &ViewNode) -> usize {
    let own = usize::from(node_element_props(node).is_some());
    own + node_child_groups(node)
        .into_iter()
        .flat_map(|children| children.iter())
        .map(count_inspector_elements)
        .sum::<usize>()
}

fn inspector_id(index: usize, source: &InspectorSource) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in format!(
        "{}:{}:{}:{}:{}",
        source.path, source.start_line, source.end_line, source.kind, index
    )
    .bytes()
    {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("dn_{hash:016x}")
}
