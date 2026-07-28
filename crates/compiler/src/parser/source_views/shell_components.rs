fn lower_scaffold_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Scaffold)?;
    let mut app_bar = None;
    let mut start = None;
    let mut main = None;
    let mut end = None;
    let mut bottom_bar = None;
    let mut overlays = None;

    for child in &node.children {
        if !matches!(
            child.name.as_str(),
            "appBar" | "start" | "main" | "end" | "bottomBar" | "overlays"
        ) {
            return Err(node_error(
                child,
                "Scaffold only accepts appBar, start, main, end, bottomBar or overlays regions",
            ));
        }
        if !child.args.is_empty() || !child.props.is_empty() {
            return Err(node_error(
                child,
                "Scaffold regions cannot declare args or props",
            ));
        }
        let children = lower_node_sequence(&child.children, allow_children)?;
        match child.name.as_str() {
            "appBar" if app_bar.is_none() => app_bar = Some(children),
            "start" if start.is_none() => start = Some(children),
            "main" if main.is_none() => main = Some(children),
            "end" if end.is_none() => end = Some(children),
            "bottomBar" if bottom_bar.is_none() => bottom_bar = Some(children),
            "overlays" if overlays.is_none() => overlays = Some(children),
            name => {
                return Err(node_error(
                    child,
                    format!("duplicate `{name}` region in Scaffold"),
                ));
            }
        }
    }

    scaffold_component_node(
        props,
        app_bar.unwrap_or_default(),
        start.unwrap_or_default(),
        main.unwrap_or_default(),
        end.unwrap_or_default(),
        bottom_bar.unwrap_or_default(),
        overlays.unwrap_or_default(),
        allow_children,
    )
    .map_err(|error| component_error(node, error))
}

fn lower_bar_node(
    node: &SourceNode,
    component: BuiltinComponent,
    allow_children: bool,
) -> DoweResult<ViewNode> {
    let props = component_props(node, component)?;
    if component == BuiltinComponent::BottomBar {
        let mut tabs = Vec::new();
        for child in &node.children {
            if child.name != "tab" || !child.args.is_empty() {
                return Err(node_error(child, "BottomBar only accepts tab entries"));
            }
            if child.children.len() != 1 || child.children[0].name != "Icon" {
                return Err(node_error(
                    child,
                    "BottomBar tab requires exactly one Icon child",
                ));
            }
            let icon_node = &child.children[0];
            reject_children(icon_node)?;
            let icon = side_nav_icon_component(
                icon_component_node(component_props(icon_node, BuiltinComponent::Icon)?)
                    .map_err(|error| component_error(icon_node, error))?,
            )
            .map_err(|error| component_error(icon_node, error))?;
            let tab_props = side_nav_entry_props(
                child,
                &[
                    "label",
                    "i18n",
                    "featured",
                    "href",
                    "navigate",
                    "target",
                    "externalMode",
                ],
                &[],
                BuiltinComponent::BottomBar,
            )?;
            tabs.push(
                bottom_bar_tab_component(tab_props, icon)
                    .map_err(|error| component_error(child, error))?,
            );
        }
        return bottom_bar_component_node(props, tabs)
            .map_err(|error| component_error(node, error));
    }
    let mut start = None;
    let mut center = None;
    let mut end = None;
    let mut top = None;
    let mut bottom = None;

    for child in &node.children {
        if !matches!(
            child.name.as_str(),
            "top" | "start" | "center" | "end" | "bottom"
        ) {
            return Err(node_error(
                child,
                format!(
                    "{} only accepts top, start, center, end or bottom regions",
                    component.as_str()
                ),
            ));
        }
        if !child.args.is_empty() || !child.props.is_empty() {
            return Err(node_error(
                child,
                "bar regions cannot declare args or props",
            ));
        }
        let children = lower_node_sequence(&child.children, allow_children)?;
        match child.name.as_str() {
            "top" if component == BuiltinComponent::AppBar && top.is_none() => top = Some(children),
            "start" if start.is_none() => start = Some(children),
            "center" if center.is_none() => center = Some(children),
            "end" if end.is_none() => end = Some(children),
            "bottom" if component == BuiltinComponent::AppBar && bottom.is_none() => {
                bottom = Some(children)
            }
            name => {
                return Err(node_error(
                    child,
                    format!("duplicate `{name}` region in {}", component.as_str()),
                ));
            }
        }
    }

    bar_component_node(
        component,
        props,
        top.unwrap_or_default(),
        start.unwrap_or_default(),
        center.unwrap_or_default(),
        end.unwrap_or_default(),
        bottom.unwrap_or_default(),
        allow_children,
    )
    .map_err(|error| component_error(node, error))
}

fn lower_select_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Select)?;
    let mut options = Vec::new();
    let mut option_each = None;
    for child in &node.children {
        if child.name == "each" {
            if option_each.is_some() {
                return Err(node_error(child, "Select accepts one `each` option block"));
            }
            let (item, collection, key) = parse_each_header(child)?;
            if child.children.len() != 1 || child.children[0].name != "Option" {
                return Err(node_error(
                    child,
                    "Select `each` must contain exactly one Option child",
                ));
            }
            let option = &child.children[0];
            reject_children(option)?;
            let value = required_prop_string(option, "value")?;
            let label = required_prop_string(option, "label")?;
            let description = optional_prop_string(option, "description")?;
            option_each = Some(dowe_components::SelectOptionEach {
                item,
                collection,
                key,
                value,
                label,
                description,
            });
            continue;
        }
        let component = COMPONENT_REGISTRY.get(&child.name).ok_or_else(|| {
            node_error(
                child,
                ComponentError::unknown_component(&child.name).to_string(),
            )
        })?;
        if component != BuiltinComponent::Option {
            return Err(node_error(child, "Select can only contain Option children"));
        }
        reject_children(child)?;
        options.push(
            select_option_component(component_props(child, component)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    select_node_with_each(props, options, option_each).map_err(|error| component_error(node, error))
}

fn lower_combo_box_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::ComboBox)?;
    let mut options = Vec::new();
    for child in &node.children {
        let component = COMPONENT_REGISTRY.get(&child.name).ok_or_else(|| {
            node_error(
                child,
                ComponentError::unknown_component(&child.name).to_string(),
            )
        })?;
        if component != BuiltinComponent::ComboOption {
            return Err(node_error(
                child,
                "ComboBox can only contain comboOption children",
            ));
        }
        reject_children(child)?;
        options.push(
            combo_option_component(component_props(child, component)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    combo_box_component_node(props, options).map_err(|error| component_error(node, error))
}

fn lower_csv_field_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::CsvField)?;
    let mut columns = Vec::new();
    for child in &node.children {
        let component = COMPONENT_REGISTRY.get(&child.name).ok_or_else(|| {
            node_error(
                child,
                ComponentError::unknown_component(&child.name).to_string(),
            )
        })?;
        if component != BuiltinComponent::CsvColumn {
            return Err(node_error(
                child,
                "CsvField can only contain csvColumn children",
            ));
        }
        reject_children(child)?;
        columns.push(
            csv_column_component(component_props(child, component)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    csv_field_component_node(props, columns).map_err(|error| component_error(node, error))
}

fn lower_drag_drop_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::DragDrop)?;
    let mut items = Vec::new();
    let mut groups = Vec::new();
    for child in &node.children {
        let component = COMPONENT_REGISTRY.get(&child.name).ok_or_else(|| {
            node_error(
                child,
                ComponentError::unknown_component(&child.name).to_string(),
            )
        })?;
        match component {
            BuiltinComponent::DragItem => {
                reject_children(child)?;
                items.push(
                    drag_item_component(component_props(child, component)?)
                        .map_err(|error| component_error(child, error))?,
                );
            }
            BuiltinComponent::DragGroup => {
                groups.push(lower_drag_group_node(child)?);
            }
            _ => {
                return Err(node_error(
                    child,
                    "DragDrop can only contain dragItem or dragGroup children",
                ));
            }
        }
    }
    drag_drop_component_node(props, items, groups).map_err(|error| component_error(node, error))
}

fn lower_drag_group_node(node: &SourceNode) -> DoweResult<dowe_components::DragGroup> {
    let props = component_props(node, BuiltinComponent::DragGroup)?;
    let mut items = Vec::new();
    for child in &node.children {
        let component = COMPONENT_REGISTRY.get(&child.name).ok_or_else(|| {
            node_error(
                child,
                ComponentError::unknown_component(&child.name).to_string(),
            )
        })?;
        if component != BuiltinComponent::DragItem {
            return Err(node_error(
                child,
                "dragGroup can only contain dragItem children",
            ));
        }
        reject_children(child)?;
        items.push(
            drag_item_component(component_props(child, component)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    drag_group_component(props, items).map_err(|error| component_error(node, error))
}

fn lower_code_node(node: &SourceNode) -> DoweResult<ViewNode> {
    reject_children(node)?;
    if node.prop("lines").is_some() {
        return Err(node_error(
            node,
            "`Code lines` was replaced by multiline `content`",
        ));
    }
    let content = code_content(node)?;
    let props = node
        .props
        .iter()
        .filter(|prop| prop.name != "content")
        .map(|prop| component_prop(BuiltinComponent::Code, prop))
        .collect::<DoweResult<Vec<_>>>()?;
    code_node(props, content).map_err(|error| component_error(node, error))
}

fn code_content(node: &SourceNode) -> DoweResult<String> {
    let prop = node
        .prop("content")
        .ok_or_else(|| node_error(node, "`Code` requires multiline `content`"))?;
    let SourceValue::String(value) = &prop.value else {
        return Err(node_error(
            node,
            "`Code content` must be a multiline string",
        ));
    };
    Ok(value.clone())
}

fn lower_svg_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Svg)?;
    let mut paths = Vec::new();
    for child in &node.children {
        let component = COMPONENT_REGISTRY.get(&child.name).ok_or_else(|| {
            node_error(
                child,
                ComponentError::unknown_component(&child.name).to_string(),
            )
        })?;
        if component != BuiltinComponent::Path {
            return Err(node_error(
                child,
                ComponentError::invalid_prop_combination("Svg only accepts Path children")
                    .to_string(),
            ));
        }
        reject_children(child)?;
        paths.push(
            svg_path_component(component_props(child, component)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    svg_component_node(props, paths).map_err(|error| component_error(node, error))
}

