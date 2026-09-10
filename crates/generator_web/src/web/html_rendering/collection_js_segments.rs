fn collect_collection_js_node_segments(node: &ViewNode, segments: &mut Vec<JsSegment>, context: &ReactiveRenderContext) -> bool {
    match node {
        ViewNode::Each {
            item,
            collection,
            key,
            children,
        } => {
            push_literal(
                segments,
                &format!(
                    r#"<div data-dowe-each="{}" data-dowe-item="{}" data-dowe-key="{}"><template>"#,
                    escape_attr(&context.signal_path(collection)),
                    escape_attr(item),
                    escape_attr(key)
                ),
            );
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</template>");
            if let Some(values) = context.constant_array_values(collection) {
                for (index, value) in values.iter().enumerate() {
                    push_literal(
                        segments,
                        &format!(
                            r#"<div data-dowe-each-row data-dowe-each-index="{index}">"#
                        ),
                    );
                    let row_context = context.with_scope_value(item, value);
                    for child in children {
                        collect_js_segments(child, segments, &row_context);
                    }
                    push_literal(segments, "</div>");
                }
            }
            push_literal(segments, "</div>");
        }
        ViewNode::Children => segments.push(JsSegment::Children),
        _ => return false,
    }
    true
}
