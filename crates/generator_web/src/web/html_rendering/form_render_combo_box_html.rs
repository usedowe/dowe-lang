fn render_combo_box_html(
    props: &ComboBoxProps,
    options: &[ComboOption],
    context: &ReactiveRenderContext,
) -> String {
    let mut classes = variant_classes("control", &props.style);
    classes.push("combo-box-control".to_string());
    classes.push(format!(
        "is-{}",
        props.style.size.unwrap_or(ButtonSize::Md).as_str()
    ));
    if props.style.label_floating {
        classes.push("is-floating".to_string());
    }
    if props.error_text.is_some() {
        classes.push("is-error".to_string());
    }
    let selected = props
        .value
        .as_deref()
        .and_then(|value| options.iter().find(|option| option.value == value));
    let label = selected
        .map(|option| option.label.as_str())
        .unwrap_or_else(|| {
            props
                .style
                .placeholder
                .as_deref()
                .unwrap_or("Select an option")
        });
    let value = selected
        .map(|option| option.value.as_str())
        .unwrap_or_default();
    let clear = if props.clearable {
        format!(
            r#"<button class="combo-box-clear" type="button" aria-label="Clear selection" data-dowe-combo-clear{}>&times;</button>"#,
            if props.disabled { " disabled" } else { "" }
        )
    } else {
        String::new()
    };
    let hidden = props
        .name
        .as_deref()
        .map(|name| {
            format!(
                r#"<input type="hidden" name="{}" value="{}" data-dowe-combo-hidden>"#,
                escape_attr(name),
                escape_attr(value)
            )
        })
        .unwrap_or_default();
    let extra = format!(
        r#" type="button" role="combobox" aria-haspopup="listbox" aria-expanded="false" data-dowe-combo-box data-dowe-placeholder="{}" data-dowe-value="{}" data-dowe-empty-text="{}" data-dowe-loading-text="{}" data-dowe-loading-more-text="{}"{}{}"#,
        escape_attr(
            props
                .style
                .placeholder
                .as_deref()
                .unwrap_or("Select an option")
        ),
        escape_attr(value),
        escape_attr(&props.empty_text),
        escape_attr(&props.loading_text),
        escape_attr(&props.loading_more_text),
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.disabled { " disabled" } else { "" }
    );
    let options_html = options
        .iter()
        .map(render_combo_option_html)
        .collect::<String>();
    let control = format!(
        r#"<div class="combo-box">{hidden}<button{}>{}<span class="combo-box-value">{}</span>{}</button>{clear}<div class="combo-box-popover" role="listbox" data-dowe-combo-popover hidden><div class="combo-box-search-wrap">{}<input class="combo-box-search" type="search" placeholder="{}" data-dowe-combo-search></div><div class="combo-box-options">{options_html}</div><div class="combo-box-empty" hidden>{}</div><div class="combo-box-loading" hidden>{}</div></div></div>"#,
        attrs(classes, Some(&props.style.element), Some(&extra), context),
        floating_label_html(&props.style),
        escape_html(label),
        select_arrow_svg(),
        view_icon_svg(ViewIcon::Search, "combo-box-search-icon"),
        escape_attr(&props.search_placeholder),
        escape_html(&props.empty_text),
        escape_html(&props.loading_text)
    );
    render_field_block(
        &props.style,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &control,
        context,
    )
}

