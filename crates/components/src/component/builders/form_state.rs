pub fn collect_view_forms(node: &ViewNode) -> Vec<ViewForm> {
    let mut fields = std::collections::BTreeMap::<String, Vec<ViewFormField>>::new();
    collect_view_form_fields(node, &mut fields);
    fields
        .into_iter()
        .map(|(signal, fields)| ViewForm { signal, fields })
        .collect()
}

fn collect_view_form_fields(node: &ViewNode, forms: &mut BTreeMap<String, Vec<ViewFormField>>) {
    let field_kind = match node {
        ViewNode::Checkbox { .. } => Some(ViewFormFieldKind::Boolean),
        ViewNode::Date { .. }
        | ViewNode::Input { .. }
        | ViewNode::Password { .. }
        | ViewNode::Pin { .. }
        | ViewNode::Phone { .. }
        | ViewNode::Select { .. } => Some(ViewFormFieldKind::String),
        _ => None,
    };

    if let (Some(kind), Some(element)) = (field_kind, node_element_props(node)) {
        if let (Some(bind), Some(validation)) = (&element.bind, element.form_validation()) {
            if !validation.rules.is_empty() {
                if let Some((signal, path)) = bind.split_once('.') {
                    let field = ViewFormField {
                        path: path.to_string(),
                        kind,
                        rules: validation.rules.clone(),
                    };
                    let entry = forms.entry(signal.to_string()).or_default();
                    if let Some(existing) = entry.iter_mut().find(|item| item.path == field.path) {
                        for rule in field.rules {
                            if !existing.rules.contains(&rule) {
                                existing.rules.push(rule);
                            }
                        }
                    } else {
                        entry.push(field);
                    }
                }
            }
        }
    }

    for child in form_children(node) {
        collect_view_form_fields(child, forms);
    }
}

fn form_children(node: &ViewNode) -> Vec<&ViewNode> {
    let mut children = Vec::new();
    match node {
        ViewNode::Scope {
            children: values, ..
        }
        | ViewNode::Each {
            children: values, ..
        }
        | ViewNode::Box {
            children: values, ..
        }
        | ViewNode::Section {
            children: values, ..
        }
        | ViewNode::Flex {
            children: values, ..
        }
        | ViewNode::Grid {
            children: values, ..
        }
        | ViewNode::Card {
            children: values, ..
        }
        | ViewNode::Button {
            children: values, ..
        }
        | ViewNode::Brand {
            children: values, ..
        }
        | ViewNode::Banner {
            children: values, ..
        }
        | ViewNode::Badge {
            children: values, ..
        }
        | ViewNode::Tooltip {
            children: values, ..
        }
        | ViewNode::Marquee {
            children: values, ..
        }
        | ViewNode::Collapsible {
            children: values, ..
        } => children.extend(values),
        ViewNode::Splash {
            content,
            children: values,
            ..
        } => {
            children.extend(content);
            children.extend(values);
        }
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                children.extend(&tab.children);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                if let NavMenuItem::Megamenu { content, .. } = item {
                    children.extend(content);
                }
            }
        }
        ViewNode::AppBar {
            top,
            start,
            center,
            end,
            bottom,
            ..
        }
        | ViewNode::Footer {
            top,
            start,
            center,
            end,
            bottom,
            ..
        } => {
            children.extend(top);
            children.extend(start);
            children.extend(center);
            children.extend(end);
            children.extend(bottom);
        }
        ViewNode::Sidebar {
            header,
            body,
            footer,
            ..
        }
        | ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        }
        | ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            children.extend(header);
            children.extend(body);
            children.extend(footer);
        }
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
            ..
        } => {
            children.extend(app_bar);
            children.extend(start);
            children.extend(main);
            children.extend(end);
            children.extend(bottom_bar);
            children.extend(overlays);
        }
        _ => {}
    }
    children
}
