fn collect_i18n_keys(node: &ViewNode, keys: &mut BTreeSet<String>) {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => {
            for child in content.iter().chain(children) {
                collect_i18n_keys(child, keys);
            }
        }
        ViewNode::Scope { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Button { children, .. }
        | ViewNode::Brand { children, .. }
        | ViewNode::Banner { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Each { children, .. } => {
            for child in children {
                collect_i18n_keys(child, keys);
            }
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_i18n_keys(child, keys);
            }
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_i18n_keys(child, keys);
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            for child in trigger.iter().chain(header).chain(footer) {
                collect_i18n_keys(child, keys);
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
                collect_i18n_keys(child, keys);
            }
        }
        ViewNode::BottomBar { tabs, .. } => {
            for tab in tabs {
                if let Some(key) = tab.i18n.as_ref() {
                    keys.insert(key.clone());
                }
            }
        }
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                if let Some(key) = tab.i18n.as_ref() {
                    keys.insert(key.clone());
                }
                for child in &tab.children {
                    collect_i18n_keys(child, keys);
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                for child in &item.children {
                    collect_i18n_keys(child, keys);
                }
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                for child in &slide.children {
                    collect_i18n_keys(child, keys);
                }
            }
        }
        ViewNode::Marquee { children, .. } | ViewNode::Collapsible { children, .. } => {
            for child in children {
                collect_i18n_keys(child, keys);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                match item {
                    dowe_components::NavMenuItem::Item(props) => {
                        if let Some(key) = props.i18n.as_ref() {
                            keys.insert(key.clone());
                        }
                        if let Some(key) = props.description_i18n.as_ref() {
                            keys.insert(key.clone());
                        }
                    }
                    dowe_components::NavMenuItem::Submenu { props, items } => {
                        if let Some(key) = props.i18n.as_ref() {
                            keys.insert(key.clone());
                        }
                        if let Some(key) = props.description_i18n.as_ref() {
                            keys.insert(key.clone());
                        }
                        for item in items {
                            if let Some(key) = item.i18n.as_ref() {
                                keys.insert(key.clone());
                            }
                            if let Some(key) = item.description_i18n.as_ref() {
                                keys.insert(key.clone());
                            }
                        }
                    }
                    dowe_components::NavMenuItem::Megamenu { props, content } => {
                        if let Some(key) = props.i18n.as_ref() {
                            keys.insert(key.clone());
                        }
                        if let Some(key) = props.description_i18n.as_ref() {
                            keys.insert(key.clone());
                        }
                        for child in content {
                            collect_i18n_keys(child, keys);
                        }
                    }
                }
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
                collect_i18n_keys(child, keys);
            }
        }
        ViewNode::SideNav { items, .. } => {
            for item in items {
                match item {
                    dowe_components::SideNavItem::Header(props)
                    | dowe_components::SideNavItem::Item(props) => {
                        collect_side_nav_i18n_keys(props, keys);
                    }
                    dowe_components::SideNavItem::Submenu { props, items, .. } => {
                        collect_side_nav_i18n_keys(props, keys);
                        for item in items {
                            collect_side_nav_i18n_keys(item, keys);
                        }
                    }
                    dowe_components::SideNavItem::Divider => {}
                }
            }
        }
        ViewNode::RailNav { items, .. } => {
            for item in items {
                if let dowe_components::RailNavItem::Item(props) = item
                    && let Some(key) = props.i18n.as_ref()
                {
                    keys.insert(key.clone());
                }
            }
        }
        ViewNode::Title { props, .. }
        | ViewNode::Text { props, .. }
        | ViewNode::RichText { props, .. } => {
            if let Some(key) = props.i18n.as_ref() {
                keys.insert(key.clone());
            }
        }
        ViewNode::Input { .. }
        | ViewNode::ComboBox { .. }
        | ViewNode::CsvField { .. }
        | ViewNode::DragDrop { .. }
        | ViewNode::Editor { .. }
        | ViewNode::ImageCropper { .. }
        | ViewNode::Password { .. }
        | ViewNode::Phone { .. }
        | ViewNode::Pin { .. }
        | ViewNode::Textarea { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::SelectTheme { .. }
        | ViewNode::Fab { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Select { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Camera { .. }
        | ViewNode::Microphone { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Iframe { .. }
        | ViewNode::Device { .. }
        | ViewNode::Canvas { .. }
        | ViewNode::Diagram { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. }
        | ViewNode::Table { .. }
        | ViewNode::Tree { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Alert { .. }
        | ViewNode::Avatar { .. }
        | ViewNode::Chip { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Command { .. }
        | ViewNode::Svg { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::TypeWriter { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. }
        | ViewNode::Children => {}
    }
}

fn collect_side_nav_i18n_keys(
    props: &dowe_components::SideNavItemProps,
    keys: &mut BTreeSet<String>,
) {
    for key in [
        props.i18n.as_ref(),
        props.description_i18n.as_ref(),
        props.status_i18n.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        keys.insert(key.clone());
    }
}

fn reject_unknown_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.contains(&prop.name.as_str()) {
            return Err(prop_error(
                prop,
                format!("`{}` does not support `{}`", node.name, prop.name),
            ));
        }
    }
    Ok(())
}

fn required_string_prop(node: &SourceNode, name: &str) -> DoweResult<String> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("`{}` requires `{name}`", node.name)))?;
    match &prop.value {
        SourceValue::String(value) if !value.is_empty() => Ok(value.clone()),
        _ => Err(prop_error(
            prop,
            format!("`{name}` must be a non-empty quoted string"),
        )),
    }
}

fn optional_bool_prop(node: &SourceNode, name: &str) -> DoweResult<Option<bool>> {
    node.prop(name)
        .map(|prop| match prop.value {
            SourceValue::Boolean(value) => Ok(value),
            _ => Err(prop_error(prop, format!("`{name}` must be a boolean"))),
        })
        .transpose()
}

fn node_error(node: &SourceNode, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &node.location.path,
        format!(
            "{}:{}: {}",
            node.location.line,
            node.location.column,
            message.as_ref()
        ),
    )
}

fn prop_error(prop: &SourceProp, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &prop.location.path,
        format!(
            "{}:{}: {}",
            prop.location.line,
            prop.location.column,
            message.as_ref()
        ),
    )
}

