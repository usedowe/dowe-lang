pub struct ParsedViews {
    pub web: WebOutput,
    pub desktop_web: WebOutput,
    pub routes: ViewTargetRoutes,
}

pub(crate) fn client_environment_names(routes: &ViewTargetRoutes) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for route in routes
        .web
        .iter()
        .chain(&routes.desktop)
        .chain(&routes.android)
        .chain(&routes.ios)
    {
        collect_client_environment(&route.layout_tree, &mut names);
        collect_client_environment(&route.page_tree, &mut names);
    }
    names
}

fn collect_client_environment(node: &ViewNode, names: &mut BTreeSet<String>) {
    if let ViewNode::Scope { actions, .. } = node {
        for action in actions {
            collect_action_environment(&action.kind, names);
        }
    }
    let uses_backend_url = match node {
        ViewNode::Iframe { props } => props.src.starts_with('/'),
        ViewNode::Device { iframe, .. } => iframe.src.starts_with('/'),
        ViewNode::Candlestick { props } => props
            .stream
            .as_deref()
            .is_some_and(|stream| stream.starts_with('/')),
        _ => false,
    };
    if uses_backend_url {
        names.insert("BACKEND_URL".to_string());
    }
    for children in node_child_groups(node) {
        for child in children {
            collect_client_environment(child, names);
        }
    }
}

fn collect_action_environment(kind: &ViewActionKind, names: &mut BTreeSet<String>) {
    match kind {
        ViewActionKind::Sequence(statements) => collect_statement_environment(statements, names),
        ViewActionKind::Request(request) => {
            if let Some(name) = &request.base_env {
                names.insert(name.clone());
            }
        }
        ViewActionKind::Assign(_) | ViewActionKind::Reset(_) => {}
    }
}

fn collect_statement_environment(
    statements: &[ViewFunctionStatement],
    names: &mut BTreeSet<String>,
) {
    for statement in statements {
        match statement {
            ViewFunctionStatement::Request { action, .. } => {
                if let Some(name) = &action.base_env {
                    names.insert(name.clone());
                }
            }
            ViewFunctionStatement::If { success, error, .. } => {
                collect_statement_environment(success, names);
                collect_statement_environment(error, names);
            }
            ViewFunctionStatement::Assign(_)
            | ViewFunctionStatement::Reset(_)
            | ViewFunctionStatement::Toast(_)
            | ViewFunctionStatement::Redirect { .. } => {}
        }
    }
}
