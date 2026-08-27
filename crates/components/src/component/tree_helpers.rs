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
        ViewNode::Splash {
            content, children, ..
        } => contains_children(content) || contains_children(children),
        ViewNode::Scope { children, .. } | ViewNode::Each { children, .. } => {
            contains_children(children)
        }
        ViewNode::Box { children, .. }
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
        | ViewNode::Button { children, .. } => contains_children(children),
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => contains_children(header) || contains_children(body) || contains_children(footer),
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
        ViewNode::Accordion { items, .. } => {
            items.iter().any(|item| contains_children(&item.children))
        }
        ViewNode::Carousel { slides, .. } => slides
            .iter()
            .any(|slide| contains_children(&slide.children)),
        ViewNode::Tabs { tabs, .. } => tabs.iter().any(|tab| contains_children(&tab.children)),
        ViewNode::NavMenu { items, .. } => items.iter().any(nav_menu_contains_children),
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
            contains_children(top)
                || contains_children(start)
                || contains_children(center)
                || contains_children(end)
                || contains_children(bottom)
        }
        ViewNode::BottomBar { .. } | ViewNode::SideNav { .. } | ViewNode::RailNav { .. } => false,
        ViewNode::Sidebar {
            header,
            body,
            footer,
            ..
        } => contains_children(header) || contains_children(body) || contains_children(footer),
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
            ..
        } => {
            contains_children(app_bar)
                || contains_children(start)
                || contains_children(main)
                || contains_children(end)
                || contains_children(bottom_bar)
                || contains_children(overlays)
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
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Camera { .. }
        | ViewNode::Microphone { .. }
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

pub fn tree_has_dynamic_icon(node: &ViewNode) -> bool {
    if matches!(node, ViewNode::Svg { props, .. } if props.icon_name.is_some())
        || matches!(node, ViewNode::Avatar { icon: Some(icon), .. } if icon.props.icon_name.is_some())
    {
        return true;
    }
    node_child_groups(node)
        .into_iter()
        .flat_map(|group| group.iter())
        .any(tree_has_dynamic_icon)
}

pub fn dynamic_icon_names(node: &ViewNode) -> Option<BTreeSet<String>> {
    #[derive(Clone)]
    struct BindingValue {
        value: ViewSignalValue,
        is_static: bool,
    }

    fn resolve_path(
        path: &str,
        values: &std::collections::HashMap<String, BindingValue>,
        locals: &std::collections::HashMap<String, BindingValue>,
    ) -> Option<BindingValue> {
        let root = path.split('.').next().unwrap_or(path);
        let mut value = values.get(root).or_else(|| locals.get(root))?.clone();
        for field in path.split('.').skip(1) {
            let BindingValue {
                value: object,
                is_static,
            } = value;
            let ViewSignalValue::Object(fields) = object else {
                return None;
            };
            value = fields.into_iter().find_map(|(name, value)| {
                (name == field).then_some(BindingValue { value, is_static })
            })?;
        }
        Some(value)
    }

    let mut names = BTreeSet::new();
    let valid_names = all_icon_names().into_iter().collect::<BTreeSet<_>>();
    let values = std::collections::HashMap::new();
    let locals = std::collections::HashMap::new();
    let mut pending = vec![(node, values, locals)];
    while let Some((node, values, locals)) = pending.pop() {
        match node {
            ViewNode::Scope {
                constants,
                signals,
                children,
                ..
            } => {
                let mut scoped = values;
                scoped.extend(constants.iter().map(|constant| {
                    (
                        constant.name.clone(),
                        BindingValue {
                            value: constant.value.clone(),
                            is_static: true,
                        },
                    )
                }));
                scoped.extend(signals.iter().map(|signal| {
                    (
                        signal.name.clone(),
                        BindingValue {
                            value: signal.initial.clone(),
                            is_static: false,
                        },
                    )
                }));
                pending.extend(
                    children
                        .iter()
                        .map(|child| (child, scoped.clone(), locals.clone())),
                );
            }
            ViewNode::Each {
                item,
                collection,
                children,
                ..
            } => {
                if !children.iter().any(tree_has_dynamic_icon) {
                    continue;
                }
                let Some(collection) = resolve_path(collection, &values, &locals) else {
                    return None;
                };
                let is_static = collection.is_static;
                let ViewSignalValue::Array(items) = collection.value else {
                    return None;
                };
                if !is_static {
                    return None;
                }
                for item_value in items {
                    let mut scoped = locals.clone();
                    scoped.insert(
                        item.clone(),
                        BindingValue {
                            value: item_value,
                            is_static: true,
                        },
                    );
                    pending.extend(
                        children
                            .iter()
                            .map(|child| (child, values.clone(), scoped.clone())),
                    );
                }
            }
            ViewNode::Svg { props, .. }
            | ViewNode::Avatar {
                icon: Some(SideNavIcon { props, .. }),
                ..
            } => {
                let Some(binding) = props.icon_name.as_deref() else {
                    continue;
                };
                let Some(value) = resolve_path(binding, &values, &locals) else {
                    return None;
                };
                let BindingValue { value, is_static } = value;
                let ViewSignalValue::String(name) = value else {
                    return None;
                };
                if !is_static || !valid_names.contains(&name) {
                    return None;
                }
                names.insert(name);
            }
            _ => {
                pending.extend(
                    node_child_groups(node)
                        .into_iter()
                        .flatten()
                        .map(|child| (child, values.clone(), locals.clone())),
                );
            }
        }
    }
    Some(names)
}

fn is_text_like(node: &ViewNode) -> bool {
    matches!(node, ViewNode::Text { .. } | ViewNode::Title { .. })
}
