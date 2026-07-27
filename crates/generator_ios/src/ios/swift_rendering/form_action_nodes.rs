fn render_swift_theme_toggle(props: &ThemeToggleProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}Button(action: {{\n{pad}    DoweDesign.applyTheme(DoweDesign.shared.name == \"dark\" ? \"light\" : \"dark\")\n{pad}}}) {{\n{pad}    Image(systemName: DoweDesign.shared.name == \"dark\" ? \"sun.max\" : \"moon.stars\")\n{pad}        .font(.system(size: CGFloat(18), weight: .semibold))\n{pad}}}\n"
    ));
    let mut modifiers = swift_modifiers_for_style(&props.style.style);
    modifiers.push(format!(
        ".background({})",
        card_variant_container(&props.style)
    ));
    modifiers.push(format!(
        ".foregroundStyle({})",
        card_variant_content(&props.style)
    ));
    modifiers.push(".clipShape(Circle())".to_string());
    if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        modifiers.push(format!(
            ".overlay(Circle().stroke({}, lineWidth: CGFloat(1)))",
            card_variant_content(&props.style)
        ));
    }
    modifiers.push(".buttonStyle(.plain)".to_string());
    append_swift_modifiers(output, indent, &modifiers);
}

fn render_swift_theme_select(
    props: &ThemeSelectProps,
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
) {
    let pad = " ".repeat(indent);
    let options = props
        .themes
        .iter()
        .map(|theme| SelectOption {
            value: theme.clone(),
            label: theme_display_label(theme),
            description: None,
        })
        .collect::<Vec<_>>();
    let size = swift_text_size_expr(false, INPUT_TEXT_SIZE);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined)
        == ComponentVariant::Outlined
    {
        format!("Optional({})", card_variant_content(&props.style))
    } else {
        "nil".to_string()
    };
    output.push_str(&format!(
        "{pad}DoweSelectField(value: Optional(Binding(get: {{ DoweDesign.shared.name }}, set: {{ DoweDesign.applyTheme($0) }})), label: Optional({}), placeholder: {}, floating: false, options: {}, font: {}, fontSize: {size}, lineHeight: CGFloat({}), minHeight: CGFloat({}), horizontalPadding: CGFloat({}), backgroundColor: {}, contentColor: {}, borderColor: {border}, radius: {})\n",
        swift_string_literal(&props.label),
        swift_string_literal(&props.placeholder),
        swift_select_options(&options, None, &SwiftReactiveContext::default()),
        swift_font_value(
            props.style.style.font.as_ref().or(inherited_font),
            &size,
            default_family,
        ),
        text_typography(false, INPUT_TEXT_SIZE).line_height,
        INPUT_MIN_HEIGHT.native_units(),
        INPUT_HORIZONTAL_PADDING.native_units(),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
        swift_control_radius(&props.style.style)
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn theme_display_label(value: &str) -> String {
    value
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_swift_fab(
    props: &FabProps,
    actions: &[FabAction],
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
    open_state: Option<&str>,
) {
    let pad = " ".repeat(indent);
    if props.fixed && open_state.is_none() {
        output.push_str(&format!("{pad}EmptyView()\n"));
        return;
    }
    output.push_str(&format!(
        "{pad}VStack(alignment: .trailing, spacing: CGFloat(12)) {{\n"
    ));
    if let Some(open_state) = open_state.filter(|_| !actions.is_empty()) {
        output.push_str(&format!("{pad}    if {open_state} {{\n"));
    }
    let action_indent = if open_state.is_some() && !actions.is_empty() {
        indent + 8
    } else {
        indent + 4
    };
    let action_pad = " ".repeat(action_indent);
    for action in actions {
        let icon = view_icon(action.icon);
        let action_props = VariantProps {
            color: Some(action.color),
            variant: props.style.variant,
            ..VariantProps::default()
        };
        output.push_str(&format!(
            "{action_pad}Button(action: {}) {{\n{action_pad}    HStack(spacing: CGFloat(12)) {{\n{action_pad}        Text({})\n{action_pad}            .font(.system(size: CGFloat(14), weight: .semibold))\n",
            swift_component_action(action.on_click.as_deref(), action.navigation.as_ref(), context),
            swift_string_literal(&action.label),
        ));
        render_swift_button_icon(
            &icon,
            &variant_content(&action_props),
            action_indent + 8,
            output,
        );
        output.push_str(&format!(
            "{action_pad}    }}\n{action_pad}    .padding(.horizontal, CGFloat(12))\n{action_pad}    .padding(.vertical, CGFloat(8))\n{action_pad}    .contentShape(Capsule())\n{action_pad}}}\n{action_pad}.background({})\n{action_pad}.foregroundStyle({})\n{action_pad}.clipShape(Capsule())\n{action_pad}.buttonStyle(.plain)\n",
            variant_container(&action_props),
            variant_content(&action_props)
        ));
    }
    if open_state.is_some() && !actions.is_empty() {
        output.push_str(&format!("{pad}    }}\n"));
    }
    let trigger_icon = view_icon(props.icon);
    let trigger_action = open_state
        .filter(|_| !actions.is_empty())
        .map(|state| format!("{{ withAnimation {{ {state}.toggle() }} }}"))
        .unwrap_or_else(|| {
            swift_component_action(
                props.style.element.on_click.as_deref(),
                props.style.navigation.as_ref(),
                context,
            )
        });
    output.push_str(&format!(
        "{pad}    Button(action: {trigger_action}) {{\n{pad}        ZStack {{\n{pad}            Color.clear\n"
    ));
    render_swift_button_icon(
        &trigger_icon,
        &variant_content(&props.style),
        indent + 12,
        output,
    );
    output.push_str(&format!(
        "{pad}        }}\n{pad}        .frame(maxWidth: .infinity, maxHeight: .infinity)\n{pad}        .contentShape(Circle())\n{pad}    }}\n"
    ));
    let mut trigger_modifiers = swift_modifiers_for_style(&props.style.style);
    trigger_modifiers.push(format!(".background({})", variant_container(&props.style)));
    trigger_modifiers.push(format!(
        ".foregroundStyle({})",
        variant_content(&props.style)
    ));
    trigger_modifiers.push(".clipShape(Circle())".to_string());
    trigger_modifiers.push(".buttonStyle(.plain)".to_string());
    if let Some(open_state) = open_state.filter(|_| !actions.is_empty()) {
        trigger_modifiers.push(format!(
            ".rotationEffect(.degrees({open_state} ? 45 : 0))"
        ));
    }
    append_swift_modifiers(output, indent + 4, &trigger_modifiers);
    output.push_str(&format!("{pad}}}\n"));
    if props.fixed {
        let modifiers = vec![
            ".frame(maxWidth: .infinity, maxHeight: .infinity, alignment: ".to_string()
                + swift_fab_alignment(props.position)
                + ")",
            format!(
                ".padding(.horizontal, {})",
                swift_scale_literal(props.offset_x)
            ),
            format!(
                ".padding(.vertical, {})",
                swift_scale_literal(props.offset_y)
            ),
        ];
        append_swift_modifiers(output, indent, &modifiers);
    } else {
        append_swift_modifiers(
            output,
            indent,
            &swift_modifiers_for_style(&props.style.style),
        );
    }
}

fn render_swift_slider(
    props: &SliderProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let value = props.value.parse::<f64>().unwrap_or(0.0);
    let min = props.min.parse::<f64>().unwrap_or(0.0);
    let max = props.max.parse::<f64>().unwrap_or(100.0);
    let value_literal = swift_double_literal(value);
    let min_literal = swift_double_literal(min);
    let max_literal = swift_double_literal(max);
    let step_literal = props.step.as_deref().map(|step| {
        step.parse::<f64>()
            .map(swift_double_literal)
            .unwrap_or_else(|_| step.to_string())
    });
    let binding = props
        .style
        .element
        .bind
        .as_deref()
        .map(|path| {
            let path = escape_swift(&context.signal_path(path));
            format!(
                "Binding<Double>(get: {{ Double(state.text(\"{path}\")) ?? {value_literal} }}, set: {{ state.write(\"{path}\", value: $0) }})"
            )
        })
        .unwrap_or_else(|| format!("Binding<Double>.constant({value_literal})"));
    output.push_str(&format!(
        "{pad}DoweSliderView(value: {binding}, label: {}, hideLabel: {}, lowerBound: {min_literal}, upperBound: {max_literal}, step: {}, size: {}, accentColor: {})\n",
        swift_optional_literal(props.style.label.as_deref()),
        props.hide_label,
        step_literal
            .map(|step| format!("Optional({step})"))
            .unwrap_or_else(|| "nil".to_string()),
        swift_string_literal(props.size.as_str()),
        swift_scheme_color(&props.style)
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_dropzone(props: &DropzoneProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    let accept = swift_optional_literal(props.accept.as_deref());
    let max_size = props
        .max_size
        .map(|value| format!("Optional(Int64({value}))"))
        .unwrap_or_else(|| "nil".to_string());
    output.push_str(&format!(
        "{pad}DoweDropzone(label: {}, placeholder: {}, accept: {}, multiple: {}, maxSize: {}, disabled: {}, helpText: {}, errorText: {}, size: {}, backgroundColor: {}, contentColor: {}, borderColor: {}, radius: {})\n",
        swift_optional_literal(props.style.label.as_deref()),
        swift_string_literal(
            props
                .style
                .placeholder
                .as_deref()
                .unwrap_or("Drag & drop files here or click to select")
        ),
        accept,
        props.multiple,
        max_size,
        props.disabled,
        swift_optional_literal(props.help_text.as_deref()),
        swift_optional_literal(props.error_text.as_deref()),
        swift_string_literal(props.size.as_str()),
        variant_container(&props.style),
        variant_content(&props.style),
        if props.error_text.is_some() {
            color_ref(ColorToken::Danger).to_string()
        } else {
            variant_content(&props.style).to_string()
        },
        swift_card_radius(&props.style.style)
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn swift_component_action(
    action: Option<&str>,
    navigation: Option<&NavigationAction>,
    context: &SwiftReactiveContext,
) -> String {
    let value = swift_optional_component_action(action, navigation, context);
    if value == "nil" {
        "{}".to_string()
    } else {
        value
    }
}

fn swift_fab_alignment(position: OverlayCornerPosition) -> &'static str {
    match position {
        OverlayCornerPosition::TopLeft => ".topLeading",
        OverlayCornerPosition::TopRight => ".topTrailing",
        OverlayCornerPosition::BottomLeft => ".bottomLeading",
        OverlayCornerPosition::BottomRight => ".bottomTrailing",
    }
}

fn swift_scale_literal(value: ScaleValue) -> String {
    format!("CGFloat({})", value.native_units())
}

fn swift_double_literal(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}
