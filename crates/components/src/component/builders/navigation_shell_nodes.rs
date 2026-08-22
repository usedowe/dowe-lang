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
    let mut i18n = None;
    for prop in props {
        match prop.name.as_str() {
            "id" => id = Some(parse_id_prop(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "i18n" => i18n = Some(parse_i18n_key_prop(&prop.name, &prop.value)?),
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
        i18n,
        children,
    })
}

pub fn stepper_component_node(
    props: Vec<ComponentProp>,
    steps: Vec<TabItem>,
) -> ComponentResult<ViewNode> {
    if steps.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Stepper requires at least one step",
        ));
    }
    let mut seen = BTreeSet::new();
    for step in &steps {
        if !seen.insert(step.id.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate Stepper step id `{}`",
                step.id
            )));
        }
        if step.children.is_empty() {
            return Err(ComponentError::invalid_prop_combination(format!(
                "Stepper step `{}` requires at least one child",
                step.id
            )));
        }
    }
    let props = parse_stepper_props(&props)?;
    Ok(ViewNode::Tabs { props, tabs: steps })
}

pub fn stepper_step_component(
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
) -> ComponentResult<TabItem> {
    let mut id = None;
    let mut label = None;
    let mut i18n = None;
    for prop in props {
        match prop.name.as_str() {
            "id" => id = Some(parse_id_prop(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "i18n" => i18n = Some(parse_i18n_key_prop(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::Step,
                    &prop.name,
                ));
            }
        }
    }
    let id = id.ok_or_else(|| ComponentError::invalid_prop("id", "portable step id"))?;
    if children.is_empty() {
        return Err(ComponentError::invalid_prop_combination(format!(
            "Stepper step `{id}` requires at least one child"
        )));
    }
    Ok(TabItem {
        id,
        label: label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?,
        i18n,
        children,
    })
}

pub fn bar_component_node(
    component: BuiltinComponent,
    props: Vec<ComponentProp>,
    top: Vec<ViewNode>,
    start: Vec<ViewNode>,
    center: Vec<ViewNode>,
    end: Vec<ViewNode>,
    bottom: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if top.is_empty()
        && start.is_empty()
        && center.is_empty()
        && end.is_empty()
        && bottom.is_empty()
    {
        return Err(ComponentError::invalid_prop_combination(format!(
            "{} requires at least one region with content",
            component.as_str()
        )));
    }
    if !allow_children
        && (contains_children(&top)
            || contains_children(&start)
            || contains_children(&center)
            || contains_children(&end)
            || contains_children(&bottom))
    {
        return Err(ComponentError::children_outside_layout());
    }
    let props = parse_bar_props(component, &props)?;
    match component {
        BuiltinComponent::AppBar => Ok(ViewNode::AppBar {
            props,
            top,
            start,
            center,
            end,
            bottom,
        }),
        BuiltinComponent::Footer => Ok(ViewNode::Footer {
            props,
            top,
            start,
            center,
            end,
            bottom,
        }),
        BuiltinComponent::BottomBar => Err(ComponentError::invalid_prop_combination(
            "BottomBar requires tab entries",
        )),
        _ => Err(ComponentError::invalid_prop("component", "bar component")),
    }
}

pub fn bottom_bar_component_node(
    props: Vec<ComponentProp>,
    tabs: Vec<BottomBarTab>,
) -> ComponentResult<ViewNode> {
    if tabs.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "BottomBar requires at least one tab",
        ));
    }
    if tabs.iter().filter(|tab| tab.featured).count() > 1 {
        return Err(ComponentError::invalid_prop_combination(
            "BottomBar accepts at most one featured tab",
        ));
    }
    Ok(ViewNode::BottomBar {
        props: parse_bar_props(BuiltinComponent::BottomBar, &props)?,
        tabs,
    })
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
    let mut reactive_wide = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "size" => {
                if reactive_reference(&prop.value).is_some() {
                    style_props.push(prop);
                } else {
                    size = Some(parse_side_nav_size_prop(&prop.name, &prop.value)?);
                }
            }
            "wide" => {
                if let Some(reference) = reactive_reference(&prop.value) {
                    reactive_wide = Some(reference);
                } else {
                    wide = parse_static_bool(&prop.name, &prop.value)?;
                }
            }
            _ => style_props.push(prop),
        }
    }
    let style = parse_variant_props(BuiltinComponent::SideNav, &style_props)?;
    Ok(ViewNode::SideNav {
        props: SideNavProps {
            style,
            size: size.unwrap_or(SideNavSize::Md),
            wide,
            reactive_wide,
        },
        items,
    })
}

pub fn rail_nav_component_node(
    props: Vec<ComponentProp>,
    items: Vec<RailNavItem>,
) -> ComponentResult<ViewNode> {
    if items.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "RailNav requires at least one entry",
        ));
    }
    let mut size = None;
    let mut show_labels = false;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "size" => size = Some(parse_side_nav_size_prop(&prop.name, &prop.value)?),
            "showLabels" => show_labels = parse_static_bool(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    let mut style = parse_variant_props(BuiltinComponent::RailNav, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Ghost);
    style.color.get_or_insert(ColorFamily::Muted);
    Ok(ViewNode::RailNav {
        props: RailNavProps {
            style,
            size: size.unwrap_or(SideNavSize::Md),
            show_labels,
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
    let style = parse_variant_props(BuiltinComponent::Sidebar, &style_props)?;
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
    let style = parse_variant_props(BuiltinComponent::NavMenu, &style_props)?;
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
    overlays: Vec<ViewNode>,
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
            || contains_children(&bottom_bar)
            || contains_children(&overlays))
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
        overlays,
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
    let style = parse_variant_props(BuiltinComponent::Drawer, &style_props)?;
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
