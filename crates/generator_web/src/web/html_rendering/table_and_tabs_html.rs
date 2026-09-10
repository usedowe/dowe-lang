
fn render_table_html(props: &TableProps, context: &ReactiveRenderContext) -> String {
    let mut html = format!(
        "<div{}><div class=\"table-container\"><table{}{}><thead class=\"table-header\"><tr>",
        attrs(
            table_wrapper_classes(props),
            Some(&props.style.element),
            None,
            context,
        ),
        class_attr(table_classes(props)),
        table_attrs(props, context),
    );
    for column in &props.columns {
        html.push_str(&format!(
            r#"<th class="table-head" data-dowe-table-field="{}" data-dowe-table-align="{}"{}><div class="table-head-content"><span class="table-head-label">{}</span></div></th>"#,
            escape_attr(&column.field),
            column.align.as_str(),
            table_column_style(column),
            escape_html(&column.label)
        ));
    }
    html.push_str("</tr></thead><tbody class=\"table-body\">");
    html.push_str(&render_table_empty_row(props));
    html.push_str("</tbody></table></div></div>");
    html
}

fn render_tabs_html(
    props: &TabsProps,
    tabs: &[TabItem],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut extra = String::new();
    if let Some(bind) = props.style.element.bind.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-tabs-bind="{}""#,
            escape_attr(&context.signal_path(bind))
        ));
    }
    let mut html = format!(
        "<div{} data-dowe-tabs>",
        attrs(
            tabs_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context
        )
    );
    html.push_str(&format!(
        r#"<div{} role="tablist">"#,
        class_attr(tabs_list_classes(props))
    ));
    for (index, tab) in tabs.iter().enumerate() {
        let active = index == 0;
        let mut classes = vec!["tab".to_string()];
        if active {
            classes.push("on-active".to_string());
        }
        let label = localized_span("tabs-label", &tab.label, tab.i18n.as_deref());
        let content = if props.variant == TabsVariant::Stepper {
            format!(
                r#"<span class="step-indicator" aria-hidden="true">{}</span>{label}"#,
                index + 1
            )
        } else {
            label
        };
        html.push_str(&format!(
            r#"<button{} type="button" role="tab" id="{}" aria-selected="{}" aria-controls="{}" tabindex="{}"{} data-dowe-tab="{}">{}</button>"#,
            class_attr(classes),
            escape_attr(&tab_button_id(tab)),
            if active { "true" } else { "false" },
            escape_attr(&tab_panel_id(tab)),
            if active { "0" } else { "-1" },
            if props.variant == TabsVariant::Stepper && active { r#" aria-current="step""# } else { "" },
            escape_attr(&tab.id),
            content
        ));
    }
    html.push_str("</div><div class=\"tabs-wrapper\">");
    for (index, tab) in tabs.iter().enumerate() {
        let active = index == 0;
        let mut classes = vec!["tabs-content".to_string()];
        if active {
            classes.push("on-active".to_string());
        }
        html.push_str(&format!(
            r#"<div{} id="{}" role="tabpanel" aria-labelledby="{}" data-dowe-tab-panel="{}"{}>"#,
            class_attr(classes),
            escape_attr(&tab_panel_id(tab)),
            escape_attr(&tab_button_id(tab)),
            escape_attr(&tab.id),
            if active { "" } else { " hidden" }
        ));
        for child in &tab.children {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div>");
    }
    html.push_str("</div></div>");
    html
}

fn tab_button_id(tab: &TabItem) -> String {
    format!("tab-{}-button", tab.id)
}

fn tab_panel_id(tab: &TabItem) -> String {
    format!("tab-{}-panel", tab.id)
}

fn table_attrs(props: &TableProps, context: &ReactiveRenderContext) -> String {
    format!(
        r#" data-dowe-table data-dowe-table-data="{}" data-dowe-table-empty-title="{}" data-dowe-table-empty-description="{}""#,
        escape_attr(&context.signal_path(&props.data)),
        escape_attr(&props.empty_title),
        escape_attr(&props.empty_description)
    )
}

fn render_table_empty_row(props: &TableProps) -> String {
    format!(
        r#"<tr class="table-empty-row"><td class="table-empty-cell" colspan="{}"><div class="empty-state"><div class="empty-content"><h3 class="empty-title">{}</h3><p class="empty-description">{}</p></div></div></td></tr>"#,
        props.columns.len(),
        escape_html(&props.empty_title),
        escape_html(&props.empty_description)
    )
}

fn table_column_style(column: &TableColumn) -> String {
    let mut styles = vec![format!("text-align:{}", table_align_css(column.align))];
    if let Some(width) = column.width.as_deref() {
        styles.push(format!("width:{}", escape_attr(width)));
    }
    format!(r#" style="{}""#, styles.join(";"))
}

fn table_align_css(value: TableColumnAlign) -> &'static str {
    match value {
        TableColumnAlign::Start => "start",
        TableColumnAlign::Center => "center",
        TableColumnAlign::End => "end",
    }
}
