fn swift_control_icon(icon: Option<&SideNavIcon>) -> String {
    icon.map(|icon| {
        format!(
            "DoweControlIcon(viewBox: {}, paths: {})",
            swift_svg_view_box(&icon.props.view_box),
            swift_svg_paths(&icon.paths)
        )
    })
    .unwrap_or_else(|| "nil".to_string())
}

fn swift_string_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| swift_string_literal(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn swift_validation_help(element: &ElementProps) -> String {
    swift_optional_literal(
        element
            .form_validation()
            .and_then(|validation| validation.help_text.as_deref()),
    )
}

fn swift_validation_error(element: &ElementProps) -> String {
    swift_optional_literal(
        element
            .form_validation()
            .and_then(|validation| validation.error_text.as_deref()),
    )
}

fn swift_validation_rules(
    element: &ElementProps,
    context: &SwiftReactiveContext,
    boolean: bool,
) -> String {
    let Some(validation) = element.form_validation() else {
        return "[]".to_string();
    };
    let rules = validation
        .rules
        .iter()
        .map(|rule| {
            let argument = match &rule.kind {
                dowe_components::FormValidationRuleKind::Matches(path) => {
                    let path = escape_swift(&context.signal_path(path));
                    if boolean {
                        format!("String(state.bool(\"{path}\"))")
                    } else {
                        format!("state.text(\"{path}\")")
                    }
                }
                _ => rule
                    .kind
                    .argument()
                    .as_deref()
                    .map(swift_string_literal)
                    .unwrap_or_else(|| "nil".to_string()),
            };
            format!(
                "DoweValidationRule(kind: {}, argument: {argument}, message: {})",
                swift_string_literal(rule.kind.name()),
                swift_string_literal(&rule.message)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{rules}]")
}

fn render_swift_combo_box(
    props: &ComboBoxProps,
    options: &[ComboOption],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let control_size = props.style.size.unwrap_or(ButtonSize::Md);
    let text_size = form_control_text_size(control_size);
    let size = swift_text_size_expr(false, text_size);
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            format!("Optional({})", color_ref(ColorToken::Muted))
        } else {
            "nil".to_string()
        };
    output.push_str(&format!(
        "{pad}DoweComboBox(value: {}, initialValue: {}, label: {}, placeholder: {}, floating: {}, searchPlaceholder: {}, emptyText: {}, loadingText: {}, clearable: {}, disabled: {}, options: {}, font: {}, fontSize: {size}, lineHeight: CGFloat({}), minHeight: CGFloat({}), horizontalPadding: CGFloat({}), backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {}, helpText: {}, errorText: {}, validationRules: {})\n",
        swift_text_binding(props.style.element.bind.as_deref(), context),
        swift_string_literal(props.value.as_deref().unwrap_or_default()),
        swift_optional_literal(props.style.label.as_deref()),
        swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Select an option")),
        props.style.label_floating,
        swift_string_literal(&props.search_placeholder),
        swift_string_literal(&props.empty_text),
        swift_string_literal(&props.loading_text),
        props.clearable,
        props.disabled,
        swift_combo_options(options),
        swift_font_value(props.style.style.font.as_ref().or(inherited_font), &size, default_family),
        text_typography(false, text_size).line_height,
        form_control_min_height(control_size, props.style.label_floating)
        .native_units(),
        INPUT_HORIZONTAL_PADDING.native_units(),
        variant_container(&props.style),
        variant_content(&props.style),
        swift_control_radius(&props.style.style),
        swift_optional_literal(props.help_text.as_deref()),
        swift_optional_literal(props.error_text.as_deref()),
        swift_validation_rules(&props.style.element, context, false)
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn swift_text_binding(bind: Option<&str>, context: &SwiftReactiveContext) -> String {
    bind.map(|path| {
        format!(
            "state.binding(\"{}\")",
            escape_swift(&context.signal_path(path))
        )
    })
    .unwrap_or_else(|| "nil".to_string())
}

fn swift_combo_options(options: &[ComboOption]) -> String {
    format!(
        "[{}]",
        options
            .iter()
            .map(|option| format!(
                "DoweComboOption(value: {}, label: {}, description: {}, icon: {}, disabled: {})",
                swift_string_literal(&option.value),
                swift_string_literal(&option.label),
                swift_optional_literal(option.description.as_deref()),
                swift_control_icon(option.icon.map(view_icon).as_ref()),
                option.disabled
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn swift_csv_columns(columns: &[CsvColumn]) -> String {
    format!(
        "[{}]",
        columns
            .iter()
            .map(|column| format!(
                "DoweCsvColumn(name: {}, label: {})",
                swift_string_literal(&column.name),
                swift_optional_literal(column.label.as_deref())
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn swift_drag_items(items: &[DragItem]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(swift_drag_item)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn swift_drag_groups(groups: &[DragGroup]) -> String {
    format!(
        "[{}]",
        groups
            .iter()
            .map(|group| format!(
                "DoweDragGroup(id: {}, title: {}, items: {})",
                swift_string_literal(&group.id),
                swift_optional_literal(group.title.as_deref()),
                swift_drag_items(&group.items)
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn swift_drag_item(item: &DragItem) -> String {
    format!(
        "DoweDragItem(id: {}, label: {}, description: {}, disabled: {})",
        swift_string_literal(&item.id),
        swift_optional_literal(item.label.as_deref()),
        swift_optional_literal(item.description.as_deref()),
        item.disabled
    )
}
