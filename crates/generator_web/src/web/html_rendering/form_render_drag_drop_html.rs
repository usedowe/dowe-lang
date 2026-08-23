fn render_drag_drop_html(
    props: &DragDropProps,
    items: &[DragItem],
    groups: &[DragGroup],
    context: &ReactiveRenderContext,
) -> String {
    let mut classes = variant_classes("drag-drop", &props.style);
    classes.push(format!("is-{}", props.direction.as_str()));
    classes.push(format!("is-{}", props.size.as_str()));
    if props.disabled {
        classes.push("is-disabled".to_string());
    }
    let body = if groups.is_empty() {
        render_drag_drop_list("root", None, items, &props.empty_text)
    } else {
        groups
            .iter()
            .map(|group| {
                render_drag_drop_list(
                    &group.id,
                    group.title.as_deref(),
                    &group.items,
                    &props.empty_text,
                )
            })
            .collect::<String>()
    };
    let extra = format!(
        r#" data-dowe-drag-drop data-dowe-direction="{}" data-dowe-allow-group-transfer="{}""#,
        props.direction.as_str(),
        props.allow_group_transfer
    );
    let surface = format!(
        r#"<div{}>{}</div>"#,
        attrs(classes, Some(&props.style.element), Some(&extra), context),
        body
    );
    render_field_block(&props.style, None, None, &surface, context)
}

