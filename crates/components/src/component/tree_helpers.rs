fn stable_slug(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:x}", hash)[..8].to_string()
}

fn static_text(value: impl AsRef<str>, component: BuiltinComponent) -> ComponentResult<String> {
    let value = value.as_ref().trim().to_string();
    if value.is_empty() {
        Err(ComponentError::text_requires_static_text(component))
    } else {
        Ok(value)
    }
}

fn reject_children_placeholder(
    component: BuiltinComponent,
    children: &[ViewNode],
    allow_children: bool,
) -> ComponentResult<()> {
    if contains_children(children) {
        if allow_children {
            Err(ComponentError::children_not_allowed(component))
        } else {
            Err(ComponentError::children_outside_layout())
        }
    } else {
        Ok(())
    }
}

fn contains_children(nodes: &[ViewNode]) -> bool {
    nodes.iter().any(|node| match node {
        ViewNode::Children => true,
        ViewNode::Scope { children, .. } | ViewNode::Each { children, .. } => {
            contains_children(children)
        }
        ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Drawer { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Marquee { children, .. }
        | ViewNode::Collapsible { children, .. }
        | ViewNode::Button { children, .. } => contains_children(children),
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => contains_children(header) || contains_children(body) || contains_children(footer),
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => contains_children(trigger) || contains_children(header) || contains_children(footer),
        ViewNode::Accordion { items, .. } => items
            .iter()
            .any(|item| contains_children(&item.children)),
        ViewNode::Carousel { slides, .. } => slides
            .iter()
            .any(|slide| contains_children(&slide.children)),
        ViewNode::Tabs { tabs, .. } => tabs.iter().any(|tab| contains_children(&tab.children)),
        ViewNode::NavMenu { items, .. } => items.iter().any(nav_menu_contains_children),
        ViewNode::AppBar {
            start, center, end, ..
        }
        | ViewNode::Footer {
            start, center, end, ..
        }
        | ViewNode::BottomBar {
            start, center, end, ..
        } => contains_children(start) || contains_children(center) || contains_children(end),
        ViewNode::SideNav { .. } | ViewNode::Sidebar { .. } => false,
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            ..
        } => {
            contains_children(app_bar)
                || contains_children(start)
                || contains_children(main)
                || contains_children(end)
                || contains_children(bottom_bar)
        }
        ViewNode::Input { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::Fab { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Select { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::Table { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Alert { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Avatar { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. }
        | ViewNode::Chip { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Command { .. }
        | ViewNode::Svg { .. }
        | ViewNode::Title { .. }
        | ViewNode::Text { .. }
        | ViewNode::TypeWriter { .. } => false,
    })
}

fn nav_menu_contains_children(item: &NavMenuItem) -> bool {
    match item {
        NavMenuItem::Megamenu { content, .. } => contains_children(content),
        NavMenuItem::Item(_) | NavMenuItem::Submenu { .. } => false,
    }
}

fn is_text_like(node: &ViewNode) -> bool {
    matches!(node, ViewNode::Text { .. } | ViewNode::Title { .. })
}
