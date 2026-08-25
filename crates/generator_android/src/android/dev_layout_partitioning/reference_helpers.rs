fn dev_children_reference_layout_bindings(
    children: &[ViewNode],
    bindings: &DevLayoutBindings,
) -> bool {
    children
        .iter()
        .any(|child| dev_node_references_layout_bindings(child, bindings))
}

fn dev_action_references_layout_bindings(
    action: &ViewAction,
    bindings: &DevLayoutBindings,
) -> bool {
    match &action.kind {
        ViewActionKind::Sequence(statements) => statements.iter().any(|statement| match statement {
            dowe_components::ViewFunctionStatement::Request { action, .. } => action.body.as_deref().is_some_and(|value| bindings.references_signal(value)),
            dowe_components::ViewFunctionStatement::Assign(assign) => bindings.references_signal(&assign.target) || bindings.references_signal(&assign.source),
            dowe_components::ViewFunctionStatement::Reset(reset) => bindings.references_signal(&reset.target),
            dowe_components::ViewFunctionStatement::If { success, error, .. } => success.iter().chain(error).any(|step| matches!(step, dowe_components::ViewFunctionStatement::Assign(assign) if bindings.references_signal(&assign.target) || bindings.references_signal(&assign.source))),
            dowe_components::ViewFunctionStatement::Toast(_) => false,
            dowe_components::ViewFunctionStatement::Redirect { .. } => false,
            dowe_components::ViewFunctionStatement::Validate { .. } => false,
        }),
        ViewActionKind::Request(request) => [
            request.body.as_deref(),
            request.update.as_deref(),
            request.reset.as_deref(),
            request.success_alert.as_deref(),
            request.error_alert.as_deref(),
        ]
        .into_iter()
        .flatten()
        .any(|value| bindings.references_signal(value)),
        ViewActionKind::Assign(assign) => {
            bindings.references_signal(&assign.target) || bindings.references_signal(&assign.source)
        }
        ViewActionKind::Reset(reset) => bindings.references_signal(&reset.target),
    }
}

fn dev_element_references_layout_bindings(
    props: &ElementProps,
    bindings: &DevLayoutBindings,
) -> bool {
    props
        .bind
        .as_deref()
        .is_some_and(|value| bindings.references_signal(value))
        || props
            .on_click
            .as_deref()
            .is_some_and(|value| bindings.references_action(value))
        || props
            .show
            .as_ref()
            .is_some_and(|value| dev_visibility_references_layout_bindings(value, bindings))
}

fn dev_visibility_references_layout_bindings(
    value: &VisibilityCondition,
    bindings: &DevLayoutBindings,
) -> bool {
    match value {
        VisibilityCondition::Static(_) => false,
        VisibilityCondition::Signal(path) => bindings.references_signal(path),
        VisibilityCondition::NumberComparison { path, .. }
        | VisibilityCondition::StringEquality { path, .. } => bindings.references_signal(path),
    }
}

fn dev_style_references_layout_bindings(
    props: &StyleProps,
    bindings: &DevLayoutBindings,
) -> bool {
    dev_element_references_layout_bindings(&props.element, bindings)
}

fn dev_layout_references_layout_bindings(
    props: &LayoutProps,
    bindings: &DevLayoutBindings,
) -> bool {
    dev_style_references_layout_bindings(&props.style, bindings)
}

fn dev_grid_references_layout_bindings(
    props: &GridProps,
    bindings: &DevLayoutBindings,
) -> bool {
    dev_style_references_layout_bindings(&props.style, bindings)
}

fn dev_variant_references_layout_bindings(
    props: &VariantProps,
    bindings: &DevLayoutBindings,
) -> bool {
    dev_element_references_layout_bindings(&props.element, bindings)
        || dev_style_references_layout_bindings(&props.style, bindings)
}

fn dev_text_references_layout_bindings(
    props: &TextProps,
    value: &str,
    bindings: &DevLayoutBindings,
) -> bool {
    dev_style_references_layout_bindings(&props.style, bindings)
        || (props.i18n.is_none()
            && text_template_bindings(value).any(|path| bindings.references_signal(&path)))
}

fn dev_chart_references_layout_bindings(
    props: &ChartCommonProps,
    bindings: &DevLayoutBindings,
) -> bool {
    dev_variant_references_layout_bindings(&props.style, bindings)
        || props
            .data
            .as_deref()
            .is_some_and(|value| bindings.references_signal(value))
        || props
            .series
            .as_deref()
            .is_some_and(|value| bindings.references_signal(value))
}

fn dev_side_nav_items_reference_layout_bindings(
    items: &[SideNavItem],
    bindings: &DevLayoutBindings,
) -> bool {
    items.iter().any(|item| match item {
        SideNavItem::Header(props) | SideNavItem::Item(props) => props
            .on_click
            .as_deref()
            .is_some_and(|value| bindings.references_action(value)),
        SideNavItem::Submenu { props, items, .. } => {
            props
                .on_click
                .as_deref()
                .is_some_and(|value| bindings.references_action(value))
                || items.iter().any(|item| {
                    item.on_click
                        .as_deref()
                        .is_some_and(|value| bindings.references_action(value))
                })
        }
        SideNavItem::Divider => false,
    })
}

fn dev_nav_menu_items_reference_layout_bindings(
    items: &[NavMenuItem],
    bindings: &DevLayoutBindings,
) -> bool {
    items.iter().any(|item| match item {
        NavMenuItem::Item(props) => props
            .on_click
            .as_deref()
            .is_some_and(|value| bindings.references_action(value)),
        NavMenuItem::Submenu { props, items } => {
            props
                .on_click
                .as_deref()
                .is_some_and(|value| bindings.references_action(value))
                || items.iter().any(|item| {
                    item.on_click
                        .as_deref()
                        .is_some_and(|value| bindings.references_action(value))
                })
        }
        NavMenuItem::Megamenu { props, content } => {
            props
                .on_click
                .as_deref()
                .is_some_and(|value| bindings.references_action(value))
                || dev_children_reference_layout_bindings(content, bindings)
        }
    })
}

fn dev_overlay_entries_reference_layout_bindings(
    entries: &[OverlayEntry],
    bindings: &DevLayoutBindings,
) -> bool {
    entries.iter().any(|entry| match entry {
        OverlayEntry::Item(props) => props
            .on_click
            .as_deref()
            .is_some_and(|value| bindings.references_action(value)),
        OverlayEntry::Divider => false,
    })
}

fn dev_command_entries_reference_layout_bindings(
    entries: &[CommandEntry],
    bindings: &DevLayoutBindings,
) -> bool {
    entries.iter().any(|entry| match entry {
        CommandEntry::Item(props) => props
            .on_click
            .as_deref()
            .is_some_and(|value| bindings.references_action(value)),
        CommandEntry::Group { items, .. } => items.iter().any(|item| {
            item.on_click
                .as_deref()
                .is_some_and(|value| bindings.references_action(value))
        }),
    })
}

