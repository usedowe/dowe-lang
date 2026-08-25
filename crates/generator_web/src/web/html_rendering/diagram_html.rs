fn render_diagram_html(props: &DiagramProps, context: &ReactiveRenderContext) -> String {
    let mut extra = format!(
        r#" data-dowe-diagram data-dowe-diagram-fit-view="{}" data-dowe-diagram-pan-on-drag="{}" data-dowe-diagram-zoom-on-scroll="{}" data-dowe-diagram-minimap="{}" data-dowe-diagram-controls="{}" data-dowe-diagram-show-grid="{}" data-dowe-diagram-empty-label="{}""#,
        props.fit_view,
        props.pan_on_drag,
        props.zoom_on_scroll,
        props.minimap,
        props.controls,
        props.show_grid,
        escape_attr(&props.empty_label)
    );
    if let Some(nodes) = Some(context.signal_path(&props.nodes)) {
        extra.push_str(&format!(
            r#" data-dowe-diagram-nodes="{}""#,
            escape_attr(&nodes)
        ));
    }
    let edges = context.signal_path(&props.edges);
    extra.push_str(&format!(
        r#" data-dowe-diagram-edges="{}""#,
        escape_attr(&edges)
    ));
    if let Some(action) = props.on_node_click.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-diagram-on-node-click="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if let Some(action) = props.on_node_drag.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-diagram-on-node-drag="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    if let Some(action) = props.on_connect.as_ref() {
        extra.push_str(&format!(
            r#" data-dowe-diagram-on-connect="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    format!(
        r#"<div{}><div class="diagram-canvas"><svg class="diagram-edges-layer" width="100%" height="100%" preserveAspectRatio="none" aria-hidden="true"></svg><div class="diagram-nodes-layer"></div><div class="diagram-empty" hidden>{}</div></div><div class="diagram-controls" aria-hidden="true"><button type="button" class="diagram-control-zoom-in" tabindex="-1">+</button><button type="button" class="diagram-control-zoom-out" tabindex="-1">−</button><button type="button" class="diagram-control-fit" tabindex="-1">⤢</button></div><div class="diagram-minimap" aria-hidden="true"><svg class="diagram-minimap-svg"></svg></div></div>"#,
        attrs(
            diagram_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        escape_html(&props.empty_label)
    )
}
