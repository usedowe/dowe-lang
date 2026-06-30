pub fn tabs_component_node(
    props: Vec<ComponentProp>,
    tabs: Vec<TabItem>,
) -> ComponentResult<ViewNode> {
    if tabs.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Tabs requires at least one tab",
        ));
    }
    let mut seen = BTreeSet::new();
    for tab in &tabs {
        if !seen.insert(tab.id.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate Tabs tab id `{}`",
                tab.id
            )));
        }
        if tab.children.is_empty() {
            return Err(ComponentError::invalid_prop_combination(format!(
                "Tabs tab `{}` requires at least one child",
                tab.id
            )));
        }
    }
    let props = parse_tabs_props(BuiltinComponent::Tabs, &props)?;
    Ok(ViewNode::Tabs { props, tabs })
}

pub fn tabs_tab_component(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
) -> ComponentResult<TabItem> {
    let mut id = None;
    let mut label = None;
    for prop in props {
        match prop.name.as_str() {
            "id" => id = Some(parse_id_prop(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::Tab,
                    &prop.name,
                ));
            }
        }
    }
    let id = id.ok_or_else(|| ComponentError::invalid_prop("id", "portable tab id"))?;
    if children.is_empty() {
        return Err(ComponentError::invalid_prop_combination(format!(
            "Tabs tab `{id}` requires at least one child"
        )));
    }
    Ok(TabItem {
        id,
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        children,
    })
}

pub fn bar_component_node(
    component: BuiltinComponent,
    props: Vec<ComponentProp>,
    start: Vec<ViewNode>,
    center: Vec<ViewNode>,
    end: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if start.is_empty() && center.is_empty() && end.is_empty() {
        return Err(ComponentError::invalid_prop_combination(format!(
            "{} requires at least one region with content",
            component.as_str()
        )));
    }
    if !allow_children
        && (contains_children(&start) || contains_children(&center) || contains_children(&end))
    {
        return Err(ComponentError::children_outside_layout());
    }
    let props = parse_bar_props(component, &props)?;
    match component {
        BuiltinComponent::AppBar => Ok(ViewNode::AppBar {
            props,
            start,
            center,
            end,
        }),
        BuiltinComponent::Footer => Ok(ViewNode::Footer {
            props,
            start,
            center,
            end,
        }),
        BuiltinComponent::BottomBar => Ok(ViewNode::BottomBar {
            props,
            start,
            center,
            end,
        }),
        _ => Err(ComponentError::invalid_prop("component", "bar component")),
    }
}

pub fn side_nav_component_node(
    props: Vec<ComponentProp>,
    items: Vec<SideNavItem>,
) -> ComponentResult<ViewNode> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "SideNav requires at least one entry",
        ));
    }
    let mut size = None;
    let mut wide = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "size" => {
                size = Some(parse_side_nav_size_prop(&prop.name, &prop.value)?);
            }
            "wide" => wide = parse_static_bool(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::SideNav, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Ghost);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::SideNav {
        props: SideNavProps {
            style,
            size: size.unwrap_or(SideNavSize::Md),
            wide,
        },
        items,
    })
}

pub fn sidebar_component_node(
    props: Vec<ComponentProp>,
    header: Vec<ViewNode>,
    body: Vec<ViewNode>,
    footer: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if body.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Sidebar requires body children",
        ));
    }
    reject_children_placeholder(BuiltinComponent::Sidebar, &header, allow_children)?;
    reject_children_placeholder(BuiltinComponent::Sidebar, &body, allow_children)?;
    reject_children_placeholder(BuiltinComponent::Sidebar, &footer, allow_children)?;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "color" => {
                return Err(ComponentError::new(
                    "unknown prop `color` on `Sidebar`; use `scheme` for visual family",
                ));
            }
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Sidebar, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Ghost);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::Sidebar {
        props: SidebarProps { style },
        header,
        body,
        footer,
    })
}

pub fn nav_menu_component_node(
    props: Vec<ComponentProp>,
    items: Vec<NavMenuItem>,
) -> ComponentResult<ViewNode> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "NavMenu requires at least one entry",
        ));
    }
    let mut size = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "size" => size = Some(parse_side_nav_size_prop(&prop.name, &prop.value)?),
            "color" => {
                return Err(ComponentError::new(
                    "unknown prop `color` on `NavMenu`; use `scheme` for visual family",
                ));
            }
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::NavMenu, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Ghost);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::NavMenu {
        props: NavMenuProps {
            style,
            size: size.unwrap_or(SideNavSize::Md),
        },
        items,
    })
}

pub fn scaffold_component_node(
    props: Vec<ComponentProp>,
    app_bar: Vec<ViewNode>,
    start: Vec<ViewNode>,
    main: Vec<ViewNode>,
    end: Vec<ViewNode>,
    bottom_bar: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if main.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Scaffold requires a main region with content",
        ));
    }
    if !allow_children
        && (contains_children(&app_bar)
            || contains_children(&start)
            || contains_children(&main)
            || contains_children(&end)
            || contains_children(&bottom_bar))
    {
        return Err(ComponentError::children_outside_layout());
    }
    let mut boxed = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "boxed" => boxed = parse_static_bool(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    Ok(ViewNode::Scaffold {
        props: ScaffoldProps {
            style: parse_style_props(
                BuiltinComponent::Scaffold,
                &style_props,
                StylePropMode::Variant,
            )?,
            boxed,
        },
        app_bar,
        start,
        main,
        end,
        bottom_bar,
    })
}

pub fn drawer_component_node(
    props: Vec<ComponentProp>,
    header: Vec<ViewNode>,
    body: Vec<ViewNode>,
    footer: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if body.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Drawer requires body children",
        ));
    }
    reject_children_placeholder(BuiltinComponent::Drawer, &header, allow_children)?;
    reject_children_placeholder(BuiltinComponent::Drawer, &body, allow_children)?;
    reject_children_placeholder(BuiltinComponent::Drawer, &footer, allow_children)?;
    let mut open = None;
    let mut position = DrawerPosition::Start;
    let mut disable_overlay_close = false;
    let mut hide_close_button = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "open" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                if !is_reference_path(&value) {
                    return Err(ComponentError::invalid_prop("open", "signal bool path"));
                }
                open = Some(value);
            }
            "position" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                position = DrawerPosition::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("position", "start, end, top or bottom")
                })?;
            }
            "disableOverlayClose" => {
                disable_overlay_close = parse_static_bool(&prop.name, &prop.value)?
            }
            "hideCloseButton" => hide_close_button = parse_static_bool(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::Drawer, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Surface);
    Ok(ViewNode::Drawer {
        props: DrawerProps {
            style,
            open: open.ok_or_else(|| ComponentError::invalid_prop("open", "signal bool path"))?,
            position,
            disable_overlay_close,
            hide_close_button,
        },
        header,
        body,
        footer,
    })
}
