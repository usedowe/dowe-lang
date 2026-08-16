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
        ViewNode::SelectTheme { props } => {
            render_compose_theme_select(props, indent, output, flow);
        }
        ViewNode::Fab { props, actions } => {
            render_compose_fab(props, actions, indent, output, context, None);
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
            let control_size = props.size.unwrap_or(ButtonSize::Md);
            let text_size = form_control_text_size(control_size);
            let size = compose_text_size_expr(false, text_size);
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
                        "{pad}DoweInput(value = {value}, onValueChange = {change}, modifier = {}, label = {}, placeholder = {}, floating = {}, fontFamily = {}, fontSize = {size}, lineHeight = doweTextLineHeight({size}, {}f), minHeight = {}.dp, horizontalPadding = {}.dp, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border}, startIcon = {}, endIcon = {}, helpText = {}, errorText = {}, validationRules = {})\n",
                        modifier,
                        compose_optional_string(props.label.as_deref()),
                        compose_string_literal(props.placeholder.as_deref().unwrap_or_default()),
                        props.label_floating,
                        compose_font_value(props.style.font.as_ref().or(inherited_font), default_family),
                        text_typography(false, text_size).line_height,
                        form_control_min_height(control_size, props.label_floating)
                        .native_units(),
                        INPUT_HORIZONTAL_PADDING.native_units(),
                        compose_control_radius(&props.style),
                        variant_container(props),
                        variant_content(props),
                        compose_control_icon(props.icon_start.as_ref()),
                        compose_control_icon(props.icon_end.as_ref()),
                        compose_validation_help(&props.element),
                        compose_validation_error(&props.element),
                        compose_validation_rules(&props.element, context)
                    ));
        }
        ViewNode::Slider { props } => {
            render_compose_slider(props, indent, output, context);
        }
        ViewNode::Dropzone { props } => {
            render_compose_dropzone(props, indent, output);
        }
        ViewNode::Select {
            props,
            options,
            option_each,
        } => {
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
            let control_size = props.size.unwrap_or(ButtonSize::Md);
            let text_size = form_control_text_size(control_size);
            let size = compose_text_size_expr(false, text_size);
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
                        "{pad}DoweSelect(value = {value}, onValueChange = {change}, bound = {bound}, modifier = {}, label = {}, placeholder = {}, floating = {}, options = {}, fontFamily = {}, fontSize = {size}, lineHeight = doweTextLineHeight({size}, {}f), minHeight = {}.dp, horizontalPadding = {}.dp, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border}, helpText = {}, errorText = {}, validationRules = {})\n",
                        modifier,
                        compose_optional_string(props.label.as_deref()),
                        compose_string_literal(props.placeholder.as_deref().unwrap_or("Select an option")),
                        props.label_floating,
                        compose_select_options(options, option_each.as_ref(), context),
                        compose_font_value(props.style.font.as_ref().or(inherited_font), default_family),
                        text_typography(false, text_size).line_height,
                        form_control_min_height(control_size, props.label_floating)
                        .native_units(),
                        INPUT_HORIZONTAL_PADDING.native_units(),
                        compose_control_radius(&props.style),
                        variant_container(props),
                        variant_content(props),
                        compose_validation_help(&props.element),
                        compose_validation_error(&props.element),
                        compose_validation_rules(&props.element, context)
                    ));
        }
        ViewNode::ComboBox { props, options } => {
            render_compose_combo_box(
                props,
                options,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
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
                "{pad}DoweImageCropper(value = {value}, onValueChange = {change}, bound = {}, initialValue = {}, label = {}, placeholder = {}, alt = {}, accept = {}, aspectRatio = {}, minWidth = {}, minHeight = {}, maxWidth = {}, maxHeight = {}, shape = {}, size = {}, disabled = {}, helpText = {}, errorText = {}, modifier = {}, backgroundColor = {}, contentColor = {})\n",
                props.style.element.bind.is_some(),
                compose_string_literal(props.src.as_deref().unwrap_or_default()),
                compose_optional_string(props.style.label.as_deref()),
                compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Upload")),
                compose_string_literal(&props.alt),
                compose_string_literal(&props.accept),
                props
                    .aspect_ratio
                    .as_deref()
                    .map(compose_string_literal)
                    .unwrap_or_else(|| "null".to_string()),
                props.min_width,
                props.min_height,
                compose_optional_u16(props.max_width),
                compose_optional_u16(props.max_height),
                compose_string_literal(props.shape.as_str()),
                compose_string_literal(props.style.size.unwrap_or(ButtonSize::Md).as_str()),
                props.disabled,
                compose_optional_string(props.help_text.as_deref()),
                compose_optional_string(props.error_text.as_deref()),
                modifier_for_style(&props.style.style),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
        }
        ViewNode::Password { props } => {
            let show_icon = solar_control_icon("eye").expect("bundled Password reveal icon");
            let hide_icon =
                solar_control_icon("eye-closed").expect("bundled Password conceal icon");
            let (value, change) = compose_bound_text(
                props.style.element.bind.as_deref(),
                props.value.as_deref().unwrap_or_default(),
                context,
            );
            let control_size = props.style.size.unwrap_or(ButtonSize::Md);
            let text_size = form_control_text_size(control_size);
            let font_size = compose_text_size_expr(false, text_size);
            output.push_str(&format!(
                "{pad}DowePassword(value = {value}, onValueChange = {change}, label = {}, placeholder = {}, floating = {}, minHeight = {}.dp, fontSize = {font_size}, lineHeight = doweTextLineHeight({font_size}, {}f), hideStrength = {}, weakLabel = {}, mediumLabel = {}, strongLabel = {}, readOnly = {}, showIcon = {}, hideIcon = {}, modifier = {}, backgroundColor = {}, contentColor = {})\n",
                compose_optional_string(props.style.label.as_deref()),
                compose_string_literal(props.style.placeholder.as_deref().unwrap_or_default()),
                props.style.label_floating,
                form_control_min_height(control_size, props.style.label_floating)
                .native_units(),
                text_typography(false, text_size).line_height,
                props.hide_strength,
                compose_string_literal(&props.weak_label),
                compose_string_literal(&props.medium_label),
                compose_string_literal(&props.strong_label),
                props.readonly || props.disabled,
                compose_password_icon(&show_icon),
                compose_password_icon(&hide_icon),
                modifier_for_style(&props.style.style),
                variant_container(&props.style),
                variant_content(&props.style)
            ));
        }
        ViewNode::Phone { props } => {
            let (value, change) = compose_bound_text(
                props.style.element.bind.as_deref(),
                props.value.as_deref().unwrap_or_default(),
                context,
            );
            let control_size = props.style.size.unwrap_or(ButtonSize::Md);
            let text_size = form_control_text_size(control_size);
            let font_size = compose_text_size_expr(false, text_size);
            output.push_str(&format!(
                "{pad}DowePhone(value = {value}, onValueChange = {change}, label = {}, placeholder = {}, country = {}, countries = {}, priorityCountries = {}, searchPlaceholder = {}, emptyText = {}, loadingText = {}, floating = {}, minHeight = {}.dp, fontSize = {font_size}, lineHeight = doweTextLineHeight({font_size}, {}f), disabled = {}, modifier = {}, backgroundColor = {}, contentColor = {}, helpText = {}, errorText = {}, validationRules = {})\n",
                compose_optional_string(props.style.label.as_deref()),
                compose_string_literal(props.style.placeholder.as_deref().unwrap_or("Enter phone number")),
                compose_string_literal(props.country.as_deref().unwrap_or("US")),
                "dowePhoneCountries",
                compose_string_list(&props.priority_countries),
                compose_string_literal(&props.search_placeholder),
                compose_string_literal(&props.empty_text),
                compose_string_literal(&props.loading_text),
                props.style.label_floating,
                form_control_min_height(control_size, props.style.label_floating)
                .native_units(),
                text_typography(false, text_size).line_height,
                props.disabled,
                modifier_for_style(&props.style.style),
                variant_container(&props.style),
                variant_content(&props.style),
                compose_optional_string(props.help_text.as_deref()),
                compose_optional_string(props.error_text.as_deref()),
                compose_validation_rules(&props.style.element, context)
            ));
        }
        ViewNode::Pin { props } => {
            let (value, change) = compose_bound_text(
                props.style.element.bind.as_deref(),
                props.value.as_deref().unwrap_or_default(),
                context,
            );
            let size = props.style.size.unwrap_or(ButtonSize::Md);
            let text_size = form_control_text_size(size);
            let font_size = compose_text_size_expr(false, text_size);
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                color_ref(ColorToken::Muted)
            } else {
                "null"
            };
            output.push_str(&format!(
                "{pad}DowePin(value = {value}, onValueChange = {change}, label = {}, length = {}, kind = {}, size = {}, fontSize = {font_size}, lineHeight = doweTextLineHeight({font_size}, {}f), modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {}, helpText = {}, errorText = {}, validationRules = {})\n",
                compose_optional_string(props.style.label.as_deref()),
                props.length,
                compose_string_literal(props.kind.as_str()),
                compose_string_literal(size.as_str()),
                text_typography(false, text_size).line_height,
                modifier_for_style(&props.style.style),
                compose_control_radius(&props.style.style),
                variant_container(&props.style),
                variant_content(&props.style),
                border,
                compose_optional_string(props.help_text.as_deref()),
                compose_optional_string(props.error_text.as_deref()),
                compose_validation_rules(&props.style.element, context)
            ));
        }
        ViewNode::Textarea { props } => {
            let (value, change) = compose_bound_text(
                props.style.element.bind.as_deref(),
                props.value.as_deref().unwrap_or_default(),
                context,
            );
            let text_size = form_control_text_size(props.style.size.unwrap_or(ButtonSize::Md));
            let font_size = compose_text_size_expr(false, text_size);
            output.push_str(&format!(
                "{pad}DoweTextarea(value = {value}, onValueChange = {change}, label = {}, placeholder = {}, floating = {}, rows = {}, maxLength = {}, fontSize = {font_size}, lineHeight = doweTextLineHeight({font_size}, {}f), readOnly = {}, modifier = {}, backgroundColor = {}, contentColor = {})\n",
                compose_optional_string(props.style.label.as_deref()),
                compose_string_literal(props.style.placeholder.as_deref().unwrap_or_default()),
                props.style.label_floating,
                props.rows,
                props.max_length
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "null".to_string()),
                text_typography(false, text_size).line_height,
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

fn compose_control_icon(icon: Option<&SideNavIcon>) -> String {
    icon.map(|icon| format!(
        "{{ DoweSvg(viewBox = {}, modifier = Modifier.width(24.dp).height(24.dp), color = LocalContentColor.current, paths = {}) }}",
        compose_svg_view_box(&icon.props.view_box),
        compose_svg_paths(&icon.paths)
    ))
    .unwrap_or_else(|| "null".to_string())
}

fn compose_password_icon(icon: &SideNavIcon) -> String {
    format!(
        "{{ DoweSvg(viewBox = {}, modifier = Modifier.width(20.dp).height(20.dp), color = LocalContentColor.current, paths = {}) }}",
        compose_svg_view_box(&icon.props.view_box),
        compose_svg_paths(&icon.paths)
    )
}

fn compose_phone_country_catalog() -> String {
    let countries = phone_countries()
        .iter()
        .filter_map(|country| {
            let icon = phone_country_flag_icon(country.code)?;
            Some(format!(
                "DowePhoneCountry(code = {}, name = {}, dialCode = {}, viewBox = {}, paths = {})",
                compose_string_literal(country.code),
                compose_string_literal(country.name),
                compose_string_literal(country.dial),
                compose_svg_view_box(&icon.props.view_box),
                compose_svg_paths(&icon.paths)
            ))
        })
        .collect::<Vec<_>>();
    let mut output = String::new();
    let mut parts = Vec::new();
    for (index, countries) in countries.chunks(16).enumerate() {
        let name = format!("dowePhoneCountries{index}");
        output.push_str(&format!(
            "private fun {name}(): List<DowePhoneCountry> = listOf({})\n",
            countries.join(", ")
        ));
        parts.push(format!("addAll({name}())"));
    }
    output.push_str(&format!(
        "private val dowePhoneCountries: List<DowePhoneCountry> = buildList {{ {} }}",
        parts.join("; ")
    ));
    output
}

fn compose_string_list(values: &[String]) -> String {
    format!(
        "listOf({})",
        values
            .iter()
            .map(|value| compose_string_literal(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn compose_validation_help(element: &ElementProps) -> String {
    compose_optional_string(
        element
            .form_validation()
            .and_then(|validation| validation.help_text.as_deref()),
    )
}

fn compose_validation_error(element: &ElementProps) -> String {
    compose_optional_string(
        element
            .form_validation()
            .and_then(|validation| validation.error_text.as_deref()),
    )
}

fn compose_validation_rules(element: &ElementProps, context: &ComposeReactiveContext) -> String {
    let Some(validation) = element.form_validation() else {
        return "emptyList()".to_string();
    };
    let values = validation
        .rules
        .iter()
        .map(|rule| {
            let argument = match &rule.kind {
                dowe_components::FormValidationRuleKind::Matches(path) => format!(
                    "state.text(\"{}\")",
                    escape_kotlin(&context.signal_path(path))
                ),
                _ => rule
                    .kind
                    .argument()
                    .as_deref()
                    .map(compose_string_literal)
                    .unwrap_or_else(|| "null".to_string()),
            };
            format!(
                "DoweValidationRule(kind = {}, argument = {argument}, message = {})",
                compose_string_literal(rule.kind.name()),
                compose_string_literal(&rule.message)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_boolean_validation_rules(
    element: &ElementProps,
    context: &ComposeReactiveContext,
) -> String {
    let Some(validation) = element.form_validation() else {
        return "emptyList()".to_string();
    };
    let values = validation
        .rules
        .iter()
        .map(|rule| {
            let argument = match &rule.kind {
                dowe_components::FormValidationRuleKind::Matches(path) => format!(
                    "state.bool(\"{}\").toString()",
                    escape_kotlin(&context.signal_path(path))
                ),
                _ => rule
                    .kind
                    .argument()
                    .as_deref()
                    .map(compose_string_literal)
                    .unwrap_or_else(|| "null".to_string()),
            };
            format!(
                "DoweValidationRule(kind = {}, argument = {argument}, message = {})",
                compose_string_literal(rule.kind.name()),
                compose_string_literal(&rule.message)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
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
