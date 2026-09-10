fn ios_children_reference_layout_bindings(
    children: &[ViewNode],
    bindings: &IosLayoutBindings,
) -> bool {
    children
        .iter()
        .any(|child| ios_node_references_layout_bindings(child, bindings))
}

fn ios_action_references_layout_bindings(
    action: &ViewAction,
    bindings: &IosLayoutBindings,
) -> bool {
    match &action.kind {
        ViewActionKind::Sequence(statements) => statements.iter().any(|statement| match statement {
            dowe_components::ViewFunctionStatement::Request { action, .. } => action.body.as_deref().is_some_and(|value| bindings.references_signal(value)),
            dowe_components::ViewFunctionStatement::Assign(assign) => bindings.references_signal(&assign.target) || bindings.references_signal(&assign.source),
            dowe_components::ViewFunctionStatement::Reset(reset) => bindings.references_signal(&reset.target),
            dowe_components::ViewFunctionStatement::If { success, error, .. } => success.iter().chain(error).any(|step| matches!(step, dowe_components::ViewFunctionStatement::Assign(assign) if bindings.references_signal(&assign.target) || bindings.references_signal(&assign.source))),
            dowe_components::ViewFunctionStatement::Invoke { .. } => false,
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
        ViewActionKind::Invoke(_) => false,
        ViewActionKind::Assign(assign) => {
            bindings.references_signal(&assign.target) || bindings.references_signal(&assign.source)
        }
        ViewActionKind::Reset(reset) => bindings.references_signal(&reset.target),
    }
}

fn ios_element_references_layout_bindings(
    props: &ElementProps,
    bindings: &IosLayoutBindings,
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
            .is_some_and(|value| ios_visibility_references_layout_bindings(value, bindings))
}

fn ios_visibility_references_layout_bindings(
    value: &VisibilityCondition,
    bindings: &IosLayoutBindings,
) -> bool {
    match value {
        VisibilityCondition::Static(_) => false,
        VisibilityCondition::Signal(path) => bindings.references_signal(path),
        VisibilityCondition::NumberComparison { path, .. }
        | VisibilityCondition::StringEquality { path, .. } => bindings.references_signal(path),
    }
}

fn ios_style_references_layout_bindings(
    props: &StyleProps,
    bindings: &IosLayoutBindings,
) -> bool {
    ios_element_references_layout_bindings(&props.element, bindings)
}

fn ios_layout_references_layout_bindings(
    props: &LayoutProps,
    bindings: &IosLayoutBindings,
) -> bool {
    ios_style_references_layout_bindings(&props.style, bindings)
}

fn ios_grid_references_layout_bindings(
    props: &GridProps,
    bindings: &IosLayoutBindings,
) -> bool {
    ios_style_references_layout_bindings(&props.style, bindings)
}

fn ios_variant_references_layout_bindings(
    props: &VariantProps,
    bindings: &IosLayoutBindings,
) -> bool {
    ios_element_references_layout_bindings(&props.element, bindings)
        || ios_style_references_layout_bindings(&props.style, bindings)
}

fn ios_text_references_layout_bindings(
    props: &TextProps,
    value: &str,
    bindings: &IosLayoutBindings,
) -> bool {
    ios_style_references_layout_bindings(&props.style, bindings)
        || (props.i18n.is_none()
            && text_template_bindings(value).any(|path| bindings.references_signal(&path)))
}

fn ios_chart_references_layout_bindings(
    props: &ChartCommonProps,
    bindings: &IosLayoutBindings,
) -> bool {
    ios_variant_references_layout_bindings(&props.style, bindings)
        || props
            .data
            .as_deref()
            .is_some_and(|value| bindings.references_signal(value))
        || props
            .series
            .as_deref()
            .is_some_and(|value| bindings.references_signal(value))
}

fn ios_side_nav_items_reference_layout_bindings(
    items: &[SideNavItem],
    bindings: &IosLayoutBindings,
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

fn ios_nav_menu_items_reference_layout_bindings(
    items: &[NavMenuItem],
    bindings: &IosLayoutBindings,
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
                || ios_children_reference_layout_bindings(content, bindings)
        }
    })
}

fn ios_overlay_entries_reference_layout_bindings(
    entries: &[OverlayEntry],
    bindings: &IosLayoutBindings,
) -> bool {
    entries.iter().any(|entry| match entry {
        OverlayEntry::Item(props) => props
            .on_click
            .as_deref()
            .is_some_and(|value| bindings.references_action(value)),
        OverlayEntry::Divider => false,
    })
}

fn ios_command_entries_reference_layout_bindings(
    entries: &[CommandEntry],
    bindings: &IosLayoutBindings,
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
