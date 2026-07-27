pub fn side_nav_header_component(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
) -> ComponentResult<SideNavItem> {
    Ok(SideNavItem::Header(parse_side_nav_item_props(
        props, icon, false,
    )?))
}

pub fn side_nav_item_component(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
) -> ComponentResult<SideNavItem> {
    Ok(SideNavItem::Item(parse_side_nav_item_props(
        props, icon, true,
    )?))
}

pub fn side_nav_submenu_component(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
    open: bool,
    bordered: bool,
    items: Vec<SideNavItemProps>,
) -> ComponentResult<SideNavItem> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "SideNav submenu requires at least one item",
        ));
    }
    let props = parse_side_nav_item_props(props, icon, true)?;
    if props.navigation.is_some() || props.on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "SideNav submenu cannot declare navigation or onClick",
        ));
    }
    Ok(SideNavItem::Submenu {
        props,
        open,
        bordered,
        items,
    })
}

pub fn side_nav_icon_component(node: ViewNode) -> ComponentResult<SideNavIcon> {
    match node {
        ViewNode::Svg { props, paths } => Ok(SideNavIcon { props, paths }),
        _ => Err(ComponentError::invalid_prop_combination(
            "SideNav icon requires exactly one Svg child",
        )),
    }
}

pub fn rail_nav_item_component(props: Vec<ComponentProp>) -> ComponentResult<RailNavItem> {
    let mut label = None;
    let mut i18n = None;
    let mut icon = None;
    let mut href = None;
    let mut navigate = None;
    let mut target = None;
    let mut external_mode = None;
    let mut on_click = None;

    for prop in props {
        match prop.name.as_str() {
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "i18n" => i18n = Some(parse_i18n_key_prop(&prop.name, &prop.value)?),
            "icon" => {
                let name = parse_required_string(&prop.name, &prop.value)?;
                icon = Some(solar_control_icon(&name)?);
            }
            "href" => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "navigate" => navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?),
            "target" => target = Some(parse_web_target(&prop.name, &prop.value)?),
            "externalMode" => {
                external_mode = Some(parse_native_external_mode(&prop.name, &prop.value)?)
            }
            "onClick" => on_click = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::RailNav,
                    &prop.name,
                ));
            }
        }
    }

    if href.is_some() && on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "`href` and `onClick` cannot be used on the same RailNav item",
        ));
    }
    let navigation =
        parse_link_navigation_props("RailNav item", href, navigate, target, external_mode)?;
    Ok(RailNavItem::Item(RailNavItemProps {
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        i18n,
        icon: icon.ok_or_else(|| ComponentError::invalid_prop("icon", "Solar icon name"))?,
        on_click,
        navigation,
    }))
}

pub fn bottom_bar_tab_component(
    props: Vec<ComponentProp>,
    icon: SideNavIcon,
) -> ComponentResult<BottomBarTab> {
    let mut label = None;
    let mut i18n = None;
    let mut featured = false;
    let mut href = None;
    let mut navigate = None;
    let mut target = None;
    let mut external_mode = None;
    for prop in props {
        match prop.name.as_str() {
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "i18n" => i18n = Some(parse_i18n_key_prop(&prop.name, &prop.value)?),
            "featured" => featured = parse_static_bool(&prop.name, &prop.value)?,
            "href" => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "navigate" => navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?),
            "target" => target = Some(parse_web_target(&prop.name, &prop.value)?),
            "externalMode" => {
                external_mode = Some(parse_native_external_mode(&prop.name, &prop.value)?)
            }
            _ => return Err(ComponentError::unknown_prop(BuiltinComponent::BottomBar, &prop.name)),
        }
    }
    let navigation = parse_link_navigation_props(
        "BottomBar tab",
        href,
        navigate,
        target,
        external_mode,
    )?
    .ok_or_else(|| ComponentError::invalid_prop("href", "navigation destination"))?;
    Ok(BottomBarTab {
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        i18n,
        featured,
        icon,
        navigation,
    })
}

pub fn nav_menu_item_component(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
) -> ComponentResult<NavMenuItem> {
    Ok(NavMenuItem::Item(parse_nav_menu_item_props(
        props, icon, true,
    )?))
}

pub fn nav_menu_submenu_component(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
    items: Vec<NavMenuItemProps>,
) -> ComponentResult<NavMenuItem> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "NavMenu submenu requires at least one item",
        ));
    }
    let props = parse_nav_menu_item_props(props, icon, false)?;
    if props.navigation.is_some() || props.on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "NavMenu submenu cannot declare navigation or onClick",
        ));
    }
    Ok(NavMenuItem::Submenu { props, items })
}

pub fn nav_menu_megamenu_component(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
    content: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<NavMenuItem> {
    if content.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "NavMenu megamenu requires content",
        ));
    }
    if contains_children(&content) && !allow_children {
        return Err(ComponentError::children_outside_layout());
    }
    let props = parse_nav_menu_item_props(props, icon, false)?;
    if props.navigation.is_some() || props.on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "NavMenu megamenu cannot declare navigation or onClick",
        ));
    }
    Ok(NavMenuItem::Megamenu { props, content })
}

fn parse_side_nav_item_props(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
    allow_status: bool,
) -> ComponentResult<SideNavItemProps> {
    let mut icon = icon;
    let mut label = None;
    let mut i18n = None;
    let mut description = None;
    let mut description_i18n = None;
    let mut status = None;
    let mut status_i18n = None;
    let mut href = None;
    let mut navigate = None;
    let mut target = None;
    let mut external_mode = None;
    let mut on_click = None;

    for prop in props {
        match prop.name.as_str() {
            "icon" => {
                if icon.is_some() {
                    return Err(ComponentError::invalid_prop_combination(
                        "a SideNav entry can declare either an `icon` prop or an icon block",
                    ));
                }
                let name = parse_required_string(&prop.name, &prop.value)?;
                icon = Some(solar_control_icon(&name)?);
            }
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "i18n" => i18n = Some(parse_i18n_key_prop(&prop.name, &prop.value)?),
            "description" => description = Some(parse_required_string(&prop.name, &prop.value)?),
            "descriptionI18n" => {
                description_i18n = Some(parse_i18n_key_prop(&prop.name, &prop.value)?)
            }
            "status" if allow_status => {
                status = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "statusI18n" if allow_status => {
                status_i18n = Some(parse_i18n_key_prop(&prop.name, &prop.value)?)
            }
            "href" => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "navigate" => navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?),
            "target" => target = Some(parse_web_target(&prop.name, &prop.value)?),
            "externalMode" => {
                external_mode = Some(parse_native_external_mode(&prop.name, &prop.value)?)
            }
            "onClick" => on_click = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::SideNav,
                    &prop.name,
                ));
            }
        }
    }

    if href.is_some() && on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "`href` and `onClick` cannot be used on the same SideNav entry",
        ));
    }
    if description_i18n.is_some() && description.is_none() {
        return Err(ComponentError::invalid_prop_combination(
            "`descriptionI18n` requires `description` on a SideNav entry",
        ));
    }
    if status_i18n.is_some() && status.is_none() {
        return Err(ComponentError::invalid_prop_combination(
            "`statusI18n` requires `status` on a SideNav entry",
        ));
    }
    let navigation =
        parse_link_navigation_props("SideNav entry", href, navigate, target, external_mode)?;
    Ok(SideNavItemProps {
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        i18n,
        description,
        description_i18n,
        status,
        status_i18n,
        icon,
        on_click,
        navigation,
    })
}

fn parse_nav_menu_item_props(
    props: Vec<ComponentProp>,
    icon: Option<SideNavIcon>,
    allow_navigation: bool,
) -> ComponentResult<NavMenuItemProps> {
    let mut label = None;
    let mut i18n = None;
    let mut description = None;
    let mut description_i18n = None;
    let mut href = None;
    let mut navigate = None;
    let mut target = None;
    let mut external_mode = None;
    let mut on_click = None;

    for prop in props {
        match prop.name.as_str() {
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "i18n" => i18n = Some(parse_i18n_key_prop(&prop.name, &prop.value)?),
            "description" => description = Some(parse_required_string(&prop.name, &prop.value)?),
            "descriptionI18n" => {
                description_i18n = Some(parse_i18n_key_prop(&prop.name, &prop.value)?)
            }
            "href" if allow_navigation => {
                href = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "navigate" if allow_navigation => {
                navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?)
            }
            "target" if allow_navigation => {
                target = Some(parse_web_target(&prop.name, &prop.value)?)
            }
            "externalMode" if allow_navigation => {
                external_mode = Some(parse_native_external_mode(&prop.name, &prop.value)?)
            }
            "onClick" if allow_navigation => {
                on_click = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::NavMenu,
                    &prop.name,
                ));
            }
        }
    }

    if href.is_some() && on_click.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "`href` and `onClick` cannot be used on the same NavMenu entry",
        ));
    }
    if description_i18n.is_some() && description.is_none() {
        return Err(ComponentError::invalid_prop_combination(
            "`descriptionI18n` requires `description` on a NavMenu entry",
        ));
    }
    let navigation =
        parse_link_navigation_props("NavMenu entry", href, navigate, target, external_mode)?;
    Ok(NavMenuItemProps {
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        i18n,
        description,
        description_i18n,
        icon,
        on_click,
        navigation,
    })
}
