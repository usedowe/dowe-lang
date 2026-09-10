fn ios_rich_node_references_layout_bindings(node: &ViewNode, bindings: &IosLayoutBindings) -> bool {
    match node {
        ViewNode::ChatBox { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.messages)
                || props
                    .loading
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .sending
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .streaming
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .has_more
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || [
                    props.on_send.as_deref(),
                    props.on_load_more.as_deref(),
                    props.on_stop.as_deref(),
                    props.on_voice_note.as_deref(),
                    props.on_file_attach.as_deref(),
                    props.on_camera_capture.as_deref(),
                ]
                .into_iter()
                .flatten()
                .any(|value| bindings.references_action(value))
        }
        ViewNode::Marquee { props, children } => {
            ios_style_references_layout_bindings(&props.style, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::TypeWriter { props, .. } => {
            ios_style_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::RichText { props, .. } => {
            ios_style_references_layout_bindings(&props.style, bindings)
        }
        ViewNode::Record { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || [
                    props.on_start.as_deref(),
                    props.on_pause.as_deref(),
                    props.on_resume.as_deref(),
                    props.on_stop.as_deref(),
                    props.on_discard.as_deref(),
                    props.on_confirm.as_deref(),
                ]
                .into_iter()
                .flatten()
                .any(|value| bindings.references_action(value))
        }
        ViewNode::ToggleGroup { props, .. } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .value
                    .as_deref()
                    .is_some_and(|value| bindings.references_signal(value))
                || props
                    .on_change
                    .as_deref()
                .is_some_and(|value| bindings.references_action(value))
        }
        ViewNode::Collapsible { props, children } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Countdown { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .on_complete
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
        }
        ViewNode::Map {
            props,
            markers,
            waypoints: _,
        } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || [
                    props.on_location.as_deref(),
                    props.on_location_error.as_deref(),
                    props.on_route.as_deref(),
                ]
                .into_iter()
                .flatten()
                .any(|value| bindings.references_action(value))
                || markers.iter().any(|marker| {
                    marker
                        .on_click
                        .as_deref()
                        .is_some_and(|value| bindings.references_action(value))
                })
        }
        ViewNode::Accordion { props, items } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || items.iter().any(|item| {
                    ios_children_reference_layout_bindings(&item.children, bindings)
                })
        }
        ViewNode::Carousel { props, slides } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || slides.iter().any(|slide| {
                    ios_children_reference_layout_bindings(&slide.children, bindings)
                })
        }
        ViewNode::Tabs { props, tabs } => {
            ios_style_references_layout_bindings(&props.style, bindings)
                || tabs.iter().any(|tab| {
                    ios_children_reference_layout_bindings(&tab.children, bindings)
                })
        }
        ViewNode::NavMenu { props, items } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || ios_nav_menu_items_reference_layout_bindings(items, bindings)
        }
        ViewNode::Each {
            children,
            collection,
            ..
        } => {
            bindings.references_signal(collection)
                || ios_children_reference_layout_bindings(children, bindings)
        }
        ViewNode::Avatar { props, .. } => ios_variant_references_layout_bindings(&props.style, bindings),
        ViewNode::Children => false,
        _ => false
    }
}
