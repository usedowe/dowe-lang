fn render_swift_structure_node(
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
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => {
            let context = context.with_scope(signals, actions);
            for child in children {
                render_swift_node_in_flow(
                    child,
                    indent,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    &context,
                );
            }
        }
        ViewNode::Each {
            item,
            collection,
            children,
            ..
        } => {
            output.push_str(&format!(
                "{pad}ForEach(state.rows(\"{}\")) {{ row in\n",
                escape_swift(&context.signal_path(collection))
            ));
            let context = context.with_item(item, "row.value".to_string());
            for child in children {
                render_swift_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    &context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
        }
        ViewNode::Box { props, children } => {
            let current_font = props.font.as_ref().or(inherited_font);
            if props.cover.is_some() {
                output.push_str(&format!("{pad}ZStack(alignment: .topLeading) {{\n"));
                output.push_str(&format!(
                    "{pad}    DoweCoverImage(source: {} ?? \"\")\n",
                    swift_cover_value(props.cover.as_ref().expect("cover"))
                ));
                if let Some(overlay) = props.overlay.as_ref() {
                    output.push_str(&format!(
                        "{pad}    if let overlay = {} {{\n{pad}        DoweOverlayView(overlay: overlay)\n{pad}    }}\n",
                        swift_overlay_value(overlay)
                    ));
                }
                output.push_str(&format!(
                    "{pad}    VStack(alignment: .leading, spacing: 0) {{\n"
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 8,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}    }}\n"));
                output.push_str(&format!("{pad}}}\n"));
            } else {
                output.push_str(&format!(
                    "{pad}VStack(alignment: .leading, spacing: 0) {{\n"
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 4,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}}}\n"));
            }
            append_swift_modifiers(
                output,
                indent,
                &swift_modifiers_for_container_style(props, flow),
            );
        }
        ViewNode::Section { props, children } => {
            let current_font = props.font.as_ref().or(inherited_font);
            if props.cover.is_some() {
                output.push_str(&format!("{pad}ZStack(alignment: .topLeading) {{\n"));
                output.push_str(&format!(
                    "{pad}    DoweCoverImage(source: {} ?? \"\")\n",
                    swift_cover_value(props.cover.as_ref().expect("cover"))
                ));
                if let Some(overlay) = props.overlay.as_ref() {
                    output.push_str(&format!(
                        "{pad}    if let overlay = {} {{\n{pad}        DoweOverlayView(overlay: overlay)\n{pad}    }}\n",
                        swift_overlay_value(overlay)
                    ));
                }
                output.push_str(&format!(
                    "{pad}    VStack(alignment: .leading, spacing: 0) {{\n"
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 8,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}    }}\n"));
                output.push_str(&format!("{pad}}}\n"));
            } else if let Some(background) = props.background.as_ref() {
                output.push_str(&format!("{pad}ZStack(alignment: .topLeading) {{\n"));
                output.push_str(&format!(
                    "{pad}    if let background = {} {{\n{pad}        DoweSectionBackgroundView(background: background)\n{pad}    }}\n",
                    swift_section_background_value(background)
                ));
                output.push_str(&format!(
                    "{pad}    VStack(alignment: .leading, spacing: 0) {{\n"
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 8,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}    }}\n"));
                output.push_str(&format!("{pad}}}\n"));
            } else {
                output.push_str(&format!(
                    "{pad}VStack(alignment: .leading, spacing: 0) {{\n"
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 4,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}}}\n"));
            }
            append_swift_modifiers(
                output,
                indent,
                &swift_modifiers_for_container_style(props, flow),
            );
        }
        ViewNode::Flex { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            output.push_str(&format!(
                "{pad}HStack(alignment: {}, spacing: {}) {{\n",
                swift_vertical_alignment(props.align.as_ref()),
                swift_gap(props.gap.as_ref())
            ));
            for child in children {
                render_swift_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    NativeFlow::Inline,
                    current_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
            append_swift_modifiers(output, indent, &swift_modifiers_for_layout(props, flow));
        }
        ViewNode::Grid { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            output.push_str(&format!(
                "{pad}LazyVGrid(columns: {}, alignment: {}, spacing: {}) {{\n",
                swift_grid_columns(props),
                swift_grid_horizontal_alignment(props.justify.as_ref()),
                swift_gap(props.gap.as_ref())
            ));
            for child in children {
                render_swift_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    NativeFlow::Block,
                    current_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
            append_swift_modifiers(output, indent, &swift_modifiers_for_grid(props, flow));
        }
        ViewNode::Card { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            if props.style.cover.is_some() {
                output.push_str(&format!("{pad}ZStack(alignment: .topLeading) {{\n"));
                output.push_str(&format!(
                    "{pad}    DoweCoverImage(source: {} ?? \"\")\n",
                    swift_cover_value(props.style.cover.as_ref().expect("cover"))
                ));
                if let Some(overlay) = props.style.overlay.as_ref() {
                    output.push_str(&format!(
                        "{pad}    if let overlay = {} {{\n{pad}        DoweOverlayView(overlay: overlay)\n{pad}    }}\n",
                        swift_overlay_value(overlay)
                    ));
                }
                output.push_str(&format!(
                    "{pad}    VStack(alignment: .leading, spacing: 0) {{\n"
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 8,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}    }}\n"));
                output.push_str(&format!("{pad}}}\n"));
            } else {
                output.push_str(&format!(
                    "{pad}VStack(alignment: .leading, spacing: 0) {{\n"
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 4,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}}}\n"));
            }
            let mut modifiers = swift_modifiers_for_container_style(&props.style, flow);
            modifiers.push(format!(".background({})", card_variant_container(props)));
            modifiers.push(format!(".foregroundStyle({})", card_variant_content(props)));
            let radius = swift_card_radius(&props.style);
            modifiers.push(format!(
                ".clipShape(RoundedRectangle(cornerRadius: {radius}))"
            ));
            if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
                modifiers.push(format!(
                    ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke({}, lineWidth: CGFloat(1)))",
                    variant_content(props)
                ));
            }
            append_swift_modifiers(output, indent, &modifiers);
        }
        ViewNode::Children => {
            if let Some(expression) = context.children_expression.as_ref() {
                output.push_str(&format!("{pad}{expression}\n"));
            }
        }
        _ => unreachable!(),
    }
}
