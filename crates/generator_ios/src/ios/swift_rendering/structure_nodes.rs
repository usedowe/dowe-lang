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
        ViewNode::Splash {
            binding,
            content,
            children,
            ..
        } => {
            output.push_str(&format!(
                "{pad}if state.bool(\"{}\") {{\n",
                escape_swift(&context.signal_path(binding))
            ));
            for child in children {
                render_swift_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}}} else {{\n"));
            for child in content {
                render_swift_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
        }
        ViewNode::Scope {
            constants,
            signals,
            actions,
            children,
        } => {
            let context = context.with_scope(constants, signals, actions);
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
            if props.position().mode != BoxPosition::Fixed {
                render_swift_box(
                    props,
                    children,
                    indent,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    context,
                    false,
                );
            }
        }
        ViewNode::Section { props, children } => {
            let current_font = props.font.as_ref().or(inherited_font);
            let section_spacing = swift_section_vertical_spacing(props.gap.as_ref());
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
                    "{pad}    VStack(alignment: {}, spacing: {section_spacing}) {{\n",
                    swift_section_horizontal_alignment(props.center_x.as_ref())
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
                append_swift_modifiers(
                    output,
                    indent + 4,
                    &swift_modifiers_for_section_content(props),
                );
                output.push_str(&format!("{pad}}}\n"));
            } else if let Some(background) = props.background.as_ref() {
                output.push_str(&format!("{pad}ZStack(alignment: .topLeading) {{\n"));
                output.push_str(&format!(
                    "{pad}    if let background = {} {{\n{pad}        DoweSectionBackgroundView(background: background)\n{pad}    }}\n",
                    swift_section_background_value(background)
                ));
                output.push_str(&format!(
                    "{pad}    VStack(alignment: {}, spacing: {section_spacing}) {{\n",
                    swift_section_horizontal_alignment(props.center_x.as_ref())
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
                append_swift_modifiers(
                    output,
                    indent + 4,
                    &swift_modifiers_for_section_content(props),
                );
                output.push_str(&format!("{pad}}}\n"));
            } else {
                output.push_str(&format!(
                    "{pad}VStack(alignment: {}, spacing: {section_spacing}) {{\n",
                    swift_section_horizontal_alignment(props.center_x.as_ref())
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
                append_swift_modifiers(output, indent, &swift_modifiers_for_section_content(props));
            }
            append_swift_modifiers(
                output,
                indent,
                &swift_modifiers_for_section_container(props, flow),
            );
        }
        ViewNode::Flex { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let justify = swift_flex_justify(props.justify.as_ref());
            let gap = swift_gap(props.gap.as_ref());
            output.push_str(&format!("{pad}Group {{\n"));
            output.push_str(&format!(
                "{pad}    if {} == DoweFlexDirection.column {{\n",
                swift_flex_direction_value(&props.direction)
            ));
            output.push_str(&format!(
                "{pad}        VStack(alignment: {}, spacing: doweFlexStackSpacing({justify}, gap: {gap})) {{\n",
                swift_horizontal_alignment(props.align.as_ref()),
            ));
            for (index, child) in children.iter().enumerate() {
                if index > 0 {
                    output.push_str(&format!(
                        "{pad}            if let spacerGap = doweFlexBetweenSpacer({justify}, gap: {gap}) {{\n{pad}                Spacer(minLength: spacerGap)\n{pad}            }}\n"
                    ));
                }
                render_swift_node_in_flow(
                    child,
                    indent + 12,
                    output,
                    NativeFlow::Block,
                    current_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}        }}\n"));
            append_swift_modifiers(
                output,
                indent + 8,
                &swift_modifiers_for_column_layout(props, flow),
            );
            output.push_str(&format!("{pad}    }} else {{\n"));
            if props.wrap {
                output.push_str(&format!(
                    "{pad}        DoweFlowLayout(justify: {justify}, align: {}, gap: {gap}) {{\n",
                    swift_flex_align(props.align.as_ref()),
                ));
            } else {
                output.push_str(&format!(
                    "{pad}        HStack(alignment: {}, spacing: doweFlexStackSpacing({justify}, gap: {gap})) {{\n",
                    swift_vertical_alignment(props.align.as_ref()),
                ));
            }
            for (index, child) in children.iter().enumerate() {
                if index > 0 && !props.wrap {
                    output.push_str(&format!(
                        "{pad}            if let spacerGap = doweFlexBetweenSpacer({justify}, gap: {gap}) {{\n{pad}                Spacer(minLength: spacerGap)\n{pad}            }}\n"
                    ));
                }
                render_swift_node_in_flow(
                    child,
                    indent + 12,
                    output,
                    NativeFlow::Inline,
                    current_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}        }}\n"));
            append_swift_modifiers(output, indent + 8, &swift_modifiers_for_layout(props, flow));
            output.push_str(&format!("{pad}    }}\n"));
            output.push_str(&format!("{pad}}}\n"));
        }
        ViewNode::Grid { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            output.push_str(&format!(
                "{pad}DoweGridLayout(tracks: {}, rowGap: {}, columnGap: {}, justify: {}, align: {}, fillHeight: {}) {{\n",
                swift_grid_tracks(props.columns.as_ref()),
                swift_grid_row_gap(props.gap.as_ref()),
                swift_grid_column_gap(props.gap.as_ref()),
                swift_grid_alignment(props.justify.as_ref()),
                swift_grid_alignment(props.align.as_ref()),
                swift_grid_fills_height(props, flow)
            ));
            for child in children {
                render_swift_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    NativeFlow::GridItem,
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
            let mut card_style = props.style.clone();
            card_style.shadow = None;
            card_style.shadow_color = None;
            card_style.set_animation(None);
            let mut modifiers = swift_modifiers_for_container_style(&card_style, flow);
            let reactive_text = |path: &str, fallback: &str| {
                context
                    .item_value(path)
                    .map(|item| {
                        format!(
                            "state.text(\"{}\", item: {item})",
                            escape_swift(&context.item_path(path).expect("item path"))
                        )
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "state.text(\"{}\", fallback: \"{fallback}\")",
                            escape_swift(&context.signal_path(path))
                        )
                    })
            };
            let variant = props
                .reactive
                .variant
                .as_deref()
                .map(|path| reactive_text(path, "solid"))
                .unwrap_or_else(|| format!("\"{}\"", props.variant.unwrap_or(ComponentVariant::Solid).as_str()));
            let scheme = props
                .reactive
                .scheme
                .as_deref()
                .map(|path| reactive_text(path, "primary"))
                .unwrap_or_else(|| format!("\"{}\"", props.color.unwrap_or(ColorFamily::Primary).as_str()));
            if props.reactive.scheme.is_some() || props.reactive.variant.is_some() {
                modifiers.push(format!(".background(doweCardContainer({variant}, {scheme}))"));
                modifiers.push(format!(".foregroundStyle(doweCardContent({variant}, {scheme}))"));
                modifiers.push(format!(
                    ".environment(\\.doweTitleColor, doweCardTitle({variant}, {scheme}))"
                ));
            } else {
                modifiers.push(format!(".background({})", card_variant_container(props)));
                modifiers.push(format!(".foregroundStyle({})", card_variant_content(props)));
                modifiers.push(format!(
                    ".environment(\\.doweTitleColor, {})",
                    card_variant_title(props)
                ));
            }
            let radius = swift_card_radius(&props.style);
            modifiers.push(format!(
                ".clipShape(RoundedRectangle(cornerRadius: {radius}))"
            ));
            if props.style.border.is_none()
                && props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined
            {
                modifiers.push(format!(
                    ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke({}, lineWidth: CGFloat(1)))",
                    variant_content(props)
                ));
            }
            if let Some(modifier) = swift_shadow_modifier_with_radius(&props.style, &radius) {
                modifiers.push(modifier);
            }
            if let Some(animation) = props.style.animation() {
                modifiers.push(format!(
                    ".modifier(DoweAnimationModifier(preset: {}))",
                    swift_animation_preset(animation)
                ));
            }
            if props.style.element.on_click.is_some() {
                modifiers.push(format!(
                    ".onTapGesture(perform: {})",
                    swift_component_action(props.style.element.on_click.as_deref(), None, context)
                ));
            }
            append_swift_modifiers(output, indent, &modifiers);
        }
        ViewNode::Brand { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            if props.navigation.is_some() {
                output.push_str(&format!(
                    "{pad}Button(action: {}) {{\n",
                    swift_navigation_action(props.navigation.as_ref())
                ));
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
            } else {
                output.push_str(&format!("{pad}HStack(spacing: 0) {{\n"));
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
            }
            let mut modifiers =
                swift_modifiers_for_container_style(&props.style, NativeFlow::Inline);
            if props.navigation.is_some() {
                modifiers.push(".contentShape(Rectangle())".to_string());
                modifiers.push(".buttonStyle(.plain)".to_string());
            }
            if let Some(label) = props.label.as_deref() {
                modifiers.push(".accessibilityElement(children: .ignore)".to_string());
                modifiers.push(format!(
                    ".accessibilityLabel(Text(\"{}\"))",
                    escape_swift(label)
                ));
            }
            append_swift_modifiers(output, indent, &modifiers);
        }
        ViewNode::Banner { props, children } => {
            output.push_str(&format!(
                "{pad}Button(action: {}) {{\n",
                swift_navigation_action(Some(&props.navigation))
            ));
            render_swift_box(
                &props.style,
                children,
                indent + 4,
                output,
                NativeFlow::Block,
                inherited_font,
                default_family,
                context,
                false,
            );
            output.push_str(&format!("{pad}}}\n"));
            let mut modifiers = vec![
                ".contentShape(Rectangle())".to_string(),
                ".buttonStyle(.plain)".to_string(),
            ];
            if let Some(label) = props.label.as_deref() {
                modifiers.push(".accessibilityElement(children: .ignore)".to_string());
                modifiers.push(format!(
                    ".accessibilityLabel(Text(\"{}\"))",
                    escape_swift(label)
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

fn render_swift_fixed_box(
    props: &StyleProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    render_swift_box(
        props,
        children,
        indent,
        output,
        NativeFlow::Inline,
        inherited_font,
        default_family,
        context,
        true,
    );
}

fn render_swift_box(
    props: &StyleProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
    render_fixed: bool,
) {
    let pad = " ".repeat(indent);
    let current_font = props.font.as_ref().or(inherited_font);
    let position = props.position();
    let positioned = position.mode == BoxPosition::Absolute
        || position.mode == BoxPosition::Fixed && render_fixed;
    let has_absolute_children = position.mode == BoxPosition::Relative
        && children.iter().any(|child| {
            matches!(child, ViewNode::Box { props, .. } if props.position().mode == BoxPosition::Absolute)
        });
    let layered = props.cover.is_some() || has_absolute_children || positioned;

    if layered {
        output.push_str(&format!("{pad}ZStack(alignment: .topLeading) {{\n"));
        if let Some(cover) = props.cover.as_ref() {
            output.push_str(&format!(
                "{pad}    DoweCoverImage(source: {} ?? \"\")\n",
                swift_cover_value(cover)
            ));
            if let Some(overlay) = props.overlay.as_ref() {
                output.push_str(&format!(
                    "{pad}    if let overlay = {} {{\n{pad}        DoweOverlayView(overlay: overlay)\n{pad}    }}\n",
                    swift_overlay_value(overlay)
                ));
            }
        }
        output.push_str(&format!(
            "{pad}    VStack(alignment: {}, spacing: 0) {{\n",
            swift_section_horizontal_alignment(props.center_x.as_ref())
        ));
        render_swift_box_flow_children(
            children,
            indent + 8,
            output,
            current_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}    }}\n"));
        for child in children.iter().filter(|child| {
            matches!(child, ViewNode::Box { props, .. } if props.position().mode == BoxPosition::Absolute)
        }) {
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
    } else {
        output.push_str(&format!(
            "{pad}VStack(alignment: {}, spacing: 0) {{\n",
            swift_section_horizontal_alignment(props.center_x.as_ref())
        ));
        render_swift_box_flow_children(
            children,
            indent + 4,
            output,
            current_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}}}\n"));
    }

    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_container_style(
            props,
            if positioned { NativeFlow::Inline } else { flow },
        ),
    );
    if props.element.on_click.is_some() {
        append_swift_modifiers(
            output,
            indent,
            &[format!(
                ".onTapGesture(perform: {})",
                swift_component_action(props.element.on_click.as_deref(), None, context)
            )],
        );
    }
    if positioned {
        append_swift_modifiers(
            output,
            indent,
            &swift_modifiers_for_positioned_box(position),
        );
    }
}

fn render_swift_box_flow_children(
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    for child in children.iter().filter(|child| {
        !matches!(child, ViewNode::Box { props, .. } if matches!(props.position().mode, BoxPosition::Absolute | BoxPosition::Fixed))
    }) {
        render_swift_node_in_flow(
            child,
            indent,
            output,
            NativeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
}
