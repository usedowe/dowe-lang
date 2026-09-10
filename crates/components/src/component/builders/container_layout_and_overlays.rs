fn container_layout_and_overlays(
    component: BuiltinComponent,
    props: &[ComponentProp],
    children: &[ViewNode],
    allow_children: bool,
) -> ComponentResult<Option<ViewNode>> {
    match component {
        BuiltinComponent::Box => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                let props = parse_style_props(component, &props, StylePropMode::Box)?;
                container_node(component, Vec::new(), children, allow_children, props)
            })()
            .map(Some)
        }
        BuiltinComponent::Section => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                let props = parse_style_props(component, &props, StylePropMode::Section)?;
                container_node(component, Vec::new(), children, allow_children, props)
            })()
            .map(Some)
        }
        BuiltinComponent::Flex => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                let props = parse_layout_props(component, &props)?;
                if contains_children(&children) && !allow_children {
                    return Err(ComponentError::children_outside_layout());
                }
                Ok(ViewNode::Flex { props, children })
            })()
            .map(Some)
        }
        BuiltinComponent::Grid => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                let props = parse_grid_props(component, &props)?;
                if contains_children(&children) && !allow_children {
                    return Err(ComponentError::children_outside_layout());
                }
                Ok(ViewNode::Grid { props, children })
            })()
            .map(Some)
        }
        BuiltinComponent::Card => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                let props = parse_variant_props(component, &props)?;
                reject_children_placeholder(component, &children, allow_children)?;
                Ok(ViewNode::Card { props, children })
            })()
            .map(Some)
        }
        BuiltinComponent::Drawer => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                drawer_component_node(props, Vec::new(), children, Vec::new(), allow_children)
            })()
            .map(Some)
        }
        BuiltinComponent::Avatar => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    avatar_component_node(props, None)
                } else {
                    Err(ComponentError::invalid_prop_combination(
                        "Avatar only accepts an optional icon region",
                    ))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Badge => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                badge_component_node(props, children, allow_children)
            })()
            .map(Some)
        }
        BuiltinComponent::Chip => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
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
            })()
            .map(Some)
        }
        BuiltinComponent::Skeleton => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    skeleton_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Modal => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                modal_component_node(props, Vec::new(), children, Vec::new(), allow_children)
            })()
            .map(Some)
        }
        BuiltinComponent::AlertDialog => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    alert_dialog_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Tooltip => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                tooltip_component_node(props, children, allow_children)
            })()
            .map(Some)
        }
        BuiltinComponent::Toast => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    toast_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Dropdown => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "Dropdown requires trigger and item entries",
            ))
        })()
        .map(Some),
        BuiltinComponent::Command => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "Command requires item or group entries",
            ))
        })()
        .map(Some),
        BuiltinComponent::AvatarGroup => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                reject_children_placeholder(component, &children, allow_children)?;
                if children.is_empty() {
                    avatar_group_component_node(props, Vec::new())
                } else {
                    Err(ComponentError::invalid_prop_combination(
                        "AvatarGroup only accepts item entries",
                    ))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::ChatBox => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    chat_box_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Empty => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    empty_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Marquee => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                marquee_component_node(props, children, allow_children)
            })()
            .map(Some)
        }
        BuiltinComponent::TypeWriter => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "TypeWriter requires item entries",
            ))
        })()
        .map(Some),
        BuiltinComponent::RichText => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "RichText requires mark entries",
            ))
        })()
        .map(Some),
        BuiltinComponent::Record => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    record_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::ToggleGroup => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "ToggleGroup requires item entries",
            ))
        })()
        .map(Some),
        BuiltinComponent::Collapsible => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                collapsible_component_node(props, children, allow_children)
            })()
            .map(Some)
        }
        BuiltinComponent::Countdown => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    countdown_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Map => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "Map only accepts marker and waypoint entries",
            ))
        })()
        .map(Some),
        BuiltinComponent::Accordion => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "Accordion requires item entries",
            ))
        })()
        .map(Some),
        BuiltinComponent::Tree => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                reject_children_placeholder(component, &children, allow_children)?;
                tree_component_node(props)
            })()
            .map(Some)
        }
        BuiltinComponent::Carousel => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "Carousel requires slide entries",
            ))
        })()
        .map(Some),
        BuiltinComponent::RadioGroup => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "RadioGroup requires item entries",
            ))
        })()
        .map(Some),
        BuiltinComponent::RadioCard => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "RadioCard requires item entries",
            ))
        })()
        .map(Some),
        _ => Ok(None),
    }
}
