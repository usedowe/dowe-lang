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

fn validate_view_tree_with_parent(
    node: &ViewNode,
    parent_is_grid: bool,
    parent_columns: Option<u16>,
) -> ComponentResult<()> {
    if let Some(style) = node_style_props(node) {
        let grid_item = style.grid_item();
        let has_span = grid_item.col_span.is_some() || grid_item.row_span.is_some();
        if has_span && !parent_is_grid {
            return Err(ComponentError::invalid_prop_combination(
                "`colSpan` and `rowSpan` can only be used on `Box`, `Section` or `Card` children of `Grid`",
            ));
        }
        if parent_is_grid
            && let Some(columns) = parent_columns
            && let Some(span) = grid_item.col_span.as_ref()
            && span.entries.iter().any(|entry| entry.value.0 > columns)
        {
            return Err(ComponentError::invalid_prop(
                "colSpan",
                "value not greater than parent grid columns",
            ));
        }
    }

    match node {
        ViewNode::Splash {
            content, children, ..
        } => {
            for child in content.iter().chain(children) {
                validate_view_tree_with_parent(child, false, None)?;
            }
        }
        ViewNode::Scope { children, .. } | ViewNode::Each { children, .. } => {
            for child in children {
                validate_view_tree_with_parent(child, false, None)?;
            }
        }
        ViewNode::Grid { props, children } => {
            let columns = grid_static_columns(props);
            for child in children {
                validate_view_tree_with_parent(child, true, columns)?;
            }
        }
        ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Marquee { children, .. }
        | ViewNode::Collapsible { children, .. }
        | ViewNode::Brand { children, .. }
        | ViewNode::Banner { children, .. }
        | ViewNode::Button { children, .. } => {
            for child in children {
                validate_view_tree_with_parent(child, false, None)?;
            }
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                validate_view_tree_with_parent(child, false, None)?;
            }
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                validate_view_tree_with_parent(child, false, None)?;
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            for child in trigger.iter().chain(header).chain(footer) {
                validate_view_tree_with_parent(child, false, None)?;
            }
        }
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                for child in &tab.children {
                    validate_view_tree_with_parent(child, false, None)?;
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                for child in &item.children {
                    validate_view_tree_with_parent(child, false, None)?;
                }
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                for child in &slide.children {
                    validate_view_tree_with_parent(child, false, None)?;
                }
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                validate_nav_menu_item(item)?;
            }
        }
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
        } => {
            for child in top.iter().chain(start).chain(center).chain(end).chain(bottom) {
                validate_view_tree_with_parent(child, false, None)?;
            }
        }
        ViewNode::BottomBar { .. } | ViewNode::SideNav { .. } | ViewNode::RailNav { .. } => {}
        ViewNode::Sidebar {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                validate_view_tree_with_parent(child, false, None)?;
            }
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
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
                .chain(overlays)
            {
                validate_view_tree_with_parent(child, false, None)?;
            }
        }
        ViewNode::Input { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::SelectTheme { .. }
        | ViewNode::Fab { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::ComboBox { .. }
        | ViewNode::CsvField { .. }
        | ViewNode::DragDrop { .. }
        | ViewNode::Editor { .. }
        | ViewNode::ImageCropper { .. }
        | ViewNode::PasswordField { .. }
        | ViewNode::PhoneField { .. }
        | ViewNode::PinField { .. }
        | ViewNode::Textarea { .. }
        | ViewNode::Select { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Iframe { .. }
        | ViewNode::Device { .. }
        | ViewNode::Canvas { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. }
        | ViewNode::Table { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Alert { .. }
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
        | ViewNode::TypeWriter { .. }
        | ViewNode::Children => {}
    }

    Ok(())
}


fn validate_nav_menu_item(item: &NavMenuItem) -> ComponentResult<()> {
    if let NavMenuItem::Megamenu { content, .. } = item {
        for child in content {
            validate_view_tree_with_parent(child, false, None)?;
        }
    }
    Ok(())
}

fn node_style_props(node: &ViewNode) -> Option<&StyleProps> {
    match node {
        ViewNode::Box { props, .. } | ViewNode::Section { props, .. } => Some(props),
        ViewNode::Card { props, .. } => Some(&props.style),
        ViewNode::Drawer { props, .. } => Some(&props.style.style),
        ViewNode::Avatar { props, .. } => Some(&props.style.style),
        ViewNode::AvatarGroup { props, .. } => Some(&props.style.style),
        ViewNode::ChatBox { props } => Some(&props.style.style),
        ViewNode::Empty { props } => Some(&props.style.style),
        ViewNode::Marquee { props, .. } => Some(&props.style),
        ViewNode::Badge { props, .. } => Some(&props.style.style),
        ViewNode::Chip { props, .. } => Some(&props.style.style),
        ViewNode::Modal { props, .. } => Some(&props.style.style),
        ViewNode::AlertDialog { props } => Some(&props.style.style),
        ViewNode::Tooltip { props, .. } => Some(&props.style.style),
        ViewNode::Toast { props } => Some(&props.style.style),
        ViewNode::Dropdown { props, .. } => Some(&props.style.style),
        ViewNode::Command { props, .. } => Some(&props.style.style),
        ViewNode::Audio { props } => Some(&props.style.style),
        ViewNode::Image { props } => Some(&props.style.style),
        ViewNode::Accordion { props, .. } => Some(&props.style.style),
        ViewNode::Carousel { props, .. } => Some(&props.style.style),
        ViewNode::Checkbox { props } => Some(&props.style.style),
        ViewNode::Color { props } => Some(&props.style.style),
        ViewNode::Date { props } => Some(&props.style.style),
        ViewNode::DateRange { props } => Some(&props.style.style),
        ViewNode::RadioGroup { props, .. } => Some(&props.style.style),
        ViewNode::Toggle { props } => Some(&props.style.style),
        ViewNode::ToggleTheme { props } => Some(&props.style.style),
        ViewNode::SelectTheme { props } => Some(&props.style.style),
        ViewNode::Fab { props, .. } => Some(&props.style.style),
        ViewNode::Slider { props } => Some(&props.style.style),
        ViewNode::Dropzone { props } => Some(&props.style.style),
        ViewNode::ComboBox { props, .. } => Some(&props.style.style),
        ViewNode::CsvField { props, .. } => Some(&props.style.style),
        ViewNode::DragDrop { props, .. } => Some(&props.style.style),
        ViewNode::Editor { props } => Some(&props.style.style),
        ViewNode::ImageCropper { props } => Some(&props.style.style),
        ViewNode::PasswordField { props } => Some(&props.style.style),
        ViewNode::PhoneField { props } => Some(&props.style.style),
        ViewNode::PinField { props } => Some(&props.style.style),
        ViewNode::Textarea { props } => Some(&props.style.style),
        ViewNode::Skeleton { props } => Some(&props.style),
        ViewNode::Code { props } => Some(&props.style.style),
        ViewNode::Video { props } => Some(&props.style.style),
        ViewNode::Iframe { props } => Some(&props.style),
        ViewNode::Device { props, .. } => Some(&props.style),
        ViewNode::Canvas { props } => Some(&props.style),
        ViewNode::Candlestick { props } => Some(&props.style.style),
        ViewNode::ArcChart { props } => Some(&props.common.style.style),
        ViewNode::AreaChart { props } => Some(&props.common.style.style),
        ViewNode::BarChart { props } => Some(&props.common.style.style),
        ViewNode::LineChart { props } => Some(&props.common.style.style),
        ViewNode::PieChart { props } => Some(&props.common.style.style),
        ViewNode::Table { props } => Some(&props.style.style),
        ViewNode::Divider { props } => Some(&props.style),
        ViewNode::TypeWriter { props, .. } => Some(&props.style),
        _ => None,
    }
}

fn grid_static_columns(props: &GridProps) -> Option<u16> {
    let columns = props.columns.as_ref()?;
    let mut count = None;
    for entry in &columns.entries {
        let current = entry.value.count()?;
        if let Some(existing) = count {
            if existing != current {
                return None;
            }
        } else {
            count = Some(current);
        }
    }
    count
}

fn container_node(
    component: BuiltinComponent,
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
    allow_children: bool,
    style: StyleProps,
) -> ComponentResult<ViewNode> {
    if !props.is_empty() {
        return Err(ComponentError::unknown_prop(component, &props[0].name));
    }
    if contains_children(&children) && !allow_children {
        return Err(ComponentError::children_outside_layout());
    }
    match component {
        BuiltinComponent::Section => Ok(ViewNode::Section {
            props: style,
            children,
        }),
        _ => Ok(ViewNode::Box {
            props: style,
            children,
        }),
    }
}

pub fn fixed_fab_nodes(node: &ViewNode) -> Vec<&ViewNode> {
    fn collect<'a>(node: &'a ViewNode, fabs: &mut Vec<&'a ViewNode>) {
        if matches!(node, ViewNode::Fab { props, .. } if props.fixed) {
            fabs.push(node);
            return;
        }
        for group in node_child_groups(node) {
            for child in group {
                collect(child, fabs);
            }
        }
    }

    let mut fabs = Vec::new();
    collect(node, &mut fabs);
    fabs
}

pub fn fixed_box_nodes(node: &ViewNode) -> Vec<&ViewNode> {
    fn collect<'a>(node: &'a ViewNode, boxes: &mut Vec<&'a ViewNode>) {
        if matches!(node, ViewNode::Box { props, .. } if props.position().mode == BoxPosition::Fixed)
        {
            boxes.push(node);
            return;
        }
        for group in node_child_groups(node) {
            for child in group {
                collect(child, boxes);
            }
        }
    }

    let mut boxes = Vec::new();
    collect(node, &mut boxes);
    boxes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StylePropMode {
    Box,
    Banner,
    Section,
    Layout,
    Grid,
    Card,
    Variant,
    Text,
}
