fn render_csv_field_html(
    props: &CsvFieldProps,
    columns: &[CsvColumn],
    context: &ReactiveRenderContext,
) -> String {
    let source = format!(
        "{}:{}:{}",
        props.button_text,
        props.modal_title,
        columns
            .iter()
            .map(|column| column.name.as_str())
            .collect::<Vec<_>>()
            .join(",")
    );
    let uid = short_id("csv", &source);
    let mut button_classes = variant_classes("button", &props.style);
    button_classes.push(format!(
        "button-{}",
        props.style.size.unwrap_or(ButtonSize::Md).as_str()
    ));
    button_classes.push("csv-field-button".to_string());
    let columns_html = columns
        .iter()
        .map(|column| {
            format!(
                r#"<div class="csv-field-column" data-dowe-csv-column="{}"><span>{}</span><select class="csv-field-select" data-dowe-csv-select><option value="">{}</option></select></div>"#,
                escape_attr(&column.name),
                escape_html(column.label.as_deref().unwrap_or(&column.name)),
                escape_html(column.label.as_deref().unwrap_or(&column.name))
            )
        })
        .collect::<String>();
    let preview = if props.show_preview {
        format!(
            r#"<div class="csv-field-preview" data-dowe-csv-preview hidden><div class="csv-field-preview-title">{}</div><div class="csv-field-preview-table" data-dowe-csv-table></div></div>"#,
            escape_html(&props.preview_title)
        )
    } else {
        String::new()
    };
    let extra = format!(
        r#" data-dowe-csv data-dowe-csv-preview-rows="{}" data-dowe-csv-preview-page-size="{}""#,
        props.preview_rows, props.preview_page_size
    );
    let field = format!(
        r#"<div{}><input id="{uid}" class="csv-field-input" type="file" accept=".csv,text/csv"{} hidden><button{} type="button" data-dowe-csv-trigger>{}{}</button><div class="csv-field-summary" data-dowe-csv-summary hidden></div>{preview}<div class="csv-field-modal" data-dowe-csv-modal hidden><div class="csv-field-dialog"><h2 class="csv-field-title">{}</h2><p class="csv-field-instructions">{}</p><div class="csv-field-columns">{columns_html}</div><div class="csv-field-error" data-dowe-csv-error{}>{}</div><div class="csv-field-actions"><button class="csv-field-action" type="button" data-dowe-csv-cancel>{}</button><button class="csv-field-action is-primary" type="button" data-dowe-csv-confirm>{}</button><button class="csv-field-action" type="button" data-dowe-csv-clear>{}</button></div></div></div></div>"#,
        attrs(
            vec!["csv-field".to_string()],
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        if props.multiple { " multiple" } else { "" },
        class_attr(button_classes),
        view_icon_svg(ViewIcon::Upload, "csv-field-icon"),
        escape_html(&props.button_text),
        escape_html(&props.modal_title),
        escape_html(&props.instructions),
        if props.error_text.is_some() {
            ""
        } else {
            " hidden"
        },
        escape_html(props.error_text.as_deref().unwrap_or_default()),
        escape_html(&props.cancel_text),
        escape_html(&props.confirm_text),
        escape_html(&props.clear_text)
    );
    render_field_block(
        &props.style,
        None,
        props.error_text.as_deref(),
        &field,
        context,
    )
}

