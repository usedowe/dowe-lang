fn render_nav_menu_html(
    props: &NavMenuProps,
    items: &[NavMenuItem],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<nav{}>",
        attrs(
            nav_menu_classes(props),
            Some(&props.style.element),
            Some(r#" data-dowe-navmenu aria-label="Navigation menu""#),
            context,
        )
    );
    for (index, item) in items.iter().enumerate() {
        html.push_str(&render_nav_menu_item_html(index, item, context));
    }
    for (index, item) in items.iter().enumerate() {
        html.push_str(&render_nav_menu_popover_html(
            index,
            item,
            children_html,
            context,
        ));
    }
    html.push_str("</nav>");
    html
}

fn collect_nav_menu_js_segments(
    props: &NavMenuProps,
    items: &[NavMenuItem],
    segments: &mut Vec<JsSegment>,
    context: &ReactiveRenderContext,
) {
    push_literal(
        segments,
        &format!(
            "<nav{}>",
            attrs(
                nav_menu_classes(props),
                Some(&props.style.element),
                Some(r#" data-dowe-navmenu aria-label="Navigation menu""#),
                context,
            )
        ),
    );
    for (index, item) in items.iter().enumerate() {
        push_literal(segments, &render_nav_menu_item_html(index, item, context));
    }
    for (index, item) in items.iter().enumerate() {
        collect_nav_menu_popover_js_segments(index, item, segments, context);
    }
    push_literal(segments, "</nav>");
}

fn render_nav_menu_item_html(
    index: usize,
    item: &NavMenuItem,
    context: &ReactiveRenderContext,
) -> String {
    match item {
        NavMenuItem::Item(props) => render_nav_menu_entry_html(props, context),
        NavMenuItem::Submenu { props, .. } | NavMenuItem::Megamenu { props, .. } => {
            format!(
                r#"<button class="navmenu-item" type="button" data-dowe-navmenu-trigger="{index}" aria-haspopup="menu" aria-expanded="false">{}{}<span class="navmenu-arrow" aria-hidden="true">⌄</span></button>"#,
                render_nav_menu_icon_html(props.icon.as_ref(), context),
                render_nav_menu_label_html(&props.label)
            )
        }
    }
}

fn render_nav_menu_entry_html(props: &NavMenuItemProps, context: &ReactiveRenderContext) -> String {
    let (tag, attrs, close) = nav_menu_entry_tags(props, "navmenu-item", context);
    format!(
        "<{tag}{attrs}>{}{}</{close}>",
        render_nav_menu_icon_html(props.icon.as_ref(), context),
        render_nav_menu_label_html(&props.label)
    )
}

fn render_nav_menu_popover_html(
    index: usize,
    item: &NavMenuItem,
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    match item {
        NavMenuItem::Submenu { items, .. } => {
            let mut html = format!(
                r#"<div class="navmenu-popover" data-dowe-navmenu-popover="{index}" role="menu" hidden><div class="navmenu-popover-content">"#
            );
            for item in items {
                html.push_str(&render_nav_menu_subitem_html(item, context));
            }
            html.push_str("</div></div>");
            html
        }
        NavMenuItem::Megamenu { content, .. } => {
            let mut html = format!(
                r#"<div class="navmenu-popover is-megamenu" data-dowe-navmenu-popover="{index}" role="menu" hidden><div class="navmenu-popover-content">"#
            );
            for child in content {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str("</div></div>");
            html
        }
        NavMenuItem::Item(_) => String::new(),
    }
}

fn collect_nav_menu_popover_js_segments(
    index: usize,
    item: &NavMenuItem,
    segments: &mut Vec<JsSegment>,
    context: &ReactiveRenderContext,
) {
    match item {
        NavMenuItem::Submenu { .. } => {
            push_literal(
                segments,
                &render_nav_menu_popover_html(index, item, None, context),
            );
        }
        NavMenuItem::Megamenu { content, .. } => {
            push_literal(
                segments,
                &format!(
                    r#"<div class="navmenu-popover is-megamenu" data-dowe-navmenu-popover="{index}" role="menu" hidden><div class="navmenu-popover-content">"#
                ),
            );
            for child in content {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</div></div>");
        }
        NavMenuItem::Item(_) => {}
    }
}

fn render_nav_menu_subitem_html(
    props: &NavMenuItemProps,
    context: &ReactiveRenderContext,
) -> String {
    let (tag, attrs, close) = nav_menu_entry_tags(props, "navmenu-submenu-item", context);
    let description = props
        .description
        .as_deref()
        .map(|value| {
            format!(
                r#"<span class="navmenu-submenu-description">{}</span>"#,
                escape_html(value)
            )
        })
        .unwrap_or_default();
    format!(
        "<{tag}{attrs}>{}<span class=\"navmenu-submenu-content\"><span class=\"navmenu-submenu-label\">{}</span>{description}</span></{close}>",
        render_nav_menu_subitem_icon_html(props.icon.as_ref(), context),
        escape_html(&props.label)
    )
}

fn nav_menu_entry_tags(
    props: &NavMenuItemProps,
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
            format!("{classes}{}", nav_menu_navigation_attrs(action)),
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

fn nav_menu_navigation_attrs(action: &NavigationAction) -> String {
    match action {
        NavigationAction::Internal {
            path,
            fragment,
            operation,
        } => {
            let href = internal_href(path, fragment.as_deref());
            format!(
                r#"{} data-dowe-navmenu-href="{}""#,
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

fn render_nav_menu_icon_html(
    icon: Option<&SideNavIcon>,
    context: &ReactiveRenderContext,
) -> String {
    icon.map(|icon| {
        format!(
            r#"<span class="navmenu-icon">{}</span>"#,
            render_svg_html(&icon.props, &icon.paths, context)
        )
    })
    .unwrap_or_default()
}

fn render_nav_menu_subitem_icon_html(
    icon: Option<&SideNavIcon>,
    context: &ReactiveRenderContext,
) -> String {
    icon.map(|icon| {
        format!(
            r#"<span class="navmenu-submenu-icon">{}</span>"#,
            render_svg_html(&icon.props, &icon.paths, context)
        )
    })
    .unwrap_or_default()
}

fn render_nav_menu_label_html(label: &str) -> String {
    format!(
        r#"<span class="navmenu-label" data-text="{}">{}</span>"#,
        escape_attr(label),
        escape_html(label)
    )
}

fn render_scaffold_html(
    props: &ScaffoldProps,
    app_bar: &[ViewNode],
    start: &[ViewNode],
    main: &[ViewNode],
    end: &[ViewNode],
    bottom_bar: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<div{}>",
        attrs(
            scaffold_classes(props),
            Some(&props.style.element),
            None,
            context,
        )
    );
    html.push_str(&render_scaffold_region_html(
        app_bar,
        children_html,
        context,
    ));
    html.push_str("<div class=\"scaffold-body\">");
    if !start.is_empty() {
        html.push_str("<aside class=\"scaffold-start\"><div class=\"scaffold-content\">");
        html.push_str(&render_scaffold_region_html(start, children_html, context));
        html.push_str("</div></aside>");
    }
    html.push_str("<main class=\"scaffold-main\">");
    html.push_str(&render_scaffold_region_html(main, children_html, context));
    html.push_str("</main>");
    if !end.is_empty() {
        html.push_str("<aside class=\"scaffold-end\"><div class=\"scaffold-content\">");
        html.push_str(&render_scaffold_region_html(end, children_html, context));
        html.push_str("</div></aside>");
    }
    html.push_str("</div>");
    html.push_str(&render_scaffold_region_html(
        bottom_bar,
        children_html,
        context,
    ));
    html.push_str("</div>");
    html
}

fn render_scaffold_region_html(
    children: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    children
        .iter()
        .map(|child| render_html_with_context(child, children_html, context))
        .collect::<String>()
}

fn collect_scaffold_js_segments(
    props: &ScaffoldProps,
    app_bar: &[ViewNode],
    start: &[ViewNode],
    main: &[ViewNode],
    end: &[ViewNode],
    bottom_bar: &[ViewNode],
    segments: &mut Vec<JsSegment>,
    context: &ReactiveRenderContext,
) {
    push_literal(
        segments,
        &format!(
            "<div{}>",
            attrs(
                scaffold_classes(props),
                Some(&props.style.element),
                None,
                context,
            )
        ),
    );
    for child in app_bar {
        collect_js_segments(child, segments, context);
    }
    push_literal(segments, "<div class=\"scaffold-body\">");
    if !start.is_empty() {
        push_literal(
            segments,
            "<aside class=\"scaffold-start\"><div class=\"scaffold-content\">",
        );
        for child in start {
            collect_js_segments(child, segments, context);
        }
        push_literal(segments, "</div></aside>");
    }
    push_literal(segments, "<main class=\"scaffold-main\">");
    for child in main {
        collect_js_segments(child, segments, context);
    }
    push_literal(segments, "</main>");
    if !end.is_empty() {
        push_literal(
            segments,
            "<aside class=\"scaffold-end\"><div class=\"scaffold-content\">",
        );
        for child in end {
            collect_js_segments(child, segments, context);
        }
        push_literal(segments, "</div></aside>");
    }
    push_literal(segments, "</div>");
    for child in bottom_bar {
        collect_js_segments(child, segments, context);
    }
    push_literal(segments, "</div>");
}
