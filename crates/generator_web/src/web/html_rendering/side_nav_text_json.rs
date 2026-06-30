fn render_side_nav_item_html(
    base: &str,
    item: &SideNavItem,
    context: &ReactiveRenderContext,
) -> String {
    match item {
        SideNavItem::Header(props) => {
            render_side_nav_entry_html(base, props, &format!("{base}-header"), context)
        }
        SideNavItem::Item(props) => {
            render_side_nav_entry_html(base, props, &format!("{base}-entry"), context)
        }
        SideNavItem::Divider => format!(r#"<div class="{base}-divider"></div>"#),
        SideNavItem::Submenu {
            props,
            open,
            bordered,
            items,
        } => {
            let classes = if *open {
                format!("{base}-submenu is-open")
            } else {
                format!("{base}-submenu")
            };
            let classes = if *bordered {
                classes
            } else {
                format!("{classes} is-unbordered")
            };
            let mut html = format!(
                r#"<details class="{classes}" data-dowe-{base}-submenu{}><summary class="{base}-entry {base}-trigger" aria-expanded="{}">{}{}{}</summary><div class="{base}-submenu-content">"#,
                if *open { " open" } else { "" },
                if *open { "true" } else { "false" },
                render_side_nav_icon_html(base, props.icon.as_ref(), context),
                render_side_nav_content_html(base, props),
                render_side_nav_arrow_html(base)
            );
            for item in items {
                html.push_str(&render_side_nav_entry_html(
                    base,
                    item,
                    &format!("{base}-entry {base}-subitem"),
                    context,
                ));
            }
            html.push_str("</div></details>");
            html
        }
    }
}

fn render_side_nav_arrow_html(base: &str) -> String {
    format!(
        r#"<span class="{base}-chevron" aria-hidden="true"><svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path d="M0 0h24v24H0z" fill="none" /><path fill="currentColor" d="m19.704 12l-8.491-8.727a.75.75 0 1 1 1.075-1.046l9 9.25a.75.75 0 0 1 0 1.046l-9 9.25a.75.75 0 1 1-1.075-1.046z" /></svg></span>"#
    )
}

fn render_side_nav_entry_html(
    base: &str,
    props: &SideNavItemProps,
    classes: &str,
    context: &ReactiveRenderContext,
) -> String {
    let (tag, attrs, close) = side_nav_entry_tags(base, props, classes, context);
    format!(
        "<{tag}{attrs}>{}{}</{close}>",
        render_side_nav_icon_html(base, props.icon.as_ref(), context),
        render_side_nav_content_html(base, props)
    )
}

fn side_nav_entry_tags(
    base: &str,
    props: &SideNavItemProps,
    classes: &str,
    context: &ReactiveRenderContext,
) -> (&'static str, String, &'static str) {
    let classes = class_attr(
        classes
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>(),
    );
    match props.navigation.as_ref() {
        Some(action) => (
            "a",
            format!("{classes}{}", side_nav_navigation_attrs(base, action)),
            "a",
        ),
        None if props.on_click.is_some() => (
            "button",
            format!(
                r#"{classes} type="button" data-dowe-click="{}""#,
                escape_attr(&context.action_id(props.on_click.as_deref().expect("onClick")))
            ),
            "button",
        ),
        None => ("div", classes, "div"),
    }
}

fn side_nav_navigation_attrs(base: &str, action: &NavigationAction) -> String {
    match action {
        NavigationAction::Internal {
            path,
            fragment,
            operation,
        } => {
            let href = internal_href(path, fragment.as_deref());
            format!(
                r#"{} data-dowe-{base}-href="{}""#,
                navigation_attrs(&href, *operation),
                escape_attr(path)
            )
        }
        NavigationAction::Section {
            fragment,
            operation,
        } => navigation_attrs(&format!("#{fragment}"), *operation),
        NavigationAction::External {
            url,
            web_target,
            native_external_mode,
        } => external_attrs(url, *web_target, *native_external_mode),
        NavigationAction::Back => r#" data-dowe-history="back""#.to_string(),
    }
}

fn render_side_nav_icon_html(
    base: &str,
    icon: Option<&SideNavIcon>,
    context: &ReactiveRenderContext,
) -> String {
    icon.map(|icon| {
        format!(
            r#"<span class="{base}-icon">{}</span>"#,
            render_svg_html(&icon.props, &icon.paths, context)
        )
    })
    .unwrap_or_default()
}

fn render_side_nav_content_html(base: &str, props: &SideNavItemProps) -> String {
    let description = props
        .description
        .as_deref()
        .map(|value| {
            format!(
                r#"<span class="{base}-description">{}</span>"#,
                escape_html(value)
            )
        })
        .unwrap_or_default();
    let status = props
        .status
        .as_deref()
        .map(|value| {
            format!(
                r#"<span class="{base}-status">{}</span>"#,
                escape_html(value)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<span class="{base}-copy"><span class="{base}-label">{}</span>{description}</span>{status}"#,
        escape_html(&props.label)
    )
}

fn svg_path_fill(fill: SvgPathFill) -> String {
    match fill {
        SvgPathFill::None => "none".to_string(),
        SvgPathFill::CurrentColor => "currentColor".to_string(),
        SvgPathFill::Color(token) => format!("var(--dowe-{})", token.as_str()),
    }
}

fn render_text_html(
    _base: &str,
    classes: Vec<String>,
    element: Option<&ElementProps>,
    value: &str,
    i18n: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let dynamic = dynamic_text_attr(value, context);
    let mut extra = dynamic.clone();
    if let Some(key) = i18n {
        extra.push_str(&format!(r#" data-dowe-i18n="{}""#, escape_attr(key)));
    }
    let content = if dynamic.is_empty() {
        escape_html(value)
    } else {
        String::new()
    };
    format!(
        "<p{}>{}</p>",
        attrs(
            classes,
            element,
            (!extra.is_empty()).then_some(extra.as_str()),
            context,
        ),
        content
    )
}

fn bind_attr(value: Option<&str>, context: &ReactiveRenderContext) -> String {
    value
        .map(|value| {
            format!(
                r#" data-dowe-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default()
}

fn dynamic_text_attr(value: &str, context: &ReactiveRenderContext) -> String {
    if dynamic_path(value) {
        format!(
            r#" data-dowe-text="{}""#,
            escape_attr(&context.signal_path(value))
        )
    } else {
        String::new()
    }
}

fn alert_attrs(props: &AlertProps, context: &ReactiveRenderContext) -> String {
    let mut attrs = format!(
        r#" data-dowe-alert data-dowe-alert-kind="{}""#,
        props.kind.as_str()
    );
    if let Some(visible) = props.visible.as_deref() {
        attrs.push_str(&format!(
            r#" data-dowe-alert-visible="{}""#,
            escape_attr(&context.signal_path(visible))
        ));
    }
    attrs
}

fn dynamic_path(value: &str) -> bool {
    let value = value.trim();
    value.contains('.')
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
}

fn page_definition_json(tree: &ViewNode) -> String {
    match tree {
        ViewNode::Scope {
            signals, actions, ..
        } => {
            let context =
                ReactiveRenderContext::default().with_scope(signals.as_slice(), actions.as_slice());
            format!(
                r#"{{"signals":[{}],"actions":[{}]}}"#,
                signals
                    .iter()
                    .map(signal_json)
                    .collect::<Vec<_>>()
                    .join(","),
                actions
                    .iter()
                    .map(|action| action_json(action, &context))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        _ => r#"{"signals":[],"actions":[]}"#.to_string(),
    }
}

fn signal_json(signal: &ViewSignal) -> String {
    format!(
        r#"{{"id":"{}","name":"{}","initial":{}}}"#,
        escape_json(&signal.id),
        escape_json(&signal.name),
        signal_value_json(&signal.initial)
    )
}

fn signal_value_json(value: &ViewSignalValue) -> String {
    match value {
        ViewSignalValue::Null => "null".to_string(),
        ViewSignalValue::Bool(value) => value.to_string(),
        ViewSignalValue::Number(value) => value.clone(),
        ViewSignalValue::String(value) => format!(r#""{}""#, escape_json(value)),
        ViewSignalValue::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(signal_value_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        ViewSignalValue::Object(entries) => format!(
            "{{{}}}",
            entries
                .iter()
                .map(|(key, value)| format!(
                    r#""{}":{}"#,
                    escape_json(key),
                    signal_value_json(value)
                ))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn action_json(action: &ViewAction, context: &ReactiveRenderContext) -> String {
    match &action.kind {
        ViewActionKind::Request(request) => request_action_json(action, request, context),
        ViewActionKind::Assign(assign) => assign_action_json(action, assign, context),
        ViewActionKind::Reset(reset) => reset_action_json(action, reset, context),
    }
}

fn request_action_json(
    view_action: &ViewAction,
    action: &ViewRequestAction,
    context: &ReactiveRenderContext,
) -> String {
    format!(
        r#"{{"id":"{}","name":"{}","kind":"request","method":"{}","path":"{}","baseEnv":{},"body":{},"update":{},"reset":{},"successAlert":{},"successMessage":{},"errorAlert":{},"errorMessage":{},"autoload":{}}}"#,
        escape_json(&view_action.id),
        escape_json(&view_action.name),
        action.method.as_str(),
        escape_json(&action.path),
        json_optional_string(action.base_env.as_deref()),
        json_optional_path(action.body.as_deref(), context),
        json_optional_path(action.update.as_deref(), context),
        json_optional_path(action.reset.as_deref(), context),
        json_optional_path(action.success_alert.as_deref(), context),
        json_optional_string(action.success_message.as_deref()),
        json_optional_path(action.error_alert.as_deref(), context),
        json_optional_string(action.error_message.as_deref()),
        action.autoload
    )
}

fn assign_action_json(
    view_action: &ViewAction,
    action: &ViewAssignAction,
    context: &ReactiveRenderContext,
) -> String {
    format!(
        r#"{{"id":"{}","name":"{}","kind":"assign","target":"{}","source":"{}","call":{}}}"#,
        escape_json(&view_action.id),
        escape_json(&view_action.name),
        escape_json(&context.signal_path(&action.target)),
        escape_json(&context.signal_path(&action.source)),
        action
            .call
            .as_ref()
            .map(|call| stdlib_call_json(call, context))
            .unwrap_or_else(|| "null".to_string())
    )
}

fn stdlib_call_json(
    call: &dowe_components::StdlibCall,
    context: &ReactiveRenderContext,
) -> String {
    format!(
        r#"{{"namespace":"{}","function":"{}","args":[{}]}}"#,
        escape_json(&call.namespace),
        escape_json(&call.function),
        call.args
            .iter()
            .map(|arg| format!(
                r#"{{"name":"{}","value":{}}}"#,
                escape_json(&arg.name),
                stdlib_value_json(&arg.value, context)
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn stdlib_value_json(
    value: &dowe_components::StdlibValue,
    context: &ReactiveRenderContext,
) -> String {
    match value {
        dowe_components::StdlibValue::Null => r#"{"kind":"null","value":null}"#.to_string(),
        dowe_components::StdlibValue::Bool(value) => {
            format!(r#"{{"kind":"bool","value":{value}}}"#)
        }
        dowe_components::StdlibValue::Number(value) => {
            format!(r#"{{"kind":"number","value":"{}"}}"#, escape_json(value))
        }
        dowe_components::StdlibValue::String(value) => {
            format!(r#"{{"kind":"string","value":"{}"}}"#, escape_json(value))
        }
        dowe_components::StdlibValue::Reference(value) => format!(
            r#"{{"kind":"reference","value":"{}"}}"#,
            escape_json(&context.signal_path(value))
        ),
        dowe_components::StdlibValue::Array(values) => format!(
            r#"{{"kind":"array","value":[{}]}}"#,
            values
                .iter()
                .map(|value| stdlib_value_json(value, context))
                .collect::<Vec<_>>()
                .join(",")
        ),
        dowe_components::StdlibValue::Object(entries) => format!(
            r#"{{"kind":"object","value":[{}]}}"#,
            entries
                .iter()
                .map(|(key, value)| format!(
                    r#"["{}",{}]"#,
                    escape_json(key),
                    stdlib_value_json(value, context)
                ))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn reset_action_json(
    view_action: &ViewAction,
    action: &ViewResetAction,
    context: &ReactiveRenderContext,
) -> String {
    format!(
        r#"{{"id":"{}","name":"{}","kind":"reset","target":"{}"}}"#,
        escape_json(&view_action.id),
        escape_json(&view_action.name),
        escape_json(&context.signal_path(&action.target))
    )
}

fn json_optional_path(value: Option<&str>, context: &ReactiveRenderContext) -> String {
    value
        .map(|value| format!(r#""{}""#, escape_json(&context.signal_path(value))))
        .unwrap_or_else(|| "null".to_string())
}

fn box_classes(props: &StyleProps) -> Vec<String> {
    let mut classes = vec!["box".to_string()];
    append_style_classes(&mut classes, props);
    append_container_visual_classes(&mut classes, props);
    classes
}

fn section_classes(props: &StyleProps) -> Vec<String> {
    let mut classes = vec!["section".to_string()];
    append_style_classes(&mut classes, props);
    append_container_visual_classes(&mut classes, props);
    classes
}

fn layout_classes(base: &str, props: &LayoutProps) -> Vec<String> {
    let mut classes = vec![base.to_string()];
    append_style_classes(&mut classes, &props.style);
    append_responsive_classes(&mut classes, "justify", props.justify.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(&mut classes, "align", props.align.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(&mut classes, "gap", props.gap.as_ref(), |value| {
        value.class_suffix()
    });
    classes
}
