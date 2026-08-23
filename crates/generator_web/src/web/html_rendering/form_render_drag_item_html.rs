fn render_drag_item_html(item: &DragItem) -> String {
    let description = item
        .description
        .as_deref()
        .map(|description| {
            format!(
                r#"<span class="drag-drop-item-description">{}</span>"#,
                escape_html(description)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<button class="drag-drop-item" type="button" draggable="true" data-dowe-drag-item="{}"{}><span class="drag-drop-handle">::</span><span class="drag-drop-item-copy"><span class="drag-drop-item-label">{}</span>{description}</span></button>"#,
        escape_attr(&item.id),
        if item.disabled { " disabled" } else { "" },
        escape_html(item.label.as_deref().unwrap_or(&item.id))
    )
}

