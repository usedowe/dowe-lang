fn collect_navigation_node_classes(node: &ViewNode, classes: &mut BTreeSet<String>) {
    match node {
        ViewNode::AppBar {
            props,
            start,
            center,
            end,
        } => {
            collect_bar_classes("appbar", props, start, center, end, classes);
        }
        ViewNode::Footer {
            props,
            start,
            center,
            end,
        } => {
            collect_bar_classes("footer", props, start, center, end, classes);
        }
        ViewNode::BottomBar {
            props,
            start,
            center,
            end,
        } => {
            collect_bar_classes("bottombar", props, start, center, end, classes);
        }
        ViewNode::SideNav { props, items } => {
            classes.extend(side_nav_classes("sidenav", props));
            collect_side_nav_icon_classes(items, classes);
        }
        ViewNode::Sidebar { props, items } => {
            classes.extend(side_nav_classes("sidebar", props));
            collect_side_nav_icon_classes(items, classes);
        }
        ViewNode::NavMenu { props, items } => {
            classes.extend(nav_menu_classes(props));
            classes.insert("navmenu-item".to_string());
            classes.insert("navmenu-label".to_string());
            classes.insert("navmenu-icon".to_string());
            classes.insert("navmenu-arrow".to_string());
            classes.insert("navmenu-popover".to_string());
            classes.insert("navmenu-popover-content".to_string());
            classes.insert("navmenu-submenu-item".to_string());
            classes.insert("navmenu-submenu-icon".to_string());
            classes.insert("navmenu-submenu-content".to_string());
            classes.insert("navmenu-submenu-label".to_string());
            classes.insert("navmenu-submenu-description".to_string());
            collect_nav_menu_classes(items, classes);
        }
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
        } => {
            classes.extend(scaffold_classes(props));
            classes.insert("scaffold-body".to_string());
            classes.insert("scaffold-main".to_string());
            classes.insert("scaffold-start".to_string());
            classes.insert("scaffold-end".to_string());
            classes.insert("scaffold-content".to_string());
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
            {
                collect_classes(child, classes);
            }
        }
        ViewNode::Tabs { props, tabs } => {
            classes.extend(tabs_classes(props));
            classes.extend(tabs_list_classes(props));
            classes.insert("tab".to_string());
            classes.insert("tabs-label".to_string());
            classes.insert("tabs-wrapper".to_string());
            classes.insert("tabs-content".to_string());
            for tab in tabs {
                for child in &tab.children {
                    collect_classes(child, classes);
                }
            }
        }
        ViewNode::Drawer { props, children } => {
            classes.extend(drawer_panel_classes(props));
            classes.extend(drawer_classes(props));
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Children => {}
        _ => unreachable!(),
    }
}
