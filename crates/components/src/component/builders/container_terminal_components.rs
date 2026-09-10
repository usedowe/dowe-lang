fn container_terminal_components(
    component: BuiltinComponent,
    props: &[ComponentProp],
    children: &[ViewNode],
    _allow_children: bool,
) -> ComponentResult<Option<ViewNode>> {
    match component {
        BuiltinComponent::Divider => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    divider_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Alert => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    alert_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Icon => {
            let props = props.to_vec();
            let children = children.to_vec();
            (|| -> ComponentResult<ViewNode> {
                if children.is_empty() {
                    icon_component_node(props)
                } else {
                    Err(ComponentError::children_not_allowed(component))
                }
            })()
            .map(Some)
        }
        BuiltinComponent::Svg => {
            let props = props.to_vec();
            (|| -> ComponentResult<ViewNode> { svg_component_node(props, Vec::new()) })().map(Some)
        }
        BuiltinComponent::Path => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "Path can only be used inside Svg",
            ))
        })()
        .map(Some),
        BuiltinComponent::AppBar | BuiltinComponent::Footer | BuiltinComponent::BottomBar => {
            (|| -> ComponentResult<ViewNode> {
                Err(ComponentError::invalid_prop_combination(
                    "bar components require start, center or end regions",
                ))
            })()
            .map(Some)
        }
        BuiltinComponent::SideNav => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "SideNav requires header, item, divider or submenu entries",
            ))
        })()
        .map(Some),
        BuiltinComponent::RailNav => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "RailNav requires item or divider entries",
            ))
        })()
        .map(Some),
        BuiltinComponent::Sidebar => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "Sidebar requires header, body or footer regions",
            ))
        })()
        .map(Some),
        BuiltinComponent::Scaffold => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "Scaffold requires appBar, main and optional side regions",
            ))
        })()
        .map(Some),
        BuiltinComponent::Splash => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::invalid_prop_combination(
                "Splash can only be used as a direct child of a layout or page",
            ))
        })()
        .map(Some),
        BuiltinComponent::Title | BuiltinComponent::Text => (|| -> ComponentResult<ViewNode> {
            Err(ComponentError::text_cannot_contain_component_children(
                component,
            ))
        })()
        .map(Some),
        _ => Ok(None),
    }
}
