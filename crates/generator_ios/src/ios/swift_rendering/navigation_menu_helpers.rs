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

fn render_swift_nav_menu(
    props: &NavMenuProps,
    items: &[NavMenuItem],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.style.font.as_ref().or(inherited_font);
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) =
        swift_side_nav_metrics(props.size);
    let border =
        if props.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined {
            format!("Optional({})", variant_content(&props.style))
        } else {
            "nil".to_string()
        };
    let wide_indices = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            matches!(item, NavMenuItem::Megamenu { .. }).then_some(index.to_string())
        })
        .collect::<Vec<_>>()
        .join(", ");
    output.push_str(&format!(
        "{pad}DoweNavMenu(gap: CGFloat({gap}), wideIndices: [{wide_indices}], popoverBackgroundColor: DoweDesign.background, popoverContentColor: DoweDesign.backgroundText) {{ openIndex, toggle in\n"
    ));
    for (index, item) in items.iter().enumerate() {
        render_swift_nav_menu_trigger(
            index,
            item,
            indent + 4,
            output,
            props,
            padding_horizontal,
            padding_vertical,
            label_size,
            &border,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}} popover: {{ openIndex in\n"));
    for (index, item) in items.iter().enumerate() {
        render_swift_nav_menu_popover(
            index,
            item,
            indent + 4,
            output,
            props,
            padding_horizontal,
            padding_vertical,
            label_size,
            description_size,
            &border,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_container_style(&props.style.style, flow),
    );
}

fn render_swift_nav_menu_trigger(
    index: usize,
    item: &NavMenuItem,
    indent: usize,
    output: &mut String,
    nav: &NavMenuProps,
    padding_horizontal: u16,
    padding_vertical: u16,
    label_size: u16,
    border: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    match item {
        NavMenuItem::Item(props) => render_swift_nav_menu_button(
            props,
            swift_nav_menu_action(props, context),
            swift_nav_menu_active(props.navigation.as_ref()),
            false,
            indent,
            output,
            nav,
            padding_horizontal,
            padding_vertical,
            label_size,
            border,
            inherited_font,
            default_family,
            context,
        ),
        NavMenuItem::Submenu { props, .. } | NavMenuItem::Megamenu { props, .. } => {
            render_swift_nav_menu_button(
                props,
                format!("{{ toggle({index}) }}"),
                format!("openIndex == {index}"),
                true,
                indent,
                output,
                nav,
                padding_horizontal,
                padding_vertical,
                label_size,
                border,
                inherited_font,
                default_family,
                context,
            );
        }
    }
}

fn render_swift_nav_menu_popover(
    index: usize,
    item: &NavMenuItem,
    indent: usize,
    output: &mut String,
    nav: &NavMenuProps,
    padding_horizontal: u16,
    padding_vertical: u16,
    label_size: u16,
    description_size: u16,
    border: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    match item {
        NavMenuItem::Submenu { items, .. } => {
            output.push_str(&format!("{pad}if openIndex == {index} {{\n"));
            output.push_str(&format!(
                "{pad}    VStack(alignment: .leading, spacing: CGFloat(2)) {{\n"
            ));
            for item in items {
                render_swift_nav_menu_subitem(
                    item,
                    indent + 8,
                    output,
                    nav,
                    padding_horizontal,
                    padding_vertical,
                    label_size,
                    description_size,
                    border,
                    inherited_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}    }}\n"));
            output.push_str(&format!("{pad}}}\n"));
        }
        NavMenuItem::Megamenu { content, .. } => {
            output.push_str(&format!("{pad}if openIndex == {index} {{\n"));
            output.push_str(&format!(
                "{pad}    VStack(alignment: .leading, spacing: CGFloat(8)) {{\n"
            ));
            for child in content {
                render_swift_node_in_flow(
                    child,
                    indent + 8,
                    output,
                    NativeFlow::Block,
                    inherited_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}    }}\n"));
            output.push_str(&format!("{pad}}}\n"));
        }
        NavMenuItem::Item(_) => {}
    }
}

fn render_swift_nav_menu_subitem(
    props: &NavMenuItemProps,
    indent: usize,
    output: &mut String,
    nav: &NavMenuProps,
    padding_horizontal: u16,
    padding_vertical: u16,
    label_size: u16,
    description_size: u16,
    border: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    render_swift_nav_menu_button(
        props,
        swift_nav_menu_action(props, context),
        swift_nav_menu_active(props.navigation.as_ref()),
        false,
        indent,
        output,
        nav,
        padding_horizontal,
        padding_vertical,
        label_size,
        border,
        inherited_font,
        default_family,
        context,
    );
    if let Some(description) = props.description.as_deref() {
        output.push_str(&format!(
            "{pad}Text({})\n{pad}    .font({})\n{pad}    .opacity(0.72)\n{pad}    .padding(.leading, CGFloat(12))\n",
            swift_localized_literal(description, props.description_i18n.as_deref()),
            swift_font_value(
                inherited_font,
                &format!("CGFloat({description_size})"),
                default_family
            )
        ));
    }
}

fn render_swift_nav_menu_button(
    props: &NavMenuItemProps,
    action: String,
    active: String,
    arrow: bool,
    indent: usize,
    output: &mut String,
    nav: &NavMenuProps,
    padding_horizontal: u16,
    padding_vertical: u16,
    label_size: u16,
    border: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweNavMenuItem(active: {active}, paddingHorizontal: CGFloat({padding_horizontal}), paddingVertical: CGFloat({padding_vertical}), backgroundColor: {}, contentColor: {}, borderColor: {border}, action: {action}) {{\n",
        variant_container(&nav.style),
        nav_active_content(&nav.style),
    ));
    if let Some(icon) = props.icon.as_ref() {
        output.push_str(&format!(
            "{pad}    DoweSvgView(viewBox: {}, color: {}, paths: {})\n",
            swift_svg_view_box(&icon.props.view_box),
            swift_svg_color(&icon.props.style),
            swift_svg_paths(&icon.paths)
        ));
        append_swift_modifiers(
            output,
            indent + 4,
            &swift_modifiers_for_style(&icon.props.style),
        );
    }
    output.push_str(&format!(
        "{pad}    Text({})\n{pad}        .font({})\n{pad}        .fontWeight(.regular)\n",
        swift_text_expression(&props.label, props.i18n.as_deref(), context),
        swift_font_value(
            inherited_font,
            &format!("CGFloat({label_size})"),
            default_family
        )
    ));
    if arrow {
        let icon = solar_control_icon("alt-arrow-down").expect("bundled NavMenu arrow icon");
        output.push_str(&format!(
            "{pad}    DoweSvgView(viewBox: {}, color: {active} ? {} : DoweDesign.backgroundText, paths: {})\n{pad}        .frame(width: CGFloat({label_size}), height: CGFloat({label_size}))\n{pad}        .rotationEffect({active} ? .degrees(180) : .degrees(0))\n",
            swift_svg_view_box(&icon.props.view_box),
            nav_active_content(&nav.style),
            swift_svg_paths(&icon.paths)
        ));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn swift_nav_menu_action(props: &NavMenuItemProps, context: &SwiftReactiveContext) -> String {
    props
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| format!("{{ state.run(\"{}\") }}", escape_swift(id)))
        .or_else(|| {
            props
                .navigation
                .as_ref()
                .map(|action| swift_navigation_action(Some(action)))
        })
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_nav_menu_active(action: Option<&NavigationAction>) -> String {
    match action {
        Some(NavigationAction::Internal { path, .. }) => {
            format!("activePath == \"{}\"", escape_swift(path))
        }
        _ => "false".to_string(),
    }
}
