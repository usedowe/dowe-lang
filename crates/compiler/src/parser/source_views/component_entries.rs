fn avatar_group_item_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(
                prop.name.as_str(),
                "src"
                    | "name"
                    | "alt"
                    | "href"
                    | "navigate"
                    | "history"
                    | "target"
                    | "externalMode"
                    | "onClick"
            ) {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::AvatarGroup, &prop.name)
                        .to_string(),
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

fn type_writer_item_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if prop.name != "text" {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::TypeWriter, &prop.name)
                        .to_string(),
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

fn rich_text_mark_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(prop.name.as_str(), "text" | "style" | "scheme" | "color") {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::RichText, &prop.name)
                        .to_string(),
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

fn toggle_group_item_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(prop.name.as_str(), "id" | "label" | "icon") {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::ToggleGroup, &prop.name)
                        .to_string(),
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

fn map_marker_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(
                prop.name.as_str(),
                "id" | "lat" | "lng" | "label" | "popup" | "icon" | "onClick"
            ) {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Map, &prop.name).to_string(),
                ));
            }
            if prop.name != "onClick"
                && !matches!(prop.name.as_str(), "lat" | "lng")
                && static_value_has_bareword(&prop.value)
            {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn map_waypoint_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(prop.name.as_str(), "lat" | "lng") {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Map, &prop.name).to_string(),
                ));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn accordion_item_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(
                prop.name.as_str(),
                "id" | "label" | "disabled" | "defaultOpen"
            ) {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Accordion, &prop.name)
                        .to_string(),
                ));
            }
            if matches!(prop.name.as_str(), "id" | "label")
                && static_value_has_bareword(&prop.value)
            {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn carousel_slide_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if prop.name != "id" {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Carousel, &prop.name)
                        .to_string(),
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

fn radio_item_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(prop.name.as_str(), "value" | "label" | "disabled") {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::RadioGroup, &prop.name)
                        .to_string(),
                ));
            }
            if matches!(prop.name.as_str(), "value" | "label")
                && static_value_has_bareword(&prop.value)
            {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn lower_command_group(node: &SourceNode) -> DoweResult<dowe_components::CommandEntry> {
    if !node.args.is_empty() {
        return Err(node_error(node, "Command group cannot declare args"));
    }
    let mut icon = None;
    let mut items = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "icon" if icon.is_none() => {
                icon = Some(lower_overlay_icon(child, BuiltinComponent::Command)?)
            }
            "icon" => {
                return Err(node_error(
                    child,
                    "duplicate `icon` region in Command group",
                ));
            }
            "item" => items.push(lower_overlay_item(child, BuiltinComponent::Command)?),
            _ => {
                return Err(node_error(
                    child,
                    "Command group only accepts icon or item entries",
                ));
            }
        }
    }
    command_group_component(command_group_props(node)?, icon, items)
        .map_err(|error| component_error(node, error))
}

fn lower_overlay_item(
    node: &SourceNode,
    owner: BuiltinComponent,
) -> DoweResult<dowe_components::OverlayItemProps> {
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            format!("{} item cannot declare args", owner.as_str()),
        ));
    }
    let mut icon = None;
    for child in &node.children {
        if child.name != "icon" {
            return Err(node_error(
                child,
                format!("{} item only accepts an icon region", owner.as_str()),
            ));
        }
        if icon.is_some() {
            return Err(node_error(
                child,
                format!("duplicate `icon` region in {} item", owner.as_str()),
            ));
        }
        icon = Some(lower_overlay_icon(child, owner)?);
    }
    overlay_item_component(owner, overlay_item_props(node, owner)?, icon)
        .map_err(|error| component_error(node, error))
}

fn lower_overlay_icon(
    node: &SourceNode,
    owner: BuiltinComponent,
) -> DoweResult<dowe_components::SideNavIcon> {
    if !node.args.is_empty() || !node.props.is_empty() || node.children.len() != 1 {
        return Err(node_error(
            node,
            format!("{} icon requires exactly one Svg child", owner.as_str()),
        ));
    }
    let child = &node.children[0];
    if child.name != "Svg" {
        return Err(node_error(
            child,
            format!("{} icon requires exactly one Svg child", owner.as_str()),
        ));
    }
    overlay_icon_component(lower_svg_node(child)?, owner)
        .map_err(|error| component_error(node, error))
}

fn lower_region(node: &SourceNode, label: &str, allow_children: bool) -> DoweResult<Vec<ViewNode>> {
    if !node.args.is_empty() || !node.props.is_empty() {
        return Err(node_error(
            node,
            format!("{label} cannot declare args or props"),
        ));
    }
    lower_node_sequence(&node.children, allow_children)
}

fn lower_styled_region(
    node: &SourceNode,
    label: &str,
    allow_children: bool,
) -> DoweResult<Vec<ViewNode>> {
    if !node.args.is_empty() {
        return Err(node_error(node, format!("{label} cannot declare args")));
    }
    let children = lower_node_sequence(&node.children, allow_children)?;
    if node.props.is_empty() {
        return Ok(children);
    }
    let props = component_props(node, BuiltinComponent::Box)?;
    let wrapper = container_component_node(BuiltinComponent::Box, props, children, allow_children)
        .map_err(|error| component_error(node, error))?;
    Ok(vec![wrapper])
}

fn overlay_item_props(
    node: &SourceNode,
    owner: BuiltinComponent,
) -> DoweResult<Vec<ComponentProp>> {
    let allowed = [
        "label",
        "description",
        "href",
        "navigate",
        "history",
        "target",
        "externalMode",
        "onClick",
        "disabled",
    ];
    node.props
        .iter()
        .map(|prop| {
            if !allowed.contains(&prop.name.as_str()) {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(owner, &prop.name).to_string(),
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

fn command_group_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if prop.name != "label" {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Command, &prop.name).to_string(),
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

