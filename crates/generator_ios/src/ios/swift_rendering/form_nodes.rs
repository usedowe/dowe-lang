fn render_swift_form_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    match node {
        ViewNode::Button { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let action = props
                .element
                .on_click
                .as_deref()
                .and_then(|name| context.action_id(name))
                .map(|id| {
                    let item = context
                        .active_item()
                        .map(|value| format!(", item: {value}"))
                        .unwrap_or_default();
                    format!("{{ state.run(\"{}\"{item}) }}", escape_swift(id))
                })
                .unwrap_or_else(|| swift_navigation_action(props.navigation.as_ref()));
            output.push_str(&format!("{pad}Button(action: {action}) {{\n"));
            output.push_str(&format!("{pad}    HStack(spacing: 0) {{\n"));
            for child in children {
                render_swift_node_in_flow(
                    child,
                    indent + 8,
                    output,
                    NativeFlow::Inline,
                    current_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}    }}\n"));
            output.push_str(&format!("{pad}}}\n"));
            let mut modifiers = swift_modifiers_for_style(&props.style);
            if flow.is_grid_item() && props.style.sizing.w.is_none() {
                modifiers.push(".frame(maxWidth: .infinity, alignment: .center)".to_string());
            }
            modifiers.push(format!(".background({})", variant_container(props)));
            modifiers.push(format!(".foregroundStyle({})", variant_content(props)));
            let radius = swift_control_radius(&props.style);
            modifiers.push(format!(
                ".clipShape(RoundedRectangle(cornerRadius: {radius}))"
            ));
            if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                modifiers.push(format!(
                    ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke({}, lineWidth: CGFloat(1)))",
                    variant_content(props)
                ));
            }
            modifiers.push(".buttonStyle(.plain)".to_string());
            append_swift_modifiers(output, indent, &modifiers);
        }
        ViewNode::ToggleTheme { props } => render_swift_theme_toggle(props, indent, output),
        ViewNode::Fab { props, actions } => {
            render_swift_fab(props, actions, indent, output, context)
        }
        ViewNode::Input { props } => {
            let binding = props
                .element
                .bind
                .as_deref()
                .map(|path| {
                    format!(
                        "state.binding(\"{}\")",
                        escape_swift(&context.signal_path(path))
                    )
                })
                .unwrap_or_else(|| "nil".to_string());
            let size = swift_text_size_expr(false, INPUT_TEXT_SIZE);
            let border =
                if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                    format!("Optional({})", color_ref(ColorToken::Muted))
                } else {
                    "nil".to_string()
                };
            output.push_str(&format!(
                "{pad}DoweInputField(value: {binding}, label: {}, placeholder: {}, floating: {}, font: {}, fontSize: {size}, lineHeight: CGFloat({}), minHeight: CGFloat({}), horizontalPadding: CGFloat({}), backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
                swift_optional_literal(props.label.as_deref()),
                swift_string_literal(props.placeholder.as_deref().unwrap_or_default()),
                props.label_floating,
                swift_font_value(
                    props.style.font.as_ref().or(inherited_font),
                    &size,
                    default_family,
                ),
                text_typography(false, INPUT_TEXT_SIZE).line_height,
                INPUT_MIN_HEIGHT.native_units(),
                INPUT_HORIZONTAL_PADDING.native_units(),
                variant_container(props),
                variant_content(props),
                swift_control_radius(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style));
        }
        ViewNode::Slider { props } => render_swift_slider(props, indent, output, context),
        ViewNode::Dropzone { props } => render_swift_dropzone(props, indent, output),
        ViewNode::Select { props, options } => {
            let binding = props
                .element
                .bind
                .as_deref()
                .map(|path| {
                    format!(
                        "state.binding(\"{}\")",
                        escape_swift(&context.signal_path(path))
                    )
                })
                .unwrap_or_else(|| "nil".to_string());
            let size = swift_text_size_expr(false, INPUT_TEXT_SIZE);
            let border =
                if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                    format!("Optional({})", color_ref(ColorToken::Muted))
                } else {
                    "nil".to_string()
                };
            output.push_str(&format!(
                "{pad}DoweSelectField(value: {binding}, label: {}, placeholder: {}, floating: {}, options: {}, font: {}, fontSize: {size}, lineHeight: CGFloat({}), minHeight: CGFloat({}), horizontalPadding: CGFloat({}), backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
                swift_optional_literal(props.label.as_deref()),
                swift_string_literal(props.placeholder.as_deref().unwrap_or("Select an option")),
                props.label_floating,
                swift_select_options(options),
                swift_font_value(props.style.font.as_ref().or(inherited_font), &size, default_family),
                text_typography(false, INPUT_TEXT_SIZE).line_height,
                INPUT_MIN_HEIGHT.native_units(),
                INPUT_HORIZONTAL_PADDING.native_units(),
                variant_container(props),
                variant_content(props),
                swift_control_radius(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style));
        }
        ViewNode::ComboBox { props, options } => {
            render_swift_combo_box(props, options, indent, output, inherited_font, default_family, context);
        }
        ViewNode::CsvField { props, columns } => {
            output.push_str(&format!(
                "{pad}DoweCsvField(label: {}, buttonText: {}, modalTitle: {}, instructions: {}, columns: {}, backgroundColor: {}, contentColor: {})\n",
                swift_optional_literal(props.style.label.as_deref()),
                swift_string_literal(&props.button_text),
                swift_string_literal(&props.modal_title),
                swift_string_literal(&props.instructions),
                swift_csv_columns(columns),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::DragDrop {
            props,
            items,
            groups,
        } => {
            output.push_str(&format!(
                "{pad}DoweDragDrop(label: {}, emptyText: {}, direction: {}, items: {}, groups: {}, backgroundColor: {}, contentColor: {})\n",
                swift_optional_literal(props.style.label.as_deref()),
                swift_string_literal(&props.empty_text),
                swift_string_literal(props.direction.as_str()),
                swift_drag_items(items),
                swift_drag_groups(groups),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::Editor { props } => {
            output.push_str(&format!(
                "{pad}DoweEditorField(value: {}, initialValue: {}, label: {}, placeholder: {}, minHeight: CGFloat({}), hideToolbar: {}, readOnly: {}, backgroundColor: {}, contentColor: {})\n",
                swift_text_binding(props.style.element.bind.as_deref(), context),
                swift_string_literal(props.value.as_deref().unwrap_or_default()),
                swift_optional_literal(props.style.label.as_deref()),
                swift_string_literal(props.style.placeholder.as_deref().unwrap_or_default()),
                props.min_height,
                props.hide_toolbar,
                props.readonly || props.disabled,
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::ImageCropper { props } => {
            output.push_str(&format!(
                "{pad}DoweImageCropper(value: {}, initialValue: {}, label: {}, placeholder: {}, shape: {}, backgroundColor: {}, contentColor: {})\n",
                swift_text_binding(props.style.element.bind.as_deref(), context),
                swift_string_literal(props.src.as_deref().unwrap_or_default()),
                swift_optional_literal(props.style.label.as_deref()),
                swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Upload")),
                swift_string_literal(props.shape.as_str()),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::PasswordField { props } => {
            output.push_str(&format!(
                "{pad}DowePasswordField(value: {}, initialValue: {}, label: {}, placeholder: {}, floating: {}, hideStrength: {}, weakLabel: {}, mediumLabel: {}, strongLabel: {}, readOnly: {}, backgroundColor: {}, contentColor: {})\n",
                swift_text_binding(props.style.element.bind.as_deref(), context),
                swift_string_literal(props.value.as_deref().unwrap_or_default()),
                swift_optional_literal(props.style.label.as_deref()),
                swift_string_literal(props.style.placeholder.as_deref().unwrap_or_default()),
                props.style.label_floating,
                props.hide_strength,
                swift_string_literal(&props.weak_label),
                swift_string_literal(&props.medium_label),
                swift_string_literal(&props.strong_label),
                props.readonly || props.disabled,
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::PhoneField { props } => {
            output.push_str(&format!(
                "{pad}DowePhoneField(value: {}, initialValue: {}, label: {}, placeholder: {}, country: {}, floating: {}, backgroundColor: {}, contentColor: {})\n",
                swift_text_binding(props.style.element.bind.as_deref(), context),
                swift_string_literal(props.value.as_deref().unwrap_or_default()),
                swift_optional_literal(props.style.label.as_deref()),
                swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Enter phone number")),
                swift_string_literal(props.country.as_deref().unwrap_or("US")),
                props.style.label_floating,
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::PinField { props } => {
            output.push_str(&format!(
                "{pad}DowePinField(value: {}, initialValue: {}, label: {}, length: {}, kind: {}, backgroundColor: {}, contentColor: {})\n",
                swift_text_binding(props.style.element.bind.as_deref(), context),
                swift_string_literal(props.value.as_deref().unwrap_or_default()),
                swift_optional_literal(props.style.label.as_deref()),
                props.length,
                swift_string_literal(props.kind.as_str()),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::Textarea { props } => {
            output.push_str(&format!(
                "{pad}DoweTextarea(value: {}, initialValue: {}, label: {}, placeholder: {}, floating: {}, rows: {}, maxLength: {}, readOnly: {}, backgroundColor: {}, contentColor: {})\n",
                swift_text_binding(props.style.element.bind.as_deref(), context),
                swift_string_literal(props.value.as_deref().unwrap_or_default()),
                swift_optional_literal(props.style.label.as_deref()),
                swift_string_literal(props.style.placeholder.as_deref().unwrap_or_default()),
                props.style.label_floating,
                props.rows,
                props.max_length
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "nil".to_string()),
                props.readonly || props.disabled,
                variant_container(&props.style),
                variant_content(&props.style)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
        }
        ViewNode::Checkbox { props } => render_swift_checkbox(props, indent, output, context),
        ViewNode::Color { props } => render_swift_color(props, indent, output, context),
        ViewNode::Date { props } => render_swift_date(props, indent, output, context),
        ViewNode::DateRange { props } => render_swift_date_range(props, indent, output, context),
        ViewNode::RadioGroup { props, options } => {
            render_swift_radio_group(props, options, indent, output, context)
        }
        ViewNode::Toggle { props } => render_swift_toggle(props, indent, output, context),
        _ => unreachable!(),
    }
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
    let size = swift_text_size_expr(false, INPUT_TEXT_SIZE);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        format!("Optional({})", color_ref(ColorToken::Muted))
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweComboBox(value: {}, initialValue: {}, label: {}, placeholder: {}, floating: {}, searchPlaceholder: {}, emptyText: {}, clearable: {}, options: {}, font: {}, fontSize: {size}, lineHeight: CGFloat({}), minHeight: CGFloat({}), horizontalPadding: CGFloat({}), backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
        swift_text_binding(props.style.element.bind.as_deref(), context),
        swift_string_literal(props.value.as_deref().unwrap_or_default()),
        swift_optional_literal(props.style.label.as_deref()),
        swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Select an option")),
        props.style.label_floating,
        swift_string_literal(&props.search_placeholder),
        swift_string_literal(&props.empty_text),
        props.clearable,
        swift_combo_options(options),
        swift_font_value(props.style.style.font.as_ref().or(inherited_font), &size, default_family),
        text_typography(false, INPUT_TEXT_SIZE).line_height,
        INPUT_MIN_HEIGHT.native_units(),
        INPUT_HORIZONTAL_PADDING.native_units(),
        variant_container(&props.style),
        variant_content(&props.style),
        swift_control_radius(&props.style.style)
    ));
    append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style.style));
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
                "DoweSelectOption(value: {}, label: {}, description: {})",
                swift_string_literal(&option.value),
                swift_string_literal(&option.label),
                swift_optional_literal(option.description.as_deref())
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
