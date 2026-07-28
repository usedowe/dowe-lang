fn lower_pagination_node(node: &SourceNode) -> DoweResult<ViewNode> {
    reject_children(node)?;
    let _bind = required_prop_bareword(node, "bind")?;
    let total_prop = node
        .prop("total")
        .ok_or_else(|| node_error(node, "missing `total`"))?;
    let page_size = required_prop_number(node, "pageSize")?;
    let _on_change = required_prop_bareword(node, "onChange")?;
    let page_size = page_size
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| node_error(node, "Pagination pageSize must be a positive integer"))?;
    let (total, pages) = match &total_prop.value {
        SourceValue::Number(value) => {
            let total = value
                .parse::<u32>()
                .map_err(|_| node_error(node, "Pagination total must be a non-negative integer"))?;
            let pages = total.div_ceil(page_size).max(1);
            if pages > 25 {
                return Err(node_error(
                    node,
                    "Pagination supports at most 25 pages in the current portable subset",
                ));
            }
            (dowe_components::PaginationTotal::Static(total), pages)
        }
        SourceValue::Bareword(path) => (dowe_components::PaginationTotal::Signal(path.clone()), 25),
        _ => {
            return Err(prop_error(
                total_prop,
                "Pagination total must be a non-negative integer or signal number path",
            ));
        }
    };
    let mut props = Vec::new();
    for prop in &node.props {
        let name = match prop.name.as_str() {
            "bind" => "value",
            "onChange" => "onChange",
            "variant" | "scheme" | "size" | "disabled" | "wide" | "vertical" | "ariaLabel" => {
                prop.name.as_str()
            }
            "total" | "pageSize" => continue,
            _ => {
                return Err(node_error(
                    node,
                    format!("unknown prop `{}` on `Pagination`", prop.name),
                ));
            }
        };
        props.push(ComponentProp {
            name: name.to_string(),
            value: prop_value(prop)?,
        });
    }
    let items = (1..=pages)
        .map(|page| dowe_components::ToggleGroupItem {
            id: page.to_string(),
            label: page.to_string(),
            icon: None,
        })
        .collect();
    let mut pagination =
        toggle_group_component_node(props, items).map_err(|error| component_error(node, error))?;
    let ViewNode::ToggleGroup { props, .. } = &mut pagination else {
        unreachable!()
    };
    props.kind = ToggleGroupKind::Pagination;
    props.pagination = Some(dowe_components::PaginationProps { total, page_size });
    Ok(pagination)
}

fn lower_avatar_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Avatar)?;
    let mut icon = None;
    for child in &node.children {
        if child.name != "icon" {
            return Err(node_error(child, "Avatar only accepts an icon region"));
        }
        if icon.is_some() {
            return Err(node_error(child, "duplicate `icon` region in Avatar"));
        }
        icon = Some(lower_overlay_icon(child, BuiltinComponent::Avatar)?);
    }
    avatar_component_node(props, icon).map_err(|error| component_error(node, error))
}

fn lower_chip_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Chip)?;
    let mut start = None;
    let mut end = None;
    let mut labels = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "start" if start.is_none() => {
                start = Some(lower_overlay_icon(child, BuiltinComponent::Chip)?)
            }
            "start" => return Err(node_error(child, "duplicate `start` region in Chip")),
            "end" if end.is_none() => {
                end = Some(lower_overlay_icon(child, BuiltinComponent::Chip)?)
            }
            "end" => return Err(node_error(child, "duplicate `end` region in Chip")),
            _ => labels.push(text_child_line(child)?),
        }
    }
    if labels.len() != 1 {
        return Err(node_error(node, "Chip requires one direct text child"));
    }
    chip_component_node(props, &labels[0], start, end).map_err(|error| component_error(node, error))
}

fn lower_modal_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Modal)?;
    let mut header = None;
    let mut footer = None;
    let mut body_nodes = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "header" if header.is_none() => {
                header = Some(lower_region(child, "Modal header", allow_children)?)
            }
            "header" => return Err(node_error(child, "duplicate `header` region in Modal")),
            "footer" if footer.is_none() => {
                footer = Some(lower_region(child, "Modal footer", allow_children)?)
            }
            "footer" => return Err(node_error(child, "duplicate `footer` region in Modal")),
            _ => body_nodes.push(child.clone()),
        }
    }
    let body = lower_node_sequence(&body_nodes, allow_children)?;
    modal_component_node(
        props,
        header.unwrap_or_default(),
        body,
        footer.unwrap_or_default(),
        allow_children,
    )
    .map_err(|error| component_error(node, error))
}

fn lower_drawer_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Drawer)?;
    let mut header = None;
    let mut body = None;
    let mut footer = None;
    let mut body_nodes = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "header" if header.is_none() => {
                header = Some(lower_region(child, "Drawer header", allow_children)?)
            }
            "header" => return Err(node_error(child, "duplicate `header` region in Drawer")),
            "body" if body.is_none() => {
                body = Some(lower_region(child, "Drawer body", allow_children)?)
            }
            "body" => return Err(node_error(child, "duplicate `body` region in Drawer")),
            "footer" if footer.is_none() => {
                footer = Some(lower_region(child, "Drawer footer", allow_children)?)
            }
            "footer" => return Err(node_error(child, "duplicate `footer` region in Drawer")),
            _ => body_nodes.push(child.clone()),
        }
    }
    let mut lowered_body = lower_node_sequence(&body_nodes, allow_children)?;
    if let Some(mut region_body) = body {
        lowered_body.append(&mut region_body);
    }
    drawer_component_node(
        props,
        header.unwrap_or_default(),
        lowered_body,
        footer.unwrap_or_default(),
        allow_children,
    )
    .map_err(|error| component_error(node, error))
}

fn lower_dropdown_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Dropdown)?;
    let mut trigger = None;
    let mut header = None;
    let mut footer = None;
    let mut entries = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "trigger" if trigger.is_none() => {
                trigger = Some(lower_region(child, "Dropdown trigger", allow_children)?)
            }
            "trigger" => return Err(node_error(child, "duplicate `trigger` region in Dropdown")),
            "header" if header.is_none() => {
                header = Some(lower_region(child, "Dropdown header", allow_children)?)
            }
            "header" => return Err(node_error(child, "duplicate `header` region in Dropdown")),
            "footer" if footer.is_none() => {
                footer = Some(lower_region(child, "Dropdown footer", allow_children)?)
            }
            "footer" => return Err(node_error(child, "duplicate `footer` region in Dropdown")),
            "item" => entries.push(dowe_components::OverlayEntry::Item(lower_overlay_item(
                child,
                BuiltinComponent::Dropdown,
            )?)),
            "divider" => {
                if !child.args.is_empty() || !child.props.is_empty() || !child.children.is_empty() {
                    return Err(node_error(
                        child,
                        "Dropdown divider cannot declare args, props or children",
                    ));
                }
                entries.push(dowe_components::OverlayEntry::Divider);
            }
            _ => {
                return Err(node_error(
                    child,
                    "Dropdown only accepts trigger, header, footer, item or divider entries",
                ));
            }
        }
    }
    dropdown_component_node(
        props,
        trigger.unwrap_or_default(),
        header.unwrap_or_default(),
        entries,
        footer.unwrap_or_default(),
        allow_children,
    )
    .map_err(|error| component_error(node, error))
}

fn lower_command_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Command)?;
    let mut entries = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "item" => entries.push(dowe_components::CommandEntry::Item(lower_overlay_item(
                child,
                BuiltinComponent::Command,
            )?)),
            "group" => entries.push(lower_command_group(child)?),
            _ => {
                return Err(node_error(
                    child,
                    "Command only accepts item or group entries",
                ));
            }
        }
    }
    command_component_node(props, entries).map_err(|error| component_error(node, error))
}

fn lower_avatar_group_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::AvatarGroup)?;
    let mut items = Vec::new();
    for child in &node.children {
        if child.name != "item" {
            return Err(node_error(child, "AvatarGroup only accepts item entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "AvatarGroup item cannot declare args"));
        }
        reject_children(child)?;
        items.push(
            avatar_group_item_component(avatar_group_item_props(child)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    avatar_group_component_node(props, items).map_err(|error| component_error(node, error))
}

fn lower_type_writer_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::TypeWriter)?;
    let mut items = Vec::new();
    for child in &node.children {
        if child.name != "item" {
            return Err(node_error(child, "TypeWriter only accepts item entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "TypeWriter item cannot declare args"));
        }
        reject_children(child)?;
        items.push(
            type_writer_item_component(type_writer_item_props(child)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    type_writer_component_node(props, items).map_err(|error| component_error(node, error))
}

fn lower_rich_text_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::RichText)?;
    let mut marks = Vec::new();
    for child in &node.children {
        if child.name != "mark" {
            return Err(node_error(child, "RichText only accepts mark entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "RichText mark cannot declare args"));
        }
        reject_children(child)?;
        marks.push(
            rich_text_mark_component(rich_text_mark_props(child)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    rich_text_component_node(props, marks).map_err(|error| component_error(node, error))
}

fn lower_toggle_group_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::ToggleGroup)?;
    let mut items = Vec::new();
    for child in &node.children {
        if child.name != "item" {
            return Err(node_error(child, "ToggleGroup only accepts item entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "ToggleGroup item cannot declare args"));
        }
        reject_children(child)?;
        items.push(
            toggle_group_item_component(toggle_group_item_props(child)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    toggle_group_component_node(props, items).map_err(|error| component_error(node, error))
}

fn lower_map_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Map)?;
    let mut markers = Vec::new();
    let mut waypoints = Vec::new();
    for child in &node.children {
        if !matches!(child.name.as_str(), "marker" | "waypoint") {
            return Err(node_error(
                child,
                "Map only accepts marker and waypoint entries",
            ));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "Map entries cannot declare args"));
        }
        reject_children(child)?;
        if child.name == "marker" {
            markers.push(
                map_marker_component(map_marker_props(child)?)
                    .map_err(|error| component_error(child, error))?,
            );
        } else {
            waypoints.push(
                map_waypoint_component(map_waypoint_props(child)?)
                    .map_err(|error| component_error(child, error))?,
            );
        }
    }
    map_component_node(props, markers, waypoints).map_err(|error| component_error(node, error))
}

fn lower_accordion_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Accordion)?;
    let mut items = Vec::new();
    for child in &node.children {
        if child.name != "item" {
            return Err(node_error(child, "Accordion only accepts item entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "Accordion item cannot declare args"));
        }
        let children = lower_node_sequence(&child.children, allow_children)?;
        items.push(
            accordion_item_component(accordion_item_props(child)?, children)
                .map_err(|error| component_error(child, error))?,
        );
    }
    accordion_component_node(props, items).map_err(|error| component_error(node, error))
}

fn lower_carousel_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Carousel)?;
    let mut slides = Vec::new();
    for child in &node.children {
        if child.name != "slide" {
            return Err(node_error(child, "Carousel only accepts slide entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "Carousel slide cannot declare args"));
        }
        let children = lower_node_sequence(&child.children, allow_children)?;
        slides.push(
            carousel_slide_component(carousel_slide_props(child)?, children)
                .map_err(|error| component_error(child, error))?,
        );
    }
    carousel_component_node(props, slides).map_err(|error| component_error(node, error))
}

fn lower_radio_group_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::RadioGroup)?;
    let mut options = Vec::new();
    for child in &node.children {
        if child.name != "item" {
            return Err(node_error(child, "RadioGroup only accepts item entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "RadioGroup item cannot declare args"));
        }
        reject_children(child)?;
        options.push(
            radio_option_component(radio_item_props(child)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    radio_group_component_node(props, options).map_err(|error| component_error(node, error))
}

fn lower_fab_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Fab)?;
    let mut actions = Vec::new();
    for child in &node.children {
        let component = COMPONENT_REGISTRY.get(&child.name).ok_or_else(|| {
            node_error(
                child,
                ComponentError::unknown_component(&child.name).to_string(),
            )
        })?;
        if component != BuiltinComponent::FabAction {
            return Err(node_error(child, "Fab only accepts fabAction entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "fabAction cannot declare args"));
        }
        reject_children(child)?;
        actions.push(
            fab_action_component(component_props(child, component)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    fab_component_node(props, actions).map_err(|error| component_error(node, error))
}

