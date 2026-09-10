fn collect_tabs_variant_rules(node: &ViewNode, variants: &mut Vec<(ColorFamily, TabsVariant)>) {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => {
            for child in content.iter().chain(children) {
                collect_tabs_variant_rules(child, variants);
            }
        }
        ViewNode::Scope { children, .. }
        | ViewNode::Each { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Marquee { children, .. }
        | ViewNode::Collapsible { children, .. }
        | ViewNode::Brand { children, .. }
        | ViewNode::Banner { children, .. }
        | ViewNode::Button { children, .. } => {
            for child in children {
                collect_tabs_variant_rules(child, variants);
            }
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_tabs_variant_rules(child, variants);
            }
        }
        ViewNode::Modal {
            header,
            body,
            footer,
            ..
        } => {
            for child in header.iter().chain(body).chain(footer) {
                collect_tabs_variant_rules(child, variants);
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            for child in trigger.iter().chain(header).chain(footer) {
                collect_tabs_variant_rules(child, variants);
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                for child in &item.children {
                    collect_tabs_variant_rules(child, variants);
                }
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                for child in &slide.children {
                    collect_tabs_variant_rules(child, variants);
                }
            }
        }
        ViewNode::Tabs { props, tabs } => {
            let rule = (props.color, props.variant);
            if !variants.contains(&rule) {
                variants.push(rule);
            }
            for tab in tabs {
                for child in &tab.children {
                    collect_tabs_variant_rules(child, variants);
                }
            }
        }
        ViewNode::AppBar {
            top,
            start,
            center,
            end,
            bottom,
            ..
        }
        | ViewNode::Footer {
            top,
            start,
            center,
            end,
            bottom,
            ..
        } => {
            for child in top
                .iter()
                .chain(start)
                .chain(center)
                .chain(end)
                .chain(bottom)
            {
                collect_tabs_variant_rules(child, variants);
            }
        }
        ViewNode::BottomBar { .. } => {}
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                collect_nav_menu_tabs_variant_rules(item, variants);
            }
        }
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
            ..
        } => {
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
                .chain(overlays)
            {
                collect_tabs_variant_rules(child, variants);
            }
        }
        ViewNode::Input { .. }
        | ViewNode::ComboBox { .. }
        | ViewNode::CsvField { .. }
        | ViewNode::DragDrop { .. }
        | ViewNode::Editor { .. }
        | ViewNode::ImageCropper { .. }
        | ViewNode::Password { .. }
        | ViewNode::Phone { .. }
        | ViewNode::Pin { .. }
        | ViewNode::Textarea { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::SelectTheme { .. }
        | ViewNode::Fab { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Select { .. }
        | ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Camera { .. }
        | ViewNode::Microphone { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Iframe { .. }
        | ViewNode::Device { .. }
        | ViewNode::Canvas { .. }
        | ViewNode::Diagram { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. }
        | ViewNode::Table { .. }
        | ViewNode::Tree { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Alert { .. }
        | ViewNode::Avatar { .. }
        | ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. }
        | ViewNode::Chip { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Command { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::SideNav { .. }
        | ViewNode::RailNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::Svg { .. }
        | ViewNode::Title { .. }
        | ViewNode::Text { .. }
        | ViewNode::TypeWriter { .. }
        | ViewNode::Children => {}
    }
}

fn collect_nav_menu_tabs_variant_rules(
    item: &NavMenuItem,
    variants: &mut Vec<(ColorFamily, TabsVariant)>,
) {
    if let NavMenuItem::Megamenu { content, .. } = item {
        for child in content {
            collect_tabs_variant_rules(child, variants);
        }
    }
}
