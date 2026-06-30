pub fn container_component_node(
    component: BuiltinComponent,
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    match component {
        BuiltinComponent::Box => {
            let props = parse_style_props(component, &props, StylePropMode::Box)?;
            container_node(component, Vec::new(), children, allow_children, props)
        }
        BuiltinComponent::Section => {
            let props = parse_style_props(component, &props, StylePropMode::Section)?;
            container_node(component, Vec::new(), children, allow_children, props)
        }
        BuiltinComponent::Flex => {
            let props = parse_layout_props(component, &props)?;
            if contains_children(&children) && !allow_children {
                return Err(ComponentError::children_outside_layout());
            }
            Ok(ViewNode::Flex { props, children })
        }
        BuiltinComponent::Grid => {
            let props = parse_grid_props(component, &props)?;
            if contains_children(&children) && !allow_children {
                return Err(ComponentError::children_outside_layout());
            }
            Ok(ViewNode::Grid { props, children })
        }
        BuiltinComponent::Card => {
            let props = parse_variant_props(component, &props)?;
            reject_children_placeholder(component, &children, allow_children)?;
            Ok(ViewNode::Card { props, children })
        }
        BuiltinComponent::Drawer => {
            drawer_component_node(props, Vec::new(), children, Vec::new(), allow_children)
        }
        BuiltinComponent::Avatar => {
            if children.is_empty() {
                avatar_component_node(props, None)
            } else {
                Err(ComponentError::invalid_prop_combination(
                    "Avatar only accepts an optional icon region",
                ))
            }
        }
        BuiltinComponent::Badge => badge_component_node(props, children, allow_children),
        BuiltinComponent::Chip => {
            reject_children_placeholder(component, &children, allow_children)?;
            if children.iter().all(is_text_like) {
                let value = children
                    .iter()
                    .filter_map(first_text)
                    .collect::<Vec<_>>()
                    .join(" ");
                chip_component_node(props, value, None, None)
            } else {
                Err(ComponentError::text_cannot_contain_component_children(
                    component,
                ))
            }
        }
        BuiltinComponent::Skeleton => {
            if children.is_empty() {
                skeleton_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Modal => {
            modal_component_node(props, Vec::new(), children, Vec::new(), allow_children)
        }
        BuiltinComponent::AlertDialog => {
            if children.is_empty() {
                alert_dialog_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Tooltip => tooltip_component_node(props, children, allow_children),
        BuiltinComponent::Toast => {
            if children.is_empty() {
                toast_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Dropdown => Err(ComponentError::invalid_prop_combination(
            "Dropdown requires trigger and item entries",
        )),
        BuiltinComponent::Command => Err(ComponentError::invalid_prop_combination(
            "Command requires item or group entries",
        )),
        BuiltinComponent::AvatarGroup => {
            reject_children_placeholder(component, &children, allow_children)?;
            if children.is_empty() {
                avatar_group_component_node(props, Vec::new())
            } else {
                Err(ComponentError::invalid_prop_combination(
                    "AvatarGroup only accepts item entries",
                ))
            }
        }
        BuiltinComponent::ChatBox => {
            if children.is_empty() {
                chat_box_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Empty => {
            if children.is_empty() {
                empty_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Marquee => marquee_component_node(props, children, allow_children),
        BuiltinComponent::TypeWriter => Err(ComponentError::invalid_prop_combination(
            "TypeWriter requires item entries",
        )),
        BuiltinComponent::RichText => Err(ComponentError::invalid_prop_combination(
            "RichText requires mark entries",
        )),
        BuiltinComponent::Record => {
            if children.is_empty() {
                record_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::ToggleGroup => Err(ComponentError::invalid_prop_combination(
            "ToggleGroup requires item entries",
        )),
        BuiltinComponent::Collapsible => {
            collapsible_component_node(props, children, allow_children)
        }
        BuiltinComponent::Countdown => {
            if children.is_empty() {
                countdown_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Map => Err(ComponentError::invalid_prop_combination(
            "Map only accepts marker and waypoint entries",
        )),
        BuiltinComponent::Accordion => Err(ComponentError::invalid_prop_combination(
            "Accordion requires item entries",
        )),
        BuiltinComponent::Carousel => Err(ComponentError::invalid_prop_combination(
            "Carousel requires slide entries",
        )),
        BuiltinComponent::RadioGroup => Err(ComponentError::invalid_prop_combination(
            "RadioGroup requires item entries",
        )),
        BuiltinComponent::Audio => {
            if children.is_empty() {
                audio_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Image => {
            if children.is_empty() {
                image_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Checkbox => {
            if children.is_empty() {
                checkbox_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Color => {
            if children.is_empty() {
                color_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Date => {
            if children.is_empty() {
                date_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::DateRange => {
            if children.is_empty() {
                date_range_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Toggle => {
            if children.is_empty() {
                toggle_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Button => {
            let props = parse_variant_props(component, &props)?;
            reject_children_placeholder(component, &children, allow_children)?;
            if children.iter().all(is_text_like) {
                Ok(ViewNode::Button { props, children })
            } else {
                Err(ComponentError::text_cannot_contain_component_children(
                    component,
                ))
            }
        }
        BuiltinComponent::ToggleTheme => {
            if children.is_empty() {
                theme_toggle_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Fab => {
            reject_children_placeholder(component, &children, allow_children)?;
            fab_component_node(props, Vec::new())
        }
        BuiltinComponent::FabAction => Err(ComponentError::invalid_prop_combination(
            "fabAction can only be used inside Fab",
        )),
        BuiltinComponent::Input => {
            if children.is_empty() {
                input_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Slider => {
            if children.is_empty() {
                slider_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Dropzone => {
            if children.is_empty() {
                dropzone_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::ComboBox => Err(ComponentError::invalid_prop_combination(
            "ComboBox can only contain comboOption children",
        )),
        BuiltinComponent::ComboOption => Err(ComponentError::invalid_prop_combination(
            "comboOption can only be used inside ComboBox",
        )),
        BuiltinComponent::CsvField => Err(ComponentError::invalid_prop_combination(
            "CsvField can only contain csvColumn children",
        )),
        BuiltinComponent::CsvColumn => Err(ComponentError::invalid_prop_combination(
            "csvColumn can only be used inside CsvField",
        )),
        BuiltinComponent::DragDrop => Err(ComponentError::invalid_prop_combination(
            "DragDrop can only contain dragItem or dragGroup children",
        )),
        BuiltinComponent::DragGroup => Err(ComponentError::invalid_prop_combination(
            "dragGroup can only be used inside DragDrop",
        )),
        BuiltinComponent::DragItem => Err(ComponentError::invalid_prop_combination(
            "dragItem can only be used inside DragDrop or dragGroup",
        )),
        BuiltinComponent::Editor => {
            if children.is_empty() {
                editor_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::ImageCropper => {
            if children.is_empty() {
                image_cropper_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::PasswordField => {
            if children.is_empty() {
                password_field_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::PhoneField => {
            if children.is_empty() {
                phone_field_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::PinField => {
            if children.is_empty() {
                pin_field_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Textarea => {
            if children.is_empty() {
                textarea_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Select => Err(ComponentError::invalid_prop_combination(
            "Select can only contain Option children",
        )),
        BuiltinComponent::Option => Err(ComponentError::invalid_prop_combination(
            "Option can only be used inside Select",
        )),
        BuiltinComponent::Code => {
            if children.is_empty() {
                code_node(props, Vec::new())
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Video => {
            if children.is_empty() {
                video_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Candlestick => {
            if children.is_empty() {
                candlestick_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::ArcChart => {
            if children.is_empty() {
                arc_chart_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::AreaChart => {
            if children.is_empty() {
                area_chart_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::BarChart => {
            if children.is_empty() {
                bar_chart_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::LineChart => {
            if children.is_empty() {
                line_chart_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::PieChart => {
            if children.is_empty() {
                pie_chart_component_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Table => Err(ComponentError::invalid_prop_combination(
            "Table requires column entries",
        )),
        BuiltinComponent::Tabs => Err(ComponentError::invalid_prop_combination(
            "Tabs requires tab entries",
        )),
        BuiltinComponent::Tab => Err(ComponentError::invalid_prop_combination(
            "tab can only be used inside Tabs",
        )),
        BuiltinComponent::NavMenu => Err(ComponentError::invalid_prop_combination(
            "NavMenu requires item, submenu or megamenu entries",
        )),
        BuiltinComponent::Divider => {
            if children.is_empty() {
                divider_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Alert => {
            if children.is_empty() {
                alert_node(props)
            } else {
                Err(ComponentError::children_not_allowed(component))
            }
        }
        BuiltinComponent::Svg => svg_component_node(props, Vec::new()),
        BuiltinComponent::Path => Err(ComponentError::invalid_prop_combination(
            "Path can only be used inside Svg",
        )),
        BuiltinComponent::AppBar | BuiltinComponent::Footer | BuiltinComponent::BottomBar => {
            Err(ComponentError::invalid_prop_combination(
                "bar components require start, center or end regions",
            ))
        }
        BuiltinComponent::SideNav => Err(ComponentError::invalid_prop_combination(
            "SideNav requires header, item, divider or submenu entries",
        )),
        BuiltinComponent::Sidebar => Err(ComponentError::invalid_prop_combination(
            "Sidebar requires header, body or footer regions",
        )),
        BuiltinComponent::Scaffold => Err(ComponentError::invalid_prop_combination(
            "Scaffold requires appBar, main and optional side regions",
        )),
        BuiltinComponent::Title | BuiltinComponent::Text => Err(
            ComponentError::text_cannot_contain_component_children(component),
        ),
    }
}
