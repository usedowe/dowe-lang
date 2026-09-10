fn container_media_and_forms(
    component: BuiltinComponent,
    props: &[ComponentProp],
    children: &[ViewNode],
    allow_children: bool,
) -> ComponentResult<Option<ViewNode>> {
    match component {
        BuiltinComponent::Audio => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    audio_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Camera => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    camera_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Microphone => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    microphone_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Image => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    image_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Checkbox => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    checkbox_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Color => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    color_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Date => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    date_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::DateRange => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    date_range_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Toggle => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    toggle_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Button => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                let props = parse_variant_props(component, &props)?;
                reject_children_placeholder(component, &children, allow_children)?;
                if children.iter().all(is_text_like) {
                    let mut children = children;
                    if let Some(key) = props.i18n.as_ref() {
                        for child in &mut children {
                            if let ViewNode::Text { props, .. } = child {
                                props.i18n = Some(key.clone());
                            }
                        }
                    }
                    Ok(ViewNode::Button { props, children })
                } else {
                    Err(ComponentError::text_cannot_contain_component_children(
                        component,
                    ))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Brand => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                reject_children_placeholder(component, &children, allow_children)?;
                if children.is_empty() {
                    return Err(ComponentError::invalid_prop_combination(
                        "Brand requires at least one child",
                    ));
                }
                Ok(ViewNode::Brand {
                    props: parse_brand_props(component, &props)?,
                    children,
                })
            })()
            .map(Some)
        }
        BuiltinComponent::Banner => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                reject_children_placeholder(component, &children, allow_children)?;
                if children.is_empty() {
                    return Err(ComponentError::invalid_prop_combination(
                        "Banner requires at least one child",
                    ));
                }
                Ok(ViewNode::Banner {
                    props: parse_banner_props(component, &props)?,
                    children,
                })
            })()
            .map(Some)
        }
        BuiltinComponent::IconButton | BuiltinComponent::Swap => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    let mut props = parse_variant_props(component, &props)?;
                    if component == BuiltinComponent::Swap {
                        let binding = props.style.element.bind.clone().ok_or_else(|| {
                            ComponentError::invalid_prop("bind", "boolean Signal path")
                        })?;
                        props.swap_bind = Some(binding);
                    }
                    Ok(ViewNode::Button { props, children })
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::ToggleTheme => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    theme_toggle_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::SelectTheme => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    theme_select_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Fab => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                reject_children_placeholder(component, &children, allow_children)?;
                fab_component_node(props, Vec::new())
            })()
            .map(Some)
        }
        BuiltinComponent::FabAction => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "fabAction can only be used inside Fab",
            ))
        })()
        .map(Some),
        BuiltinComponent::Input => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    input_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Slider => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    slider_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Dropzone => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    dropzone_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::ComboBox => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "ComboBox can only contain comboOption children",
            ))
        })()
        .map(Some),
        BuiltinComponent::ComboOption => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "comboOption can only be used inside ComboBox",
            ))
        })()
        .map(Some),
        BuiltinComponent::CsvField => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "CsvField can only contain csvColumn children",
            ))
        })()
        .map(Some),
        BuiltinComponent::CsvColumn => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "csvColumn can only be used inside CsvField",
            ))
        })()
        .map(Some),
        BuiltinComponent::DragDrop => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "DragDrop can only contain dragItem or dragGroup children",
            ))
        })()
        .map(Some),
        BuiltinComponent::DragGroup => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "dragGroup can only be used inside DragDrop",
            ))
        })()
        .map(Some),
        BuiltinComponent::DragItem => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "dragItem can only be used inside DragDrop or dragGroup",
            ))
        })()
        .map(Some),
        _ => Ok(None),
    }
}
