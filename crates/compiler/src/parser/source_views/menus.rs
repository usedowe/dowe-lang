fn lower_tabs_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Tabs)?;
    let mut tabs = Vec::new();
    for child in &node.children {
        if child.name != "tab" {
            return Err(node_error(child, "Tabs only accepts tab entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "Tabs tab cannot declare args"));
        }
        let children = lower_node_sequence(&child.children, allow_children)?;
        tabs.push(
            tabs_tab_component(tabs_tab_props(child)?, children)
                .map_err(|error| component_error(child, error))?,
        );
    }
    tabs_component_node(props, tabs).map_err(|error| component_error(node, error))
}

fn tabs_tab_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(prop.name.as_str(), "id" | "label" | "i18n") {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Tab, &prop.name).to_string(),
                ));
            }
            if static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn lower_stepper_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Stepper)?;
    let mut steps = Vec::new();
    for child in &node.children {
        if child.name != "step" {
            return Err(node_error(child, "Stepper only accepts step entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "Stepper step cannot declare args"));
        }
        let children = lower_node_sequence(&child.children, allow_children)?;
        steps.push(
            stepper_step_component(stepper_step_props(child)?, children)
                .map_err(|error| component_error(child, error))?,
        );
    }
    stepper_component_node(props, steps).map_err(|error| component_error(node, error))
}

fn stepper_step_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(prop.name.as_str(), "id" | "label" | "i18n") {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Step, &prop.name).to_string(),
                ));
            }
            if static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn lower_table_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Table)?;
    let mut columns = Vec::new();
    for child in &node.children {
        if child.name != "column" {
            return Err(node_error(child, "Table only accepts column entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "Table column cannot declare args"));
        }
        reject_children(child)?;
        columns.push(
            table_column_component(table_column_props(child)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    table_node(props, columns).map_err(|error| component_error(node, error))
}

fn table_column_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(prop.name.as_str(), "field" | "label" | "align" | "width") {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Table, &prop.name).to_string(),
                ));
            }
            if static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn lower_nav_menu_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::NavMenu)?;
    let mut items = Vec::new();
    for child in &node.children {
        items.push(match child.name.as_str() {
            "item" => lower_nav_menu_item(child)?,
            "submenu" => lower_nav_menu_submenu(child)?,
            "megamenu" => lower_nav_menu_megamenu(child, allow_children)?,
            _ => {
                return Err(node_error(
                    child,
                    "NavMenu only accepts item, submenu or megamenu entries",
                ));
            }
        });
    }
    nav_menu_component_node(props, items).map_err(|error| component_error(node, error))
}

fn lower_nav_menu_item(node: &SourceNode) -> DoweResult<dowe_components::NavMenuItem> {
    if !node.args.is_empty() {
        return Err(node_error(node, "NavMenu entries cannot declare args"));
    }
    let icon = lower_nav_menu_icon_children(node)?;
    let props = nav_menu_entry_props(
        node,
        &[
            "label",
            "i18n",
            "description",
            "descriptionI18n",
            "href",
            "navigate",
            "target",
            "externalMode",
            "onClick",
        ],
    )?;
    nav_menu_item_component(props, icon).map_err(|error| component_error(node, error))
}

fn lower_nav_menu_submenu(node: &SourceNode) -> DoweResult<dowe_components::NavMenuItem> {
    if !node.args.is_empty() {
        return Err(node_error(node, "NavMenu submenu cannot declare args"));
    }
    let props = nav_menu_entry_props(node, &["label", "i18n", "description", "descriptionI18n"])?;
    let mut icon = None;
    let mut items = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "icon" if icon.is_none() => icon = Some(lower_nav_menu_icon(child)?),
            "icon" => {
                return Err(node_error(
                    child,
                    "duplicate `icon` block in NavMenu submenu",
                ));
            }
            "item" => {
                let item = lower_nav_menu_item(child)?;
                let dowe_components::NavMenuItem::Item(props) = item else {
                    unreachable!("NavMenu submenu item");
                };
                items.push(props);
            }
            _ => {
                return Err(node_error(
                    child,
                    "NavMenu submenu only accepts icon or item children",
                ));
            }
        }
    }
    nav_menu_submenu_component(props, icon, items).map_err(|error| component_error(node, error))
}

fn lower_nav_menu_megamenu(
    node: &SourceNode,
    allow_children: bool,
) -> DoweResult<dowe_components::NavMenuItem> {
    if !node.args.is_empty() {
        return Err(node_error(node, "NavMenu megamenu cannot declare args"));
    }
    let props = nav_menu_entry_props(node, &["label", "i18n", "description", "descriptionI18n"])?;
    let mut icon = None;
    let mut content = None;
    for child in &node.children {
        match child.name.as_str() {
            "icon" if icon.is_none() => icon = Some(lower_nav_menu_icon(child)?),
            "icon" => {
                return Err(node_error(
                    child,
                    "duplicate `icon` block in NavMenu megamenu",
                ));
            }
            "content" if content.is_none() => {
                if !child.args.is_empty() || !child.props.is_empty() {
                    return Err(node_error(
                        child,
                        "NavMenu megamenu content cannot declare args or props",
                    ));
                }
                content = Some(lower_node_sequence(&child.children, allow_children)?);
            }
            "content" => {
                return Err(node_error(
                    child,
                    "duplicate `content` region in NavMenu megamenu",
                ));
            }
            _ => {
                return Err(node_error(
                    child,
                    "NavMenu megamenu only accepts icon and content children",
                ));
            }
        }
    }
    nav_menu_megamenu_component(props, icon, content.unwrap_or_default(), allow_children)
        .map_err(|error| component_error(node, error))
}

fn lower_nav_menu_icon_children(
    node: &SourceNode,
) -> DoweResult<Option<dowe_components::SideNavIcon>> {
    let mut icon = None;
    for child in &node.children {
        if child.name != "icon" {
            return Err(node_error(
                child,
                "NavMenu entry only accepts an icon block",
            ));
        }
        if icon.is_some() {
            return Err(node_error(child, "duplicate `icon` block in NavMenu entry"));
        }
        icon = Some(lower_nav_menu_icon(child)?);
    }
    Ok(icon)
}

fn lower_nav_menu_icon(node: &SourceNode) -> DoweResult<dowe_components::SideNavIcon> {
    if !node.args.is_empty() || !node.props.is_empty() || node.children.len() != 1 {
        return Err(node_error(
            node,
            "NavMenu icon requires exactly one Svg child",
        ));
    }
    let child = &node.children[0];
    if child.name != "Svg" {
        return Err(node_error(
            child,
            "NavMenu icon requires exactly one Svg child",
        ));
    }
    side_nav_icon_component(lower_svg_node(child)?).map_err(|error| component_error(node, error))
}

fn nav_menu_entry_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !allowed.contains(&prop.name.as_str()) {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::NavMenu, &prop.name).to_string(),
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
