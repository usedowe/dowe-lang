fn collect_display_node_classes(node: &ViewNode, classes: &mut BTreeSet<String>) {
    match node {
        ViewNode::Avatar { props, .. } => {
            classes.extend(avatar_classes(props));
            classes.insert("avatar-image".to_string());
            classes.insert("avatar-icon".to_string());
            classes.insert("avatar-name".to_string());
            classes.insert("avatar-status".to_string());
            classes.insert("avatar-indicator".to_string());
        }
        ViewNode::AvatarGroup { props, items } => {
            classes.extend(avatar_group_classes(props));
            classes.insert("avatar-group-list".to_string());
            classes.insert("avatar-group-counter".to_string());
            for item in items {
                let avatar = AvatarProps {
                    style: props.style.clone(),
                    src: item.src.clone(),
                    name: item.name.clone(),
                    alt: item.alt.clone().unwrap_or_default(),
                    size: props.size,
                    status: None,
                    bordered: props.bordered,
                };
                classes.extend(avatar_classes(&avatar));
            }
        }
        ViewNode::ChatBox { props } => {
            classes.extend(chat_box_classes(props));
            classes.extend([
                "chat-box-header".to_string(),
                "chat-box-user".to_string(),
                "chat-box-avatar".to_string(),
                "chat-box-user-copy".to_string(),
                "chat-box-header-actions".to_string(),
                "chat-box-icon".to_string(),
                "chat-box-body".to_string(),
                "chat-box-messages".to_string(),
                "chat-message".to_string(),
                "chat-bubble".to_string(),
                "chat-meta".to_string(),
                "chat-box-typing".to_string(),
                "chat-box-footer".to_string(),
                "chat-box-input-wrap".to_string(),
                "chat-box-input".to_string(),
                "chat-box-tool".to_string(),
                "chat-box-send".to_string(),
                "chat-box-stop".to_string(),
            ]);
        }
        ViewNode::Empty { props } => {
            classes.extend(empty_classes(props));
            classes.extend([
                "empty-icon".to_string(),
                "empty-content".to_string(),
                "empty-title".to_string(),
                "empty-description".to_string(),
                "empty-actions".to_string(),
                "button".to_string(),
                "button-md".to_string(),
            ]);
        }
        ViewNode::Marquee { props, children } => {
            classes.extend(marquee_classes(props));
            classes.insert("marquee-track".to_string());
            classes.insert("marquee-content".to_string());
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::TypeWriter { props, .. } => {
            classes.extend(type_writer_classes(props));
            classes.insert("typewriter-text".to_string());
            classes.insert("typewriter-caret".to_string());
        }
        ViewNode::RichText { props, marks } => {
            classes.extend(rich_text_classes(props));
            classes.insert("rich-mark".to_string());
            for mark in marks {
                classes.insert(format!("rich-mark-{}", mark.style.as_str()));
                classes.insert(format!("is-{}", mark.color.as_str()));
            }
        }
        ViewNode::Record { props } => {
            classes.extend(record_classes(props));
            classes.extend([
                "record-main".to_string(),
                "record-wave".to_string(),
                "record-bar".to_string(),
                "record-meta".to_string(),
                "record-time".to_string(),
                "record-status".to_string(),
                "record-actions".to_string(),
                "record-btn".to_string(),
                "record-start".to_string(),
                "record-pause".to_string(),
                "record-stop".to_string(),
                "record-discard".to_string(),
                "record-confirm".to_string(),
                "record-file".to_string(),
            ]);
        }
        ViewNode::ToggleGroup { props, .. } => {
            classes.extend(toggle_group_classes(props));
            classes.extend([
                "toggle-group-item".to_string(),
                "toggle-group-icon".to_string(),
                "is-active".to_string(),
            ]);
        }
        ViewNode::Collapsible { props, children } => {
            classes.extend(collapsible_classes(props));
            classes.extend([
                "collapsible-header".to_string(),
                "collapsible-label".to_string(),
                "collapsible-arrow".to_string(),
                "collapsible-content".to_string(),
            ]);
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Countdown { props } => {
            classes.extend(countdown_classes(props));
            classes.extend([
                "countdown-unit".to_string(),
                "countdown-box".to_string(),
                "countdown-digit".to_string(),
                "countdown-label".to_string(),
                "countdown-separator".to_string(),
            ]);
        }
        ViewNode::Map {
            props,
            markers,
            waypoints,
        } => {
            classes.extend(map_classes(props));
            classes.extend([
                "map-container".to_string(),
                "map-grid".to_string(),
                "map-route".to_string(),
                "map-marker".to_string(),
                "map-marker-pin".to_string(),
                "map-marker-label".to_string(),
                "map-waypoint".to_string(),
                "map-controls".to_string(),
                "map-scale".to_string(),
                "map-location-btn".to_string(),
            ]);
            for marker in markers {
                classes.insert(format!("is-{}", marker.icon.as_str()));
            }
            if !waypoints.is_empty() {
                classes.insert("map-waypoint".to_string());
            }
        }
        ViewNode::Badge { props, children } => {
            classes.extend(badge_classes(props));
            classes.insert("badge-content".to_string());
            classes.insert("badge-text".to_string());
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Chip { props, .. } => {
            classes.extend(chip_classes(props));
            classes.insert("chip-label".to_string());
            classes.insert("chip-icon".to_string());
            classes.insert("chip-close".to_string());
        }
        ViewNode::Skeleton { props } => {
            classes.extend(skeleton_classes(props));
        }
        _ => unreachable!(),
    }
}
