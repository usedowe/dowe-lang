fn collect_layout_node_classes(node: &ViewNode, classes: &mut BTreeSet<String>) {
    match node {
        ViewNode::Scope { children, .. } | ViewNode::Each { children, .. } => {
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Box { props, children } => {
            classes.extend(box_classes(props));
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Section { props, children } => {
            classes.extend(section_classes(props));
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Flex { props, children } => {
            classes.extend(layout_classes("flex", props));
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Grid { props, children } => {
            classes.extend(grid_classes(props));
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Card { props, children } => {
            classes.extend(variant_classes("card", props));
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Button { props, children } => {
            classes.extend(variant_classes("button", props));
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::ToggleTheme { props } => {
            classes.extend(variant_classes("theme-toggle", &props.style));
            classes.insert("theme-toggle-icon".to_string());
            classes.insert("theme-icon".to_string());
            classes.insert("theme-icon-moon".to_string());
            classes.insert("theme-icon-sun".to_string());
        }
        ViewNode::Fab { props, actions } => {
            classes.extend([
                "fab-container".to_string(),
                "fab-actions".to_string(),
                "fab-action".to_string(),
                "fab-action-label".to_string(),
                "fab-action-button".to_string(),
                "fab-icon".to_string(),
                "fab-icon-svg".to_string(),
                format!("is-{}", props.position.as_str()),
            ]);
            if props.fixed {
                classes.insert("is-fixed".to_string());
            }
            classes.extend(variant_classes("fab-trigger", &props.style));
            for action in actions {
                classes.extend(variant_classes(
                    "fab-action-button",
                    &VariantProps {
                        color: Some(action.color),
                        variant: props.style.variant,
                        ..VariantProps::default()
                    },
                ));
            }
        }
        _ => unreachable!(),
    }
}
