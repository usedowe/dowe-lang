#[allow(unused_variables)]
fn render_swift_structure_card(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let ViewNode::Card { props, children } = node else { return; };
    let pad = " ".repeat(indent);
            let current_font = props.style.font.as_ref().or(inherited_font);
            let mut card_outer_style = props.style.clone();
            let has_cover = card_outer_style.cover.is_some();
            if has_cover {
                card_outer_style.spacing = Default::default();
            }
            let mut content_style = StyleProps::default();
            content_style.spacing = props.style.spacing.clone();
            let content_modifiers = swift_modifiers_for_style_with_width_alignment(&content_style, None).join("");
            let content_color_modifier = props
                .style
                .text
                .as_ref()
                .map(|color| {
                    let color = swift_color_value(color);
                    format!(
                        ".foregroundStyle({color} ?? Color.clear).environment(\\.doweTitleColor, {color} ?? Color.clear)"
                    )
                })
                .unwrap_or_default();
            if props.style.cover.is_some() {
                output.push_str(&format!("{pad}ZStack(alignment: .topLeading) {{\n"));
                output.push_str(&format!(
                    "{pad}    DoweCoverImage(source: {} ?? \"\")\n        .frame(maxWidth: .infinity, maxHeight: .infinity)\n        .clipped()\n",
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
                output.push_str(&format!("{pad}    }}{content_modifiers}{content_color_modifier}\n"));
                output.push_str(&format!("{pad}}}.frame(maxWidth: .infinity, alignment: .leading)\n"));
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
            let mut modifiers = swift_modifiers_for_container_style(&card_outer_style, flow);
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
                modifiers.push(format!(".background({})", card_surface_container(props)));
                modifiers.push(format!(".foregroundStyle({})", card_surface_content(props)));
                modifiers.push(format!(
                    ".environment(\\.doweTitleColor, {})",
                    card_surface_title(props)
                ));
            }
            if let Some(color) = props.style.text.as_ref() {
                let color = swift_color_value(color);
                modifiers.push(format!(".foregroundStyle({color} ?? Color.clear)"));
                modifiers.push(format!(".environment(\\.doweTitleColor, {color} ?? Color.clear)"));
            }
            let radius = swift_card_radius(&props.style);
            modifiers.push(format!(
                ".clipShape(RoundedRectangle(cornerRadius: {radius}))"
            ));
            if props.reactive.variant.is_some() || props.reactive.scheme.is_some() {
                modifiers.push(format!(
                    ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke(doweCardContent({variant}, {scheme}), lineWidth: ({variant} == \"outlined\" ? CGFloat(1) : CGFloat(0))))"
                ));
            } else if props.style.border.is_none()
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

#[allow(unused_variables)]
fn render_swift_structure_brand(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let ViewNode::Brand { props, children } = node else { return; };
    let pad = " ".repeat(indent);
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

#[allow(unused_variables)]
fn render_swift_structure_banner(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let ViewNode::Banner { props, children } = node else { return; };
    let pad = " ".repeat(indent);
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

#[allow(unused_variables)]
fn render_swift_structure_children(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let ViewNode::Children = node else { return; };
    let pad = " ".repeat(indent);
            if let Some(expression) = context.children_expression.as_ref() {
                output.push_str(&format!("{pad}{expression}\n"));
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

