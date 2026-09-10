fn container_editor_and_charts(
    component: BuiltinComponent,
    props: &[ComponentProp],
    children: &[ViewNode],
    _allow_children: bool,
) -> ComponentResult<Option<ViewNode>> {
    match component {
        BuiltinComponent::Editor => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    editor_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::ImageCropper => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    image_cropper_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Password => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    password_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Phone => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    phone_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Pin => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    pin_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Textarea => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    textarea_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Select => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "Select can only contain Option children",
            ))
        })()
        .map(Some),
        BuiltinComponent::Option => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "Option can only be used inside Select",
            ))
        })()
        .map(Some),
        BuiltinComponent::Code => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    code_node(props, String::new())
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Video => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    video_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Iframe => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    iframe_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Device => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> { device_node(props, children) })().map(Some)
        }
        BuiltinComponent::Canvas => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    canvas_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Draw => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    draw_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Candlestick => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    candlestick_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Diagram => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    diagram_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::ArcChart => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    arc_chart_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::AreaChart => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    area_chart_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::BarChart => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    bar_chart_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::LineChart => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    line_chart_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::PieChart => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    pie_chart_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Table => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "Table requires column entries",
            ))
        })()
        .map(Some),
        BuiltinComponent::Tabs => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "Tabs requires tab entries",
            ))
        })()
        .map(Some),
        BuiltinComponent::Tab => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "tab can only be used inside Tabs",
            ))
        })()
        .map(Some),
        BuiltinComponent::Stepper => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "Stepper requires step entries",
            ))
        })()
        .map(Some),
        BuiltinComponent::Step => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "step can only be used inside Stepper",
            ))
        })()
        .map(Some),
        BuiltinComponent::NavMenu => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "NavMenu requires item, submenu or megamenu entries",
            ))
        })()
        .map(Some),
        _ => Ok(None),
    }
}
