fn collect_shell_js_node_segments(node: &ViewNode, segments: &mut Vec<JsSegment>, context: &ReactiveRenderContext) -> bool {
    match node {
        ViewNode::Divider { props } => {
            push_literal(segments, &render_divider_html(props, context));
        }
        ViewNode::Title { props, value } => {
            push_literal(
                segments,
                &render_text_html(
                    "title",
                    text_classes("title", props),
                    Some(&props.style.element),
                    value,
                    props.i18n.as_deref(),
                    props.as_tag.as_deref(),
                    context,
                ),
            );
        }
        ViewNode::Text { props, value } => {
            push_literal(
                segments,
                &render_text_html(
                    "text",
                    text_classes("text", props),
                    Some(&props.style.element),
                    value,
                    props.i18n.as_deref(),
                    None,
                    context,
                ),
            );
        }
        ViewNode::Alert { .. } => {
            push_literal(segments, &render_html_with_context(node, None, context))
        }
        ViewNode::Avatar { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::Marquee { .. }
        | ViewNode::TypeWriter { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Collapsible { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. }
        | ViewNode::Badge { .. }
        | ViewNode::Chip { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::Modal { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Tooltip { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Dropdown { .. }
        | ViewNode::Command { .. }
        | ViewNode::Accordion { .. }
        | ViewNode::Carousel { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. } => {
            push_literal(segments, &render_html_with_context(node, None, context))
        }
        ViewNode::Svg { props, paths } => {
            push_literal(segments, &render_svg_html(props, paths, context));
        }
        ViewNode::AppBar {
            props,
            top,
            start,
            center,
            end,
            bottom,
            mobile_menu,
        } => collect_bar_js_segments(
            "header", "appbar", props, top, start, center, end, bottom, mobile_menu.as_ref(), segments, context,
        ),
        ViewNode::Footer {
            props,
            top,
            start,
            center,
            end,
            bottom,
        } => collect_bar_js_segments(
            "footer", "footer", props, top, start, center, end, bottom, None, segments, context,
        ),
        ViewNode::BottomBar { props, tabs, .. } => {
            push_literal(segments, &render_bottom_bar_html(props, tabs, context))
        }
        ViewNode::SideNav { props, items } => {
            push_literal(
                segments,
                &render_side_nav_html("sidenav", props, items, context),
            );
        }
        ViewNode::RailNav { props, items } => {
            push_literal(segments, &render_rail_nav_html(props, items, context));
        }
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => {
            push_literal(
                segments,
                &format!(
                    "<aside{}>",
                    attrs(
                        sidebar_classes(props),
                        Some(&props.style.element),
                        None,
                        context
                    )
                ),
            );
            if !header.is_empty() {
                push_literal(segments, "<div class=\"sidebar-header\">");
                for child in header {
                    collect_js_segments(child, segments, context);
                }
                push_literal(segments, "</div>");
            }
            push_literal(segments, "<div class=\"sidebar-body\">");
            for child in body {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</div>");
            if !footer.is_empty() {
                push_literal(segments, "<div class=\"sidebar-footer\">");
                for child in footer {
                    collect_js_segments(child, segments, context);
                }
                push_literal(segments, "</div>");
            }
            push_literal(segments, "</aside>");
        }
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
        } => {
            collect_scaffold_js_segments(
                props, app_bar, start, main, end, bottom_bar, overlays, segments, context,
            );
        }
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => {
            let extra = drawer_panel_attrs(props, context);
            push_literal(
                segments,
                &format!(
                    "<div{} hidden><button class=\"drawer-overlay\" type=\"button\" aria-label=\"Close drawer\" data-dowe-drawer-overlay></button><div{} role=\"dialog\" aria-modal=\"true\">",
                    attrs(
                        drawer_panel_classes(props),
                        Some(&props.style.element),
                        Some(&extra),
                        context,
                    ),
                    class_attr(drawer_classes(props))
                ),
            );
            if !header.is_empty() {
                push_literal(segments, "<div class=\"drawer-header\">");
                for child in header {
                    collect_js_segments(child, segments, context);
                }
                push_literal(segments, "</div>");
            }
            push_literal(segments, "<div class=\"drawer-body\">");
            for child in body {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</div>");
            if !footer.is_empty() {
                push_literal(segments, "<div class=\"drawer-footer\">");
                for child in footer {
                    collect_js_segments(child, segments, context);
                }
                push_literal(segments, "</div>");
            }
            push_literal(segments, "</div>");
            if !props.hide_close_button {
                push_literal(segments, &drawer_close_html(props.position.as_str()));
            }
            push_literal(segments, "</div>");
        }
        _ => return false,
    }
    true
}
