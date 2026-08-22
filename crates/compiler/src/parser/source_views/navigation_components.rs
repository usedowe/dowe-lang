fn lower_side_nav_node(node: &SourceNode, component: BuiltinComponent) -> DoweResult<ViewNode> {
    let props = component_props(node, component)?;
    let mut items = Vec::new();
    for child in &node.children {
        items.push(match child.name.as_str() {
            "header" => lower_side_nav_entry(child, true, component)?,
            "item" => lower_side_nav_entry(child, false, component)?,
            "divider" => {
                if !child.args.is_empty() || !child.props.is_empty() || !child.children.is_empty() {
                    return Err(node_error(
                        child,
                        format!(
                            "{} divider cannot declare args, props or children",
                            component.as_str()
                        ),
                    ));
                }
                dowe_components::SideNavItem::Divider
            }
            "submenu" => lower_side_nav_submenu(child, component)?,
            _ => {
                return Err(node_error(
                    child,
                    format!(
                        "{} only accepts header, item, divider or submenu entries",
                        component.as_str()
                    ),
                ));
            }
        });
    }
    match component {
        BuiltinComponent::SideNav => {
            side_nav_component_node(props, items).map_err(|error| component_error(node, error))
        }
        _ => unreachable!("navigation component"),
    }
}

fn lower_rail_nav_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::RailNav)?;
    let mut items = Vec::new();
    for child in &node.children {
        items.push(match child.name.as_str() {
            "item" => {
                if !child.args.is_empty() || !child.children.is_empty() {
                    return Err(node_error(
                        child,
                        "RailNav item cannot declare args or children",
                    ));
                }
                let props = side_nav_entry_props(
                    child,
                    &[
                        "label",
                        "i18n",
                        "icon",
                        "href",
                        "navigate",
                        "target",
                        "externalMode",
                        "onClick",
                    ],
                    &[],
                    BuiltinComponent::RailNav,
                )?;
                rail_nav_item_component(props).map_err(|error| component_error(child, error))?
            }
            "divider" => {
                if !child.args.is_empty() || !child.props.is_empty() || !child.children.is_empty() {
                    return Err(node_error(
                        child,
                        "RailNav divider cannot declare args, props or children",
                    ));
                }
                dowe_components::RailNavItem::Divider
            }
            _ => {
                return Err(node_error(
                    child,
                    "RailNav only accepts item or divider entries",
                ));
            }
        });
    }
    rail_nav_component_node(props, items).map_err(|error| component_error(node, error))
}

fn lower_sidebar_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Sidebar)?;
    let mut header = None;
    let mut body = None;
    let mut footer = None;
    for child in &node.children {
        match child.name.as_str() {
            "header" if header.is_none() => {
                header = Some(lower_styled_region(child, "Sidebar header", allow_children)?)
            }
            "header" => return Err(node_error(child, "duplicate `header` region in Sidebar")),
            "body" if body.is_none() => {
                body = Some(lower_styled_region(child, "Sidebar body", allow_children)?)
            }
            "body" => return Err(node_error(child, "duplicate `body` region in Sidebar")),
            "footer" if footer.is_none() => {
                footer = Some(lower_styled_region(child, "Sidebar footer", allow_children)?)
            }
            "footer" => return Err(node_error(child, "duplicate `footer` region in Sidebar")),
            _ => {
                return Err(node_error(
                    child,
                    "Sidebar only accepts header, body or footer regions",
                ));
            }
        }
    }
    sidebar_component_node(
        props,
        header.unwrap_or_default(),
        body.unwrap_or_default(),
        footer.unwrap_or_default(),
        allow_children,
    )
    .map_err(|error| component_error(node, error))
}

fn lower_side_nav_entry(
    node: &SourceNode,
    header: bool,
    component: BuiltinComponent,
) -> DoweResult<dowe_components::SideNavItem> {
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            format!("{} entries cannot declare args", component.as_str()),
        ));
    }
    let icon = lower_side_nav_icon_children(node, component)?;
    let props = side_nav_entry_props(
        node,
        if header {
            &[
                "label",
                "icon",
                "i18n",
                "description",
                "descriptionI18n",
                "href",
                "navigate",
                "target",
                "externalMode",
                "onClick",
            ]
        } else {
            &[
                "label",
                "icon",
                "i18n",
                "description",
                "descriptionI18n",
                "status",
                "statusI18n",
                "href",
                "navigate",
                "target",
                "externalMode",
                "onClick",
            ]
        },
        &[],
        component,
    )?;
    if header {
        side_nav_header_component(props, icon).map_err(|error| component_error(node, error))
    } else {
        side_nav_item_component(props, icon).map_err(|error| component_error(node, error))
    }
}

fn lower_side_nav_submenu(
    node: &SourceNode,
    component: BuiltinComponent,
) -> DoweResult<dowe_components::SideNavItem> {
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            format!("{} submenu cannot declare args", component.as_str()),
        ));
    }
    let open = optional_prop_bool(node, "open")?.unwrap_or(false);
    let bordered = optional_prop_bool(node, "bordered")?.unwrap_or(true);
    let props = side_nav_entry_props(
        node,
        &[
            "label",
            "icon",
            "i18n",
            "description",
            "descriptionI18n",
            "status",
            "statusI18n",
        ],
        &["open", "bordered"],
        component,
    )?;
    let mut icon = None;
    let mut items = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "icon" if icon.is_none() => icon = Some(lower_side_nav_icon(child, component)?),
            "icon" => {
                return Err(node_error(
                    child,
                    format!("duplicate `icon` block in {} submenu", component.as_str()),
                ));
            }
            "item" => {
                let item = lower_side_nav_entry(child, false, component)?;
                let dowe_components::SideNavItem::Item(props) = item else {
                    unreachable!("SideNav submenu item");
                };
                items.push(props);
            }
            _ => {
                return Err(node_error(
                    child,
                    format!(
                        "{} submenu only accepts icon or item children",
                        component.as_str()
                    ),
                ));
            }
        }
    }
    side_nav_submenu_component(props, icon, open, bordered, items)
        .map_err(|error| component_error(node, error))
}

fn lower_side_nav_icon_children(
    node: &SourceNode,
    component: BuiltinComponent,
) -> DoweResult<Option<dowe_components::SideNavIcon>> {
    let mut icon = None;
    for child in &node.children {
        if child.name != "icon" {
            return Err(node_error(
                child,
                format!("{} entry only accepts an icon block", component.as_str()),
            ));
        }
        if icon.is_some() {
            return Err(node_error(
                child,
                format!("duplicate `icon` block in {} entry", component.as_str()),
            ));
        }
        icon = Some(lower_side_nav_icon(child, component)?);
    }
    Ok(icon)
}

fn lower_side_nav_icon(
    node: &SourceNode,
    component: BuiltinComponent,
) -> DoweResult<dowe_components::SideNavIcon> {
    if !node.args.is_empty() || !node.props.is_empty() || node.children.len() != 1 {
        return Err(node_error(
            node,
            format!("{} icon requires exactly one Svg child", component.as_str()),
        ));
    }
    let child = &node.children[0];
    if child.name != "Svg" {
        return Err(node_error(
            child,
            format!("{} icon requires exactly one Svg child", component.as_str()),
        ));
    }
    side_nav_icon_component(lower_svg_node(child)?).map_err(|error| component_error(node, error))
}

fn side_nav_entry_props(
    node: &SourceNode,
    allowed: &[&str],
    ignored: &[&str],
    component: BuiltinComponent,
) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .filter(|prop| !ignored.contains(&prop.name.as_str()))
        .map(|prop| {
            if !allowed.contains(&prop.name.as_str()) {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(component, &prop.name).to_string(),
                ));
            }
            if prop.name != "onClick" && static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

