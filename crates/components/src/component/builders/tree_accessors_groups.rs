
pub fn node_child_groups(node: &ViewNode) -> Vec<&[ViewNode]> {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => vec![content.as_slice(), children.as_slice()],
        ViewNode::AppBar {
            top,
            start,
            center,
            end,
            bottom,
            ..
        }
        | ViewNode::Footer {
            top,
            start,
            center,
            end,
            bottom,
            ..
        } => vec![
            top.as_slice(),
            start.as_slice(),
            center.as_slice(),
            end.as_slice(),
            bottom.as_slice(),
        ],
        ViewNode::BottomBar { .. } => Vec::new(),
        ViewNode::Tabs { tabs, .. } => tabs
            .iter()
            .map(|tab| tab.children.as_slice())
            .collect::<Vec<_>>(),
        ViewNode::NavMenu { items, .. } => items.iter().filter_map(nav_menu_child_group).collect(),
        ViewNode::SideNav { .. } | ViewNode::RailNav { .. } => Vec::new(),
        ViewNode::Sidebar {
            header,
            body,
            footer,
            ..
        } => vec![header.as_slice(), body.as_slice(), footer.as_slice()],
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => vec![header.as_slice(), body.as_slice(), footer.as_slice()],
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => vec![header.as_slice(), body.as_slice(), footer.as_slice()],
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => vec![trigger.as_slice(), header.as_slice(), footer.as_slice()],
        ViewNode::Command { .. } => Vec::new(),
        ViewNode::Accordion { items, .. } => items
            .iter()
            .map(|item| item.children.as_slice())
            .collect::<Vec<_>>(),
        ViewNode::Collapsible { children, .. } => vec![children.as_slice()],
        ViewNode::Carousel { slides, .. } => slides
            .iter()
            .map(|slide| slide.children.as_slice())
            .collect::<Vec<_>>(),
        ViewNode::RadioGroup { .. } | ViewNode::ToggleGroup { .. } => Vec::new(),
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
            ..
        } => vec![
            app_bar.as_slice(),
            start.as_slice(),
            main.as_slice(),
            end.as_slice(),
            bottom_bar.as_slice(),
            overlays.as_slice(),
        ],
        _ => vec![node_children(node)],
    }
}

pub fn node_child_groups_mut(node: &mut ViewNode) -> Vec<&mut [ViewNode]> {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => vec![content.as_mut_slice(), children.as_mut_slice()],
        ViewNode::AppBar {
            top,
            start,
            center,
            end,
            bottom,
            ..
        }
        | ViewNode::Footer {
            top,
            start,
            center,
            end,
            bottom,
            ..
        } => vec![
            top.as_mut_slice(),
            start.as_mut_slice(),
            center.as_mut_slice(),
            end.as_mut_slice(),
            bottom.as_mut_slice(),
        ],
        ViewNode::Tabs { tabs, .. } => tabs
            .iter_mut()
            .map(|tab| tab.children.as_mut_slice())
            .collect(),
        ViewNode::NavMenu { items, .. } => items
            .iter_mut()
            .filter_map(nav_menu_child_group_mut)
            .collect(),
        ViewNode::Sidebar {
            header,
            body,
            footer,
            ..
        }
        | ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        }
        | ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => vec![
            header.as_mut_slice(),
            body.as_mut_slice(),
            footer.as_mut_slice(),
        ],
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => vec![
            trigger.as_mut_slice(),
            header.as_mut_slice(),
            footer.as_mut_slice(),
        ],
        ViewNode::Accordion { items, .. } => items
            .iter_mut()
            .map(|item| item.children.as_mut_slice())
            .collect(),
        ViewNode::Carousel { slides, .. } => slides
            .iter_mut()
            .map(|slide| slide.children.as_mut_slice())
            .collect(),
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
            ..
        } => vec![
            app_bar.as_mut_slice(),
            start.as_mut_slice(),
            main.as_mut_slice(),
            end.as_mut_slice(),
            bottom_bar.as_mut_slice(),
            overlays.as_mut_slice(),
        ],
        ViewNode::Scope { children, .. }
        | ViewNode::Each { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Marquee { children, .. }
        | ViewNode::Collapsible { children, .. }
        | ViewNode::Brand { children, .. }
        | ViewNode::Banner { children, .. }
        | ViewNode::Button { children, .. } => vec![children.as_mut_slice()],
        _ => Vec::new(),
    }
}

