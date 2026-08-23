fn render_swift_side_nav_item(
    item: &SideNavItem,
    indent: usize,
    output: &mut String,
    nav: &SideNavProps,
    wide: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
    memory_key: &str,
) {
    let pad = " ".repeat(indent);
    match item {
        SideNavItem::Header(props) => render_swift_side_nav_row(
            props,
            true,
            swift_side_nav_action(props, context),
            indent,
            output,
            nav,
            wide,
            inherited_font,
            default_family,
            None,
        ),
        SideNavItem::Item(props) => render_swift_side_nav_row(
            props,
            false,
            swift_side_nav_action(props, context),
            indent,
            output,
            nav,
            wide,
            inherited_font,
            default_family,
            None,
        ),
        SideNavItem::Divider => {
            output.push_str(&format!(
                "{pad}Divider()\n{pad}    .padding(.vertical, CGFloat(8))\n"
            ));
        }
        SideNavItem::Submenu {
            props,
            open,
            bordered,
            items,
        } => {
            output.push_str(&format!(
                "{pad}DoweSideNavSubmenu(stateKey: \"{}\", open: {open}, bordered: {bordered}, wide: {wide}) {{\n",
                escape_swift(memory_key)
            ));
            for item in items {
                render_swift_side_nav_row(
                    item,
                    false,
                    swift_side_nav_action(item, context),
                    indent + 4,
                    output,
                    nav,
                    wide,
                    inherited_font,
                    default_family,
                    None,
                );
            }
            output.push_str(&format!("{pad}}} label: {{ expanded in\n"));
            render_swift_side_nav_row(
                props,
                false,
                "nil".to_string(),
                indent + 4,
                output,
                nav,
                wide,
                inherited_font,
                default_family,
                Some("expanded"),
            );
            output.push_str(&format!("{pad}}}\n"));
        }
    }
}

fn render_swift_side_nav_row(
    props: &SideNavItemProps,
    header: bool,
    action: String,
    indent: usize,
    output: &mut String,
    nav: &SideNavProps,
    wide: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    submenu_expanded: Option<&str>,
) {
    let pad = " ".repeat(indent);
    let active = swift_side_nav_active(props.navigation.as_ref());
    let label_color = if header {
        side_nav_header_content(&nav.style).to_string()
    } else {
        format!(
            "{active} ? {} : DoweDesign.backgroundText",
            variant_content(&nav.style)
        )
    };
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) =
        swift_side_nav_metrics(nav.size);
    let border =
        if nav.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined {
            format!("Optional({})", variant_content(&nav.style))
        } else {
            "nil".to_string()
        };
    output.push_str(&format!(
        "{pad}DoweSideNavRow(active: {}, wide: {}, paddingHorizontal: CGFloat({padding_horizontal}), paddingVertical: CGFloat({padding_vertical}), gap: CGFloat({gap}), backgroundColor: {}, contentColor: {}, borderColor: {border}, action: {action}) {{\n",
        active,
        wide,
        variant_container(&nav.style),
        variant_content(&nav.style),
    ));
    if let Some(icon) = props.icon.as_ref() {
        output.push_str(&format!(
            "{pad}    DoweSvgView(viewBox: {}, color: {}, paths: {})\n",
            swift_svg_view_box(&icon.props.view_box),
            swift_side_nav_explicit_icon_color(icon, nav, &active, header),
            swift_svg_paths(&icon.paths)
        ));
        append_swift_modifiers(
            output,
            indent + 4,
            &swift_modifiers_for_style(&icon.props.style),
        );
    }
    output.push_str(&format!(
        "{pad}    VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
    ));
    output.push_str(&format!(
        "{pad}        Text({})\n{pad}            .font({})\n{pad}            .fontWeight({})\n{pad}            .foregroundStyle({})\n",
        swift_localized_literal(&props.label, props.i18n.as_deref()),
        swift_font_value(
            inherited_font,
            &format!("CGFloat({label_size})"),
            default_family
        ),
        if header { ".semibold" } else { ".regular" },
        label_color
    ));
    if let Some(description) = props.description.as_deref() {
        output.push_str(&format!(
            "{pad}        Text({})\n{pad}            .font({})\n{pad}            .opacity(0.72)\n",
            swift_localized_literal(description, props.description_i18n.as_deref()),
            swift_font_value(
                inherited_font,
                &format!("CGFloat({description_size})"),
                default_family
            )
        ));
    }
    output.push_str(&format!(
        "{pad}    }}\n{pad}    .frame(maxWidth: {wide} ? .infinity : nil, alignment: .leading)\n"
    ));
    if props.status.is_some() || submenu_expanded.is_some() {
        output.push_str(&format!("{pad}    HStack(spacing: CGFloat({gap})) {{\n"));
        if let Some(status) = props.status.as_deref() {
            output.push_str(&format!(
                "{pad}        DoweSideNavStatus(text: {}, font: {})\n",
                swift_localized_literal(status, props.status_i18n.as_deref()),
                swift_font_value(
                    inherited_font,
                    &format!("CGFloat({description_size})"),
                    default_family
                )
            ));
        }
        if let Some(expanded) = submenu_expanded {
            output.push_str(&format!(
                "{pad}        DoweSideNavArrow(expanded: {expanded})\n"
            ));
        }
        output.push_str(&format!("{pad}    }}\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn swift_side_nav_action(props: &SideNavItemProps, context: &SwiftReactiveContext) -> String {
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

fn swift_side_nav_active(action: Option<&NavigationAction>) -> String {
    match action {
        Some(NavigationAction::Internal { path, .. }) => {
            format!("activePath == \"{}\"", escape_swift(path))
        }
        _ => "false".to_string(),
    }
}

fn swift_side_nav_data_icon_color(icon: &SideNavIcon) -> String {
    if icon.props.style.text.is_some() {
        format!("Optional({})", swift_svg_color(&icon.props.style))
    } else {
        "nil".to_string()
    }
}

fn swift_side_nav_explicit_icon_color(
    icon: &SideNavIcon,
    nav: &SideNavProps,
    active: &str,
    header: bool,
) -> String {
    if icon.props.style.text.is_some() {
        swift_svg_color(&icon.props.style)
    } else if header {
        side_nav_header_content(&nav.style).to_string()
    } else {
        format!(
            "{active} ? {} : DoweDesign.backgroundText",
            nav_active_content(&nav.style)
        )
    }
}

fn swift_side_nav_metrics(size: SideNavSize) -> (u16, u16, u16, u16, u16) {
    match size {
        SideNavSize::Sm => (8, 6, 8, 12, 10),
        SideNavSize::Md => (12, 8, 10, 14, 12),
        SideNavSize::Lg => (16, 12, 12, 16, 14),
    }
}

