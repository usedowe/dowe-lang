fn render_swift_region_children(
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    if children.is_empty() {
        output.push_str(&format!("{}EmptyView()\n", " ".repeat(indent)));
        return;
    }
    for child in children {
        if matches!(child, ViewNode::Box { props, .. } if region_box_has_style(props)) {
            render_swift_region_box_direct(
                child,
                indent,
                output,
                inherited_font,
                default_family,
                context,
            );
        } else {
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
}

fn region_box_has_style(props: &StyleProps) -> bool {
    props.spacing.p.is_some()
        || props.spacing.px.is_some()
        || props.spacing.py.is_some()
        || props.spacing.pl.is_some()
        || props.spacing.pr.is_some()
        || props.spacing.pt.is_some()
        || props.spacing.pb.is_some()
}

fn render_swift_region_box_direct(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    if let Some(show) = node_element_props(node).and_then(|props| props.show.as_ref()) {
        let pad = " ".repeat(indent);
        output.push_str(&format!("{pad}if {} {{\\n", swift_show_condition(show, context)));
        render_swift_node_body(
            node,
            indent + 4,
            output,
            NativeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}}}\\n"));
    } else {
        render_swift_node_body(
            node,
            indent,
            output,
            NativeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
}

fn render_swift_side_icon(icon: &SideNavIcon, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweSvgView(viewBox: {}, color: {}, paths: {})\n",
        swift_svg_view_box(&icon.props.view_box),
        swift_svg_color(&icon.props.style),
        swift_svg_paths(&icon.paths)
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&icon.props.style),
    );
}

fn render_swift_button_icon(icon: &SideNavIcon, color: &str, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweSvgView(viewBox: {}, color: {color}, paths: {})\n",
        swift_svg_view_box(&icon.props.view_box),
        swift_svg_paths(&icon.paths)
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&icon.props.style),
    );
}

fn render_swift_button_spinner(
    icon: &SideNavIcon,
    color: &str,
    indent: usize,
    output: &mut String,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweSvgView(viewBox: {}, color: {color}, paths: {}, animated: true)\n",
        swift_svg_view_box(&icon.props.view_box),
        swift_svg_paths(&icon.paths)
    ));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_style(&icon.props.style),
    );
}

fn swift_optional_component_action(
    action: Option<&str>,
    navigation: Option<&NavigationAction>,
    context: &SwiftReactiveContext,
) -> String {
    action
        .and_then(|name| context.action_id(name))
        .map(|id| {
            let item = context
                .active_item()
                .map(|value| format!(", item: {value}"))
                .unwrap_or_default();
            format!("{{ state.run(\"{}\"{item}) }}", escape_swift(id))
        })
        .or_else(|| navigation.map(|action| swift_navigation_action(Some(action))))
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_close_action(path: &str, action: Option<&str>, context: &SwiftReactiveContext) -> String {
    let after_close = action
        .and_then(|name| context.action_id(name))
        .map(|id| format!("; state.run(\"{}\")", escape_swift(id)))
        .unwrap_or_default();
    format!("{{ state.write(\"{path}\", value: false){after_close} }}")
}

fn swift_variant_border(props: &VariantProps) -> String {
    if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        format!("Optional({})", variant_content(props))
    } else {
        "nil".to_string()
    }
}

fn render_swift_tabs(
    props: &TabsProps,
    tabs: &[TabItem],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.font.as_ref().or(inherited_font);
    let initial = tabs.first().map(|tab| tab.id.as_str()).unwrap_or_default();
    output.push_str(&format!(
        "{pad}DoweTabs(items: {}, initialId: {}, position: {}, variant: {}, backgroundColor: {}, contentColor: {}, activeBackgroundColor: {}, activeContentColor: {}, accentColor: {}, borderColor: {}, radius: {}, font: {}) {{ activeTab in\n",
        swift_tabs_items(tabs),
        swift_string_literal(initial),
        swift_string_literal(props.position.as_str()),
        swift_string_literal(props.variant.as_str()),
        tabs_list_background(props),
        tabs_list_content(props),
        tabs_active_background(props),
        tabs_active_content(props),
        tabs_accent(props),
        tabs_border(props),
        swift_control_radius(&props.style),
        swift_font_value(current_font, "CGFloat(16)", default_family),
    ));
    for (index, tab) in tabs.iter().enumerate() {
        output.push_str(&format!(
            "{pad}    {} activeTab == {} {{\n",
            if index == 0 { "if" } else { "else if" },
            swift_string_literal(&tab.id)
        ));
        for child in &tab.children {
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
    }
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_container_style(&props.style, flow),
    );
}

fn swift_show_condition(show: &VisibilityCondition, context: &SwiftReactiveContext) -> String {
    match show {
        VisibilityCondition::Static(value) => {
            format!("{} ?? true", swift_bool_value(value))
        }
        VisibilityCondition::Signal(path) => {
            if let Some(item) = context.item_value(path) {
                let path = context.item_path(path).unwrap_or_else(|| path.to_string());
                format!("state.bool(\"{}\", item: {item})", escape_swift(&path))
            } else {
                format!(
                    "state.bool(\"{}\")",
                    escape_swift(&context.signal_path(path))
                )
            }
        }
        VisibilityCondition::StringEquality { path, value } => {
            format!("state.text(\"{}\") == \"{}\"", escape_swift(&context.signal_path(path)), escape_swift(value))
        }
        VisibilityCondition::NumberComparison { path, comparison } => {
            let value = if let Some(item) = context.item_value(path) {
                let path = context.item_path(path).unwrap_or_else(|| path.to_string());
                format!("state.text(\"{}\", item: {item})", escape_swift(&path))
            } else {
                format!(
                    "state.text(\"{}\", fallback: \"0\")",
                    escape_swift(&context.signal_path(path))
                )
            };
            format!(
                "(Double({value}) ?? 0) {} {}",
                comparison.operator.as_str(),
                comparison.value
            )
        }
    }
}

#[derive(Clone, Copy)]
struct SwiftBarOptions {
    start_padding: usize,
    center_padding: usize,
    end_padding: usize,
    boxed_width: usize,
    boxed_regions: bool,
}

fn swift_nav_menu_metrics(size: SideNavSize) -> (u16, u16, u16, u16, u16) {
    match size {
        SideNavSize::Sm => (8, 6, 8, 12, 10),
        SideNavSize::Md => (12, 8, 10, 14, 12),
        SideNavSize::Lg => (12, 8, 12, 16, 14),
    }
}

