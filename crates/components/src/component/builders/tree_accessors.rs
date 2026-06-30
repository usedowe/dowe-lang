pub fn first_text(node: &ViewNode) -> Option<String> {
    match node {
        ViewNode::Scope { children, .. } | ViewNode::Each { children, .. } => {
            children.iter().find_map(first_text)
        }
        ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Marquee { children, .. }
        | ViewNode::Button { children, .. } => children.iter().find_map(first_text),
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => header.iter().chain(body).chain(footer).find_map(first_text),
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => header.iter().chain(body).chain(footer).find_map(first_text),
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            entries,
            ..
        } => trigger
            .iter()
            .chain(header)
            .chain(footer)
            .find_map(first_text)
            .or_else(|| entries.iter().find_map(overlay_entry_first_text)),
        ViewNode::Command { entries, .. } => entries.iter().find_map(command_entry_first_text),
        ViewNode::AvatarGroup { items, .. } => items.iter().find_map(|item| {
            item.name
                .clone()
                .or_else(|| item.alt.clone())
                .or_else(|| item.src.clone())
        }),
        ViewNode::ChatBox { props } => Some(props.assistant_name.clone()),
        ViewNode::Empty { props } => props
            .title
            .clone()
            .or_else(|| props.description.clone())
            .or_else(|| Some(props.action_label.clone())),
        ViewNode::TypeWriter { items, .. } => items.first().map(|item| item.text.clone()),
        ViewNode::RichText { marks, .. } => marks.first().map(|mark| mark.text.clone()),
        ViewNode::Record { props } => props
            .style
            .label
            .clone()
            .or_else(|| Some(props.name.clone())),
        ViewNode::ToggleGroup { props, items } => props
            .style
            .label
            .clone()
            .or_else(|| items.first().map(|item| item.label.clone())),
        ViewNode::Collapsible {
            props, children, ..
        } => Some(props.label.clone()).or_else(|| children.iter().find_map(first_text)),
        ViewNode::Countdown { props } => Some(props.target.clone()),
        ViewNode::Map { markers, .. } => markers
            .iter()
            .find_map(|marker| marker.label.clone().or_else(|| marker.popup.clone())),
        ViewNode::Accordion { items, .. } => items.iter().find_map(|item| Some(item.label.clone())),
        ViewNode::Carousel { props, slides } => props.title.clone().or_else(|| {
            slides
                .iter()
                .find_map(|slide| slide.children.iter().find_map(first_text))
        }),
        ViewNode::Tabs { tabs, .. } => tabs
            .iter()
            .find_map(|tab| tab.children.iter().find_map(first_text)),
        ViewNode::NavMenu { items, .. } => items.iter().find_map(nav_menu_first_text),
        ViewNode::AppBar {
            start, center, end, ..
        }
        | ViewNode::Footer {
            start, center, end, ..
        }
        | ViewNode::BottomBar {
            start, center, end, ..
        } => start.iter().chain(center).chain(end).find_map(first_text),
        ViewNode::SideNav { items, .. } => {
            items.iter().find_map(side_nav_first_text)
        }
        ViewNode::Sidebar {
            header,
            body,
            footer,
            ..
        } => header
            .iter()
            .chain(body)
            .chain(footer)
            .find_map(first_text),
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            ..
        } => app_bar
            .iter()
            .chain(start)
            .chain(main)
            .chain(end)
            .chain(bottom_bar)
            .find_map(first_text),
        ViewNode::Table { props } => props
            .columns
            .first()
            .map(|column| column.label.clone())
            .or_else(|| Some(props.empty_title.clone())),
        ViewNode::Title { value, .. } | ViewNode::Text { value, .. } => Some(value.clone()),
        ViewNode::Alert { props } => Some(props.message.clone()),
        ViewNode::Avatar { props, .. } => props.name.clone().or_else(|| Some(props.alt.clone())),
        ViewNode::Chip { value, .. } => Some(value.clone()),
        ViewNode::AlertDialog { props } => Some(props.title.clone()),
        ViewNode::Toast { props } => props
            .title
            .clone()
            .or_else(|| (!props.description.is_empty()).then(|| props.description.clone())),
        ViewNode::Audio { props } => props.subtitle.clone(),
        ViewNode::Image { props } => (!props.alt.is_empty()).then(|| props.alt.clone()),
        ViewNode::Checkbox { props } => props.style.label.clone(),
        ViewNode::Color { props } => props.style.label.clone(),
        ViewNode::Date { props } => props.style.label.clone(),
        ViewNode::DateRange { props } => props.style.label.clone(),
        ViewNode::RadioGroup { props, options } => props
            .style
            .label
            .clone()
            .or_else(|| options.first().map(|option| option.label.clone())),
        ViewNode::Toggle { props } => props.style.label.clone(),
        ViewNode::ToggleTheme { props } => Some(props.dark_label.clone()),
        ViewNode::Fab { props, actions } => props
            .style
            .label
            .clone()
            .or_else(|| actions.first().map(|action| action.label.clone())),
        ViewNode::Slider { props } => props.style.label.clone(),
        ViewNode::Dropzone { props } => props.style.label.clone(),
        ViewNode::ComboBox { props, options } => props
            .style
            .label
            .clone()
            .or_else(|| options.first().map(|option| option.label.clone())),
        ViewNode::CsvField { props, columns } => Some(props.button_text.clone())
            .or_else(|| columns.first().map(|column| column.name.clone())),
        ViewNode::DragDrop { props, items, groups } => items
            .first()
            .and_then(|item| item.label.clone().or_else(|| Some(item.id.clone())))
            .or_else(|| groups.first().and_then(|group| group.title.clone()))
            .or_else(|| Some(props.empty_text.clone())),
        ViewNode::Editor { props } => props.style.label.clone(),
        ViewNode::ImageCropper { props } => props.style.label.clone(),
        ViewNode::PasswordField { props } => props.style.label.clone(),
        ViewNode::PhoneField { props } => props.style.label.clone(),
        ViewNode::PinField { props } => props.style.label.clone(),
        ViewNode::Textarea { props } => props.style.label.clone(),
        ViewNode::Input { .. }
        | ViewNode::Select { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::Svg { .. }
        | ViewNode::Children => None,
    }
}

pub fn node_element_props(node: &ViewNode) -> Option<&ElementProps> {
    match node {
        ViewNode::Scope { .. } | ViewNode::Each { .. } => None,
        ViewNode::Box { props, .. } | ViewNode::Section { props, .. } => Some(&props.element),
        ViewNode::Flex { props, .. } => Some(&props.style.element),
        ViewNode::Grid { props, .. } => Some(&props.style.element),
        ViewNode::Card { props, .. }
        | ViewNode::Button { props, .. }
        | ViewNode::Input { props }
        | ViewNode::Select { props, .. } => Some(&props.element),
        ViewNode::AvatarGroup { props, .. } => Some(&props.style.element),
        ViewNode::ChatBox { props } => Some(&props.style.element),
        ViewNode::Empty { props } => Some(&props.style.element),
        ViewNode::Marquee { props, .. } => Some(&props.style.element),
        ViewNode::TypeWriter { props, .. } => Some(&props.style.element),
        ViewNode::RichText { props, .. } => Some(&props.style.element),
        ViewNode::Record { props } => Some(&props.style.element),
        ViewNode::ToggleGroup { props, .. } => Some(&props.style.element),
        ViewNode::Collapsible { props, .. } => Some(&props.style.element),
        ViewNode::Countdown { props } => Some(&props.style.element),
        ViewNode::Map { props, .. } => Some(&props.style.element),
        ViewNode::ToggleTheme { props } => Some(&props.style.element),
        ViewNode::Fab { props, .. } => Some(&props.style.element),
        ViewNode::Slider { props } => Some(&props.style.element),
        ViewNode::Dropzone { props } => Some(&props.style.element),
        ViewNode::ComboBox { props, .. } => Some(&props.style.element),
        ViewNode::CsvField { props, .. } => Some(&props.style.element),
        ViewNode::DragDrop { props, .. } => Some(&props.style.element),
        ViewNode::Editor { props } => Some(&props.style.element),
        ViewNode::ImageCropper { props } => Some(&props.style.element),
        ViewNode::PasswordField { props } => Some(&props.style.element),
        ViewNode::PhoneField { props } => Some(&props.style.element),
        ViewNode::PinField { props } => Some(&props.style.element),
        ViewNode::Textarea { props } => Some(&props.style.element),
        ViewNode::Avatar { props, .. } => Some(&props.style.element),
        ViewNode::Badge { props, .. } => Some(&props.style.element),
        ViewNode::Chip { props, .. } => Some(&props.style.element),
        ViewNode::Modal { props, .. } => Some(&props.style.element),
        ViewNode::AlertDialog { props } => Some(&props.style.element),
        ViewNode::Tooltip { props, .. } => Some(&props.style.element),
        ViewNode::Toast { props } => Some(&props.style.element),
        ViewNode::Dropdown { props, .. } => Some(&props.style.element),
        ViewNode::Command { props, .. } => Some(&props.style.element),
        ViewNode::Audio { props } => Some(&props.style.element),
        ViewNode::Image { props } => Some(&props.style.element),
        ViewNode::Accordion { props, .. } => Some(&props.style.element),
        ViewNode::Carousel { props, .. } => Some(&props.style.element),
        ViewNode::Checkbox { props } => Some(&props.style.element),
        ViewNode::Color { props } => Some(&props.style.element),
        ViewNode::Date { props } => Some(&props.style.element),
        ViewNode::DateRange { props } => Some(&props.style.element),
        ViewNode::RadioGroup { props, .. } => Some(&props.style.element),
        ViewNode::Toggle { props } => Some(&props.style.element),
        ViewNode::Skeleton { props } => Some(&props.style.element),
        ViewNode::Tabs { props, .. } => Some(&props.style.element),
        ViewNode::NavMenu { props, .. } => Some(&props.style.element),
        ViewNode::Code { props } => Some(&props.style.element),
        ViewNode::Video { props } => Some(&props.style.element),
        ViewNode::Candlestick { props } => Some(&props.style.element),
        ViewNode::ArcChart { props } => Some(&props.common.style.element),
        ViewNode::AreaChart { props } => Some(&props.common.style.element),
        ViewNode::BarChart { props } => Some(&props.common.style.element),
        ViewNode::LineChart { props } => Some(&props.common.style.element),
        ViewNode::PieChart { props } => Some(&props.common.style.element),
        ViewNode::Table { props } => Some(&props.style.element),
        ViewNode::Divider { props } => Some(&props.style.element),
        ViewNode::Alert { props } => Some(&props.style.element),
        ViewNode::Svg { props, .. } => Some(&props.style.element),
        ViewNode::AppBar { props, .. }
        | ViewNode::Footer { props, .. }
        | ViewNode::BottomBar { props, .. } => Some(&props.style.element),
        ViewNode::SideNav { props, .. } => Some(&props.style.element),
        ViewNode::Sidebar { props, .. } => {
            Some(&props.style.element)
        }
        ViewNode::Scaffold { props, .. } => Some(&props.style.element),
        ViewNode::Drawer { props, .. } => Some(&props.style.element),
        ViewNode::Title { props, .. } | ViewNode::Text { props, .. } => Some(&props.style.element),
        ViewNode::Children => None,
    }
}

pub fn navigation_action(node: &ViewNode) -> Option<&NavigationAction> {
    match node {
        ViewNode::Button { props, .. } => props.navigation.as_ref(),
        ViewNode::Avatar { props, .. } => props.style.navigation.as_ref(),
        ViewNode::Empty { props } => props.style.navigation.as_ref(),
        _ => None,
    }
}

pub fn node_children(node: &ViewNode) -> &[ViewNode] {
    match node {
        ViewNode::Scope { children, .. } | ViewNode::Each { children, .. } => children,
        ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Marquee { children, .. }
        | ViewNode::Collapsible { children, .. }
        | ViewNode::Button { children, .. } => children,
        ViewNode::Drawer { body, .. } => body,
        ViewNode::Modal { body, .. } => body,
        ViewNode::Tabs { .. }
        | ViewNode::NavMenu { .. }
        | ViewNode::Dropdown { .. }
        | ViewNode::Command { .. }
        | ViewNode::Accordion { .. }
        | ViewNode::Carousel { .. }
        | ViewNode::RadioGroup { .. } => &[],
        ViewNode::AppBar { .. }
        | ViewNode::Footer { .. }
        | ViewNode::BottomBar { .. }
        | ViewNode::SideNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::Scaffold { .. } => &[],
        ViewNode::Input { .. }
        | ViewNode::ToggleTheme { .. }
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
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. }
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
        | ViewNode::Toggle { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Svg { .. }
        | ViewNode::Title { .. }
        | ViewNode::Text { .. }
        | ViewNode::TypeWriter { .. }
        | ViewNode::Children => &[],
    }
}

pub fn node_child_groups(node: &ViewNode) -> Vec<&[ViewNode]> {
    match node {
        ViewNode::AppBar {
            start, center, end, ..
        }
        | ViewNode::Footer {
            start, center, end, ..
        }
        | ViewNode::BottomBar {
            start, center, end, ..
        } => vec![start.as_slice(), center.as_slice(), end.as_slice()],
        ViewNode::Tabs { tabs, .. } => tabs
            .iter()
            .map(|tab| tab.children.as_slice())
            .collect::<Vec<_>>(),
        ViewNode::NavMenu { items, .. } => items.iter().filter_map(nav_menu_child_group).collect(),
        ViewNode::SideNav { .. } => Vec::new(),
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
            ..
        } => vec![
            app_bar.as_slice(),
            start.as_slice(),
            main.as_slice(),
            end.as_slice(),
            bottom_bar.as_slice(),
        ],
        _ => vec![node_children(node)],
    }
}

fn nav_menu_child_group(item: &NavMenuItem) -> Option<&[ViewNode]> {
    match item {
        NavMenuItem::Megamenu { content, .. } => Some(content.as_slice()),
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
        PropValue::Responsive(_) => Err(ComponentError::invalid_prop(name, "static scalar")),
    }
}
