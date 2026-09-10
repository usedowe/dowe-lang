fn dev_node_references_structure_and_media(
    node: &ViewNode,
    bindings: &DevLayoutBindings,
) -> Option<bool> {
    match node {
        ViewNode::Splash {
            binding,
            content,
            children,
            ..
        } => Some({
            bindings.references_signal(binding)
                || dev_children_reference_layout_bindings(content, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }),
        ViewNode::Scope {
            constants,
            signals,
            actions,
            children,
            ..
        } => Some({
            let bindings = bindings.with_scope_shadowed(constants, signals, actions);
            actions
                .iter()
                .any(|action| dev_action_references_layout_bindings(action, &bindings))
                || dev_children_reference_layout_bindings(children, &bindings)
        }),
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => Some({
            dev_style_references_layout_bindings(props, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }),
        ViewNode::Flex { props, children } => Some({
            dev_layout_references_layout_bindings(props, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }),
        ViewNode::Grid { props, children } => Some({
            dev_grid_references_layout_bindings(props, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }),
        ViewNode::Brand { props, children } => Some({
            dev_style_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }),
        ViewNode::Banner { props, children } => Some({
            dev_style_references_layout_bindings(&props.style, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }),
        ViewNode::Card { props, children } | ViewNode::Button { props, children } => Some({
            dev_variant_references_layout_bindings(props, bindings)
                || dev_children_reference_layout_bindings(children, bindings)
        }),
        ViewNode::Text { props, value } | ViewNode::Title { props, value } => Some({
            dev_text_references_layout_bindings(props, value, bindings)
        }),
        ViewNode::Input { props } | ViewNode::Select { props, .. } => Some({
            dev_variant_references_layout_bindings(props, bindings)
        }),
        ViewNode::Audio { props } => Some(dev_variant_references_layout_bindings(&props.style, bindings)),
        ViewNode::Image { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .reactive_src
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }),
        ViewNode::Camera { props } => Some(dev_variant_references_layout_bindings(&props.style, bindings)),
        _ => None,
    }
}
