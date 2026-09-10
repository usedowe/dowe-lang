pub fn compose_tree(layout: &ViewNode, page: &ViewNode) -> ViewNode {
    let mut composed = layout.clone();
    compose_tree_in_place(&mut composed, page);
    composed
}

fn compose_tree_in_place(node: &mut ViewNode, page: &ViewNode) {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => {
            compose_children_in_place(content, page);
            compose_children_in_place(children, page);
        }
        ViewNode::Scope { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Marquee { children, .. }
        | ViewNode::Collapsible { children, .. }
        | ViewNode::Button { children, .. }
        | ViewNode::Brand { children, .. }
        | ViewNode::Banner { children, .. }
        | ViewNode::Each { children, .. } => compose_children_in_place(children, page),
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                compose_children_in_place(&mut tab.children, page);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                if let NavMenuItem::Megamenu { content, .. } = item {
                    compose_children_in_place(content, page);
                }
            }
        }
        ViewNode::Drawer {
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
        }
        | ViewNode::Sidebar {
            header,
            body,
            footer,
            ..
        } => {
            compose_children_in_place(header, page);
            compose_children_in_place(body, page);
            compose_children_in_place(footer, page);
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            compose_children_in_place(trigger, page);
            compose_children_in_place(header, page);
            compose_children_in_place(footer, page);
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                compose_children_in_place(&mut item.children, page);
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                compose_children_in_place(&mut slide.children, page);
            }
        }
        ViewNode::AppBar {
            top,
            start,
            center,
            end,
            bottom,
            ..
        } => {
            compose_children_in_place(top, page);
            compose_children_in_place(start, page);
            compose_children_in_place(center, page);
            compose_children_in_place(end, page);
            compose_children_in_place(bottom, page);
        }
        ViewNode::Footer {
            top,
            start,
            center,
            end,
            bottom,
            ..
        } => {
            compose_children_in_place(top, page);
            compose_children_in_place(start, page);
            compose_children_in_place(center, page);
            compose_children_in_place(end, page);
            compose_children_in_place(bottom, page);
        }
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
            ..
        } => {
            compose_children_in_place(app_bar, page);
            compose_children_in_place(start, page);
            compose_children_in_place(main, page);
            compose_children_in_place(end, page);
            compose_children_in_place(bottom_bar, page);
            compose_children_in_place(overlays, page);
        }
        ViewNode::Children => *node = page.clone(),
        _ => {}
    }
}

fn compose_children_in_place(children: &mut [ViewNode], page: &ViewNode) {
    for child in children {
        compose_tree_in_place(child, page);
    }
}

pub fn validate_view_tree(node: &ViewNode) -> ComponentResult<()> {
    validate_box_positioning(node, None, false)?;
    validate_view_tree_with_parent(node, false, None)
}

fn validate_box_positioning(
    node: &ViewNode,
    parent_box_position: Option<BoxPosition>,
    fixed_forbidden: bool,
) -> ComponentResult<()> {
    if let ViewNode::Box { props, .. } = node {
        if props.position().mode == BoxPosition::Absolute
            && parent_box_position != Some(BoxPosition::Relative)
        {
            return Err(ComponentError::invalid_prop_combination(
                "`Box position:\"absolute\"` must be a direct child of `Box position:\"relative\"`",
            ));
        }
        if props.position().mode == BoxPosition::Fixed && fixed_forbidden {
            return Err(ComponentError::invalid_prop_combination(
                "`Box position:\"fixed\"` cannot be nested inside `each` or `Splash`",
            ));
        }
    }

    let child_parent_position = match node {
        ViewNode::Box { props, .. } => Some(props.position().mode),
        _ => None,
    };
    let child_fixed_forbidden =
        fixed_forbidden || matches!(node, ViewNode::Each { .. } | ViewNode::Splash { .. });
    for group in node_child_groups(node) {
        for child in group {
            validate_box_positioning(child, child_parent_position, child_fixed_forbidden)?;
        }
    }
    Ok(())
}

