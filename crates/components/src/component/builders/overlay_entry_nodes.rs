pub fn overlay_item_component(
    owner: BuiltinComponent,
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
) -> ComponentResult<OverlayItemProps> {
    let mut label = None;
    let mut description = None;
    let mut href = None;
    let mut navigate = None;
    let mut history = None;
    let mut target = None;
    let mut external_mode = None;
    let mut on_click = None;
    let mut disabled = false;
    for prop in props {
        match prop.name.as_str() {
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "description" => description = Some(parse_required_string(&prop.name, &prop.value)?),
            "href" => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "navigate" => navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?),
            "history" => history = Some(parse_history_prop(&prop.name, &prop.value)?),
            "target" => target = Some(parse_web_target(&prop.name, &prop.value)?),
            "externalMode" => {
                external_mode = Some(parse_native_external_mode(&prop.name, &prop.value)?)
            }
            "onClick" => on_click = Some(parse_required_string(&prop.name, &prop.value)?),
            "disabled" => disabled = parse_static_bool(&prop.name, &prop.value)?,
            _ => return Err(ComponentError::unknown_prop(owner, &prop.name)),
        }
    }
    if href.is_some() && on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(format!(
            "`href` and `onClick` cannot be used on the same {} item",
            owner.as_str()
        )));
    }
    if href.is_some() && history.is_some() {
        return Err(ComponentError::invalid_prop_combination(format!(
            "`href` and `history` cannot be used on the same {} item",
            owner.as_str()
        )));
    }
    let navigation = if history.is_some() {
        if navigate.is_some() || target.is_some() || external_mode.is_some() {
            return Err(ComponentError::invalid_prop_combination(format!(
                "`navigate`, `target` and `externalMode` require `href` on {} item",
                owner.as_str()
            )));
        }
        history
    } else {
        parse_link_navigation_props(owner.as_str(), href, navigate, target, external_mode)?
    };
    Ok(OverlayItemProps {
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        description,
        icon,
        on_click,
        navigation,
        disabled,
    })
}

pub fn command_group_component(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
    items: Vec<OverlayItemProps>,
) -> ComponentResult<CommandEntry> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Command group requires at least one item",
        ));
    }
    let mut label = None;
    for prop in props {
        match prop.name.as_str() {
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::Command,
                    &prop.name,
                ));
            }
        }
    }
    Ok(CommandEntry::Group {
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        icon,
        items,
    })
}

pub fn overlay_icon_component(
    node: ViewNode,
    owner: BuiltinComponent,
) -> ComponentResult<SideNavIcon> {
    match node {
        ViewNode::Svg { props, paths } => Ok(SideNavIcon { props, paths }),
        _ => Err(ComponentError::invalid_prop_combination(format!(
            "{} icon requires exactly one Svg child",
            owner.as_str()
        ))),
    }
}
