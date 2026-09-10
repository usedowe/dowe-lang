fn compose_fixed_fab_splash_condition(root: &ViewNode, target: &ViewNode) -> Option<String> {
    fn find(
        root: &ViewNode,
        node: &ViewNode,
        target: &ViewNode,
        conditions: &mut Vec<(String, bool)>,
    ) -> bool {
        if std::ptr::eq(node, target) {
            return true;
        }
        if let ViewNode::Splash {
            binding,
            content,
            children,
            ..
        } = node
        {
            let context = compose_reactive_context_for_node(root, node).unwrap_or_default();
            let binding = context.signal_path(binding);
            conditions.push((binding.clone(), false));
            if content
                .iter()
                .any(|child| find(root, child, target, conditions))
            {
                return true;
            }
            conditions.pop();
            conditions.push((binding, true));
            if children
                .iter()
                .any(|child| find(root, child, target, conditions))
            {
                return true;
            }
            conditions.pop();
            return false;
        }
        for group in node_child_groups(node) {
            for child in group {
                if find(root, child, target, conditions) {
                    return true;
                }
            }
        }
        false
    }

    let mut conditions = Vec::new();
    find(root, root, target, &mut conditions);
    (!conditions.is_empty()).then(|| {
        conditions
            .iter()
            .map(|(binding, active)| {
                if *active {
                    format!("state.bool(\"{}\")", escape_kotlin(binding))
                } else {
                    format!("!state.bool(\"{}\")", escape_kotlin(binding))
                }
            })
            .collect::<Vec<_>>()
            .join(" && ")
    })
}

fn compose_tree_has_persistent_scaffold_app_bar(node: &ViewNode) -> bool {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => content
            .iter()
            .chain(children)
            .any(compose_tree_has_persistent_scaffold_app_bar),
        ViewNode::Scope { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. } => children
            .iter()
            .any(compose_tree_has_persistent_scaffold_app_bar),
        ViewNode::Scaffold { app_bar, .. } => app_bar.iter().any(|node| {
            matches!(node, ViewNode::AppBar { props, .. } if props.position != BarPosition::Static)
        }),
        _ => false,
    }
}

