fn render_swift_text_svg_alert_node(
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
        ViewNode::Divider { props } => {
            output.push_str(&format!("{pad}Rectangle()\n"));
            append_swift_modifiers(output, indent, &swift_modifiers_for_divider(props, flow));
        }
        ViewNode::Title { props, value } => {
            output.push_str(&format!(
                "{pad}Text({})\n",
                swift_text_expression(value, props.i18n.as_deref(), context)
            ));
            let modifiers = swift_modifiers_for_text(
                true,
                props,
                props.style.font.as_ref().or(inherited_font),
                default_family,
            );
            append_swift_modifiers(output, indent, &modifiers);
        }
        ViewNode::Text { props, value } => {
            output.push_str(&format!(
                "{pad}Text({})\n",
                swift_text_expression(value, props.i18n.as_deref(), context)
            ));
            let modifiers = swift_modifiers_for_text(
                false,
                props,
                props.style.font.as_ref().or(inherited_font),
                default_family,
            );
            append_swift_modifiers(output, indent, &modifiers);
        }
        ViewNode::Alert { props } => {
            if let Some(visible) = props.visible.as_deref() {
                output.push_str(&format!(
                    "{pad}if state.bool(\"{}\") {{\n",
                    escape_swift(&context.signal_path(visible))
                ));
            }
            let alert_pad = if props.visible.is_some() {
                format!("{pad}    ")
            } else {
                pad.clone()
            };
            let radius = swift_control_radius(&props.style.style);
            output.push_str(&format!("{alert_pad}HStack(spacing: CGFloat(12)) {{\n"));
            output.push_str(&format!(
                "{alert_pad}    Text({})\n{alert_pad}        .frame(maxWidth: .infinity, alignment: .leading)\n",
                swift_text_expression(&props.message, None, context)
            ));
            if let Some(action) = props
                .on_close
                .as_deref()
                .and_then(|name| context.action_id(name))
            {
                output.push_str(&format!(
                    "{alert_pad}    Button(action: {{ state.run(\"{}\") }}) {{ Text(\"x\") }}\n",
                    escape_swift(action)
                ));
            }
            output.push_str(&format!("{alert_pad}}}\n"));
            let mut modifiers = swift_modifiers_for_style(&props.style.style);
            modifiers.push(".padding(.horizontal, CGFloat(14))".to_string());
            modifiers.push(".padding(.vertical, CGFloat(10))".to_string());
            modifiers.push(format!(".background({})", variant_container(&props.style)));
            modifiers.push(format!(
                ".foregroundStyle({})",
                variant_content(&props.style)
            ));
            modifiers.push(format!(
                ".clipShape(RoundedRectangle(cornerRadius: {radius}))"
            ));
            if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined
            {
                modifiers.push(format!(
                    ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke({}, lineWidth: CGFloat(1)))",
                    variant_content(&props.style)
                ));
            }
            append_swift_modifiers(
                output,
                if props.visible.is_some() {
                    indent + 4
                } else {
                    indent
                },
                &modifiers,
            );
            if props.visible.is_some() {
                output.push_str(&format!("{pad}}}\n"));
            }
        }
        ViewNode::Svg { props, paths } => {
            output.push_str(&format!(
                "{pad}DoweSvgView(viewBox: {}, color: {}, paths: {})\n",
                swift_svg_view_box(&props.view_box),
                swift_svg_color(&props.style),
                swift_svg_paths(paths)
            ));
            append_swift_modifiers(output, indent, &swift_modifiers_for_style(&props.style));
        }
        _ => unreachable!(),
    }
}
