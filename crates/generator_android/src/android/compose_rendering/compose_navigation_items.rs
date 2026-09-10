fn compose_side_nav_modifier(props: &SideNavProps, flow: ComposeFlow, wide: &str) -> String {
    let modifier = modifier_for_container_style(&props.style.style, flow);
    format!("{modifier}.then(if ({wide}) Modifier.fillMaxWidth() else Modifier)")
}

fn compose_side_nav_can_use_data_renderer(items: &[SideNavItem]) -> bool {
    items.iter().all(|item| match item {
        SideNavItem::Header(props) | SideNavItem::Item(props) => {
            compose_side_nav_item_can_use_data_renderer(props, true)
        }
        SideNavItem::Divider => true,
        SideNavItem::Submenu { props, items, .. } => {
            compose_side_nav_item_can_use_data_renderer(props, false)
                && items
                    .iter()
                    .all(|item| compose_side_nav_item_can_use_data_renderer(item, true))
        }
    })
}

fn compose_side_nav_item_can_use_data_renderer(
    props: &SideNavItemProps,
    allow_navigation: bool,
) -> bool {
    props.icon.is_none()
        && props.on_click.is_none()
        && (allow_navigation || props.navigation.is_none())
        && props
            .navigation
            .as_ref()
            .is_none_or(compose_side_nav_navigation_supported)
}

fn compose_side_nav_navigation_supported(action: &NavigationAction) -> bool {
    matches!(
        action,
        NavigationAction::Internal { .. } | NavigationAction::Section { .. }
    )
}

fn compose_side_nav_entries(items: &[SideNavItem], indent: usize) -> String {
    compose_side_nav_entries_with_prefix(items, indent, "item")
}

fn compose_side_nav_entries_with_prefix(
    items: &[SideNavItem],
    indent: usize,
    prefix: &str,
) -> String {
    if items.is_empty() {
        return "emptyList()".to_string();
    }
    let pad = " ".repeat(indent);
    let item_pad = " ".repeat(indent + 4);
    let mut output = "listOf(\n".to_string();
    for (index, item) in items.iter().enumerate() {
        let id = format!("{prefix}-{index}");
        output.push_str(&format!(
            "{item_pad}{},\n",
            compose_side_nav_entry(item, indent + 4, &id)
        ));
    }
    output.push_str(&format!("{pad})"));
    output
}

fn compose_side_nav_child_entries(
    items: &[SideNavItemProps],
    indent: usize,
    prefix: &str,
) -> String {
    if items.is_empty() {
        return "emptyList()".to_string();
    }
    let pad = " ".repeat(indent);
    let item_pad = " ".repeat(indent + 4);
    let mut output = "listOf(\n".to_string();
    for (index, item) in items.iter().enumerate() {
        let id = format!("{prefix}-{index}");
        output.push_str(&format!(
            "{item_pad}{},\n",
            compose_side_nav_entry_props("item", item, false, false, "", &id)
        ));
    }
    output.push_str(&format!("{pad})"));
    output
}

fn compose_side_nav_entry(item: &SideNavItem, indent: usize, id: &str) -> String {
    match item {
        SideNavItem::Header(props) => {
            compose_side_nav_entry_props("header", props, false, false, "", id)
        }
        SideNavItem::Item(props) => compose_side_nav_entry_props("item", props, false, false, "", id),
        SideNavItem::Divider => format!(
            "DoweSideNavEntry(id = \"{}\", kind = \"divider\", label = \"\", description = null, status = null, operation = null, path = null, fragment = null, bordered = false)",
            escape_kotlin(id)
        ),
        SideNavItem::Submenu {
            props,
            open,
            bordered,
            items,
        } => {
            let children = compose_side_nav_child_entries(items, indent + 4, id);
            compose_side_nav_entry_props("submenu", props, *open, *bordered, &children, id)
        }
    }
}

fn compose_side_nav_entry_props(
    kind: &str,
    props: &SideNavItemProps,
    open: bool,
    bordered: bool,
    children: &str,
    id: &str,
) -> String {
    let (operation, path, fragment) = compose_side_nav_navigation_values(props.navigation.as_ref());
    let children = if children.is_empty() {
        "emptyList()"
    } else {
        children
    };
    format!(
        "DoweSideNavEntry(id = \"{}\", kind = \"{}\", label = {}, description = {}, status = {}, operation = {}, path = {}, fragment = {}, open = {}, bordered = {}, children = {})",
        escape_kotlin(id),
        kind,
        compose_localized_literal(&props.label, props.i18n.as_deref()),
        props.description.as_deref().map(|value| compose_localized_literal(value, props.description_i18n.as_deref())).unwrap_or_else(|| "null".to_string()),
        props.status.as_deref().map(|value| compose_localized_literal(value, props.status_i18n.as_deref())).unwrap_or_else(|| "null".to_string()),
        compose_side_nav_optional_string(operation),
        compose_side_nav_optional_string(path),
        compose_side_nav_optional_string(fragment),
        open,
        bordered,
        children
    )
}

fn compose_side_nav_navigation_values(
    action: Option<&NavigationAction>,
) -> (Option<&str>, Option<&str>, Option<&str>) {
    match action {
        Some(NavigationAction::Internal {
            path,
            fragment,
            operation,
        }) => (
            Some(operation.as_str()),
            Some(path.as_str()),
            fragment.as_deref(),
        ),
        Some(NavigationAction::Section {
            fragment,
            operation,
        }) => (Some(operation.as_str()), Some(""), Some(fragment.as_str())),
        _ => (None, None, None),
    }
}

fn compose_side_nav_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", escape_kotlin(value)))
        .unwrap_or_else(|| "null".to_string())
}

fn render_compose_side_nav_item(
    item: &SideNavItem,
    indent: usize,
    output: &mut String,
    nav: &SideNavProps,
    wide: &str,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
    memory_key: &str,
) {
    let pad = " ".repeat(indent);
    match item {
        SideNavItem::Header(props) => render_compose_side_nav_row(
            props,
            true,
            compose_side_nav_action(props, context),
            indent,
            output,
            nav,
            wide,
            inherited_font,
            default_family,
            None,
        ),
        SideNavItem::Item(props) => render_compose_side_nav_row(
            props,
            false,
            compose_side_nav_action(props, context),
            indent,
            output,
            nav,
            wide,
            inherited_font,
            default_family,
            None,
        ),
        SideNavItem::Divider => output.push_str(&format!(
            "{pad}Box(modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp).height(1.dp).background(DoweDesign.muted))\n"
        )),
        SideNavItem::Submenu {
            props,
            open,
            bordered,
            items,
        } => {
            output.push_str(&format!("{pad}DoweSideNavSubmenu(stateKey = \"{}\", open = {open}, bordered = {bordered}, wide = {wide}, trigger = {{ expanded, toggle ->\n", escape_kotlin(memory_key)));
            render_compose_side_nav_row(
                props,
                false,
                "toggle".to_string(),
                indent + 4,
                output,
                nav,
                wide,
                inherited_font,
                default_family,
                Some("expanded"),
            );
            output.push_str(&format!("{pad}}}) {{\n"));
            for item in items {
                render_compose_side_nav_row(
                    item,
                    false,
                    compose_side_nav_action(item, context),
                    indent + 4,
                    output,
                    nav,
                    wide,
                    inherited_font,
                    default_family,
                    None,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
        }
    }
}

fn render_compose_side_nav_row(
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
    let (padding_horizontal, padding_vertical, gap, label_size, description_size) =
        compose_side_nav_metrics(nav.size);
    let border =
        if nav.style.variant.unwrap_or(ComponentVariant::Ghost) == ComponentVariant::Outlined {
            variant_content(&nav.style)
        } else {
            "null"
        };
    output.push_str(&format!(
        "{pad}DoweSideNavRow(active = {}, wide = {}, paddingHorizontal = {padding_horizontal}.dp, paddingVertical = {padding_vertical}.dp, gap = {gap}.dp, backgroundColor = {}, contentColor = {}, borderColor = {border}, onClick = {action}) {{\n",
        compose_side_nav_active(props.navigation.as_ref()),
        wide,
        variant_container(&nav.style),
        variant_content(&nav.style),
    ));
    if let Some(icon) = props.icon.as_ref() {
        let icon_color = if icon.props.style.text.is_some() {
            compose_svg_color(&icon.props.style)
        } else if header {
            side_nav_header_content(&nav.style).to_string()
        } else {
            "LocalContentColor.current".to_string()
        };
        output.push_str(&format!(
            "{pad}    DoweSvg(viewBox = {}, modifier = {}, color = {}, paths = {})\n",
            compose_svg_view_box(&icon.props.view_box),
            modifier_for_style(&icon.props.style),
            icon_color,
            compose_svg_paths(&icon.paths)
        ));
    }
    output.push_str(&format!(
        "{pad}    Column(modifier = Modifier.weight(1f)) {{\n"
    ));
    output.push_str(&format!(
        "{pad}        Text(text = {}, fontSize = {label_size}.sp, fontFamily = {}, fontWeight = {}, color = {})\n",
        compose_localized_literal(&props.label, props.i18n.as_deref()),
        compose_font_value(inherited_font, default_family),
        if header {
            "FontWeight.SemiBold"
        } else {
            "FontWeight.Normal"
        },
        if header { side_nav_header_content(&nav.style) } else { "LocalContentColor.current" }
    ));
    if let Some(description) = props.description.as_deref() {
        output.push_str(&format!(
            "{pad}        Text(text = {}, fontSize = {description_size}.sp, fontFamily = {}, color = LocalContentColor.current.copy(alpha = 0.72f))\n",
            compose_localized_literal(description, props.description_i18n.as_deref()),
            compose_font_value(inherited_font, default_family),
        ));
    }
    output.push_str(&format!("{pad}    }}\n"));
    if props.status.is_some() || submenu_expanded.is_some() {
        output.push_str(&format!(
            "{pad}    Row(horizontalArrangement = Arrangement.spacedBy({gap}.dp), verticalAlignment = Alignment.CenterVertically) {{\n"
        ));
        if let Some(status) = props.status.as_deref() {
            output.push_str(&format!(
                "{pad}        DoweSideNavStatus(text = {}, descriptionSize = {description_size}f, fontFamily = {})\n",
                compose_localized_literal(status, props.status_i18n.as_deref()),
                compose_font_value(inherited_font, default_family)
            ));
        }
        if let Some(expanded) = submenu_expanded {
            output.push_str(&format!(
                "{pad}        DoweSideNavArrow(expanded = {expanded})\n"
            ));
        }
        output.push_str(&format!("{pad}    }}\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn compose_side_nav_action(props: &SideNavItemProps, context: &ComposeReactiveContext) -> String {
    props
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
        .map(|id| {
            format!(
                "{{ actionScope.launch {{ state.run(\"{}\") }} }}",
                escape_kotlin(id)
            )
        })
        .or_else(|| {
            props
                .navigation
                .as_ref()
                .map(|action| compose_navigation_action(Some(action)))
        })
        .unwrap_or_else(|| "null".to_string())
}

fn compose_side_nav_active(action: Option<&NavigationAction>) -> String {
    match action {
        Some(NavigationAction::Internal { path, .. }) => {
            format!("activePath == \"{}\"", escape_kotlin(path))
        }
        _ => "false".to_string(),
    }
}

