fn render_swift_form_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    _flow: NativeFlow,
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
