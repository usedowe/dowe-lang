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
                r#"<details class="{classes}" data-dowe-{base}-submenu{}><summary class="{base}-entry {base}-trigger" aria-expanded="{}">{}{}{}</summary><div class="{base}-submenu-content"><div class="{base}-submenu-content-inner">"#,
                if *open { " open" } else { "" },
                if *open { "true" } else { "false" },
                render_side_nav_icon_html(base, props.icon.as_ref(), context),
                render_side_nav_content_html(base, props),
                render_side_nav_arrow_html(base, context)
            );
            for item in items {
                html.push_str(&render_side_nav_entry_html(
                    base,
                    item,
                    &format!("{base}-entry {base}-subitem"),
                    context,
                ));
            }
            html.push_str("</div></div></details>");
            html
        }
    }
}

fn render_rail_nav_html(
    props: &RailNavProps,
    items: &[RailNavItem],
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<nav{}>",
        attrs(
            rail_nav_classes(props),
            Some(&props.style.element),
            Some(r#" aria-label="Rail navigation""#),
            context,
        )
    );
    for item in items {
        match item {
            RailNavItem::Item(item) => {
                html.push_str(&render_rail_nav_item_html(item, props.show_labels, context))
            }
            RailNavItem::Divider => html.push_str(r#"<div class="railnav-divider"></div>"#),
        }
    }
    html.push_str("</nav>");
    html
}

fn render_rail_nav_item_html(
    props: &RailNavItemProps,
    show_label: bool,
    context: &ReactiveRenderContext,
) -> String {
    let classes = class_attr(vec!["railnav-item".to_string()]);
    let aria = format!(r#" aria-label="{}""#, escape_attr(&props.label));
    let (tag, attributes) = match props.navigation.as_ref() {
        Some(action) => (
            "a",
            format!(
                "{classes}{aria}{}",
                side_nav_navigation_attrs("railnav", action)
            ),
        ),
        None => (
            "button",
            format!(
                r#"{classes}{aria} type="button"{}"#,
                props
                    .on_click
                    .as_deref()
                    .map(|action| format!(
                        r#" data-dowe-click="{}""#,
                        escape_attr(&context.action_id(action))
                    ))
                    .unwrap_or_default()
            ),
        ),
    };
    let icon = format!(
        r#"<span class="railnav-icon">{}</span>"#,
        render_svg_html(&props.icon.props, &props.icon.paths, context)
    );
    let label = localized_span("railnav-label", &props.label, props.i18n.as_deref());
    let item = format!("<{tag}{attributes}>{icon}{}</{tag}>", if show_label { label.as_str() } else { "" });
    if show_label {
        item
    } else {
        format!(
            r#"<span class="tooltip railnav-tooltip" data-dowe-tooltip>{item}<span class="tooltip-popover is-solid is-muted position-end" role="tooltip"><span class="tooltip-arrow"></span>{label}</span></span>"#
        )
    }
}

fn render_side_nav_arrow_html(base: &str, context: &ReactiveRenderContext) -> String {
    let arrow = side_nav_submenu_arrow_icon();
    format!(
        r#"<span class="{base}-chevron" aria-hidden="true">{}</span>"#,
        render_svg_html(&arrow.props, &arrow.paths, context)
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
        .map(|value| localized_span(&format!("{base}-description"), value, props.description_i18n.as_deref()))
        .unwrap_or_default();
    let status = props
        .status
        .as_deref()
        .map(|value| localized_span(&format!("{base}-status"), value, props.status_i18n.as_deref()))
        .unwrap_or_default();
    format!(
        r#"<span class="{base}-copy">{}{description}</span>{status}"#,
        localized_span(&format!("{base}-label"), &props.label, props.i18n.as_deref())
    )
}

fn localized_span(class: &str, value: &str, i18n: Option<&str>) -> String {
    let i18n = i18n
        .map(|key| format!(r#" data-dowe-i18n="{}""#, escape_attr(key)))
        .unwrap_or_default();
    format!(
        r#"<span class="{}"{i18n}>{}</span>"#,
        escape_attr(class),
        escape_html(value)
    )
}

fn svg_path_fill(fill: SvgPathFill) -> String {
    match fill {
        SvgPathFill::None => "none".to_string(),
        SvgPathFill::CurrentColor => "currentColor".to_string(),
        SvgPathFill::Color(token) => format!("var(--dowe-{})", token.as_str()),
        SvgPathFill::RawFill { color, .. } | SvgPathFill::RawStroke { color, .. } => color.to_string(),
        SvgPathFill::LiteralFill { red, green, blue, .. }
        | SvgPathFill::LiteralStroke { red, green, blue, .. } => {
            format!("#{red:02x}{green:02x}{blue:02x}")
        }
        SvgPathFill::Fill { color, .. } | SvgPathFill::Stroke { color, .. } => color
            .map(|token| format!("var(--dowe-{})", token.as_str()))
            .unwrap_or_else(|| "currentColor".to_string()),
    }
}

fn svg_path_attributes(paint: SvgPathFill) -> String {
    match paint {
        SvgPathFill::RawFill { color, opacity, even_odd } => format!(
            " fill=\"{}\"{}{}",
            escape_attr(color),
            if opacity == 255 { String::new() } else { format!(" opacity=\"{:.3}\"", opacity as f32 / 255.0) },
            if even_odd { " fill-rule=\"evenodd\" clip-rule=\"evenodd\"" } else { "" }
        ),
        SvgPathFill::RawStroke { color, opacity, width, line_cap, line_join } => format!(
            " fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\" stroke-linecap=\"{}\" stroke-linejoin=\"{}\"{}",
            escape_attr(color),
            width as f32 / 100.0,
            match line_cap { SvgLineCap::Butt => "butt", SvgLineCap::Round => "round", SvgLineCap::Square => "square" },
            match line_join { SvgLineJoin::Miter => "miter", SvgLineJoin::Round => "round", SvgLineJoin::Bevel => "bevel" },
            if opacity == 255 { String::new() } else { format!(" opacity=\"{:.3}\"", opacity as f32 / 255.0) }
        ),
        SvgPathFill::LiteralFill { red, green, blue, opacity, even_odd } => format!(
            " fill=\"#{red:02x}{green:02x}{blue:02x}\"{}{}",
            if opacity == 255 { String::new() } else { format!(" opacity=\"{:.3}\"", opacity as f32 / 255.0) },
            if even_odd { " fill-rule=\"evenodd\" clip-rule=\"evenodd\"" } else { "" }
        ),
        SvgPathFill::LiteralStroke { red, green, blue, opacity, width, line_cap, line_join } => format!(
            " fill=\"none\" stroke=\"#{red:02x}{green:02x}{blue:02x}\" stroke-width=\"{:.2}\" stroke-linecap=\"{}\" stroke-linejoin=\"{}\"{}",
            width as f32 / 100.0,
            match line_cap { SvgLineCap::Butt => "butt", SvgLineCap::Round => "round", SvgLineCap::Square => "square" },
            match line_join { SvgLineJoin::Miter => "miter", SvgLineJoin::Round => "round", SvgLineJoin::Bevel => "bevel" },
            if opacity == 255 { String::new() } else { format!(" opacity=\"{:.3}\"", opacity as f32 / 255.0) }
        ),
        SvgPathFill::Fill { color, opacity, even_odd } => format!(
            " fill=\"{}\"{}{}",
            escape_attr(&color.map(|token| format!("var(--dowe-{})", token.as_str())).unwrap_or_else(|| "currentColor".to_string())),
            if opacity == 255 { String::new() } else { format!(" opacity=\"{:.3}\"", opacity as f32 / 255.0) },
            if even_odd { " fill-rule=\"evenodd\" clip-rule=\"evenodd\"" } else { "" }
        ),
        SvgPathFill::Stroke { color, opacity, width, line_cap, line_join } => format!(
            " fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\" stroke-linecap=\"{}\" stroke-linejoin=\"{}\"{}",
            escape_attr(&color.map(|token| format!("var(--dowe-{})", token.as_str())).unwrap_or_else(|| "currentColor".to_string())),
            width as f32 / 100.0,
            match line_cap { SvgLineCap::Butt => "butt", SvgLineCap::Round => "round", SvgLineCap::Square => "square" },
            match line_join { SvgLineJoin::Miter => "miter", SvgLineJoin::Round => "round", SvgLineJoin::Bevel => "bevel" },
            if opacity == 255 { String::new() } else { format!(" opacity=\"{:.3}\"", opacity as f32 / 255.0) }
        ),
        _ => format!(" fill=\"{}\"", escape_attr(&svg_path_fill(paint))),
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
    let dynamic = visible_dynamic_text_attr(value, context);
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
    if is_dynamic_path(value) {
        format!(
            r#" data-dowe-text="{}""#,
            escape_attr(&context.signal_path(value))
        )
    } else {
        String::new()
    }
}

fn visible_dynamic_text_attr(value: &str, context: &ReactiveRenderContext) -> String {
    text_binding_path(value)
        .map(|path| {
            format!(
                r#" data-dowe-text="{}""#,
                escape_attr(&context.signal_path(path))
            )
        })
        .unwrap_or_default()
}

fn is_dynamic_path(value: &str) -> bool {
    let value = value.trim();
    value.contains('.')
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '.')
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

fn page_definition_json(tree: &ViewNode) -> String {
    match tree {
        ViewNode::Scope {
            constants, signals, actions, ..
        } => {
            let context = ReactiveRenderContext::default().with_scope(
                constants.as_slice(),
                signals.as_slice(),
                actions.as_slice(),
            );
            format!(
                r#"{{"constants":[{}],"signals":[{}],"actions":[{}]}}"#,
                constants
                    .iter()
                    .map(constant_json)
                    .collect::<Vec<_>>()
                    .join(","),
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

fn constant_json(constant: &ViewConstant) -> String {
    format!(
        r#"{{"id":"{}","name":"{}","value":{}}}"#,
        escape_json(&constant.id),
        escape_json(&constant.name),
        signal_value_json(&constant.value)
    )
}

fn signal_json(signal: &ViewSignal) -> String {
    format!(
        r#"{{"id":"{}","name":"{}","storageKey":"{}","scope":"{}","storage":"{}","initial":{}}}"#,
        escape_json(&signal.id),
        escape_json(&signal.name),
        escape_json(&signal.storage_key),
        signal.scope.as_str(),
        signal.storage.as_str(),
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
        ViewActionKind::Sequence(statements) => format!(
            r#"{{"id":"{}","name":"{}","params":{},"returnType":{},"kind":"sequence","steps":[{}],"autoload":{},"init":{}}}"#,
            escape_json(&action.id),
            escape_json(&action.name),
            function_params_json(action),
            function_return_json(action),
            statements.iter().map(|statement| statement_json(statement, context)).collect::<Vec<_>>().join(","),
            action.is_init() || statements.iter().any(|statement| matches!(statement, dowe_components::ViewFunctionStatement::Request { action, .. } if action.autoload)),
            action.is_init()
        ),
        ViewActionKind::Request(request) => request_action_json(action, request, context),
        ViewActionKind::Assign(assign) => assign_action_json(action, assign, context),
        ViewActionKind::Reset(reset) => reset_action_json(action, reset, context),
    }
}

fn statement_json(statement: &dowe_components::ViewFunctionStatement, context: &ReactiveRenderContext) -> String {
    match statement {
        dowe_components::ViewFunctionStatement::Request { result, action } => format!(
            r#"{{"kind":"request","result":"{}","method":"{}","path":"{}","baseEnv":{},"headers":{},"body":{}}}"#,
            escape_json(result), action.method.as_str(), escape_json(&action.path), json_optional_string(action.base_env.as_deref()), request_headers_json(action, context), json_optional_path(action.body.as_deref(), context)
        ),
        dowe_components::ViewFunctionStatement::If { result, success, error } => format!(
            r#"{{"kind":"if","result":"{}","success":[{}],"error":[{}]}}"#,
            escape_json(result),
            success.iter().map(|step| statement_json(step, context)).collect::<Vec<_>>().join(","),
            error.iter().map(|step| statement_json(step, context)).collect::<Vec<_>>().join(",")
        ),
        dowe_components::ViewFunctionStatement::Assign(assign) => format!(
            r#"{{"kind":"assign","target":"{}","source":"{}","literal":{},"call":{}}}"#,
            escape_json(&context.signal_path(&assign.target)),
            escape_json(&context.signal_path(&assign.source)),
            assign.literal.as_ref().map(signal_value_json).unwrap_or_else(|| "null".to_string()),
            assign.call.as_ref().map(|call| stdlib_call_json(call, context)).unwrap_or_else(|| "null".to_string())
        ),
        dowe_components::ViewFunctionStatement::Reset(reset) => format!(r#"{{"kind":"reset","target":"{}"}}"#, escape_json(&context.signal_path(&reset.target))),
        dowe_components::ViewFunctionStatement::Toast(toast) => format!(
            r#"{{"kind":"toast","type":"{}","title":"{}","message":"{}","duration":{},"scheme":{},"variant":{},"position":{}}}"#,
            escape_json(&toast.kind), escape_json(&toast.title), escape_json(&toast.message), toast.duration.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string()), json_optional_string(toast.scheme.as_deref()), json_optional_string(toast.variant.as_deref()), json_optional_string(toast.position.as_deref())
        ),
        dowe_components::ViewFunctionStatement::Redirect { path } => format!(
            r#"{{"kind":"redirect","path":"{}"}}"#,
            escape_json(path)
        ),
    }
}

fn request_action_json(
    view_action: &ViewAction,
    action: &ViewRequestAction,
    context: &ReactiveRenderContext,
) -> String {
    let headers = request_headers_json(action, context);
    format!(
        r#"{{"id":"{}","name":"{}","params":{},"returnType":{},"kind":"request","method":"{}","path":"{}","baseEnv":{},"headers":{},"body":{},"update":{},"reset":{},"successAlert":{},"successMessage":{},"errorAlert":{},"errorMessage":{},"autoload":{}}}"#,
        escape_json(&view_action.id),
        escape_json(&view_action.name),
        function_params_json(view_action),
        function_return_json(view_action),
        action.method.as_str(),
        escape_json(&action.path),
        json_optional_string(action.base_env.as_deref()),
        headers,
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

fn request_headers_json(action: &ViewRequestAction, context: &ReactiveRenderContext) -> String {
    format!(
        "[{}]",
        action
            .headers
            .iter()
            .map(|header| match &header.value {
                dowe_components::ViewRequestHeaderValue::Static(value) => format!(
                    r#"{{"name":"{}","kind":"static","value":"{}"}}"#,
                    escape_json(&header.name),
                    escape_json(value)
                ),
                dowe_components::ViewRequestHeaderValue::Signal(value) => format!(
                    r#"{{"name":"{}","kind":"signal","value":"{}"}}"#,
                    escape_json(&header.name),
                    escape_json(&context.signal_path(value))
                ),
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn assign_action_json(
    view_action: &ViewAction,
    action: &ViewAssignAction,
    context: &ReactiveRenderContext,
) -> String {
    format!(
        r#"{{"id":"{}","name":"{}","params":{},"returnType":{},"kind":"assign","target":"{}","source":"{}","literal":{},"call":{}}}"#,
        escape_json(&view_action.id),
        escape_json(&view_action.name),
        function_params_json(view_action),
        function_return_json(view_action),
        escape_json(&context.signal_path(&action.target)),
        escape_json(&context.signal_path(&action.source)),
        action.literal.as_ref().map(signal_value_json).unwrap_or_else(|| "null".to_string()),
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
        r#"{{"id":"{}","name":"{}","params":{},"returnType":{},"kind":"reset","target":"{}"}}"#,
        escape_json(&view_action.id),
        escape_json(&view_action.name),
        function_params_json(view_action),
        function_return_json(view_action),
        escape_json(&context.signal_path(&action.target))
    )
}

fn function_params_json(action: &ViewAction) -> String {
    format!(
        "[{}]",
        action
            .params
            .iter()
            .map(|parameter| format!(
                r#"{{"name":"{}","type":"{}"}}"#,
                escape_json(&parameter.name),
                escape_json(&parameter.type_name)
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn function_return_json(action: &ViewAction) -> String {
    json_optional_string(action.return_type.as_ref().map(|value| value.type_name.as_str()))
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

fn brand_classes(props: &StyleProps) -> Vec<String> {
    let mut classes = vec!["brand".to_string()];
    append_style_classes(&mut classes, props);
    classes
}

fn banner_classes(props: &StyleProps) -> Vec<String> {
    let mut classes = vec!["banner".to_string()];
    append_style_classes(&mut classes, props);
    append_container_visual_classes(&mut classes, props);
    classes
}

fn section_classes(props: &StyleProps) -> Vec<String> {
    let mut classes = vec!["section".to_string()];
    append_style_classes(&mut classes, props);
    classes.retain(|class_name| !section_spacing_class(class_name));
    append_container_visual_classes(&mut classes, props);
    classes
}

fn section_body_classes(props: &StyleProps) -> Vec<String> {
    let mut classes = vec!["section-body".to_string()];
    if props.boxed {
        classes.push("is-boxed".to_string());
    }
    let mut content = props.clone();
    content.spacing = dowe_components::section_content_spacing(&props.spacing);
    let mut style_classes = Vec::new();
    append_style_classes(&mut style_classes, &content);
    classes.extend(
        style_classes
            .into_iter()
            .filter(|class_name| section_spacing_class(class_name)),
    );
    classes
}

fn section_spacing_class(class_name: &str) -> bool {
    let class_name = class_name.rsplit(':').next().unwrap_or(class_name);
    ["p-", "px-", "py-", "pl-", "pr-", "pt-", "pb-"]
        .iter()
        .any(|prefix| class_name.starts_with(prefix))
}

fn layout_classes(base: &str, props: &LayoutProps) -> Vec<String> {
    let mut classes = vec![base.to_string()];
    append_style_classes(&mut classes, &props.style);
    append_responsive_classes(
        &mut classes,
        "direction",
        Some(&props.direction),
        |value| value.as_str().to_string(),
    );
    if props.wrap {
        classes.push("flex-wrap".to_string());
    }
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
