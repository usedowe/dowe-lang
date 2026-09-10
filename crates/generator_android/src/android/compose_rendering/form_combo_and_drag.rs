fn render_compose_combo_box(
    props: &ComboBoxProps,
    options: &[ComboOption],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let (value, change, bound) = props
        .style
        .element
        .bind
        .as_deref()
        .map(|path| {
            let path = escape_kotlin(&context.signal_path(path));
            (
                format!("state.text(\"{path}\")"),
                format!("{{ state.write(\"{path}\", it) }}"),
                "true",
            )
        })
        .unwrap_or_else(|| {
            (
                compose_string_literal(props.value.as_deref().unwrap_or_default()),
                "{}".to_string(),
                "false",
            )
        });
    let control_size = props.style.size.unwrap_or(ButtonSize::Md);
    let text_size = form_control_text_size(control_size);
    let size = compose_text_size_expr(false, text_size);
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            color_ref(ColorToken::Muted)
        } else {
            "null"
        };
    let modifier = if flow == ComposeFlow::Inline && props.style.style.sizing.w.is_none() {
        format!("{}.weight(1f)", modifier_for_style(&props.style.style))
    } else {
        modifier_for_style(&props.style.style)
    };
    output.push_str(&format!(
        "{pad}DoweComboBox(value = {value}, onValueChange = {change}, bound = {bound}, label = {}, placeholder = {}, floating = {}, searchPlaceholder = {}, emptyText = {}, loadingText = {}, clearable = {}, disabled = {}, options = {}, modifier = {}, fontFamily = {}, fontSize = {size}, lineHeight = doweTextLineHeight({size}, {}f), minHeight = {}.dp, horizontalPadding = {}.dp, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border}, helpText = {}, errorText = {}, validationRules = {})\n",
        compose_optional_string(props.style.label.as_deref()),
        compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Select an option")),
        props.style.label_floating,
        compose_string_literal(&props.search_placeholder),
        compose_string_literal(&props.empty_text),
        compose_string_literal(&props.loading_text),
        props.clearable,
        props.disabled,
        compose_combo_options(options),
        modifier,
        compose_font_value(props.style.style.font.as_ref().or(inherited_font), default_family),
        text_typography(false, text_size).line_height,
        form_control_min_height(control_size, props.style.label_floating)
        .native_units(),
        INPUT_HORIZONTAL_PADDING.native_units(),
        compose_control_radius(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style),
        compose_validation_help(&props.style.element),
        compose_validation_error(&props.style.element),
        compose_validation_rules(&props.style.element, context)
    ));
}

fn compose_bound_text(
    bind: Option<&str>,
    fallback: &str,
    context: &ComposeReactiveContext,
) -> (String, String) {
    bind.map(|path| {
        let path = escape_kotlin(&context.signal_path(path));
        (
            format!("state.text(\"{path}\")"),
            format!("{{ state.write(\"{path}\", it) }}"),
        )
    })
    .unwrap_or_else(|| (compose_string_literal(fallback), "{}".to_string()))
}

fn compose_combo_options(options: &[ComboOption]) -> String {
    format!(
        "listOf({})",
        options
            .iter()
            .map(|option| format!(
                "DoweComboOption({}, {}, {}, {}, {})",
                compose_string_literal(&option.value),
                compose_string_literal(&option.label),
                compose_optional_string(option.description.as_deref()),
                compose_control_icon(option.icon.as_ref().map(|icon| view_icon(*icon)).as_ref()),
                option.disabled
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn compose_csv_columns(columns: &[CsvColumn]) -> String {
    format!(
        "listOf({})",
        columns
            .iter()
            .map(|column| format!(
                "DoweCsvColumn({}, {})",
                compose_string_literal(&column.name),
                compose_optional_string(column.label.as_deref())
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn compose_drag_items(items: &[DragItem]) -> String {
    format!(
        "listOf({})",
        items
            .iter()
            .map(compose_drag_item)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn compose_drag_groups(groups: &[DragGroup]) -> String {
    format!(
        "listOf({})",
        groups
            .iter()
            .map(|group| format!(
                "DoweDragGroup({}, {}, {})",
                compose_string_literal(&group.id),
                compose_optional_string(group.title.as_deref()),
                compose_drag_items(&group.items)
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn compose_drag_item(item: &DragItem) -> String {
    format!(
        "DoweDragItem({}, {}, {}, {})",
        compose_string_literal(&item.id),
        compose_optional_string(item.label.as_deref()),
        compose_optional_string(item.description.as_deref()),
        item.disabled
    )
}

