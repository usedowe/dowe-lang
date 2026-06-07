pub fn collect_route_font_families(routes: &[ViewRoute]) -> BTreeSet<FontFamily> {
    let mut fonts = BTreeSet::new();
    for route in routes {
        collect_node_font_families(&route.layout_tree, &mut fonts);
        collect_node_font_families(&route.page_tree, &mut fonts);
    }
    fonts
}

pub fn collect_node_font_families(node: &ViewNode, fonts: &mut BTreeSet<FontFamily>) {
    match node {
        ViewNode::Scope { children, .. } | ViewNode::Each { children, .. } => {
            for child in children {
                collect_node_font_families(child, fonts);
            }
        }
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => {
            collect_style_font_families(props, fonts);
            for child in children {
                collect_node_font_families(child, fonts);
            }
        }
        ViewNode::Flex { props, children } => {
            collect_style_font_families(&props.style, fonts);
            for child in children {
                collect_node_font_families(child, fonts);
            }
        }
        ViewNode::Grid { props, children } => {
            collect_style_font_families(&props.style, fonts);
            for child in children {
                collect_node_font_families(child, fonts);
            }
        }
        ViewNode::Card { props, children } | ViewNode::Button { props, children } => {
            collect_style_font_families(&props.style, fonts);
            for child in children {
                collect_node_font_families(child, fonts);
            }
        }
        ViewNode::Tabs { props, tabs } => {
            collect_style_font_families(&props.style, fonts);
            for tab in tabs {
                for child in &tab.children {
                    collect_node_font_families(child, fonts);
                }
            }
        }
        ViewNode::NavMenu { props, items } => {
            collect_style_font_families(&props.style.style, fonts);
            for item in items {
                collect_nav_menu_font_families(item, fonts);
            }
        }
        ViewNode::Drawer { props, children } => {
            collect_style_font_families(&props.style.style, fonts);
            for child in children {
                collect_node_font_families(child, fonts);
            }
        }
        ViewNode::Avatar { props, .. } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Badge { props, children } => {
            collect_style_font_families(&props.style.style, fonts);
            for child in children {
                collect_node_font_families(child, fonts);
            }
        }
        ViewNode::Tooltip { props, children } => {
            collect_style_font_families(&props.style.style, fonts);
            for child in children {
                collect_node_font_families(child, fonts);
            }
        }
        ViewNode::Chip { props, .. } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Skeleton { props } => collect_style_font_families(&props.style, fonts),
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            collect_style_font_families(&props.style.style, fonts);
            for child in header.iter().chain(body).chain(footer) {
                collect_node_font_families(child, fonts);
            }
        }
        ViewNode::AlertDialog { props } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Toast { props } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            footer,
            ..
        } => {
            collect_style_font_families(&props.style.style, fonts);
            for child in trigger.iter().chain(header).chain(footer) {
                collect_node_font_families(child, fonts);
            }
        }
        ViewNode::Command { props, .. } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Audio { props } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Image { props } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Accordion { props, items } => {
            collect_style_font_families(&props.style.style, fonts);
            for item in items {
                for child in &item.children {
                    collect_node_font_families(child, fonts);
                }
            }
        }
        ViewNode::Carousel { props, slides } => {
            collect_style_font_families(&props.style.style, fonts);
            for slide in slides {
                for child in &slide.children {
                    collect_node_font_families(child, fonts);
                }
            }
        }
        ViewNode::Checkbox { props } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Color { props } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Date { props } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::DateRange { props } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::RadioGroup { props, .. } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Toggle { props } => collect_style_font_families(&props.style.style, fonts),
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
            collect_style_font_families(&props.style.style, fonts);
            for child in start.iter().chain(center).chain(end) {
                collect_node_font_families(child, fonts);
            }
        }
        ViewNode::SideNav { props, .. } | ViewNode::Sidebar { props, .. } => {
            collect_style_font_families(&props.style.style, fonts);
        }
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
        } => {
            collect_style_font_families(&props.style, fonts);
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
            {
                collect_node_font_families(child, fonts);
            }
        }
        ViewNode::Input { props } | ViewNode::Select { props, .. } => {
            collect_style_font_families(&props.style, fonts)
        }
        ViewNode::Code { props } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Video { props } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Candlestick { props } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Table { props } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Divider { props } => collect_style_font_families(&props.style, fonts),
        ViewNode::Alert { props } => collect_style_font_families(&props.style.style, fonts),
        ViewNode::Svg { .. } => {}
        ViewNode::Title { props, .. } | ViewNode::Text { props, .. } => {
            collect_style_font_families(&props.style, fonts);
        }
        ViewNode::Children => {}
    }
}

fn collect_nav_menu_font_families(item: &NavMenuItem, fonts: &mut BTreeSet<FontFamily>) {
    match item {
        NavMenuItem::Megamenu { content, .. } => {
            for child in content {
                collect_node_font_families(child, fonts);
            }
        }
        NavMenuItem::Item(_) | NavMenuItem::Submenu { .. } => {}
    }
}

fn collect_style_font_families(props: &StyleProps, fonts: &mut BTreeSet<FontFamily>) {
    if let Some(value) = props.font.as_ref() {
        for entry in &value.entries {
            fonts.insert(entry.value);
        }
    }
}
