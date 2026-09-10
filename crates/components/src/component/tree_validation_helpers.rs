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
        ViewNode::Camera { props } => Some(&props.style.style),
        ViewNode::Microphone { props } => Some(&props.style.style),
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
        ViewNode::Password { props } => Some(&props.style.style),
        ViewNode::Phone { props } => Some(&props.style.style),
        ViewNode::Pin { props } => Some(&props.style.style),
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
        let current = entry
            .value
            .count()
            .or_else(|| entry.value.weights().map(|weights| weights.len() as u16))?;
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

