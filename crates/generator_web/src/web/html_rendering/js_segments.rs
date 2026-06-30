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
        ViewNode::ToggleTheme { props } => {
            push_literal(segments, &render_theme_toggle_html(props, context));
        }
        ViewNode::Fab { props, actions } => {
            push_literal(segments, &render_fab_html(props, actions, context));
        }
        ViewNode::Input { props } => {
            push_literal(segments, &render_input_html(props, context));
        }
        ViewNode::Slider { props } => {
            push_literal(segments, &render_slider_html(props, context));
        }
        ViewNode::Dropzone { props } => {
            push_literal(segments, &render_dropzone_html(props, context));
        }
        ViewNode::Select { props, options } => {
            push_literal(segments, &render_select_html(props, options, context));
        }
        ViewNode::ComboBox { props, options } => {
            push_literal(segments, &render_combo_box_html(props, options, context));
        }
        ViewNode::CsvField { props, columns } => {
            push_literal(segments, &render_csv_field_html(props, columns, context));
        }
        ViewNode::DragDrop {
            props,
            items,
            groups,
        } => {
            push_literal(
                segments,
                &render_drag_drop_html(props, items, groups, context),
            );
        }
        ViewNode::Editor { props } => {
            push_literal(segments, &render_editor_html(props, context));
        }
        ViewNode::ImageCropper { props } => {
            push_literal(segments, &render_image_cropper_html(props, context));
        }
        ViewNode::PasswordField { props } => {
            push_literal(segments, &render_password_field_html(props, context));
        }
        ViewNode::PhoneField { props } => {
            push_literal(segments, &render_phone_field_html(props, context));
        }
        ViewNode::PinField { props } => {
            push_literal(segments, &render_pin_field_html(props, context));
        }
        ViewNode::Textarea { props } => {
            push_literal(segments, &render_textarea_html(props, context));
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
        ViewNode::ArcChart { props } => {
            push_literal(segments, &render_arc_chart_html(props, context));
        }
        ViewNode::AreaChart { props } => {
            push_literal(segments, &render_area_chart_html(props, context));
        }
        ViewNode::BarChart { props } => {
            push_literal(segments, &render_bar_chart_html(props, context));
        }
        ViewNode::LineChart { props } => {
            push_literal(segments, &render_line_chart_html(props, context));
        }
        ViewNode::PieChart { props } => {
            push_literal(segments, &render_pie_chart_html(props, context));
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
            push_literal(
                segments,
                &render_side_nav_html("sidenav", props, items, context),
            );
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
                    attrs(sidebar_classes(props), Some(&props.style.element), None, context)
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
        } => {
            collect_scaffold_js_segments(
                props, app_bar, start, main, end, bottom_bar, segments, context,
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
            if !props.hide_close_button {
                push_literal(segments, drawer_close_html());
            }
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
