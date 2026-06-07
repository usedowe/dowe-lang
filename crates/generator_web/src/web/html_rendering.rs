fn text_line_css(value: TextSize) -> &'static str {
    text_typography(false, value).line_height
}

fn title_text_size_css(value: TextSize) -> String {
    fluid_text_size_css(text_typography(true, value).font_size)
}

fn title_text_line_css(value: TextSize) -> &'static str {
    text_typography(true, value).line_height
}

fn title_text_weight_css(value: TextSize) -> &'static str {
    text_weight_number(text_typography(true, value).weight)
}

fn title_text_spacing_css(value: TextSize) -> String {
    format!("{}em", text_typography(true, value).letter_spacing_em)
}

fn text_weight_css(value: TextWeight) -> &'static str {
    text_weight_number(value)
}

fn text_spacing_css(value: TextSpacing) -> String {
    format!("{}em", text_spacing_em(value))
}

fn on_token(family: ColorFamily) -> &'static str {
    match family {
        ColorFamily::Primary => "onPrimary",
        ColorFamily::Secondary => "onSecondary",
        ColorFamily::Tertiary => "onTertiary",
        ColorFamily::Muted => "onMuted",
        ColorFamily::Background => "onBackground",
        ColorFamily::Surface => "onSurface",
        ColorFamily::Success => "onSuccess",
        ColorFamily::Info => "onInfo",
        ColorFamily::Warning => "onWarning",
        ColorFamily::Danger => "onDanger",
    }
}

fn soft_token(family: ColorFamily) -> &'static str {
    match family {
        ColorFamily::Primary => "softPrimary",
        ColorFamily::Secondary => "softSecondary",
        ColorFamily::Tertiary => "softTertiary",
        ColorFamily::Muted => "softMuted",
        ColorFamily::Background => "background",
        ColorFamily::Surface => "surface",
        ColorFamily::Success => "softSuccess",
        ColorFamily::Info => "softInfo",
        ColorFamily::Warning => "softWarning",
        ColorFamily::Danger => "softDanger",
    }
}

fn on_soft_token(family: ColorFamily) -> &'static str {
    match family {
        ColorFamily::Primary => "onSoftPrimary",
        ColorFamily::Secondary => "onSoftSecondary",
        ColorFamily::Tertiary => "onSoftTertiary",
        ColorFamily::Muted => "onSoftMuted",
        ColorFamily::Background => "onBackground",
        ColorFamily::Surface => "onSurface",
        ColorFamily::Success => "onSoftSuccess",
        ColorFamily::Info => "onSoftInfo",
        ColorFamily::Warning => "onSoftWarning",
        ColorFamily::Danger => "onSoftDanger",
    }
}

fn append_responsive_rule(css: &mut String, breakpoint: Breakpoint, class_name: &str, body: &str) {
    css.push_str(&format!(
        ".{}\\:{}{{{body}}}",
        breakpoint.as_str(),
        css_class_name(class_name)
    ));
}

fn append_rule(css: &mut String, class_name: &str, body: &str) {
    css.push_str(&format!(".{}{{{body}}}", css_class_name(class_name)));
}

fn css_class_name(value: &str) -> String {
    value.replace(':', "\\:").replace('.', "\\.")
}

fn page_file_name(page: &ViewPage) -> String {
    let file_name = page.route_path.trim_matches('/').replace('/', "-");
    if file_name.is_empty() {
        "index".to_string()
    } else {
        file_name
    }
}

#[derive(Clone, Default)]
struct ReactiveRenderContext {
    signals: Vec<(String, String)>,
    actions: Vec<(String, String)>,
}

impl ReactiveRenderContext {
    fn with_scope(&self, signals: &[ViewSignal], actions: &[ViewAction]) -> Self {
        let mut context = self.clone();
        context.signals.extend(
            signals
                .iter()
                .map(|signal| (signal.name.clone(), signal.id.clone())),
        );
        context.actions.extend(
            actions
                .iter()
                .map(|action| (action.name.clone(), action.id.clone())),
        );
        context
    }

    fn signal_path(&self, value: &str) -> String {
        let (root, suffix) = value
            .split_once('.')
            .map(|(root, suffix)| (root, Some(suffix)))
            .unwrap_or((value, None));
        let Some((_, id)) = self.signals.iter().rev().find(|(name, _)| name == root) else {
            return value.to_string();
        };
        suffix
            .map(|suffix| format!("{id}.{suffix}"))
            .unwrap_or_else(|| id.clone())
    }

    fn action_id(&self, value: &str) -> String {
        self.actions
            .iter()
            .rev()
            .find(|(name, _)| name == value)
            .map(|(_, id)| id.clone())
            .unwrap_or_else(|| value.to_string())
    }
}

fn render_html(node: &ViewNode, children_html: Option<&str>) -> String {
    render_html_with_context(node, children_html, &ReactiveRenderContext::default())
}

fn render_html_with_context(
    node: &ViewNode,
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    match node {
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => {
            let context = context.with_scope(signals, actions);
            children
                .iter()
                .map(|child| render_html_with_context(child, children_html, &context))
                .collect::<String>()
        }
        ViewNode::Box { props, children } => {
            let mut html = format!(
                "<div{}>",
                attrs(box_classes(props), Some(&props.element), None, context)
            );
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str("</div>");
            html
        }
        ViewNode::Section { props, children } => {
            let mut html = format!(
                "<section{}>",
                attrs(section_classes(props), Some(&props.element), None, context)
            );
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str("</section>");
            html
        }
        ViewNode::Flex { props, children } => {
            let mut html = format!(
                "<div{}>",
                attrs(
                    layout_classes("flex", props),
                    Some(&props.style.element),
                    None,
                    context
                )
            );
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str("</div>");
            html
        }
        ViewNode::Grid { props, children } => {
            let mut html = format!(
                "<div{}>",
                attrs(
                    grid_classes(props),
                    Some(&props.style.element),
                    None,
                    context
                )
            );
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str("</div>");
            html
        }
        ViewNode::Card { props, children } => {
            let mut html = format!(
                "<article{}>",
                attrs(
                    variant_classes("card", props),
                    Some(&props.element),
                    None,
                    context,
                )
            );
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str("</article>");
            html
        }
        ViewNode::Tabs { props, tabs } => render_tabs_html(props, tabs, children_html, context),
        ViewNode::NavMenu { props, items } => {
            render_nav_menu_html(props, items, children_html, context)
        }
        ViewNode::Button { props, children } => {
            let (open, close) = button_tags(props, context);
            let mut html = open;
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str(close);
            html
        }
        ViewNode::Input { props } => render_input_html(props, context),
        ViewNode::Select { props, options } => render_select_html(props, options, context),
        ViewNode::Audio { props } => render_audio_html(props, context),
        ViewNode::Image { props } => render_image_html(props, context),
        ViewNode::Code { props } => render_code_html(props, context),
        ViewNode::Video { props } => render_video_html(props, context),
        ViewNode::Candlestick { props } => render_candlestick_html(props, context),
        ViewNode::Table { props } => render_table_html(props, context),
        ViewNode::Divider { props } => render_divider_html(props, context),
        ViewNode::Title { props, value } => render_text_html(
            "title",
            text_classes("title", props),
            Some(&props.style.element),
            value,
            props.i18n.as_deref(),
            context,
        ),
        ViewNode::Text { props, value } => render_text_html(
            "text",
            text_classes("text", props),
            Some(&props.style.element),
            value,
            props.i18n.as_deref(),
            context,
        ),
        ViewNode::Alert { props } => {
            let message = dynamic_text_attr(&props.message, context);
            let content = if message.is_empty() {
                escape_html(&props.message)
            } else {
                String::new()
            };
            let close = props
                .on_close
                .as_ref()
                .map(|action| {
                    format!(
                        r#"<button class="alert-close" type="button" data-dowe-click="{}">&times;</button>"#,
                        escape_attr(&context.action_id(action))
                    )
                })
                .unwrap_or_default();
            format!(
                r#"<div{}><span data-dowe-alert-message{}>{}</span>{}</div>"#,
                attrs(
                    variant_classes("alert", &props.style),
                    Some(&props.style.element),
                    Some(&alert_attrs(props, context)),
                    context
                ),
                message,
                content,
                close
            )
        }
        ViewNode::Avatar { props, icon } => render_avatar_html(props, icon.as_ref(), context),
        ViewNode::Badge { props, children } => {
            render_badge_html(props, children, children_html, context)
        }
        ViewNode::Chip {
            props,
            value,
            start,
            end,
        } => render_chip_html(props, value, start.as_ref(), end.as_ref(), context),
        ViewNode::Skeleton { props } => render_skeleton_html(props, context),
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => render_modal_html(props, header, body, footer, children_html, context),
        ViewNode::AlertDialog { props } => render_alert_dialog_html(props, context),
        ViewNode::Tooltip { props, children } => {
            render_tooltip_html(props, children, children_html, context)
        }
        ViewNode::Toast { props } => render_toast_html(props, context),
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            entries,
            footer,
        } => render_dropdown_html(props, trigger, header, entries, footer, children_html, context),
        ViewNode::Command { props, entries } => render_command_html(props, entries, context),
        ViewNode::Accordion { props, items } => {
            render_accordion_html(props, items, children_html, context)
        }
        ViewNode::Carousel { props, slides } => {
            render_carousel_html(props, slides, children_html, context)
        }
        ViewNode::Checkbox { props } => render_checkbox_html(props, context),
        ViewNode::Color { props } => render_color_html(props, context),
        ViewNode::Date { props } => render_date_html(props, context),
        ViewNode::DateRange { props } => render_date_range_html(props, context),
        ViewNode::RadioGroup { props, options } => render_radio_group_html(props, options, context),
        ViewNode::Toggle { props } => render_toggle_html(props, context),
        ViewNode::Svg { props, paths } => render_svg_html(props, paths, context),
        ViewNode::AppBar {
            props,
            start,
            center,
            end,
        } => render_bar_html(
            "header",
            "appbar",
            props,
            start,
            center,
            end,
            children_html,
            context,
        ),
        ViewNode::Footer {
            props,
            start,
            center,
            end,
        } => render_bar_html(
            "footer",
            "footer",
            props,
            start,
            center,
            end,
            children_html,
            context,
        ),
        ViewNode::BottomBar {
            props,
            start,
            center,
            end,
        } => render_bar_html(
            "nav",
            "bottombar",
            props,
            start,
            center,
            end,
            children_html,
            context,
        ),
        ViewNode::SideNav { props, items } => render_side_nav_html("sidenav", props, items, context),
        ViewNode::Sidebar { props, items } => {
            render_side_nav_html("sidebar", props, items, context)
        }
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
        } => render_scaffold_html(
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            children_html,
            context,
        ),
        ViewNode::Drawer { props, children } => {
            render_drawer_html(props, children, children_html, context)
        }
        ViewNode::Each {
            item,
            collection,
            key,
            children,
        } => {
            let children = children
                .iter()
                .map(|child| render_html_with_context(child, children_html, context))
                .collect::<String>();
            format!(
                r#"<div data-dowe-each="{}" data-dowe-item="{}" data-dowe-key="{}"><template>{}</template></div>"#,
                escape_attr(&context.signal_path(collection)),
                escape_attr(item),
                escape_attr(key),
                children
            )
        }
        ViewNode::Children => children_html.unwrap_or_default().to_string(),
    }
}

fn js_render_expression(node: &ViewNode) -> String {
    let mut segments = Vec::new();
    collect_js_segments(node, &mut segments, &ReactiveRenderContext::default());

    if segments.is_empty() {
        return "\"\"".to_string();
    }

    segments
        .into_iter()
        .map(|segment| match segment {
            JsSegment::Literal(value) => js_string_literal(&value),
            JsSegment::Children => "children".to_string(),
        })
        .collect::<Vec<_>>()
        .join("+")
}

fn collect_js_segments(
    node: &ViewNode,
    segments: &mut Vec<JsSegment>,
    context: &ReactiveRenderContext,
) {
    match node {
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => {
            let context = context.with_scope(signals, actions);
            for child in children {
                collect_js_segments(child, segments, &context);
            }
        }
        ViewNode::Box { props, children } => {
            push_literal(
                segments,
                &format!(
                    "<div{}>",
                    attrs(box_classes(props), Some(&props.element), None, context)
                ),
            );
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</div>");
        }
        ViewNode::Section { props, children } => {
            push_literal(
                segments,
                &format!(
                    "<section{}>",
                    attrs(section_classes(props), Some(&props.element), None, context)
                ),
            );
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</section>");
        }
        ViewNode::Flex { props, children } => {
            push_literal(
                segments,
                &format!(
                    "<div{}>",
                    attrs(
                        layout_classes("flex", props),
                        Some(&props.style.element),
                        None,
                        context
                    )
                ),
            );
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</div>");
        }
        ViewNode::Grid { props, children } => {
            push_literal(
                segments,
                &format!(
                    "<div{}>",
                    attrs(
                        grid_classes(props),
                        Some(&props.style.element),
                        None,
                        context
                    )
                ),
            );
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</div>");
        }
        ViewNode::Card { props, children } => {
            push_literal(
                segments,
                &format!(
                    "<article{}>",
                    attrs(
                        variant_classes("card", props),
                        Some(&props.element),
                        None,
                        context,
                    )
                ),
            );
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</article>");
        }
        ViewNode::Tabs { props, tabs } => collect_tabs_js_segments(props, tabs, segments, context),
        ViewNode::NavMenu { props, items } => {
            collect_nav_menu_js_segments(props, items, segments, context)
        }
        ViewNode::Button { props, children } => {
            let (open, close) = button_tags(props, context);
            push_literal(segments, &open);
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, close);
        }
        ViewNode::Input { props } => {
            push_literal(segments, &render_input_html(props, context));
        }
        ViewNode::Select { props, options } => {
            push_literal(segments, &render_select_html(props, options, context));
        }
        ViewNode::Code { props } => {
            push_literal(segments, &render_code_html(props, context));
        }
        ViewNode::Video { props } => {
            push_literal(segments, &render_video_html(props, context));
        }
        ViewNode::Audio { props } => {
            push_literal(segments, &render_audio_html(props, context));
        }
        ViewNode::Image { props } => {
            push_literal(segments, &render_image_html(props, context));
        }
        ViewNode::Candlestick { props } => {
            push_literal(segments, &render_candlestick_html(props, context));
        }
        ViewNode::Table { props } => {
            push_literal(segments, &render_table_html(props, context));
        }
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
                    context,
                ),
            );
        }
        ViewNode::Alert { .. } => {
            push_literal(segments, &render_html_with_context(node, None, context))
        }
        ViewNode::Avatar { .. }
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
            start,
            center,
            end,
        } => collect_bar_js_segments(
            "header", "appbar", props, start, center, end, segments, context,
        ),
        ViewNode::Footer {
            props,
            start,
            center,
            end,
        } => collect_bar_js_segments(
            "footer", "footer", props, start, center, end, segments, context,
        ),
        ViewNode::BottomBar {
            props,
            start,
            center,
            end,
        } => collect_bar_js_segments(
            "nav",
            "bottombar",
            props,
            start,
            center,
            end,
            segments,
            context,
        ),
        ViewNode::SideNav { props, items } => {
            push_literal(segments, &render_side_nav_html("sidenav", props, items, context));
        }
        ViewNode::Sidebar { props, items } => {
            push_literal(segments, &render_side_nav_html("sidebar", props, items, context));
        }
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
        } => {
            collect_scaffold_js_segments(
                props, app_bar, start, main, end, bottom_bar, segments, context,
            );
        }
        ViewNode::Drawer { props, children } => {
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
            if !props.hide_close_button {
                push_literal(segments, drawer_close_html());
            }
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</div></div>");
        }
        ViewNode::Each {
            item,
            collection,
            key,
            children,
        } => {
            push_literal(
                segments,
                &format!(
                    r#"<div data-dowe-each="{}" data-dowe-item="{}" data-dowe-key="{}"><template>"#,
                    escape_attr(&context.signal_path(collection)),
                    escape_attr(item),
                    escape_attr(key)
                ),
            );
            for child in children {
                collect_js_segments(child, segments, context);
            }
            push_literal(segments, "</template></div>");
        }
        ViewNode::Children => segments.push(JsSegment::Children),
    }
}

fn render_bar_html(
    tag: &str,
    base: &str,
    props: &BarProps,
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<{tag}{}><div{}>",
        attrs(
            bar_classes(base, props),
            Some(&props.style.element),
            None,
            context,
        ),
        class_attr(bar_content_classes(base, props))
    );
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
    html.push_str(&format!("</div></{tag}>"));
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
    start: &[ViewNode],
    center: &[ViewNode],
    end: &[ViewNode],
    segments: &mut Vec<JsSegment>,
    context: &ReactiveRenderContext,
) {
    push_literal(
        segments,
        &format!(
            "<{tag}{}><div{}>",
            attrs(
                bar_classes(base, props),
                Some(&props.style.element),
                None,
                context,
            ),
            class_attr(bar_content_classes(base, props))
        ),
    );
    collect_bar_region_js_segments(base, "start", start, segments, context);
    collect_bar_region_js_segments(base, "center", center, segments, context);
    collect_bar_region_js_segments(base, "end", end, segments, context);
    push_literal(segments, &format!("</div></{tag}>"));
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
            attrs(tabs_classes(props), Some(&props.style.element), None, context)
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
        push_literal(
            segments,
            &format!(
                r#"<button{} type="button" role="tab" id="{}" aria-selected="{}" aria-controls="{}" data-dowe-tab="{}"><span class="tabs-label">{}</span></button>"#,
                class_attr(classes),
                escape_attr(&tab_button_id(tab)),
                if active { "true" } else { "false" },
                escape_attr(&tab_panel_id(tab)),
                escape_attr(&tab.id),
                escape_html(&tab.label)
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

fn render_svg_html(props: &SvgProps, paths: &[SvgPath], context: &ReactiveRenderContext) -> String {
    let mut html = format!(
        r#"<svg{} xmlns="http://www.w3.org/2000/svg" viewBox="{}" aria-hidden="true">"#,
        attrs(
            svg_classes(&props.style),
            Some(&props.style.element),
            None,
            context
        ),
        escape_attr(&props.view_box.as_str())
    );
    for path in paths {
        html.push_str(&format!(
            r#"<path d="{}" fill="{}"></path>"#,
            escape_attr(&path.data),
            escape_attr(&svg_path_fill(path.fill))
        ));
    }
    html.push_str("</svg>");
    html
}

fn render_candlestick_html(props: &CandlestickProps, context: &ReactiveRenderContext) -> String {
    let mut extra = format!(
        r#" role="figure" aria-label="Candlestick chart" data-dowe-candlestick data-dowe-candlestick-data="{}" data-dowe-candlestick-up="{}" data-dowe-candlestick-down="{}" data-dowe-candlestick-max="{}""#,
        escape_attr(&context.signal_path(&props.data)),
        props.up_color.as_str(),
        props.down_color.as_str(),
        props.max_points
    );
    if let Some(stream) = props.stream.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-candlestick-stream="{}""#,
            escape_attr(stream)
        ));
    }
    format!(
        r#"<figure{}><canvas class="candlestick-canvas"></canvas><figcaption class="candlestick-empty">{}</figcaption></figure>"#,
        attrs(
            candlestick_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context,
        ),
        escape_html(&props.empty_label)
    )
}

fn render_table_html(props: &TableProps, context: &ReactiveRenderContext) -> String {
    let mut html = format!(
        "<div{}><div class=\"table-container\"><table{}{}><thead class=\"table-header\"><tr>",
        attrs(
            table_wrapper_classes(props),
            Some(&props.style.element),
            None,
            context,
        ),
        class_attr(table_classes(props)),
        table_attrs(props, context),
    );
    for column in &props.columns {
        html.push_str(&format!(
            r#"<th class="table-head" data-dowe-table-field="{}" data-dowe-table-align="{}"{}><div class="table-head-content"><span class="table-head-label">{}</span></div></th>"#,
            escape_attr(&column.field),
            column.align.as_str(),
            table_column_style(column),
            escape_html(&column.label)
        ));
    }
    html.push_str("</tr></thead><tbody class=\"table-body\">");
    html.push_str(&render_table_empty_row(props));
    html.push_str("</tbody></table></div></div>");
    html
}

fn render_tabs_html(
    props: &TabsProps,
    tabs: &[TabItem],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<div{} data-dowe-tabs>",
        attrs(tabs_classes(props), Some(&props.style.element), None, context)
    );
    html.push_str(&format!(
        r#"<div{} role="tablist">"#,
        class_attr(tabs_list_classes(props))
    ));
    for (index, tab) in tabs.iter().enumerate() {
        let active = index == 0;
        let mut classes = vec!["tab".to_string()];
        if active {
            classes.push("on-active".to_string());
        }
        html.push_str(&format!(
            r#"<button{} type="button" role="tab" id="{}" aria-selected="{}" aria-controls="{}" tabindex="{}" data-dowe-tab="{}"><span class="tabs-label">{}</span></button>"#,
            class_attr(classes),
            escape_attr(&tab_button_id(tab)),
            if active { "true" } else { "false" },
            escape_attr(&tab_panel_id(tab)),
            if active { "0" } else { "-1" },
            escape_attr(&tab.id),
            escape_html(&tab.label)
        ));
    }
    html.push_str("</div><div class=\"tabs-wrapper\">");
    for (index, tab) in tabs.iter().enumerate() {
        let active = index == 0;
        let mut classes = vec!["tabs-content".to_string()];
        if active {
            classes.push("on-active".to_string());
        }
        html.push_str(&format!(
            r#"<div{} id="{}" role="tabpanel" aria-labelledby="{}" data-dowe-tab-panel="{}"{}>"#,
            class_attr(classes),
            escape_attr(&tab_panel_id(tab)),
            escape_attr(&tab_button_id(tab)),
            escape_attr(&tab.id),
            if active { "" } else { " hidden" }
        ));
        for child in &tab.children {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div>");
    }
    html.push_str("</div></div>");
    html
}

fn tab_button_id(tab: &TabItem) -> String {
    format!("tab-{}-button", tab.id)
}

fn tab_panel_id(tab: &TabItem) -> String {
    format!("tab-{}-panel", tab.id)
}

fn table_attrs(props: &TableProps, context: &ReactiveRenderContext) -> String {
    format!(
        r#" data-dowe-table data-dowe-table-data="{}" data-dowe-table-empty-title="{}" data-dowe-table-empty-description="{}""#,
        escape_attr(&context.signal_path(&props.data)),
        escape_attr(&props.empty_title),
        escape_attr(&props.empty_description)
    )
}

fn render_table_empty_row(props: &TableProps) -> String {
    format!(
        r#"<tr class="table-empty-row"><td class="table-empty-cell" colspan="{}"><div class="empty-state"><div class="empty-content"><h3 class="empty-title">{}</h3><p class="empty-description">{}</p></div></div></td></tr>"#,
        props.columns.len(),
        escape_html(&props.empty_title),
        escape_html(&props.empty_description)
    )
}

fn table_column_style(column: &TableColumn) -> String {
    let mut styles = vec![format!("text-align:{}", table_align_css(column.align))];
    if let Some(width) = column.width.as_deref() {
        styles.push(format!("width:{}", escape_attr(width)));
    }
    format!(r#" style="{}""#, styles.join(";"))
}

fn table_align_css(value: TableColumnAlign) -> &'static str {
    match value {
        TableColumnAlign::Start => "start",
        TableColumnAlign::Center => "center",
        TableColumnAlign::End => "end",
    }
}

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
            push_literal(segments, &render_nav_menu_popover_html(index, item, None, context));
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
    html.push_str(&render_scaffold_region_html(app_bar, children_html, context));
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
    html.push_str(&render_scaffold_region_html(bottom_bar, children_html, context));
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

fn render_audio_html(props: &AudioProps, context: &ReactiveRenderContext) -> String {
    let bars = (0..50)
        .map(|index| {
            let height = 28 + ((index * 17) % 58);
            format!(
                r#"<span class="media-bar" style="height:{}%"></span>"#,
                height
            )
        })
        .collect::<String>();
    let subtitle = props
        .subtitle
        .as_deref()
        .map(|value| {
            format!(
                r#"<span class="media-subtitle">{}</span>"#,
                escape_html(value)
            )
        })
        .unwrap_or_default();
    let avatar = props
        .avatar_src
        .as_deref()
        .map(|src| {
            format!(
                r#"<img class="media-avatar" src="{}" alt="">"#,
                escape_attr(src)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<div{} data-dowe-audio><audio src="{}" preload="metadata" data-dowe-audio-el></audio><button class="media-button" type="button" aria-label="Play audio" data-dowe-audio-toggle><span data-dowe-audio-icon>▶</span></button><div class="media-content"><div class="media-waveform" role="slider" tabindex="0" aria-valuemin="0" aria-valuemax="100" aria-valuenow="0" data-dowe-audio-waveform><div class="media-bars">{}</div></div><div class="media-footer"><span class="media-time" data-dowe-audio-time>0:00</span>{}</div></div>{}</div>"#,
        attrs(
            variant_classes("media", &props.style),
            Some(&props.style.element),
            None,
            context
        ),
        escape_attr(&props.src),
        bars,
        subtitle,
        avatar
    )
}

fn render_image_html(props: &ImageProps, context: &ReactiveRenderContext) -> String {
    let controls = if props.hide_controls {
        String::new()
    } else {
        r#"<div class="image-controls"><div class="image-actions"><button class="image-action" type="button" aria-label="Download image" data-dowe-image-download>↓</button><button class="image-action" type="button" aria-label="Toggle fullscreen" data-dowe-image-fullscreen>⛶</button></div></div>"#.to_string()
    };
    let mut classes = variant_classes("image", &props.style);
    classes.push(props.aspect.as_str().to_string());
    classes.push(format!("fit-{}", props.object_fit.as_str()));
    format!(
        r#"<figure{} data-dowe-image><img class="image-element" src="{}" alt="{}" loading="{}">{}</figure>"#,
        attrs(classes, Some(&props.style.element), None, context),
        escape_attr(&props.src),
        escape_attr(&props.alt),
        props.loading.as_str(),
        controls
    )
}

fn render_accordion_html(
    props: &AccordionProps,
    items: &[AccordionItem],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut extra = format!(r#" data-dowe-accordion data-dowe-accordion-multiple="{}""#, props.multiple);
    if props.multiple {
        extra.push_str(r#" aria-multiselectable="true""#);
    }
    let mut html = format!(
        "<div{}>",
        attrs(
            variant_classes("accordion", &props.style),
            Some(&props.style.element),
            Some(&extra),
            context
        )
    );
    for item in items {
        let mut item_classes = vec!["accordion-item".to_string()];
        if item.disabled {
            item_classes.push("is-disabled".to_string());
        }
        if item.default_open {
            item_classes.push("is-open".to_string());
        }
        let hidden = if item.default_open { "" } else { " hidden" };
        let expanded = if item.default_open { "true" } else { "false" };
        html.push_str(&format!(
            r#"<div{} data-dowe-accordion-item><button class="accordion-header{}" type="button" aria-expanded="{}" data-dowe-accordion-trigger{}><span class="accordion-start"><span class="accordion-label">{}</span></span><span class="accordion-end"><span class="accordion-arrow">⌄</span></span></button><div class="accordion-content" data-dowe-accordion-content{}>"#,
            class_attr(item_classes),
            if item.default_open { " is-open" } else { "" },
            expanded,
            if item.disabled { " disabled" } else { "" },
            escape_html(&item.label),
            hidden
        ));
        for child in &item.children {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div></div>");
    }
    html.push_str("</div>");
    html
}

fn render_carousel_html(
    props: &CarouselProps,
    slides: &[CarouselSlide],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut classes = variant_classes("carousel", &props.style);
    if props.orientation == CarouselOrientation::Vertical {
        classes.push("is-vertical".to_string());
    }
    let title = props
        .title
        .as_deref()
        .map(|value| {
            format!(
                r#"<div class="carousel-header"><div class="carousel-title"><h2>{}</h2></div></div>"#,
                escape_html(value)
            )
        })
        .unwrap_or_default();
    let extra = format!(
        r#" data-dowe-carousel data-dowe-carousel-index="0" data-dowe-carousel-loop="{}" data-dowe-carousel-autoplay="{}" data-dowe-carousel-interval="{}" data-dowe-carousel-orientation="{}""#,
        !props.disable_loop,
        props.autoplay,
        props.autoplay_interval,
        props.orientation.as_str()
    );
    let mut html = format!(
        "<div{}>{}<div class=\"carousel-viewport\"><div class=\"carousel-container\" data-dowe-carousel-track style=\"gap:{}px;\">",
        attrs(classes, Some(&props.style.element), Some(&extra), context),
        title,
        props.gap
    );
    for slide in slides {
        let mut style = String::new();
        if let Some(width) = props.slide_width {
            style.push_str(&format!("width:{width}px;"));
        }
        if let Some(height) = props.slide_height {
            style.push_str(&format!("height:{height}px;"));
        }
        html.push_str(&format!(
            r#"<div class="carousel-slide" data-dowe-carousel-slide="{}"{}>"#,
            escape_attr(&slide.id),
            if style.is_empty() {
                String::new()
            } else {
                format!(r#" style="{}""#, escape_attr(&style))
            }
        ));
        for child in &slide.children {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div>");
    }
    html.push_str("</div>");
    if props.show_navigation {
        html.push_str(r#"<button class="carousel-nav is-prev" type="button" aria-label="Previous slide" data-dowe-carousel-prev>‹</button><button class="carousel-nav is-next" type="button" aria-label="Next slide" data-dowe-carousel-next>›</button>"#);
    }
    html.push_str("</div>");
    if !props.hide_controls || !props.hide_indicators || props.show_counter {
        html.push_str("<div class=\"carousel-controls\">");
        if !props.hide_controls {
            html.push_str(r#"<button class="carousel-control" type="button" aria-label="Previous slide" data-dowe-carousel-prev>‹</button>"#);
        }
        if !props.hide_indicators {
            html.push_str("<div class=\"carousel-indicators\">");
            for (index, _slide) in slides.iter().enumerate() {
                let mut classes = vec![
                    "carousel-indicator".to_string(),
                    format!("is-{}", props.size.as_str()),
                    format!("is-{}", props.style.color.unwrap_or(ColorFamily::Primary).as_str()),
                ];
                if index == 0 {
                    classes.push("is-active".to_string());
                }
                if props.indicator_type == CarouselIndicatorType::Dot {
                    classes.push("is-dot".to_string());
                }
                html.push_str(&format!(
                    r#"<button{} type="button" aria-label="Go to slide {}" data-dowe-carousel-indicator="{}"></button>"#,
                    class_attr(classes),
                    index + 1,
                    index
                ));
            }
            html.push_str("</div>");
        }
        if props.show_counter {
            html.push_str(&format!(
                r#"<div class="carousel-counter" data-dowe-carousel-counter>1 / {}</div>"#,
                slides.len()
            ));
        }
        if !props.hide_controls {
            html.push_str(r#"<button class="carousel-control" type="button" aria-label="Next slide" data-dowe-carousel-next>›</button>"#);
        }
        html.push_str("</div>");
    }
    html.push_str("</div>");
    html
}

fn render_checkbox_html(props: &CheckboxProps, context: &ReactiveRenderContext) -> String {
    let mut input = format!(
        r#"<input type="checkbox" class="checkbox-input is-{}"{}{}{}{}>"#,
        props.style.color.unwrap_or(ColorFamily::Primary).as_str(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.checked { " checked" } else { "" },
        if props.disabled { " disabled" } else { "" }
    );
    input.push_str(
        &props
            .style
            .label
            .as_deref()
            .map(|label| format!(r#"<span class="label-md">{}</span>"#, escape_html(label)))
            .unwrap_or_default(),
    );
    format!(
        "<label{}>{}</label>",
        attrs(
            vec!["checkbox".to_string()],
            Some(&props.style.element),
            None,
            context
        ),
        input
    )
}

fn render_color_html(props: &ColorProps, context: &ReactiveRenderContext) -> String {
    let input = format!(
        r#"<input class="color-input" type="color" value="{}"{}{}>"#,
        escape_attr(&props.value),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context)
    );
    let preview = format!(
        r#"<span class="color-field-swatch is-{}" style="background-color:{}"></span><span class="color-field-value">{}</span>"#,
        props.size.as_str(),
        escape_attr(&props.value),
        escape_html(&props.value.to_ascii_uppercase())
    );
    let values = render_color_values(props);
    render_field_control(
        "color-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &format!("{input}<span class=\"color-field-display\">{preview}</span>{values}"),
        context,
    )
}

fn render_date_html(props: &DateProps, context: &ReactiveRenderContext) -> String {
    let input = format!(
        r#"<input class="date-input" type="date"{}{}{}{}{}>"#,
        props
            .value
            .as_deref()
            .map(|value| format!(r#" value="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        props
            .min
            .as_deref()
            .map(|value| format!(r#" min="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .max
            .as_deref()
            .map(|value| format!(r#" max="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context)
    );
    render_field_control(
        "date-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &input,
        context,
    )
}

fn render_date_range_html(props: &DateRangeProps, context: &ReactiveRenderContext) -> String {
    let start_bind = props
        .start
        .as_deref()
        .map(|value| {
            format!(
                r#" data-dowe-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default();
    let end_bind = props
        .end
        .as_deref()
        .map(|value| {
            format!(
                r#" data-dowe-bind="{}""#,
                escape_attr(&context.signal_path(value))
            )
        })
        .unwrap_or_default();
    let common = format!(
        "{}{}",
        props
            .min
            .as_deref()
            .map(|value| format!(r#" min="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .max
            .as_deref()
            .map(|value| format!(r#" max="{}""#, escape_attr(value)))
            .unwrap_or_default()
    );
    let input = format!(
        r#"<span class="date-range-inputs"><input class="date-input" type="date"{}{}{}><span class="date-range-separator">-</span><input class="date-input" type="date"{}{}{}></span>"#,
        props
            .start_value
            .as_deref()
            .map(|value| format!(r#" value="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}Start""#, escape_attr(name)))
            .unwrap_or_default(),
        format!("{common}{start_bind}"),
        props
            .end_value
            .as_deref()
            .map(|value| format!(r#" value="{}""#, escape_attr(value)))
            .unwrap_or_default(),
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}End""#, escape_attr(name)))
            .unwrap_or_default(),
        format!("{common}{end_bind}")
    );
    render_field_control(
        "date-range-field",
        &props.style,
        props.size,
        props.help_text.as_deref(),
        props.error_text.as_deref(),
        &input,
        context,
    )
}

fn render_radio_group_html(
    props: &RadioGroupProps,
    options: &[RadioOption],
    context: &ReactiveRenderContext,
) -> String {
    let name = props
        .name
        .clone()
        .unwrap_or_else(|| format!("radio-{}", short_id("radio", &options[0].value)));
    let mut group = String::from("<div class=\"radio-group\">");
    for option in options {
        group.push_str(&format!(
            r#"<label class="radio-item"><input type="radio" class="radio is-{} is-{}" name="{}" value="{}"{}{}><span class="label">{}</span></label>"#,
            props.style.color.unwrap_or(ColorFamily::Primary).as_str(),
            props.size.as_str(),
            escape_attr(&name),
            escape_attr(&option.value),
            bind_attr(props.style.element.bind.as_deref(), context),
            if option.disabled { " disabled" } else { "" },
            escape_html(&option.label)
        ));
    }
    group.push_str("</div>");
    render_field_block(
        &props.style,
        props.info.as_deref(),
        props.error.as_deref(),
        &group,
        context,
    )
}

fn render_toggle_html(props: &ToggleProps, context: &ReactiveRenderContext) -> String {
    let left = props
        .label_left
        .as_deref()
        .map(|label| {
            format!(
                r#"<span class="toggle-label-left{}">{}</span>"#,
                if props.checked { "" } else { " is-active" },
                escape_html(label)
            )
        })
        .unwrap_or_default();
    let right = props
        .label_right
        .as_deref()
        .map(|label| {
            format!(
                r#"<span class="toggle-label-right{}">{}</span>"#,
                if props.checked { " is-active" } else { "" },
                escape_html(label)
            )
        })
        .unwrap_or_default();
    let label = props
        .style
        .label
        .as_deref()
        .map(|label| format!(r#"<span class="label-md">{}</span>"#, escape_html(label)))
        .unwrap_or_default();
    let input = format!(
        r#"<input type="checkbox" role="switch" class="toggle-input is-{}" aria-checked="{}"{}{}{}{}>"#,
        props.style.color.unwrap_or(ColorFamily::Primary).as_str(),
        props.checked,
        props
            .name
            .as_deref()
            .map(|name| format!(r#" name="{}""#, escape_attr(name)))
            .unwrap_or_default(),
        bind_attr(props.style.element.bind.as_deref(), context),
        if props.checked { " checked" } else { "" },
        if props.disabled { " disabled" } else { "" }
    );
    format!(
        "<label{}>{left}{input}{right}{label}</label>",
        attrs(
            vec!["toggle".to_string()],
            Some(&props.style.element),
            None,
            context
        )
    )
}

fn render_field_control(
    base: &str,
    props: &VariantProps,
    size: ButtonSize,
    help_text: Option<&str>,
    error_text: Option<&str>,
    control_html: &str,
    context: &ReactiveRenderContext,
) -> String {
    let mut classes = variant_classes("control", props);
    classes.push(base.to_string());
    classes.push(format!("is-{}", size.as_str()));
    if props.label_floating {
        classes.push("is-floating".to_string());
    }
    if error_text.is_some() {
        classes.push("is-error".to_string());
    }
    let control = format!(
        "<span{}>{}{}</span>",
        attrs(classes, Some(&props.element), None, context),
        floating_label_html(props),
        control_html
    );
    render_field_block(props, help_text, error_text, &control, context)
}

fn render_field_block(
    props: &VariantProps,
    help_text: Option<&str>,
    error_text: Option<&str>,
    body_html: &str,
    context: &ReactiveRenderContext,
) -> String {
    let label = if props.label.is_some() && !props.label_floating {
        format!(
            r#"<span class="field-label">{}</span>"#,
            escape_html(props.label.as_deref().unwrap_or_default())
        )
    } else {
        String::new()
    };
    let help = error_text.or(help_text).map(|value| {
        format!(
            r#"<span class="field-help{}">{}</span>"#,
            if error_text.is_some() { " is-error" } else { "" },
            escape_html(value)
        )
    }).unwrap_or_default();
    format!(
        r#"<div{}>{}{body_html}{}</div>"#,
        attrs(
            vec!["field".to_string()],
            None,
            None,
            context
        ),
        label,
        help
    )
}

fn render_color_values(props: &ColorProps) -> String {
    if !(props.show_hex || props.show_rgb || props.show_cmyk || props.show_oklch) {
        return String::new();
    }
    let mut html = String::from("<span class=\"color-picker-values\">");
    if props.show_hex {
        html.push_str(&format!(
            r#"<code class="color-picker-value-code">hex: {}</code>"#,
            escape_html(&props.value)
        ));
    }
    if props.show_rgb {
        html.push_str(&format!(
            r#"<code class="color-picker-value-code">rgb: {}</code>"#,
            escape_html(&hex_rgb_label(&props.value))
        ));
    }
    if props.show_cmyk {
        html.push_str(&format!(
            r#"<code class="color-picker-value-code">cmyk: {}</code>"#,
            escape_html(&hex_cmyk_label(&props.value))
        ));
    }
    if props.show_oklch {
        html.push_str(r#"<code class="color-picker-value-code">oklch: target-derived</code>"#);
    }
    html.push_str("</span>");
    html
}

fn hex_rgb_label(value: &str) -> String {
    let Some((r, g, b)) = parse_hex_rgb(value) else {
        return "rgb(0, 0, 0)".to_string();
    };
    format!("rgb({r}, {g}, {b})")
}

fn hex_cmyk_label(value: &str) -> String {
    let Some((r, g, b)) = parse_hex_rgb(value) else {
        return "cmyk(0%, 0%, 0%, 100%)".to_string();
    };
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let k = 1.0 - r.max(g).max(b);
    if k >= 1.0 {
        return "cmyk(0%, 0%, 0%, 100%)".to_string();
    }
    let c = ((1.0 - r - k) / (1.0 - k) * 100.0).round() as u8;
    let m = ((1.0 - g - k) / (1.0 - k) * 100.0).round() as u8;
    let y = ((1.0 - b - k) / (1.0 - k) * 100.0).round() as u8;
    let k = (k * 100.0).round() as u8;
    format!("cmyk({c}%, {m}%, {y}%, {k}%)")
}

fn parse_hex_rgb(value: &str) -> Option<(u8, u8, u8)> {
    let hex = value.strip_prefix('#')?;
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        return Some((r, g, b));
    }
    if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
        return Some((r, g, b));
    }
    None
}

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

fn render_avatar_html(
    props: &AvatarProps,
    icon: Option<&SideNavIcon>,
    context: &ReactiveRenderContext,
) -> String {
    let content = if let Some(src) = props.src.as_deref() {
        format!(
            r#"<img class="avatar-image" src="{}" alt="{}">"#,
            escape_attr(src),
            escape_attr(&props.alt)
        )
    } else if let Some(icon) = icon {
        format!(
            r#"<span class="avatar-icon">{}</span>"#,
            render_svg_html(&icon.props, &icon.paths, context)
        )
    } else {
        format!(
            r#"<span class="avatar-name">{}</span>"#,
            escape_html(&avatar_initial(props))
        )
    };
    let status = props
        .status
        .map(|status| {
            format!(
                r#"<span class="avatar-status"><span class="avatar-indicator is-{}"></span></span>"#,
                status.as_str()
            )
        })
        .unwrap_or_default();
    let (tag, tag_attrs, close) = avatar_tags(props, context);
    format!("<{tag}{tag_attrs}>{status}{content}</{close}>")
}

fn avatar_tags(props: &AvatarProps, context: &ReactiveRenderContext) -> (&'static str, String, &'static str) {
    let classes = avatar_classes(props);
    match props.style.navigation.as_ref() {
        Some(action) => (
            "a",
            attrs(
                classes,
                Some(&props.style.element),
                Some(&side_nav_navigation_attrs("avatar", action)),
                context,
            ),
            "a",
        ),
        None if props.style.element.on_click.is_some() => (
            "button",
            attrs(
                classes,
                Some(&props.style.element),
                Some(&format!(
                    r#" type="button" data-dowe-click="{}""#,
                    escape_attr(&context.action_id(
                        props.style.element.on_click.as_deref().expect("onClick")
                    ))
                )),
                context,
            ),
            "button",
        ),
        None => (
            "div",
            attrs(classes, Some(&props.style.element), None, context),
            "div",
        ),
    }
}

fn avatar_initial(props: &AvatarProps) -> String {
    props
        .name
        .as_deref()
        .unwrap_or(&props.alt)
        .chars()
        .next()
        .map(|value| value.to_uppercase().collect::<String>())
        .unwrap_or_else(|| "A".to_string())
}

fn render_badge_html(
    props: &BadgeProps,
    children: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<div{}>",
        attrs(badge_classes(props), Some(&props.style.element), None, context)
    );
    for child in children {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str(&format!(
        r#"<span class="badge-content"><span class="badge-text">{}</span></span></div>"#,
        escape_html(&props.text)
    ));
    html
}

fn render_chip_html(
    props: &ChipProps,
    value: &str,
    start: Option<&SideNavIcon>,
    end: Option<&SideNavIcon>,
    context: &ReactiveRenderContext,
) -> String {
    let start = start
        .map(|icon| render_overlay_icon_html("chip-icon", icon, context))
        .unwrap_or_default();
    let end = end
        .map(|icon| render_overlay_icon_html("chip-icon", icon, context))
        .unwrap_or_default();
    let close = props
        .on_close
        .as_deref()
        .map(|action| {
            format!(
                r#"<button class="chip-close" type="button" aria-label="Close" data-dowe-click="{}">&times;</button>"#,
                escape_attr(&context.action_id(action))
            )
        })
        .unwrap_or_default();
    format!(
        r#"<span{}>{start}<span class="chip-label">{}</span>{end}{close}</span>"#,
        attrs(chip_classes(props), Some(&props.style.element), None, context),
        escape_html(value)
    )
}

fn render_skeleton_html(props: &SkeletonProps, context: &ReactiveRenderContext) -> String {
    format!(
        r#"<div{} aria-hidden="true"></div>"#,
        attrs(skeleton_classes(props), Some(&props.style.element), None, context)
    )
}

fn render_modal_html(
    props: &ModalProps,
    header: &[ViewNode],
    body: &[ViewNode],
    footer: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        r#"<div{} hidden><button class="modal-overlay" type="button" aria-label="Close modal" data-dowe-modal-overlay></button><div{} role="dialog" aria-modal="true">"#,
        attrs(
            modal_panel_classes(props),
            Some(&props.style.element),
            Some(&modal_attrs(props, context)),
            context,
        ),
        class_attr(modal_classes(props))
    );
    if !header.is_empty() {
        html.push_str("<div class=\"modal-header\">");
        for child in header {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div>");
    }
    html.push_str("<div class=\"modal-body\">");
    for child in body {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str("</div>");
    if !footer.is_empty() {
        html.push_str("<div class=\"modal-footer\">");
        for child in footer {
            html.push_str(&render_html_with_context(child, children_html, context));
        }
        html.push_str("</div>");
    }
    if !props.hide_close_button {
        html.push_str(modal_close_html());
    }
    html.push_str("</div></div>");
    html
}

fn modal_attrs(props: &ModalProps, context: &ReactiveRenderContext) -> String {
    let mut attrs = format!(
        r#" data-dowe-modal data-dowe-modal-open="{}" data-dowe-modal-disable-overlay-close="{}""#,
        escape_attr(&context.signal_path(&props.open)),
        props.disable_overlay_close
    );
    if let Some(action) = props.on_close.as_deref() {
        attrs.push_str(&format!(
            r#" data-dowe-modal-on-close="{}""#,
            escape_attr(&context.action_id(action))
        ));
    }
    attrs
}

fn modal_close_html() -> &'static str {
    r#"<button class="modal-close" type="button" aria-label="Close modal" data-dowe-modal-close>&times;</button>"#
}

fn render_alert_dialog_html(props: &AlertDialogProps, context: &ReactiveRenderContext) -> String {
    let modal = alert_dialog_modal_props(props);
    let confirm_action = props
        .on_confirm
        .as_deref()
        .map(|action| {
            format!(
                r#" data-dowe-click="{}""#,
                escape_attr(&context.action_id(action))
            )
        })
        .unwrap_or_default();
    let cancel_action = props
        .on_cancel
        .as_deref()
        .map(|action| {
            format!(
                r#" data-dowe-click="{}""#,
                escape_attr(&context.action_id(action))
            )
        })
        .unwrap_or_default();
    format!(
        r#"<div{} hidden><button class="modal-overlay" type="button" aria-label="Close dialog" data-dowe-modal-overlay></button><div{} role="alertdialog" aria-modal="true"><div class="modal-header"><h3 class="alert-dialog-title">{}</h3></div><div class="modal-body"><p class="alert-dialog-description">{}</p></div><div class="modal-footer"><div class="alert-dialog-actions"><button class="button button-md is-outlined is-muted" type="button" data-dowe-modal-close{}>{}</button><button class="button button-md is-solid is-{}" type="button"{}{}>{}</button></div></div></div></div>"#,
        attrs(
            modal_panel_classes(&modal),
            Some(&modal.style.element),
            Some(&modal_attrs(&modal, context)),
            context,
        ),
        class_attr(modal_classes(&modal)),
        escape_html(&props.title),
        escape_html(&props.description),
        cancel_action,
        escape_html(&props.cancel_text),
        props.style.color.unwrap_or(ColorFamily::Danger).as_str(),
        confirm_action,
        if props.loading { " disabled aria-busy=\"true\"" } else { "" },
        escape_html(&props.confirm_text)
    )
}

fn render_tooltip_html(
    props: &TooltipProps,
    children: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<span{} data-dowe-tooltip>",
        attrs(tooltip_classes(props), Some(&props.style.element), None, context)
    );
    for child in children {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str(&format!(
        r#"<span{} role="tooltip"><span class="tooltip-arrow"></span>{}</span></span>"#,
        class_attr(tooltip_popover_classes(props)),
        escape_html(&props.label)
    ));
    html
}

fn render_toast_html(props: &ToastProps, context: &ReactiveRenderContext) -> String {
    let mut extra = String::from(r#" data-dowe-toast"#);
    if let Some(source) = props.source.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-toast-source="{}""#,
            escape_attr(&context.signal_path(source))
        ));
    }
    let hidden = if props.source.is_some() { " hidden" } else { "" };
    let title = props
        .title
        .as_deref()
        .map(|title| format!(r#"<div class="toast-title">{}</div>"#, escape_html(title)))
        .unwrap_or_default();
    format!(
        r#"<div{}{}><div class="toast-content">{title}<div class="toast-description">{}</div></div><button class="toast-close" type="button" aria-label="Close toast" data-dowe-toast-close>&times;</button></div>"#,
        attrs(toast_classes(props), Some(&props.style.element), Some(&extra), context),
        hidden,
        escape_html(&props.description)
    )
}

fn render_dropdown_html(
    props: &DropdownProps,
    trigger: &[ViewNode],
    header: &[ViewNode],
    entries: &[OverlayEntry],
    footer: &[ViewNode],
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<span{} data-dowe-dropdown>",
        attrs(dropdown_classes(props), Some(&props.style.element), None, context)
    );
    html.push_str("<span class=\"dropdown-trigger\" data-dowe-dropdown-trigger>");
    for child in trigger {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str("</span><div");
    html.push_str(&class_attr(dropdown_popover_classes(props)));
    html.push_str(" role=\"menu\" hidden>");
    for child in header {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str("<div class=\"dropdown-options\">");
    for entry in entries {
        html.push_str(&render_overlay_entry_html("dropdown", entry, context));
    }
    html.push_str("</div>");
    for child in footer {
        html.push_str(&render_html_with_context(child, children_html, context));
    }
    html.push_str("</div></span>");
    html
}

fn render_command_html(
    props: &CommandProps,
    entries: &[CommandEntry],
    context: &ReactiveRenderContext,
) -> String {
    let mut extra = format!(
        r#" data-dowe-command data-dowe-command-shortcut="{}" data-dowe-command-disable-global="{}""#,
        escape_attr(&props.shortcut),
        props.disable_global_shortcut
    );
    if let Some(open) = props.open.as_deref() {
        extra.push_str(&format!(
            r#" data-dowe-command-open="{}""#,
            escape_attr(&context.signal_path(open))
        ));
    }
    let mut html = format!(
        r#"<div{} hidden><button class="modal-overlay" type="button" aria-label="Close command" data-dowe-command-close></button><div{} role="dialog" aria-modal="true"><div class="command-header"><input class="command-input" type="search" placeholder="{}" data-dowe-command-input><span class="command-kbd"><kbd>Esc</kbd><span>{}</span></span></div><div class="command-results" data-dowe-command-results>"#,
        attrs(
            command_panel_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context,
        ),
        class_attr(command_classes(props)),
        escape_attr(&props.placeholder),
        escape_html(&props.close_text)
    );
    for entry in entries {
        html.push_str(&render_command_entry_html(entry, context));
    }
    html.push_str(&format!(
        r#"<div class="command-empty" hidden>{}</div></div>"#,
        escape_html(&props.empty_text)
    ));
    if props.show_footer {
        html.push_str(&format!(
            r#"<div class="command-shortcuts"><span><kbd>↑</kbd><kbd>↓</kbd> {}</span><span><kbd>↵</kbd> {}</span><span><kbd>Ctrl</kbd><kbd>{}</kbd> {}</span></div>"#,
            escape_html(&props.navigate_text),
            escape_html(&props.select_text),
            escape_html(&props.shortcut.to_ascii_uppercase()),
            escape_html(&props.toggle_text)
        ));
    }
    html.push_str("</div></div>");
    html
}

fn render_command_entry_html(entry: &CommandEntry, context: &ReactiveRenderContext) -> String {
    match entry {
        CommandEntry::Item(props) => render_overlay_item_html("command", props, context),
        CommandEntry::Group { label, icon, items } => {
            let icon = icon
                .as_ref()
                .map(|icon| render_overlay_icon_html("command-group-icon", icon, context))
                .unwrap_or_default();
            let mut html = format!(
                r#"<div class="command-group"><div class="command-group-label">{icon}{}</div><div class="command-group-items">"#,
                escape_html(label)
            );
            for item in items {
                html.push_str(&render_overlay_item_html("command", item, context));
            }
            html.push_str("</div></div>");
            html
        }
    }
}

fn render_overlay_entry_html(
    base: &str,
    entry: &OverlayEntry,
    context: &ReactiveRenderContext,
) -> String {
    match entry {
        OverlayEntry::Item(props) => render_overlay_item_html(base, props, context),
        OverlayEntry::Divider => r#"<div class="dropdown-divider" role="separator"></div>"#.to_string(),
    }
}

fn render_overlay_item_html(
    base: &str,
    props: &OverlayItemProps,
    context: &ReactiveRenderContext,
) -> String {
    let icon = props
        .icon
        .as_ref()
        .map(|icon| render_overlay_icon_html(&format!("{base}-item-icon"), icon, context))
        .unwrap_or_default();
    let description = props
        .description
        .as_deref()
        .map(|description| {
            format!(
                r#"<span class="{base}-item-description">{}</span>"#,
                escape_html(description)
            )
        })
        .unwrap_or_default();
    let content = format!(
        r#"{icon}<span class="{base}-item-content"><span class="{base}-item-label">{}</span>{description}</span>"#,
        escape_html(&props.label)
    );
    let attrs = overlay_item_attrs(base, props, context);
    format!(r#"<{}{}>{}</{}>"#, attrs.0, attrs.1, content, attrs.2)
}

fn overlay_item_attrs(
    base: &str,
    props: &OverlayItemProps,
    context: &ReactiveRenderContext,
) -> (&'static str, String, &'static str) {
    let class_name = if props.disabled {
        format!("{base}-item is-disabled")
    } else {
        format!("{base}-item")
    };
    let classes = class_attr(class_name.split_whitespace().map(str::to_string).collect());
    if props.disabled {
        return (
            "div",
            format!(r#"{classes} aria-disabled="true""#),
            "div",
        );
    }
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
        None => ("button", format!(r#"{classes} type="button""#), "button"),
    }
}

fn render_overlay_icon_html(
    class_name: &str,
    icon: &SideNavIcon,
    context: &ReactiveRenderContext,
) -> String {
    format!(
        r#"<span class="{class_name}">{}</span>"#,
        render_svg_html(&icon.props, &icon.paths, context)
    )
}

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
        SideNavItem::Submenu { props, open, items } => {
            let classes = if *open {
                format!("{base}-submenu is-open")
            } else {
                format!("{base}-submenu")
            };
            let mut html = format!(
                r#"<details class="{classes}" data-dowe-{base}-submenu{}><summary class="{base}-entry {base}-trigger" aria-expanded="{}">{}{}<span class="{base}-chevron" aria-hidden="true">›</span></summary><div class="{base}-submenu-content">"#,
                if *open { " open" } else { "" },
                if *open { "true" } else { "false" },
                render_side_nav_icon_html(base, props.icon.as_ref(), context),
                render_side_nav_content_html(base, props)
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
        r#"{{"id":"{}","name":"{}","kind":"assign","target":"{}","source":"{}"}}"#,
        escape_json(&view_action.id),
        escape_json(&view_action.name),
        escape_json(&context.signal_path(&action.target)),
        escape_json(&context.signal_path(&action.source))
    )
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
