fn render_drag_drop_list(
    id: &str,
    title: Option<&str>,
    items: &[DragItem],
    empty_text: &str,
) -> String {
    let title = title
        .map(|title| {
            format!(
                r#"<div class="drag-drop-group-title">{}</div>"#,
                escape_html(title)
            )
        })
        .unwrap_or_default();
    let mut html = format!(
        r#"<div class="drag-drop-group" data-dowe-drag-group="{}">{title}<div class="drag-drop-list">"#,
        escape_attr(id)
    );
    for item in items {
        html.push_str(&render_drag_item_html(item));
    }
    if items.is_empty() {
        html.push_str(&format!(
            r#"<div class="drag-drop-empty">{}</div>"#,
            escape_html(empty_text)
        ));
    }
    html.push_str("</div></div>");
    html
}

