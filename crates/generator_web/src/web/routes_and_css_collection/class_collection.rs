fn css_for_tree(tree: &ViewNode) -> String {
    let mut classes = BTreeSet::new();
    collect_classes(tree, &mut classes);
    let mut variants = Vec::new();
    collect_variant_rules(tree, &mut variants);
    let mut tabs_variants = Vec::new();
    collect_tabs_variant_rules(tree, &mut tabs_variants);
    let mut custom_rules = Vec::new();
    collect_custom_rules(tree, &mut custom_rules);
    let mut css = String::new();

    for class_name in &classes {
        append_class_css(&mut css, class_name);
    }

    for (base, family, variant) in variants {
        append_single_variant_css(&mut css, base, family, variant);
    }

    for (family, variant) in tabs_variants {
        append_tabs_variant_css(&mut css, family, variant);
    }

    for rule in custom_rules {
        css.push_str(&rule);
    }

    css
}

fn collect_classes(node: &ViewNode, classes: &mut BTreeSet<String>) {
    match node {
        ViewNode::Scope { .. }
        | ViewNode::Each { .. }
        | ViewNode::Box { .. }
        | ViewNode::Section { .. }
        | ViewNode::Flex { .. }
        | ViewNode::Grid { .. }
        | ViewNode::Card { .. }
        | ViewNode::Button { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::Fab { .. } => collect_layout_node_classes(node, classes),
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
        | ViewNode::Skeleton { .. } => collect_display_node_classes(node, classes),
        ViewNode::Modal { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Tooltip { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Dropdown { .. }
        | ViewNode::Command { .. } => collect_overlay_node_classes(node, classes),
        ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Accordion { .. }
        | ViewNode::Carousel { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Input { .. }
        | ViewNode::Select { .. }
        | ViewNode::ComboBox { .. }
        | ViewNode::CsvField { .. }
        | ViewNode::DragDrop { .. }
        | ViewNode::Editor { .. }
        | ViewNode::ImageCropper { .. }
        | ViewNode::PasswordField { .. }
        | ViewNode::PhoneField { .. }
        | ViewNode::PinField { .. }
        | ViewNode::Textarea { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. }
        | ViewNode::Table { .. }
        | ViewNode::Divider { .. }
        | ViewNode::Alert { .. }
        | ViewNode::Svg { .. }
        | ViewNode::Title { .. }
        | ViewNode::Text { .. } => collect_media_form_node_classes(node, classes),
        ViewNode::AppBar { .. }
        | ViewNode::Footer { .. }
        | ViewNode::BottomBar { .. }
        | ViewNode::SideNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::NavMenu { .. }
        | ViewNode::Scaffold { .. }
        | ViewNode::Tabs { .. }
        | ViewNode::Drawer { .. }
        | ViewNode::Children => collect_navigation_node_classes(node, classes),
    }
}
