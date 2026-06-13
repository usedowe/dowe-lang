fn render_swift_theme_toggle(props: &ThemeToggleProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}Button(action: {{\n{pad}    let current = UserDefaults.standard.string(forKey: \"theme-preference\") ?? \"light\"\n{pad}    UserDefaults.standard.set(current == \"dark\" ? \"light\" : \"dark\", forKey: \"theme-preference\")\n{pad}}}) {{\n{pad}    Image(systemName: \"moon.stars\")\n{pad}        .font(.system(size: CGFloat(18), weight: .semibold))\n{pad}}}\n"
    ));
    let mut modifiers = swift_modifiers_for_style(&props.style.style);
    modifiers.push(format!(".background({})", variant_container(&props.style)));
    modifiers.push(format!(
        ".foregroundStyle({})",
        variant_content(&props.style)
    ));
    modifiers.push(".clipShape(Circle())".to_string());
    if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        modifiers.push(format!(
            ".overlay(Circle().stroke({}, lineWidth: CGFloat(1)))",
            variant_content(&props.style)
        ));
    }
    modifiers.push(".buttonStyle(.plain)".to_string());
    append_swift_modifiers(output, indent, &modifiers);
}

fn render_swift_fab(
    props: &FabProps,
    actions: &[FabAction],
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}VStack(alignment: .trailing, spacing: CGFloat(12)) {{\n"
    ));
    for action in actions {
        output.push_str(&format!(
            "{pad}    Button(action: {}) {{\n{pad}        HStack(spacing: CGFloat(12)) {{\n{pad}            Text({})\n{pad}                .font(.system(size: CGFloat(14), weight: .semibold))\n{pad}            Image(systemName: {})\n{pad}        }}\n{pad}    }}\n{pad}    .padding(.horizontal, CGFloat(12))\n{pad}    .padding(.vertical, CGFloat(8))\n{pad}    .background({})\n{pad}    .foregroundStyle({})\n{pad}    .clipShape(Capsule())\n{pad}    .buttonStyle(.plain)\n",
            swift_component_action(action.on_click.as_deref(), action.navigation.as_ref(), context),
            swift_string_literal(&action.label),
            swift_string_literal(swift_view_icon_system(action.icon)),
            variant_container(&VariantProps {
                color: Some(action.color),
                variant: props.style.variant,
                ..VariantProps::default()
            }),
            variant_content(&VariantProps {
                color: Some(action.color),
                variant: props.style.variant,
                ..VariantProps::default()
            })
        ));
    }
    output.push_str(&format!(
        "{pad}    Button(action: {}) {{\n{pad}        Image(systemName: {})\n{pad}            .font(.system(size: CGFloat(20), weight: .semibold))\n{pad}    }}\n",
        swift_component_action(
            props.style.element.on_click.as_deref(),
            props.style.navigation.as_ref(),
            context,
        ),
        swift_string_literal(swift_view_icon_system(props.icon))
    ));
    let mut trigger_modifiers = swift_modifiers_for_style(&props.style.style);
    trigger_modifiers.push(format!(".background({})", variant_container(&props.style)));
    trigger_modifiers.push(format!(
        ".foregroundStyle({})",
        variant_content(&props.style)
    ));
    trigger_modifiers.push(".clipShape(Circle())".to_string());
    trigger_modifiers.push(".buttonStyle(.plain)".to_string());
    append_swift_modifiers(output, indent + 4, &trigger_modifiers);
    output.push_str(&format!("{pad}}}\n"));
    if props.fixed {
        let mut modifiers = vec![
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
        modifiers.extend(swift_modifiers_for_style(&props.style.style));
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
    let (binding, display_value) = props
        .style
        .element
        .bind
        .as_deref()
        .map(|path| {
            let path = escape_swift(&context.signal_path(path));
            (
                format!(
                    "Binding<Double>(get: {{ Double(state.text(\"{path}\")) ?? {value_literal} }}, set: {{ state.write(\"{path}\", value: $0) }})"
                ),
                format!("Double(state.text(\"{path}\")) ?? {value_literal}"),
            )
        })
        .unwrap_or_else(|| (
            format!("Binding<Double>.constant({value_literal})"),
            value_literal.clone(),
        ));
    output.push_str(&format!("{pad}VStack(spacing: CGFloat(2)) {{\n"));
    if !props.hide_label {
        output.push_str(&format!(
            "{pad}    HStack {{\n{pad}        Text({})\n{pad}        Spacer()\n{pad}        Text(String(format: \"%.0f\", {display_value}))\n{pad}    }}\n{pad}    .font(.system(size: CGFloat(14), weight: .semibold))\n",
            swift_string_literal(props.style.label.as_deref().unwrap_or_default()),
        ));
    }
    let slider = if let Some(step) = step_literal {
        format!("Slider(value: {binding}, in: {min_literal}...{max_literal}, step: {step})")
    } else {
        format!("Slider(value: {binding}, in: {min_literal}...{max_literal})")
    };
    output.push_str(&format!(
        "{pad}    {slider}\n{pad}        .tint({})\n",
        swift_scheme_color(&props.style)
    ));
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&props.style.style),
    );
}

fn render_swift_dropzone(props: &DropzoneProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}VStack(alignment: .leading, spacing: CGFloat(8)) {{\n"
    ));
    if let Some(label) = props.style.label.as_deref() {
        output.push_str(&format!(
            "{pad}    Text({})\n{pad}        .font(.system(size: CGFloat(14), weight: .semibold))\n",
            swift_string_literal(label)
        ));
    }
    output.push_str(&format!(
        "{pad}    Button(action: {{ }}) {{\n{pad}        VStack(spacing: CGFloat(8)) {{\n{pad}            Image(systemName: \"paperclip\")\n{pad}                .font(.system(size: CGFloat(28), weight: .regular))\n{pad}                .opacity(0.5)\n{pad}            Text({})\n{pad}                .font(.system(size: CGFloat(14)))\n{pad}                .multilineTextAlignment(.center)\n{pad}                .opacity(0.7)\n{pad}        }}\n{pad}        .frame(maxWidth: .infinity, minHeight: CGFloat({}))\n{pad}    }}\n{pad}    .background({})\n{pad}    .foregroundStyle({})\n{pad}    .clipShape(RoundedRectangle(cornerRadius: {}))\n{pad}    .overlay(RoundedRectangle(cornerRadius: {}).stroke({}, style: StrokeStyle(lineWidth: CGFloat(2), dash: [CGFloat(6)])))\n{pad}    .buttonStyle(.plain)\n",
        swift_string_literal(
            props
                .style
                .placeholder
                .as_deref()
                .unwrap_or("Drag & drop files here or click to select")
        ),
        swift_dropzone_height(props.size),
        variant_container(&props.style),
        variant_content(&props.style),
        swift_card_radius(&props.style.style),
        swift_card_radius(&props.style.style),
        if props.error_text.is_some() {
            color_ref(ColorToken::Danger).to_string()
        } else {
            variant_content(&props.style).to_string()
        }
    ));
    if let Some(text) = props.error_text.as_deref().or(props.help_text.as_deref()) {
        output.push_str(&format!(
            "{pad}    Text({})\n{pad}        .font(.system(size: CGFloat(13)))\n{pad}        .foregroundStyle({})\n",
            swift_string_literal(text),
            if props.error_text.is_some() {
                color_ref(ColorToken::Danger).to_string()
            } else {
                color_ref(ColorToken::Muted).to_string()
            }
        ));
    }
    output.push_str(&format!("{pad}}}\n"));
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

fn swift_view_icon_system(icon: ViewIcon) -> &'static str {
    match icon {
        ViewIcon::Plus => "plus",
        ViewIcon::Link => "link",
        ViewIcon::Edit => "pencil",
        ViewIcon::Trash => "trash",
        ViewIcon::Search => "magnifyingglass",
        ViewIcon::Settings => "gearshape",
        ViewIcon::Upload => "paperclip",
        ViewIcon::File => "doc",
        ViewIcon::Dismiss => "xmark",
        ViewIcon::Moon => "moon.stars",
        ViewIcon::Sun => "sun.max",
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

fn swift_dropzone_height(size: ButtonSize) -> u16 {
    match size {
        ButtonSize::Xs | ButtonSize::Sm => 128,
        ButtonSize::Md => 192,
        ButtonSize::Lg | ButtonSize::Xl => 256,
    }
}
