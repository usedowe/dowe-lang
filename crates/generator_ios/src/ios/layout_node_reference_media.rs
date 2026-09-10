fn ios_media_node_references_layout_bindings(node: &ViewNode, bindings: &IosLayoutBindings) -> bool {
    match node {
        ViewNode::Audio { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Image { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .reactive_src
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Camera { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Microphone { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Code { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Video { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Iframe { props } => ios_style_references_layout_bindings(&props.style, bindings),
        ViewNode::Device { props, .. } => ios_style_references_layout_bindings(&props.style, bindings),
        ViewNode::Canvas { props } => {
            ios_style_references_layout_bindings(&props.style, bindings)
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
        }
        ViewNode::Diagram { props } => {
            ios_style_references_layout_bindings(&props.style.style, bindings)
                || bindings.references_signal(&props.nodes)
                || bindings.references_signal(&props.edges)
                || props
                    .on_node_click
                    .iter()
                    .chain(&props.on_node_drag)
                    .chain(&props.on_connect)
                    .any(|value| bindings.references_action(value))
        }
        ViewNode::Checkbox { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::RadioGroup { props, .. } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::Toggle { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::ToggleTheme { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::SelectTheme { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::Empty { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::ComboBox { props, .. } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::CsvField { props, .. } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::DragDrop { props, .. } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::Editor { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::ImageCropper { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .src
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Password { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Phone { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Pin { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Textarea { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::Slider { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.value)
        }
        ViewNode::Dropzone { props } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Color { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.value)
        }
        ViewNode::Date { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        ViewNode::DateRange { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .start
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .end
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
        }
        _ => false
    }
}
