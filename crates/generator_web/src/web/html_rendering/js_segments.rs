fn js_render_expression_with_inspector(
    node: &ViewNode,
    inspector: Option<&ViewInspectorMap>,
) -> String {
    if inspector.is_none() {
        let mut segments = Vec::new();
        collect_js_segments(node, &mut segments, &ReactiveRenderContext::default());
        return if segments.is_empty() {
            "\"\"".to_string()
        } else {
            segments
                .into_iter()
                .map(|segment| match segment {
                    JsSegment::Literal(value) => js_string_literal(&value),
                    JsSegment::Children => "children".to_string(),
                })
                .collect::<Vec<_>>()
                .join("+")
        };
    }
    with_view_inspector(inspector, || {
        let mut segments = Vec::new();
        collect_js_segments(node, &mut segments, &ReactiveRenderContext::default());

        if segments.is_empty() {
            return "\"\"".to_string();
        }

        segments
            .into_iter()
            .map(|segment| match segment {
                JsSegment::Literal(value) => js_string_literal(&value),
                JsSegment::Children => "children".to_string(),
            })
            .collect::<Vec<_>>()
            .join("+")
    })
}

fn collect_js_segments(
    node: &ViewNode,
    segments: &mut Vec<JsSegment>,
    context: &ReactiveRenderContext,
) {
    with_view_inspector_node(|| collect_js_node_segments(node, segments, context));
}

fn collect_js_node_segments(
    node: &ViewNode,
    segments: &mut Vec<JsSegment>,
    context: &ReactiveRenderContext,
) {
    if collect_layout_js_node_segments(node, segments, context)
        || collect_control_js_node_segments(node, segments, context)
        || collect_shell_js_node_segments(node, segments, context)
        || collect_collection_js_node_segments(node, segments, context)
    {
        return;
    }
}
