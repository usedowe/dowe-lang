fn render_bar_html(
    tag: &str,
    base: &str,
    props: &BarProps,
    top: &[ViewNode],
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
    bottom: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<{tag}{}>",
        attrs(
            bar_classes(base, props),
            Some(&props.style.element),
            None,
            context,
        )
    );
    if base == "footer" {
        html.push_str(&format!("<div{}>", class_attr(footer_inner_classes(props))));
    }
    html.push_str(&render_bar_region_html(
        base,
        "top",
        top,
        children_html,
        context,
    ));
    html.push_str(&format!(
        "<div{}>",
        class_attr(bar_content_classes(base, props))
    ));
    html.push_str(&render_bar_region_html(
        base,
        "start",
        start,
        children_html,
        context,
    ));
    html.push_str(&render_bar_region_html(
        base,
        "center",
        center,
        children_html,
        context,
    ));
    html.push_str(&render_bar_region_html(
        base,
        "end",
        end,
        children_html,
        context,
    ));
    html.push_str("</div>");
    html.push_str(&render_bar_region_html(
        base,
        "bottom",
        bottom,
        children_html,
        context,
    ));
    if base == "footer" {
        html.push_str("</div>");
    }
    html.push_str(&format!("</{tag}>"));
    html
}

fn render_bottom_bar_html(
    props: &BarProps,
    tabs: &[BottomBarTab],
    context: &ReactiveRenderContext,
) -> String {
    let boxed = if props.boxed { " is-boxed" } else { "" };
    let mut html = format!(
        "<nav{}><div class=\"bottombar-tabs{boxed}\">",
        attrs(
            bar_classes("bottombar", props),
            Some(&props.style.element),
            None,
            context
        )
    );
    for tab in tabs {
        let featured = if tab.featured { " is-featured" } else { "" };
        html.push_str(&format!(
            "<a class=\"bottombar-tab{featured}\" aria-label=\"{}\"{}><span class=\"bottombar-tab-icon\">{}</span>{}</a>",
            escape_attr(&tab.label),
            side_nav_navigation_attrs("bottombar", &tab.navigation),
            render_svg_html(&tab.icon.props, &tab.icon.paths, context),
            localized_span("bottombar-tab-label", &tab.label, tab.i18n.as_deref()),
        ));
    }
    html.push_str("</div></nav>");
    html
}

fn render_bar_region_html(
    base: &str,
    name: &str,
    children: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    if children.is_empty() {
        return String::new();
    }
    let mut html = format!(r#"<div class="{base}-{name}">"#);
    for child in children {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str("</div>");
    html
}

fn collect_bar_js_segments(
    tag: &str,
    base: &str,
    props: &BarProps,
    top: &[ViewNode],
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
    bottom: &[ViewNode],
    segments: &mut Vec<JsSegment>,
    context: &ReactiveRenderContext,
) {
    push_literal(
        segments,
        &format!(
            "<{tag}{}>",
            attrs(
                bar_classes(base, props),
                Some(&props.style.element),
                None,
                context,
            )
        ),
    );
    if base == "footer" {
        push_literal(
            segments,
            &format!("<div{}>", class_attr(footer_inner_classes(props))),
        );
    }
    collect_bar_region_js_segments(base, "top", top, segments, context);
    push_literal(
        segments,
        &format!("<div{}>", class_attr(bar_content_classes(base, props))),
    );
    collect_bar_region_js_segments(base, "start", start, segments, context);
    collect_bar_region_js_segments(base, "center", center, segments, context);
    collect_bar_region_js_segments(base, "end", end, segments, context);
    push_literal(segments, "</div>");
    collect_bar_region_js_segments(base, "bottom", bottom, segments, context);
    if base == "footer" {
        push_literal(segments, "</div>");
    }
    push_literal(segments, &format!("</{tag}>"));
}

fn collect_bar_region_js_segments(
    base: &str,
    name: &str,
    children: &[ViewNode],
    segments: &mut Vec<JsSegment>,
    context: &ReactiveRenderContext,
) {
    if children.is_empty() {
        return;
    }
    push_literal(segments, &format!(r#"<div class="{base}-{name}">"#));
    for child in children {
        collect_js_segments(child, segments, context);
    }
    push_literal(segments, "</div>");
}

fn collect_tabs_js_segments(
    props: &TabsProps,
    tabs: &[TabItem],
    segments: &mut Vec<JsSegment>,
    context: &ReactiveRenderContext,
) {
    push_literal(
        segments,
        &format!(
            "<div{} data-dowe-tabs>",
            attrs(
                tabs_classes(props),
                Some(&props.style.element),
                None,
                context
            )
        ),
    );
    push_literal(
        segments,
        &format!(
            r#"<div{} role="tablist">"#,
            class_attr(tabs_list_classes(props))
        ),
    );
    for (index, tab) in tabs.iter().enumerate() {
        let active = index == 0;
        let mut classes = vec!["tab".to_string()];
        if active {
            classes.push("on-active".to_string());
        }
        let label = localized_span("tabs-label", &tab.label, tab.i18n.as_deref());
        let content = if props.variant == TabsVariant::Stepper {
            format!(
                r#"<span class="step-indicator" aria-hidden="true">{}</span>{label}"#,
                index + 1
            )
        } else {
            label
        };
        push_literal(
            segments,
            &format!(
                r#"<button{} type="button" role="tab" id="{}" aria-selected="{}" aria-controls="{}"{} data-dowe-tab="{}">{}</button>"#,
                class_attr(classes),
                escape_attr(&tab_button_id(tab)),
                if active { "true" } else { "false" },
                escape_attr(&tab_panel_id(tab)),
                if props.variant == TabsVariant::Stepper && active {
                    r#" aria-current="step""#
                } else {
                    ""
                },
                escape_attr(&tab.id),
                content
            ),
        );
    }
    push_literal(segments, "</div><div class=\"tabs-wrapper\">");
    for (index, tab) in tabs.iter().enumerate() {
        let active = index == 0;
        let mut classes = vec!["tabs-content".to_string()];
        if active {
            classes.push("on-active".to_string());
        }
        push_literal(
            segments,
            &format!(
                r#"<div{} id="{}" role="tabpanel" aria-labelledby="{}" data-dowe-tab-panel="{}"{}>"#,
                class_attr(classes),
                escape_attr(&tab_panel_id(tab)),
                escape_attr(&tab_button_id(tab)),
                escape_attr(&tab.id),
                if active { "" } else { " hidden" }
            ),
        );
        for child in &tab.children {
            collect_js_segments(child, segments, context);
        }
        push_literal(segments, "</div>");
    }
    push_literal(segments, "</div></div>");
}
