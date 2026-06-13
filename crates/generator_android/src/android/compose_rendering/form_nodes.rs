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
