fn collect_custom_rules(node: &ViewNode, rules: &mut Vec<String>) {
    match node {
        ViewNode::Scope { children, .. } | ViewNode::Each { children, .. } => {
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => {
            collect_style_custom_rules(props, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Flex { props, children } => {
            collect_gap_custom_rules(props.gap.as_ref(), rules);
            collect_style_custom_rules(&props.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Grid { props, children } => {
            collect_grid_custom_rules(props, rules);
            collect_style_custom_rules(&props.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Card { props, children } => {
            collect_style_custom_rules(&props.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => {
            collect_style_custom_rules(&props.style.style, rules);
            for child in header.iter().chain(body).chain(footer) {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Avatar { props, .. } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::AvatarGroup { props, .. } => {
            collect_style_custom_rules(&props.style.style, rules)
        }
        ViewNode::ChatBox { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Empty { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Marquee { props, children } => {
            collect_style_custom_rules(&props.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::TypeWriter { props, .. } => collect_style_custom_rules(&props.style, rules),
        ViewNode::RichText { props, .. } => collect_style_custom_rules(&props.style, rules),
        ViewNode::Record { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::ToggleGroup { props, .. } => {
            collect_style_custom_rules(&props.style.style, rules)
        }
        ViewNode::Collapsible { props, children } => {
            collect_style_custom_rules(&props.style.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Countdown { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Map { props, .. } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Badge { props, children } => {
            collect_style_custom_rules(&props.style.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Chip { props, .. } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Skeleton { props } => collect_style_custom_rules(&props.style, rules),
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            collect_style_custom_rules(&props.style.style, rules);
            for child in header.iter().chain(body).chain(footer) {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::AlertDialog { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Tooltip { props, children } => {
            collect_style_custom_rules(&props.style.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Toast { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            footer,
            ..
        } => {
            collect_style_custom_rules(&props.style.style, rules);
            for child in trigger.iter().chain(header).chain(footer) {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Command { props, .. } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Audio { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Image { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Accordion { props, items } => {
            collect_style_custom_rules(&props.style.style, rules);
            for item in items {
                for child in &item.children {
                    collect_custom_rules(child, rules);
                }
            }
        }
        ViewNode::Carousel { props, slides } => {
            collect_style_custom_rules(&props.style.style, rules);
            for slide in slides {
                for child in &slide.children {
                    collect_custom_rules(child, rules);
                }
            }
        }
        ViewNode::Checkbox { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Color { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Date { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::DateRange { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::RadioGroup { props, .. } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Toggle { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::ToggleTheme { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Fab { props, .. } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Slider { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Dropzone { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Button { props, children } => {
            collect_style_custom_rules(&props.style, rules);
            for child in children {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Input { props } | ViewNode::Select { props, .. } => {
            collect_style_custom_rules(&props.style, rules)
        }
        ViewNode::ComboBox { props, .. } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::CsvField { props, .. } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::DragDrop { props, .. } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Editor { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::ImageCropper { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::PasswordField { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::PhoneField { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::PinField { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Textarea { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Code { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Video { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Candlestick { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::ArcChart { props } => collect_style_custom_rules(&props.common.style.style, rules),
        ViewNode::AreaChart { props } => {
            collect_style_custom_rules(&props.common.style.style, rules)
        }
        ViewNode::BarChart { props } => collect_style_custom_rules(&props.common.style.style, rules),
        ViewNode::LineChart { props } => {
            collect_style_custom_rules(&props.common.style.style, rules)
        }
        ViewNode::PieChart { props } => collect_style_custom_rules(&props.common.style.style, rules),
        ViewNode::Table { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::Divider { props } => {
            collect_divider_custom_rules(props, rules);
            collect_style_custom_rules(&props.style, rules);
        }
        ViewNode::Alert { props } => collect_style_custom_rules(&props.style.style, rules),
        ViewNode::AppBar {
            props,
            start,
            center,
            end,
        }
        | ViewNode::Footer {
            props,
            start,
            center,
            end,
        }
        | ViewNode::BottomBar {
            props,
            start,
            center,
            end,
        } => {
            collect_style_custom_rules(&props.style.style, rules);
            for child in start.iter().chain(center).chain(end) {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::SideNav { props, .. } => {
            collect_style_custom_rules(&props.style.style, rules);
        }
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => {
            collect_style_custom_rules(&props.style.style, rules);
            for child in header.iter().chain(body).chain(footer) {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::NavMenu { props, items } => {
            collect_style_custom_rules(&props.style.style, rules);
            for item in items {
                collect_nav_menu_custom_rules(item, rules);
            }
        }
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
        } => {
            collect_style_custom_rules(&props.style, rules);
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
            {
                collect_custom_rules(child, rules);
            }
        }
        ViewNode::Tabs { props, tabs } => {
            collect_style_custom_rules(&props.style, rules);
            for tab in tabs {
                for child in &tab.children {
                    collect_custom_rules(child, rules);
                }
            }
        }
        ViewNode::Svg { props, .. } => collect_style_custom_rules(&props.style, rules),
        ViewNode::Title { props, .. } | ViewNode::Text { props, .. } => {
            collect_style_custom_rules(&props.style, rules)
        }
        ViewNode::Children => {}
    }
}

fn collect_nav_menu_custom_rules(item: &NavMenuItem, rules: &mut Vec<String>) {
    if let NavMenuItem::Megamenu { content, .. } = item {
        for child in content {
            collect_custom_rules(child, rules);
        }
    }
}

fn collect_divider_custom_rules(props: &DividerProps, rules: &mut Vec<String>) {
    let token = props.color.as_str();
    let rule = format!(
        ".divider.is-{token}{{background-color:var(--dowe-{token});color:var(--dowe-{token});}}"
    );
    if !rules.contains(&rule) {
        rules.push(rule);
    }
}

fn collect_style_custom_rules(props: &StyleProps, rules: &mut Vec<String>) {
    if let Some(cover) = props.cover.as_ref() {
        for entry in &cover.entries {
            push_custom_rule(
                rules,
                entry.breakpoint,
                &format!(
                    ".{}{{background-image:url(\"{}\");}}",
                    css_class_name(&responsive_custom_class(
                        entry.breakpoint,
                        &format!("cover-{}", cover_suffix(&entry.value))
                    )),
                    escape_css_string(&entry.value.0)
                ),
            );
        }
    }
    if let Some(overlay) = props.overlay.as_ref() {
        for entry in &overlay.entries {
            push_custom_rule(
                rules,
                entry.breakpoint,
                &format!(
                    ".{}::before{{background:{};}}",
                    css_class_name(&responsive_custom_class(
                        entry.breakpoint,
                        &format!("overlay-{}", overlay_suffix(&entry.value))
                    )),
                    overlay_css(&entry.value)
                ),
            );
        }
    }
}

fn collect_gap_custom_rules(value: Option<&ResponsiveValue<GapValue>>, rules: &mut Vec<String>) {
    let Some(value) = value else {
        return;
    };
    for entry in &value.entries {
        if let GapValue::Pair(row, column) = &entry.value {
            push_custom_rule(
                rules,
                entry.breakpoint,
                &format!(
                    ".{}{{row-gap:{};column-gap:{};}}",
                    css_class_name(&responsive_custom_class(
                        entry.breakpoint,
                        &format!("gap-{}", entry.value.class_suffix())
                    )),
                    gap_size_css(row),
                    gap_size_css(column)
                ),
            );
        }
    }
}

fn collect_grid_custom_rules(props: &GridProps, rules: &mut Vec<String>) {
    collect_gap_custom_rules(props.gap.as_ref(), rules);
    collect_grid_track_rules(
        "grid-cols",
        "grid-template-columns",
        props.columns.as_ref(),
        rules,
    );
    collect_grid_track_rules(
        "grid-rows",
        "grid-template-rows",
        props.rows.as_ref(),
        rules,
    );
}

fn collect_grid_track_rules(
    prefix: &str,
    property: &str,
    value: Option<&ResponsiveValue<GridTracks>>,
    rules: &mut Vec<String>,
) {
    let Some(value) = value else {
        return;
    };
    for entry in &value.entries {
        if let GridTracks::Template(template) = &entry.value {
            push_custom_rule(
                rules,
                entry.breakpoint,
                &format!(
                    ".{}{{{property}:{};}}",
                    css_class_name(&responsive_custom_class(
                        entry.breakpoint,
                        &format!("{prefix}-{}", entry.value.class_suffix())
                    )),
                    template
                ),
            );
        }
    }
}
