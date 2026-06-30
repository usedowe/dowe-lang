fn render_side_nav_html(
    base: &str,
    props: &SideNavProps,
    items: &[SideNavItem],
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<nav{}>",
        attrs(
            side_nav_classes(base, props),
            Some(&props.style.element),
            Some(r#" aria-label="Side navigation""#),
            context,
        )
    );
    for item in items {
        html.push_str(&render_side_nav_item_html(base, item, context));
    }
    html.push_str("</nav>");
    html
}

fn render_sidebar_html(
    props: &SidebarProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<aside{}>",
        attrs(sidebar_classes(props), Some(&props.style.element), None, context)
    );
    if !header.is_empty() {
        html.push_str("<div class=\"sidebar-header\">");
        for child in header {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div>");
    }
    html.push_str("<div class=\"sidebar-body\">");
    for child in body {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str("</div>");
    if !footer.is_empty() {
        html.push_str("<div class=\"sidebar-footer\">");
        for child in footer {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div>");
    }
    html.push_str("</aside>");
    html
}

fn render_drawer_html(
    props: &DrawerProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let extra = drawer_panel_attrs(props, context);
    let mut html = format!(
        "<div{} hidden><button class=\"drawer-overlay\" type=\"button\" aria-label=\"Close drawer\" data-dowe-drawer-overlay></button><div{} role=\"dialog\" aria-modal=\"true\">",
        attrs(
            drawer_panel_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context,
        ),
        class_attr(drawer_classes(props))
    );
    if !props.hide_close_button {
        html.push_str(drawer_close_html());
    }
    if !header.is_empty() {
        html.push_str("<div class=\"drawer-header\">");
        for child in header {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div>");
    }
    html.push_str("<div class=\"drawer-body\">");
    for child in body {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str("</div>");
    if !footer.is_empty() {
        html.push_str("<div class=\"drawer-footer\">");
        for child in footer {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div>");
    }
    html.push_str("</div></div>");
    html
}

fn drawer_panel_attrs(props: &DrawerProps, context: &ReactiveRenderContext) -> String {
    format!(
        r#" data-dowe-drawer data-dowe-drawer-open="{}" data-dowe-drawer-disable-overlay-close="{}""#,
        escape_attr(&context.signal_path(&props.open)),
        props.disable_overlay_close
    )
}

fn drawer_close_html() -> &'static str {
    r#"<button class="drawer-close" type="button" aria-label="Close drawer" data-dowe-drawer-close><svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24" aria-hidden="true" focusable="false"><path d="M0 0h24v24H0z" fill="none"/><path fill="currentColor" d="m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073l.084.073L12 10.939l6.47-6.47a.75.75 0 1 1 1.06 1.061L13.061 12l6.47 6.47a.75.75 0 0 1 .072.976l-.073.084a.75.75 0 0 1-.976.073l-.084-.073L12 13.061l-6.47 6.47a.75.75 0 0 1-1.06-1.061L10.939 12l-6.47-6.47a.75.75 0 0 1-.072-.976l.073-.084z"/></svg></button>"#
}
