fn dev_node_references_forms_and_data(
    node: &ViewNode,
    bindings: &DevLayoutBindings,
) -> Option<bool> {
    match node {
        ViewNode::Diagram { props } => Some({
            dev_style_references_layout_bindings(&props.style.style, bindings)
                || bindings.references_signal(&props.nodes)
                || bindings.references_signal(&props.edges)
                || props
                    .on_node_click
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }),
        ViewNode::Microphone { props } => Some(dev_variant_references_layout_bindings(&props.style, bindings)),
        ViewNode::Code { props } => Some(dev_variant_references_layout_bindings(&props.style, bindings)),
        ViewNode::Video { props } => Some(dev_variant_references_layout_bindings(&props.style, bindings)),
        ViewNode::Iframe { props } => Some(dev_style_references_layout_bindings(&props.style, bindings)),
        ViewNode::Device { props, .. } => Some(dev_style_references_layout_bindings(&props.style, bindings)),
        ViewNode::Canvas { props } => Some({
            dev_style_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.scene)
                || props
                    .layer_bind
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .selected_layer
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || (props.draw_mode_binding && bindings.references_signal(&props.draw_mode))
                || props
                    .on_pointer
                    .iter()
                    .chain(&props.on_key)
                    .chain(&props.on_motion)
                    .chain(&props.on_layer_add)
                    .chain(&props.on_layer_change)
                    .chain(&props.on_layer_remove)
                    .chain(&props.on_layer_select)
                    .any(|value| bindings.references_action(value))
        }),
        ViewNode::Checkbox { props } => Some(dev_variant_references_layout_bindings(&props.style, bindings)),
        ViewNode::RadioGroup { props, .. } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
        }),
        ViewNode::Toggle { props } => Some(dev_variant_references_layout_bindings(&props.style, bindings)),
        ViewNode::ToggleTheme { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
        }),
        ViewNode::SelectTheme { props } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
        }),
        ViewNode::Empty { props } => Some(dev_variant_references_layout_bindings(&props.style, bindings)),
        ViewNode::ComboBox { props, .. } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }),
        ViewNode::CsvField { props, .. } => Some(dev_variant_references_layout_bindings(&props.style, bindings)),
        ViewNode::DragDrop { props, .. } => Some({
            dev_variant_references_layout_bindings(&props.style, bindings)
        }),
        _ => None,
    }
}
