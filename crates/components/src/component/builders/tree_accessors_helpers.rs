fn nav_menu_child_group(item: &NavMenuItem) -> Option<&[ViewNode]> {
    match item {
        NavMenuItem::Megamenu { content, .. } => Some(content.as_slice()),
        NavMenuItem::Item(_) | NavMenuItem::Submenu { .. } => None,
    }
}

fn nav_menu_child_group_mut(item: &mut NavMenuItem) -> Option<&mut [ViewNode]> {
    match item {
        NavMenuItem::Megamenu { content, .. } => Some(content.as_mut_slice()),
        NavMenuItem::Item(_) | NavMenuItem::Submenu { .. } => None,
    }
}

fn overlay_entry_first_text(item: &OverlayEntry) -> Option<String> {
    match item {
        OverlayEntry::Item(props) => Some(props.label.clone()),
        OverlayEntry::Divider => None,
    }
}

fn command_entry_first_text(item: &CommandEntry) -> Option<String> {
    match item {
        CommandEntry::Item(props) => Some(props.label.clone()),
        CommandEntry::Group { label, items, .. } => items
            .iter()
            .find_map(|item| Some(item.label.clone()))
            .or_else(|| Some(label.clone())),
    }
}

fn side_nav_first_text(item: &SideNavItem) -> Option<String> {
    match item {
        SideNavItem::Header(props) | SideNavItem::Item(props) => Some(props.label.clone()),
        SideNavItem::Submenu { props, .. } => Some(props.label.clone()),
        SideNavItem::Divider => None,
    }
}

fn rail_nav_first_text(item: &RailNavItem) -> Option<String> {
    match item {
        RailNavItem::Item(props) => Some(props.label.clone()),
        RailNavItem::Divider => None,
    }
}

fn nav_menu_first_text(item: &NavMenuItem) -> Option<String> {
    match item {
        NavMenuItem::Item(props)
        | NavMenuItem::Submenu { props, .. }
        | NavMenuItem::Megamenu { props, .. } => Some(props.label.clone()),
    }
}

fn prop_value_string(name: &str, value: &PropValue) -> ComponentResult<String> {
    match value {
        PropValue::String(value) => Ok(value.clone()),
        PropValue::Boolean(value) => Ok(value.to_string()),
        PropValue::Number(value) => Ok(value.clone()),
        PropValue::Responsive(_) | PropValue::Binding(_) => Err(ComponentError::invalid_prop(name, "static scalar")),
    }
}
