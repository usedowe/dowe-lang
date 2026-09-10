fn ios_structure_node_references_layout_bindings(node: &ViewNode, bindings: &IosLayoutBindings) -> bool {
    match node {
        ViewNode::Splash {
            binding,
            content,
            children,
            ..
        } => {
            bindings.references_signal(binding)
                || ios_children_reference_layout_bindings(content, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Scope {
            constants,
            signals,
            actions,
            children,
            ..
        } => {
            let bindings = bindings.with_scope_shadowed(constants, signals, actions);
            actions
                .iter()
                .any(|action| ios_action_references_layout_bindings(action, &bindings))
                || ios_children_reference_layout_bindings(children, &bindings)
        }
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => {
            ios_style_references_layout_bindings(props, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Flex { props, children } => {
            ios_layout_references_layout_bindings(props, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Grid { props, children } => {
            ios_grid_references_layout_bindings(props, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Brand { props, children } => {
            ios_style_references_layout_bindings(&props.style, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Banner { props, children } => {
            ios_style_references_layout_bindings(&props.style, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Card { props, children } | ViewNode::Button { props, children } => {
            ios_variant_references_layout_bindings(props, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Text { props, value } | ViewNode::Title { props, value } => {
            ios_text_references_layout_bindings(props, value, bindings)
        }
        ViewNode::Input { props } | ViewNode::Select { props, .. } => {
            ios_variant_references_layout_bindings(props, bindings)
        }
        _ => false
    }
}
