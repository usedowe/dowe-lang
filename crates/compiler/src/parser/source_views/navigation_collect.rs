fn collect_sections(path: &Path, tree: &ViewNode) -> DoweResult<Vec<ViewSection>> {
    let mut sections = Vec::new();
    let mut seen = HashSet::new();
    collect_sections_from_node(path, tree, &mut sections, &mut seen)?;
    Ok(sections)
}

fn collect_sections_from_node(
    path: &Path,
    node: &ViewNode,
    sections: &mut Vec<ViewSection>,
    seen: &mut HashSet<String>,
) -> DoweResult<()> {
    if let Some(id) = node_element_props(node).and_then(|props| props.id.as_ref()) {
        if !seen.insert(id.clone()) {
            return Err(DoweError::at_path(
                path,
                format!("duplicate section id `{id}` in route"),
            ));
        }
        sections.push(ViewSection { id: id.clone() });
    }
    for group in node_child_groups(node) {
        for child in group {
            collect_sections_from_node(path, child, sections, seen)?;
        }
    }
    Ok(())
}

fn collect_navigation_actions(tree: &ViewNode, route_id: &str) -> Vec<ViewNavigationAction> {
    let mut actions = Vec::new();
    collect_navigation_actions_from_node(tree, route_id, &mut actions);
    actions
}

fn collect_navigation_actions_from_node(
    node: &ViewNode,
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    if let Some(action) = navigation_action(node) {
        actions.push(ViewNavigationAction {
            id: format!("nav-{}-{}", route_id, actions.len()),
            action: action.clone(),
        });
    }
    match node {
        ViewNode::SideNav { items, .. } => {
            collect_side_nav_navigation_actions(items, route_id, actions);
        }
        ViewNode::RailNav { items, .. } => {
            for item in items {
                if let dowe_components::RailNavItem::Item(props) = item
                    && let Some(action) = props.navigation.as_ref()
                {
                    actions.push(ViewNavigationAction {
                        id: format!("nav-{}-{}", route_id, actions.len()),
                        action: action.clone(),
                    });
                }
            }
        }
        ViewNode::NavMenu { items, .. } => {
            collect_nav_menu_navigation_actions(items, route_id, actions);
        }
        ViewNode::Dropdown { entries, .. } => {
            collect_overlay_entry_navigation_actions(entries, route_id, actions);
        }
        ViewNode::Command { entries, .. } => {
            collect_command_entry_navigation_actions(entries, route_id, actions);
        }
        ViewNode::AvatarGroup { items, .. } => {
            collect_avatar_group_navigation_actions(items, route_id, actions);
        }
        ViewNode::Fab {
            actions: fab_actions,
            ..
        } => {
            for action in fab_actions {
                if let Some(navigation) = action.navigation.as_ref() {
                    actions.push(ViewNavigationAction {
                        id: format!("nav-{}-{}", route_id, actions.len()),
                        action: navigation.clone(),
                    });
                }
            }
        }
        _ => {}
    }
    for group in node_child_groups(node) {
        for child in group {
            collect_navigation_actions_from_node(child, route_id, actions);
        }
    }
}

fn collect_avatar_group_navigation_actions(
    items: &[dowe_components::AvatarGroupItem],
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    for item in items {
        if let Some(action) = item.navigation.as_ref() {
            actions.push(ViewNavigationAction {
                id: format!("nav-{}-{}", route_id, actions.len()),
                action: action.clone(),
            });
        }
    }
}

fn collect_side_nav_navigation_actions(
    items: &[dowe_components::SideNavItem],
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    for item in items {
        match item {
            dowe_components::SideNavItem::Header(props)
            | dowe_components::SideNavItem::Item(props) => {
                if let Some(action) = props.navigation.as_ref() {
                    actions.push(ViewNavigationAction {
                        id: format!("nav-{}-{}", route_id, actions.len()),
                        action: action.clone(),
                    });
                }
            }
            dowe_components::SideNavItem::Submenu { items, .. } => {
                for props in items {
                    if let Some(action) = props.navigation.as_ref() {
                        actions.push(ViewNavigationAction {
                            id: format!("nav-{}-{}", route_id, actions.len()),
                            action: action.clone(),
                        });
                    }
                }
            }
            dowe_components::SideNavItem::Divider => {}
        }
    }
}

fn collect_nav_menu_navigation_actions(
    items: &[dowe_components::NavMenuItem],
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    for item in items {
        match item {
            dowe_components::NavMenuItem::Item(props) => {
                collect_nav_menu_entry_navigation_action(props, route_id, actions);
            }
            dowe_components::NavMenuItem::Submenu { items, .. } => {
                for props in items {
                    collect_nav_menu_entry_navigation_action(props, route_id, actions);
                }
            }
            dowe_components::NavMenuItem::Megamenu { .. } => {}
        }
    }
}

fn collect_nav_menu_entry_navigation_action(
    props: &dowe_components::NavMenuItemProps,
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    if let Some(action) = props.navigation.as_ref() {
        actions.push(ViewNavigationAction {
            id: format!("nav-{}-{}", route_id, actions.len()),
            action: action.clone(),
        });
    }
}

fn collect_overlay_entry_navigation_actions(
    entries: &[dowe_components::OverlayEntry],
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    for entry in entries {
        if let dowe_components::OverlayEntry::Item(props) = entry {
            collect_overlay_item_navigation_action(props, route_id, actions);
        }
    }
}

fn collect_command_entry_navigation_actions(
    entries: &[dowe_components::CommandEntry],
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    for entry in entries {
        match entry {
            dowe_components::CommandEntry::Item(props) => {
                collect_overlay_item_navigation_action(props, route_id, actions)
            }
            dowe_components::CommandEntry::Group { items, .. } => {
                for props in items {
                    collect_overlay_item_navigation_action(props, route_id, actions);
                }
            }
        }
    }
}

fn collect_overlay_item_navigation_action(
    props: &dowe_components::OverlayItemProps,
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    if let Some(action) = props.navigation.as_ref() {
        actions.push(ViewNavigationAction {
            id: format!("nav-{}-{}", route_id, actions.len()),
            action: action.clone(),
        });
    }
}

