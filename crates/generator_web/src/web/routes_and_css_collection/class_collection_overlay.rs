fn collect_overlay_node_classes(node: &ViewNode, classes: &mut BTreeSet<String>) {
    match node {
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            classes.extend(modal_panel_classes(props));
            classes.extend(modal_classes(props));
            classes.insert("modal-overlay".to_string());
            classes.insert("modal-header".to_string());
            classes.insert("modal-body".to_string());
            classes.insert("modal-footer".to_string());
            classes.insert("modal-close".to_string());
            for child in header.iter().chain(body).chain(footer) {
                collect_classes(child, classes);
            }
        }
        ViewNode::AlertDialog { props } => {
            let modal = alert_dialog_modal_props(props);
            classes.extend(modal_panel_classes(&modal));
            classes.extend(modal_classes(&modal));
            classes.insert("modal-overlay".to_string());
            classes.insert("modal-header".to_string());
            classes.insert("modal-body".to_string());
            classes.insert("modal-footer".to_string());
            classes.insert("alert-dialog-title".to_string());
            classes.insert("alert-dialog-description".to_string());
            classes.insert("alert-dialog-actions".to_string());
            classes.extend(vec![
                "button".to_string(),
                "button-md".to_string(),
                "is-outlined".to_string(),
                "is-muted".to_string(),
            ]);
        }
        ViewNode::Tooltip { props, children } => {
            classes.extend(tooltip_classes(props));
            classes.extend(tooltip_popover_classes(props));
            classes.insert("tooltip-arrow".to_string());
            for child in children {
                collect_classes(child, classes);
            }
        }
        ViewNode::Toast { props } => {
            classes.extend(toast_classes(props));
            classes.insert("toast-content".to_string());
            classes.insert("toast-title".to_string());
            classes.insert("toast-description".to_string());
            classes.insert("toast-close".to_string());
        }
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            entries,
            footer,
        } => {
            classes.extend(dropdown_classes(props));
            classes.extend(dropdown_popover_classes(props));
            classes.insert("dropdown-trigger".to_string());
            classes.insert("dropdown-options".to_string());
            classes.insert("dropdown-divider".to_string());
            collect_overlay_entry_classes("dropdown", entries, classes);
            for child in trigger.iter().chain(header).chain(footer) {
                collect_classes(child, classes);
            }
        }
        ViewNode::Command { props, entries } => {
            classes.extend(command_panel_classes(props));
            classes.extend(command_classes(props));
            classes.insert("modal-overlay".to_string());
            classes.insert("command-header".to_string());
            classes.insert("command-input".to_string());
            classes.insert("command-kbd".to_string());
            classes.insert("command-results".to_string());
            classes.insert("command-empty".to_string());
            classes.insert("command-group".to_string());
            classes.insert("command-group-label".to_string());
            classes.insert("command-group-icon".to_string());
            classes.insert("command-group-items".to_string());
            classes.insert("command-shortcuts".to_string());
            collect_command_entry_classes(entries, classes);
        }
        _ => unreachable!(),
    }
}
