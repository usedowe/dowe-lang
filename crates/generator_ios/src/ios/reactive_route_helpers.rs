fn swift_reactive_context_for_node(
    root: &ViewNode,
    target: &ViewNode,
) -> Option<SwiftReactiveContext> {
    fn find(
        node: &ViewNode,
        target: &ViewNode,
        context: &SwiftReactiveContext,
    ) -> Option<SwiftReactiveContext> {
        if std::ptr::eq(node, target) {
            return Some(context.clone());
        }
        let next = match node {
            ViewNode::Scope {
                constants,
                signals,
                actions,
                ..
            } => context.with_scope(constants, signals, actions),
            ViewNode::Each { item, .. } => context.with_item(item, "row.value".to_string()),
            _ => context.clone(),
        };
        node_child_groups(node)
            .into_iter()
            .flat_map(|group| group.iter())
            .find_map(|child| find(child, target, &next))
    }

    find(root, target, &SwiftReactiveContext::default())
}

fn swift_node_key(node: &ViewNode) -> usize {
    node as *const ViewNode as usize
}

fn swift_reactive_route(tree: &ViewNode) -> SwiftReactiveRoute {
    let mut constants = Vec::new();
    let mut signals = Vec::new();
    let mut metadata = Vec::new();
    let mut actions = Vec::new();
    let mut init = Vec::new();
    let mut autoload = Vec::new();
    collect_swift_reactive(
        tree,
        &SwiftReactiveContext::default(),
        &mut constants,
        &mut signals,
        &mut metadata,
        &mut actions,
        &mut init,
        &mut autoload,
    );
    let form_ids = swift_form_signal_ids(tree);
    let forms = collect_view_forms(tree)
        .iter()
        .map(|form| swift_form_value(form, &form_ids))
        .collect::<Vec<_>>();
    SwiftReactiveRoute {
        constants: swift_dictionary(&constants),
        initial: swift_dictionary(&signals),
        signals: swift_dictionary(&metadata),
        actions: swift_dictionary(&actions),
        forms: swift_dictionary(&forms),
        init,
        autoload,
    }
}

fn swift_form_signal_ids(tree: &ViewNode) -> BTreeMap<String, String> {
    fn collect(node: &ViewNode, output: &mut BTreeMap<String, String>) {
        match node {
            ViewNode::Scope { signals, children, .. } => {
                output.extend(signals.iter().map(|signal| (signal.name.clone(), signal.id.clone())));
                for child in children { collect(child, output); }
            }
            _ => for child in node_child_groups(node).into_iter().flat_map(|group| group.iter()) { collect(child, output); },
        }
    }
    let mut output = BTreeMap::new();
    collect(tree, &mut output);
    output
}

fn swift_form_value(form: &ViewForm, signal_ids: &BTreeMap<String, String>) -> String {
    let signal = signal_ids.get(&form.signal).cloned().unwrap_or_else(|| form.signal.clone());
    let fields = form.fields.iter().map(|field| {
        let rules = field.rules.iter().map(|rule| {
            let argument = match &rule.kind {
                FormValidationRuleKind::Matches(path) => {
                    let (root, suffix) = path.split_once('.').unwrap_or((path.as_str(), ""));
                    let resolved = signal_ids.get(root).cloned().unwrap_or_else(|| root.to_string());
                    let resolved = if suffix.is_empty() { resolved } else { format!("{resolved}.{suffix}") };
                    swift_optional_string(Some(resolved.as_str()))
                }
                _ => swift_optional_string(rule.kind.argument().as_deref()),
            };
            format!("DoweValidationRule(kind: \"{}\", argument: {}, message: \"{}\")", escape_swift(rule.kind.name()), argument, escape_swift(&rule.message))
        }).collect::<Vec<_>>().join(", ");
        format!("DoweFormFieldMetadata(path: \"{}\", kind: \"{}\", rules: [{}])", escape_swift(&field.path), match field.kind { ViewFormFieldKind::Boolean => "boolean", ViewFormFieldKind::String => "string" }, rules)
    }).collect::<Vec<_>>().join(", ");
    format!("\"{}\": [ {} ]", escape_swift(&signal), fields)
}

fn swift_dictionary(values: &[String]) -> String {
    if values.is_empty() {
        "[:]".to_string()
    } else {
        format!("[{}]", values.join(", "))
    }
}

