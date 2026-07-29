fn lower_node_sequence(nodes: &[SourceNode], allow_children: bool) -> DoweResult<Vec<ViewNode>> {
    let mut output = Vec::new();
    let mut index = 0usize;
    while index < nodes.len() {
        let node = &nodes[index];
        match node.name.as_str() {
            "if" => {
                let else_node = nodes.get(index + 1).filter(|next| next.name == "else");
                output.extend(lower_if(node, else_node, allow_children)?);
                index += if else_node.is_some() { 2 } else { 1 };
            }
            "else" => {
                return Err(node_error(
                    node,
                    "`else` must follow an `if` at the same indentation level",
                ));
            }
            "each" => {
                output.push(lower_each(node, allow_children)?);
                index += 1;
            }
            _ => {
                output.push(lower_view_node(node, allow_children)?);
                index += 1;
            }
        }
    }
    Ok(output)
}

fn lower_if(
    node: &SourceNode,
    else_node: Option<&SourceNode>,
    allow_children: bool,
) -> DoweResult<Vec<ViewNode>> {
    if node.children.is_empty() {
        return Err(node_error(node, "`if` must contain view nodes"));
    }
    if !node.props.is_empty() || node.args.len() != 1 {
        return Err(node_error(node, "`if` must declare one condition"));
    }
    let condition = node.args[0].as_string_like().unwrap_or_default();
    match condition.as_str() {
        "true" => lower_node_sequence(&node.children, allow_children),
        "false" => else_node
            .map(|node| lower_node_sequence(&node.children, allow_children))
            .unwrap_or_else(|| Ok(Vec::new())),
        _ => Err(node_error(
            node,
            "condition cannot be resolved by the current Dowe data surface",
        )),
    }
}

fn lower_each(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let (item, collection, key) = parse_each_header(node)?;
    if node.children.is_empty() {
        return Err(node_error(node, "`each` must contain view nodes"));
    }
    Ok(ViewNode::Each {
        item,
        collection,
        key,
        children: lower_node_sequence(&node.children, allow_children)?,
    })
}

fn parse_each_header(node: &SourceNode) -> DoweResult<(String, String, String)> {
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            "`each` must use `each in:collection as:item key:item.id`",
        ));
    }
    for prop in &node.props {
        if !["in", "as", "key"].contains(&prop.name.as_str()) {
            return Err(prop_error(
                prop,
                format!("unknown prop `{}` on `each`", prop.name),
            ));
        }
    }
    let collection = required_prop_bareword(node, "in")?;
    let item = required_prop_bareword(node, "as")?;
    let key = required_prop_bareword(node, "key")?;
    Ok((item, collection, key))
}

fn lower_view_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    if node.name == "children" {
        return children_node(allow_children).map_err(|error| node_error(node, error.to_string()));
    }
    if node.name == "Splash" {
        return Err(node_error(
            node,
            "Splash can only be used as a direct child of a layout or page",
        ));
    }
    if node.name == "Pagination" {
        return lower_pagination_node(node);
    }
    let component = COMPONENT_REGISTRY.get(&node.name).ok_or_else(|| {
        node_error(
            node,
            ComponentError::unknown_component(&node.name).to_string(),
        )
    })?;
    if component == BuiltinComponent::Code {
        return lower_code_node(node);
    }
    let props = component_props(node, component)?;
    match component {
        BuiltinComponent::Input => {
            reject_children(node)?;
            input_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Select => lower_select_node(node),
        BuiltinComponent::ComboBox => lower_combo_box_node(node),
        BuiltinComponent::CsvField => lower_csv_field_node(node),
        BuiltinComponent::DragDrop => lower_drag_drop_node(node),
        BuiltinComponent::Option => Err(node_error(
            node,
            ComponentError::invalid_prop_combination("Option can only be used inside Select")
                .to_string(),
        )),
        BuiltinComponent::ComboOption => Err(node_error(
            node,
            ComponentError::invalid_prop_combination(
                "comboOption can only be used inside ComboBox",
            )
            .to_string(),
        )),
        BuiltinComponent::CsvColumn => Err(node_error(
            node,
            ComponentError::invalid_prop_combination("csvColumn can only be used inside CsvField")
                .to_string(),
        )),
        BuiltinComponent::DragGroup => Err(node_error(
            node,
            ComponentError::invalid_prop_combination("dragGroup can only be used inside DragDrop")
                .to_string(),
        )),
        BuiltinComponent::DragItem => Err(node_error(
            node,
            ComponentError::invalid_prop_combination(
                "dragItem can only be used inside DragDrop or dragGroup",
            )
            .to_string(),
        )),
        BuiltinComponent::Code => unreachable!("Code lowers before scalar props"),
        BuiltinComponent::Video => {
            reject_children(node)?;
            video_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Iframe => {
            reject_children(node)?;
            iframe_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Device => {
            let children = lower_node_sequence(&node.children, allow_children)?;
            device_node(props, children).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Canvas => {
            reject_children(node)?;
            canvas_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Audio => {
            reject_children(node)?;
            audio_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Image => {
            reject_children(node)?;
            image_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Candlestick => {
            reject_children(node)?;
            candlestick_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::ArcChart => {
            reject_children(node)?;
            arc_chart_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::AreaChart => {
            reject_children(node)?;
            area_chart_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::BarChart => {
            reject_children(node)?;
            bar_chart_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::LineChart => {
            reject_children(node)?;
            line_chart_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::PieChart => {
            reject_children(node)?;
            pie_chart_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Table => lower_table_node(node),
        BuiltinComponent::Tabs => lower_tabs_node(node, allow_children),
        BuiltinComponent::Tab => Err(node_error(
            node,
            ComponentError::invalid_prop_combination("tab can only be used inside Tabs")
                .to_string(),
        )),
        BuiltinComponent::NavMenu => lower_nav_menu_node(node, allow_children),
        BuiltinComponent::Divider => {
            reject_children(node)?;
            divider_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Alert => {
            reject_children(node)?;
            container_component_node(component, props, Vec::new(), allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Avatar => lower_avatar_node(node),
        BuiltinComponent::Badge => {
            let children = lower_node_sequence(&node.children, allow_children)?;
            badge_component_node(props, children, allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Chip => lower_chip_node(node),
        BuiltinComponent::Skeleton => {
            reject_children(node)?;
            skeleton_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Drawer => lower_drawer_node(node, allow_children),
        BuiltinComponent::Modal => lower_modal_node(node, allow_children),
        BuiltinComponent::AlertDialog => {
            reject_children(node)?;
            alert_dialog_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Tooltip => {
            let children = lower_node_sequence(&node.children, allow_children)?;
            tooltip_component_node(props, children, allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Toast => {
            reject_children(node)?;
            toast_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Dropdown => lower_dropdown_node(node, allow_children),
        BuiltinComponent::Command => lower_command_node(node),
        BuiltinComponent::AvatarGroup => lower_avatar_group_node(node),
        BuiltinComponent::ChatBox => {
            reject_children(node)?;
            chat_box_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Empty => {
            reject_children(node)?;
            empty_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Marquee => {
            let children = lower_node_sequence(&node.children, allow_children)?;
            marquee_component_node(props, children, allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::TypeWriter => lower_type_writer_node(node),
        BuiltinComponent::RichText => lower_rich_text_node(node),
        BuiltinComponent::Record => {
            reject_children(node)?;
            record_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::ToggleGroup => lower_toggle_group_node(node),
        BuiltinComponent::Collapsible => {
            let children = lower_node_sequence(&node.children, allow_children)?;
            collapsible_component_node(props, children, allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Countdown => {
            reject_children(node)?;
            countdown_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Map => lower_map_node(node),
        BuiltinComponent::Accordion => lower_accordion_node(node, allow_children),
        BuiltinComponent::Carousel => lower_carousel_node(node, allow_children),
        BuiltinComponent::Checkbox => {
            reject_children(node)?;
            checkbox_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Color => {
            reject_children(node)?;
            color_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Date => {
            reject_children(node)?;
            date_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::DateRange => {
            reject_children(node)?;
            date_range_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::RadioGroup => lower_radio_group_node(node),
        BuiltinComponent::Toggle => {
            reject_children(node)?;
            toggle_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Title | BuiltinComponent::Text => {
            reject_text_prop(node, component)?;
            let value = required_text_child(node, component)?;
            text_component_node(component, props, value)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Button => {
            reject_text_prop(node, component)?;
            let value = required_text_child(node, component)?;
            let children = vec![text_node(value).map_err(|error| component_error(node, error))?];
            container_component_node(component, props, children, allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Brand => {
            let children = lower_node_sequence(&node.children, allow_children)?;
            container_component_node(component, props, children, allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Banner => {
            let children = lower_node_sequence(&node.children, allow_children)?;
            container_component_node(component, props, children, allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::IconButton => {
            reject_children(node)?;
            container_component_node(component, props, Vec::new(), allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::ToggleTheme => {
            reject_children(node)?;
            theme_toggle_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::SelectTheme => {
            reject_children(node)?;
            theme_select_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Fab => lower_fab_node(node),
        BuiltinComponent::FabAction => Err(node_error(
            node,
            ComponentError::invalid_prop_combination("fabAction can only be used inside Fab")
                .to_string(),
        )),
        BuiltinComponent::Slider => {
            reject_children(node)?;
            slider_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Dropzone => {
            reject_children(node)?;
            dropzone_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Editor => {
            reject_children(node)?;
            editor_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::ImageCropper => {
            reject_children(node)?;
            image_cropper_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::PasswordField => {
            reject_children(node)?;
            password_field_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::PhoneField => {
            reject_children(node)?;
            phone_field_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::PinField => {
            reject_children(node)?;
            pin_field_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Textarea => {
            reject_children(node)?;
            textarea_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Svg => lower_svg_node(node),
        BuiltinComponent::Icon => {
            reject_children(node)?;
            icon_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Path => Err(node_error(
            node,
            ComponentError::invalid_prop_combination("Path can only be used inside Svg")
                .to_string(),
        )),
        BuiltinComponent::AppBar | BuiltinComponent::Footer | BuiltinComponent::BottomBar => {
            lower_bar_node(node, component, allow_children)
        }
        BuiltinComponent::SideNav => lower_side_nav_node(node, component),
        BuiltinComponent::RailNav => lower_rail_nav_node(node),
        BuiltinComponent::Sidebar => lower_sidebar_node(node, allow_children),
        BuiltinComponent::Scaffold => lower_scaffold_node(node, allow_children),
        BuiltinComponent::Splash => Err(node_error(
            node,
            "Splash can only be used as a direct child of a layout or page",
        )),
        BuiltinComponent::Box
        | BuiltinComponent::Section
        | BuiltinComponent::Flex
        | BuiltinComponent::Grid
        | BuiltinComponent::Card => {
            let children = lower_node_sequence(&node.children, allow_children)?;
            container_component_node(component, props, children, allow_children)
                .map_err(|error| component_error(node, error))
        }
    }
}
