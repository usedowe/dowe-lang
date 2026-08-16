fn css_for_tree(tree: &ViewNode) -> String {
    let mut classes = BTreeSet::new();
    collect_classes(tree, &mut classes);
    let mut variants = Vec::new();
    collect_variant_rules(tree, &mut variants);
    let mut tabs_variants = Vec::new();
    collect_tabs_variant_rules(tree, &mut tabs_variants);
    let mut custom_rules = Vec::new();
    collect_custom_rules(tree, &mut custom_rules);
    let mut rules = Vec::new();

    for class_name in &classes {
        let mut fragment = String::new();
        append_class_css(&mut fragment, class_name);
        if !fragment.is_empty() {
            push_css_rule_fragment(&mut rules, fragment);
        }
    }

    for (base, family, variant) in variants {
        let mut fragment = String::new();
        append_single_variant_css(&mut fragment, base, family, variant);
        push_css_rule_fragment(&mut rules, fragment);
    }

    for (family, variant) in tabs_variants {
        let mut fragment = String::new();
        append_tabs_variant_css(&mut fragment, family, variant);
        push_css_rule_fragment(&mut rules, fragment);
    }

    for rule in custom_rules {
        push_css_rule_fragment(&mut rules, rule);
    }

    let mut css = String::new();
    append_css_rule_fragments(&mut css, &mut rules);
    css
}

fn collect_classes(node: &ViewNode, classes: &mut BTreeSet<String>) {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => {
            for child in content.iter().chain(children) {
                collect_classes(child, classes);
            }
        }
        ViewNode::Scope { .. }
        | ViewNode::Each { .. }
        | ViewNode::Box { .. }
        | ViewNode::Section { .. }
        | ViewNode::Flex { .. }
        | ViewNode::Grid { .. }
        | ViewNode::Card { .. }
        | ViewNode::Button { .. }
        | ViewNode::Brand { .. }
        | ViewNode::Banner { .. }
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
        | ViewNode::Camera { .. }
        | ViewNode::Microphone { .. }
        | ViewNode::Accordion { .. }
        | ViewNode::Carousel { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::SelectTheme { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Input { .. }
        | ViewNode::Select { .. }
        | ViewNode::ComboBox { .. }
        | ViewNode::CsvField { .. }
        | ViewNode::DragDrop { .. }
        | ViewNode::Editor { .. }
        | ViewNode::ImageCropper { .. }
        | ViewNode::Password { .. }
        | ViewNode::Phone { .. }
        | ViewNode::Pin { .. }
        | ViewNode::Textarea { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Iframe { .. }
        | ViewNode::Device { .. }
        | ViewNode::Canvas { .. }
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
        | ViewNode::RailNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::NavMenu { .. }
        | ViewNode::Scaffold { .. }
        | ViewNode::Tabs { .. }
        | ViewNode::Drawer { .. }
        | ViewNode::Children => collect_navigation_node_classes(node, classes),
    }
}
