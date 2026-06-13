fn render_side_nav_html(
    base: &str,
    props: &SideNavProps,
    items: &[SideNavItem],
    context: &ReactiveRenderContext,
) -> String {
    let label = if base == "sidebar" {
        "Sidebar navigation"
    } else {
        "Side navigation"
    };
    let mut html = format!(
        "<nav{}>",
        attrs(
            side_nav_classes(base, props),
            Some(&props.style.element),
            Some(&format!(r#" aria-label="{label}""#)),
            context,
        )
    );
    for item in items {
        html.push_str(&render_side_nav_item_html(base, item, context));
    }
    html.push_str("</nav>");
    html
}

fn render_drawer_html(
    props: &DrawerProps,
    children: &[ViewNode],
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
    for child in children {
        html.push_str(&render_html_with_context(child, children_html, context));
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
    r#"<button class="drawer-close" type="button" aria-label="Close drawer" data-dowe-drawer-close>&times;</button>"#
}
