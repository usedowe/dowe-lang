fn collect_side_nav_icon_classes(items: &[SideNavItem], classes: &mut BTreeSet<String>) {
    for item in items {
        match item {
            SideNavItem::Header(props) | SideNavItem::Item(props) => {
                collect_side_nav_item_icon_classes(props, classes);
            }
            SideNavItem::Submenu { props, items, .. } => {
                collect_side_nav_item_icon_classes(props, classes);
                for props in items {
                    collect_side_nav_item_icon_classes(props, classes);
                }
            }
            SideNavItem::Divider => {}
        }
    }
}

fn collect_side_nav_item_icon_classes(props: &SideNavItemProps, classes: &mut BTreeSet<String>) {
    if let Some(icon) = props.icon.as_ref() {
        classes.extend(svg_classes(&icon.props.style));
    }
}

fn collect_nav_menu_classes(items: &[NavMenuItem], classes: &mut BTreeSet<String>) {
    for item in items {
        match item {
            NavMenuItem::Item(props) => collect_nav_menu_item_icon_classes(props, classes),
            NavMenuItem::Submenu { props, items } => {
                collect_nav_menu_item_icon_classes(props, classes);
                for props in items {
                    collect_nav_menu_item_icon_classes(props, classes);
                }
            }
            NavMenuItem::Megamenu { props, content } => {
                collect_nav_menu_item_icon_classes(props, classes);
                for child in content {
                    collect_classes(child, classes);
                }
            }
        }
    }
}

fn collect_nav_menu_item_icon_classes(props: &NavMenuItemProps, classes: &mut BTreeSet<String>) {
    if let Some(icon) = props.icon.as_ref() {
        classes.extend(svg_classes(&icon.props.style));
    }
}

fn collect_bar_classes(
    base: &str,
    props: &BarProps,
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
    classes: &mut BTreeSet<String>,
) {
    classes.extend(bar_classes(base, props));
    classes.extend(bar_content_classes(base, props));
    if !start.is_empty() {
        classes.insert(format!("{base}-start"));
    }
    if !center.is_empty() {
        classes.insert(format!("{base}-center"));
    }
    if !end.is_empty() {
        classes.insert(format!("{base}-end"));
    }
    for child in start.iter().chain(center).chain(end) {
        collect_classes(child, classes);
    }
}

fn collect_overlay_entry_classes(
    base: &str,
    entries: &[OverlayEntry],
    classes: &mut BTreeSet<String>,
) {
    for entry in entries {
        if let OverlayEntry::Item(_) = entry {
            classes.insert(format!("{base}-item"));
            classes.insert(format!("{base}-item-icon"));
            classes.insert(format!("{base}-item-content"));
            classes.insert(format!("{base}-item-label"));
            classes.insert(format!("{base}-item-description"));
        }
    }
}

fn collect_command_entry_classes(entries: &[CommandEntry], classes: &mut BTreeSet<String>) {
    for entry in entries {
        match entry {
            CommandEntry::Item(_) => {
                classes.insert("command-item".to_string());
                classes.insert("command-item-icon".to_string());
                classes.insert("command-item-content".to_string());
                classes.insert("command-item-label".to_string());
                classes.insert("command-item-description".to_string());
            }
            CommandEntry::Group { items, .. } => {
                for _ in items {
                    classes.insert("command-item".to_string());
                    classes.insert("command-item-icon".to_string());
                    classes.insert("command-item-content".to_string());
                    classes.insert("command-item-label".to_string());
                    classes.insert("command-item-description".to_string());
                }
            }
        }
    }
}
