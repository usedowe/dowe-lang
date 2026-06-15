fn render_compose_form_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    match node {
        ViewNode::ToggleTheme { props } => {
            render_compose_theme_toggle(props, indent, output);
        }
        ViewNode::Fab { props, actions } => {
            render_compose_fab(props, actions, indent, output, context);
        }
        ViewNode::Input { props } => {
            let (value, change) = props
                .element
                .bind
                .as_deref()
                .map(|path| {
                    let path = escape_kotlin(&context.signal_path(path));
                    (
                        format!("state.text(\"{path}\")"),
                        format!("{{ state.write(\"{path}\", it) }}"),
                    )
                })
                .unwrap_or_else(|| ("\"\"".to_string(), "{}".to_string()));
            let size = compose_text_size_expr(false, INPUT_TEXT_SIZE);
            let border =
                if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                    color_ref(ColorToken::Muted)
                } else {
                    "null"
                };
            let modifier = if flow == ComposeFlow::Inline && props.style.sizing.w.is_none() {
                format!("{}.weight(1f)", modifier_for_style(&props.style))
            } else {
                modifier_for_style(&props.style)
            };
            output.push_str(&format!(
                        "{pad}DoweInput(value = {value}, onValueChange = {change}, modifier = {}, label = {}, placeholder = {}, floating = {}, fontFamily = {}, fontSize = {size}, lineHeight = doweTextLineHeight({size}, {}f), minHeight = {}.dp, horizontalPadding = {}.dp, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
                        modifier,
                        compose_optional_string(props.label.as_deref()),
                        compose_string_literal(props.placeholder.as_deref().unwrap_or_default()),
                        props.label_floating,
                        compose_font_value(props.style.font.as_ref().or(inherited_font), default_family),
                        text_typography(false, INPUT_TEXT_SIZE).line_height,
                        INPUT_MIN_HEIGHT.native_units(),
                        INPUT_HORIZONTAL_PADDING.native_units(),
                        compose_control_radius(&props.style),
                        variant_container(props),
                        variant_content(props)
                    ));
        }
        ViewNode::Slider { props } => {
            render_compose_slider(props, indent, output, context);
        }
        ViewNode::Dropzone { props } => {
            render_compose_dropzone(props, indent, output);
        }
        ViewNode::Select { props, options } => {
            let (value, change, bound) = props
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
                .unwrap_or_else(|| ("\"\"".to_string(), "{}".to_string(), "false"));
            let size = compose_text_size_expr(false, INPUT_TEXT_SIZE);
            let border =
                if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                    color_ref(ColorToken::Muted)
                } else {
                    "null"
                };
            let modifier = if flow == ComposeFlow::Inline && props.style.sizing.w.is_none() {
                format!("{}.weight(1f)", modifier_for_style(&props.style))
            } else {
                modifier_for_style(&props.style)
            };
            output.push_str(&format!(
                        "{pad}DoweSelect(value = {value}, onValueChange = {change}, bound = {bound}, modifier = {}, label = {}, placeholder = {}, floating = {}, options = {}, fontFamily = {}, fontSize = {size}, lineHeight = doweTextLineHeight({size}, {}f), minHeight = {}.dp, horizontalPadding = {}.dp, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
                        modifier,
                        compose_optional_string(props.label.as_deref()),
                        compose_string_literal(props.placeholder.as_deref().unwrap_or("Select an option")),
                        props.label_floating,
                        compose_select_options(options),
                        compose_font_value(props.style.font.as_ref().or(inherited_font), default_family),
                        text_typography(false, INPUT_TEXT_SIZE).line_height,
                        INPUT_MIN_HEIGHT.native_units(),
                        INPUT_HORIZONTAL_PADDING.native_units(),
                        compose_control_radius(&props.style),
                        variant_container(props),
                        variant_content(props)
                    ));
        }
        ViewNode::ComboBox { props, options } => {
            render_compose_combo_box(props, options, indent, output, flow, inherited_font, default_family, context);
        }
        ViewNode::CsvField { props, columns } => {
            output.push_str(&format!(
                "{pad}DoweCsvField(label = {}, buttonText = {}, modalTitle = {}, instructions = {}, columns = {}, modifier = {}, backgroundColor = {}, contentColor = {})\n",
                compose_optional_string(props.style.label.as_deref()),
                compose_string_literal(&props.button_text),
                compose_string_literal(&props.modal_title),
                compose_string_literal(&props.instructions),
                compose_csv_columns(columns),
                modifier_for_style(&props.style.style),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
        }
        ViewNode::DragDrop {
            props,
            items,
            groups,
        } => {
            output.push_str(&format!(
                "{pad}DoweDragDrop(label = {}, emptyText = {}, direction = {}, items = {}, groups = {}, modifier = {}, backgroundColor = {}, contentColor = {})\n",
                compose_optional_string(props.style.label.as_deref()),
                compose_string_literal(&props.empty_text),
                compose_string_literal(props.direction.as_str()),
                compose_drag_items(items),
                compose_drag_groups(groups),
                modifier_for_style(&props.style.style),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
        }
        ViewNode::Editor { props } => {
            let (value, change) = compose_bound_text(
                props.style.element.bind.as_deref(),
                props.value.as_deref().unwrap_or_default(),
                context,
            );
            output.push_str(&format!(
                "{pad}DoweEditorField(value = {value}, onValueChange = {change}, label = {}, placeholder = {}, minHeight = {}.dp, hideToolbar = {}, readOnly = {}, modifier = {}, backgroundColor = {}, contentColor = {})\n",
                compose_optional_string(props.style.label.as_deref()),
                compose_string_literal(props.style.placeholder.as_deref().unwrap_or_default()),
                props.min_height,
                props.hide_toolbar,
                props.readonly || props.disabled,
                modifier_for_style(&props.style.style),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
        }
        ViewNode::ImageCropper { props } => {
            let (value, change) = compose_bound_text(
                props.style.element.bind.as_deref(),
                props.src.as_deref().unwrap_or_default(),
                context,
            );
            output.push_str(&format!(
                "{pad}DoweImageCropper(value = {value}, onValueChange = {change}, label = {}, placeholder = {}, shape = {}, modifier = {}, backgroundColor = {}, contentColor = {})\n",
                compose_optional_string(props.style.label.as_deref()),
                compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Upload")),
                compose_string_literal(props.shape.as_str()),
                modifier_for_style(&props.style.style),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
        }
        ViewNode::PasswordField { props } => {
            let (value, change) = compose_bound_text(
                props.style.element.bind.as_deref(),
                props.value.as_deref().unwrap_or_default(),
                context,
            );
            output.push_str(&format!(
                "{pad}DowePasswordField(value = {value}, onValueChange = {change}, label = {}, placeholder = {}, floating = {}, hideStrength = {}, weakLabel = {}, mediumLabel = {}, strongLabel = {}, readOnly = {}, modifier = {}, backgroundColor = {}, contentColor = {})\n",
                compose_optional_string(props.style.label.as_deref()),
                compose_string_literal(props.style.placeholder.as_deref().unwrap_or_default()),
                props.style.label_floating,
                props.hide_strength,
                compose_string_literal(&props.weak_label),
                compose_string_literal(&props.medium_label),
                compose_string_literal(&props.strong_label),
                props.readonly || props.disabled,
                modifier_for_style(&props.style.style),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
        }
        ViewNode::PhoneField { props } => {
            let (value, change) = compose_bound_text(
                props.style.element.bind.as_deref(),
                props.value.as_deref().unwrap_or_default(),
                context,
            );
            output.push_str(&format!(
                "{pad}DowePhoneField(value = {value}, onValueChange = {change}, label = {}, placeholder = {}, country = {}, floating = {}, modifier = {}, backgroundColor = {}, contentColor = {})\n",
                compose_optional_string(props.style.label.as_deref()),
                compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Enter phone number")),
                compose_string_literal(props.country.as_deref().unwrap_or("US")),
                props.style.label_floating,
                modifier_for_style(&props.style.style),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
        }
        ViewNode::PinField { props } => {
            let (value, change) = compose_bound_text(
                props.style.element.bind.as_deref(),
                props.value.as_deref().unwrap_or_default(),
                context,
            );
            output.push_str(&format!(
                "{pad}DowePinField(value = {value}, onValueChange = {change}, label = {}, length = {}, kind = {}, modifier = {}, backgroundColor = {}, contentColor = {})\n",
                compose_optional_string(props.style.label.as_deref()),
                props.length,
                compose_string_literal(props.kind.as_str()),
                modifier_for_style(&props.style.style),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
        }
        ViewNode::Textarea { props } => {
            let (value, change) = compose_bound_text(
                props.style.element.bind.as_deref(),
                props.value.as_deref().unwrap_or_default(),
                context,
            );
            output.push_str(&format!(
                "{pad}DoweTextarea(value = {value}, onValueChange = {change}, label = {}, placeholder = {}, floating = {}, rows = {}, maxLength = {}, readOnly = {}, modifier = {}, backgroundColor = {}, contentColor = {})\n",
                compose_optional_string(props.style.label.as_deref()),
                compose_string_literal(props.style.placeholder.as_deref().unwrap_or_default()),
                props.style.label_floating,
                props.rows,
                props.max_length
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "null".to_string()),
                props.readonly || props.disabled,
                modifier_for_style(&props.style.style),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
        }
        ViewNode::Checkbox { props } => {
            render_compose_checkbox(props, indent, output, context);
        }
        ViewNode::Color { props } => {
            render_compose_color(props, indent, output, context);
        }
        ViewNode::Date { props } => {
            render_compose_date(props, indent, output, context);
        }
        ViewNode::DateRange { props } => {
            render_compose_date_range(props, indent, output, context);
        }
        ViewNode::RadioGroup { props, options } => {
            render_compose_radio_group(props, options, indent, output, context)
        }
        ViewNode::Toggle { props } => {
            render_compose_toggle(props, indent, output, context);
        }
        _ => {}
    }
}

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
    let size = compose_text_size_expr(false, INPUT_TEXT_SIZE);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
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
        "{pad}DoweComboBox(value = {value}, onValueChange = {change}, bound = {bound}, label = {}, placeholder = {}, floating = {}, searchPlaceholder = {}, emptyText = {}, clearable = {}, options = {}, modifier = {}, fontFamily = {}, fontSize = {size}, lineHeight = doweTextLineHeight({size}, {}f), minHeight = {}.dp, horizontalPadding = {}.dp, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        compose_optional_string(props.style.label.as_deref()),
        compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Select an option")),
        props.style.label_floating,
        compose_string_literal(&props.search_placeholder),
        compose_string_literal(&props.empty_text),
        props.clearable,
        compose_combo_options(options),
        modifier,
        compose_font_value(props.style.style.font.as_ref().or(inherited_font), default_family),
        text_typography(false, INPUT_TEXT_SIZE).line_height,
        INPUT_MIN_HEIGHT.native_units(),
        INPUT_HORIZONTAL_PADDING.native_units(),
        compose_control_radius(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style)
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
                "DoweSelectOption({}, {}, {})",
                compose_string_literal(&option.value),
                compose_string_literal(&option.label),
                compose_optional_string(option.description.as_deref())
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
