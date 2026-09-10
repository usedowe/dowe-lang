fn swift_fixed_fab_splash_condition(root: &ViewNode, target: &ViewNode) -> Option<String> {
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
            let context = swift_reactive_context_for_node(root, node).unwrap_or_default();
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
                    format!("state.bool(\"{}\")", escape_swift(binding))
                } else {
                    format!("!state.bool(\"{}\")", escape_swift(binding))
                }
            })
            .collect::<Vec<_>>()
            .join(" && ")
    })
}

fn swift_route_body_nodes(tree: &ViewNode) -> (&[ViewNode], SwiftReactiveContext) {
    let context = SwiftReactiveContext::default();
    match tree {
        ViewNode::Scope {
            constants,
            signals,
            actions,
            children,
        } => (
            children.as_slice(),
            context.with_scope(constants, signals, actions),
        ),
        _ => (std::slice::from_ref(tree), context),
    }
}

fn swift_tree_has_persistent_scaffold_app_bar(node: &ViewNode) -> bool {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => content
            .iter()
            .chain(children)
            .any(swift_tree_has_persistent_scaffold_app_bar),
        ViewNode::Scope { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. } => children
            .iter()
            .any(swift_tree_has_persistent_scaffold_app_bar),
        ViewNode::Scaffold { app_bar, .. } => app_bar.iter().any(|node| {
            matches!(node, ViewNode::AppBar { props, .. } if props.position != BarPosition::Static)
        }),
        _ => false,
    }
}

