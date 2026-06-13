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
        ViewNode::ToggleTheme { props } => render_theme_toggle_html(props, context),
        ViewNode::Fab { props, actions } => render_fab_html(props, actions, context),
        ViewNode::Input { props } => render_input_html(props, context),
        ViewNode::Slider { props } => render_slider_html(props, context),
        ViewNode::Dropzone { props } => render_dropzone_html(props, context),
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
        ViewNode::AvatarGroup { props, items } => render_avatar_group_html(props, items, context),
        ViewNode::ChatBox { props } => render_chat_box_html(props, context),
        ViewNode::Empty { props } => render_empty_html(props, context),
        ViewNode::Marquee { props, children } => {
            render_marquee_html(props, children, children_html, context)
        }
        ViewNode::TypeWriter { props, items } => render_type_writer_html(props, items, context),
        ViewNode::RichText { props, marks } => render_rich_text_html(props, marks, context),
        ViewNode::Record { props } => render_record_html(props, context),
        ViewNode::ToggleGroup { props, items } => render_toggle_group_html(props, items, context),
        ViewNode::Collapsible { props, children } => {
            render_collapsible_html(props, children, children_html, context)
        }
        ViewNode::Countdown { props } => render_countdown_html(props, context),
        ViewNode::Map {
            props,
            markers,
            waypoints,
        } => render_map_html(props, markers, waypoints, context),
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
        } => render_dropdown_html(
            props,
            trigger,
            header,
            entries,
            footer,
            children_html,
            context,
        ),
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
        ViewNode::SideNav { props, items } => {
            render_side_nav_html("sidenav", props, items, context)
        }
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
