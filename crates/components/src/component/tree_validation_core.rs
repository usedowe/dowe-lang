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
            for child in top
                .iter()
                .chain(start)
                .chain(center)
                .chain(end)
                .chain(bottom)
            {
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
        | ViewNode::Password { .. }
        | ViewNode::Phone { .. }
        | ViewNode::Pin { .. }
        | ViewNode::Textarea { .. }
        | ViewNode::Select { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Camera { .. }
        | ViewNode::Microphone { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Iframe { .. }
        | ViewNode::Device { .. }
        | ViewNode::Canvas { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::Diagram { .. }
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
        | ViewNode::Tree { .. }
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

