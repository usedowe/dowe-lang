fn collect_layout_js_node_segments(node: &ViewNode, segments: &mut Vec<JsSegment>, context: &ReactiveRenderContext) -> bool {
    match node {
        ViewNode::Scope {
            constants,
            signals,
            actions,
            children,
        } => {
            let context = context.with_scope(constants, signals, actions);
            for child in children {
                collect_js_segments(child, segments, &context);
            }
        }
        ViewNode::Splash {
            binding,
            initial,
            content,
            children,
        } => {
            push_literal(
                segments,
                &format!(
                    r#"<div data-dowe-splash="{}"><div data-dowe-splash-main{}>"#,
                    escape_attr(&context.signal_path(binding)),
                    if *initial { " hidden" } else { "" }
                ),
            );
            for child in content {
                collect_js_segments(child, segments, context);
            }
            push_literal(
                segments,
                if *initial {
                    "</div><div data-dowe-splash-content>"
                } else {
                    "</div><div data-dowe-splash-content hidden>"
                },
            );
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</div></div>");
        }
        ViewNode::Box { props, children } => {
            push_literal(
                segments,
                &format!(
                    "<div{}>",
                    attrs(box_classes(props), Some(&props.element), None, context)
                ),
            );
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</div>");
        }
        ViewNode::Section { props, children } => {
            push_literal(
                segments,
                &format!(
                    "<section{}><div{}>",
                    attrs(section_classes(props), Some(&props.element), None, context),
                    attrs(section_body_classes(props), None, None, context)
                ),
            );
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</div></section>");
        }
        ViewNode::Flex { props, children } => {
            push_literal(
                segments,
                &format!(
                    "<div{}>",
                    attrs(
                        layout_classes("flex", props),
                        Some(&props.style.element),
                        None,
                        context
                    )
                ),
            );
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</div>");
        }
        ViewNode::Grid { props, children } => {
            push_literal(
                segments,
                &format!(
                    "<div{}>",
                    attrs(
                        grid_classes(props),
                        Some(&props.style.element),
                        None,
                        context
                    )
                ),
            );
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</div>");
        }
        ViewNode::Card { props, children } => {
            let mut card_attrs = attrs(
                variant_classes("card", props),
                Some(&props.element),
                None,
                context,
            );
            if let Some(path) = props.reactive.scheme.as_deref() {
                card_attrs.push_str(&format!(
                    r#" data-dowe-variant-binding="true" data-dowe-scheme="{}""#,
                    escape_attr(&context.signal_path(path))
                ));
            }
            push_literal(segments, &format!("<article{}>", card_attrs));
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</article>");
        }
        ViewNode::Tabs { props, tabs } => collect_tabs_js_segments(props, tabs, segments, context),
        _ => return false,
    }
    true
}
