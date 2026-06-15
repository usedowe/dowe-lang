fn render_swift_node_in_flow(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    if let Some(expression) = context.node_expression(node) {
        let pad = " ".repeat(indent);
        output.push_str(&format!("{pad}{expression}\n"));
        return;
    }
    if let Some(show) = node_element_props(node).and_then(|props| props.show.as_ref()) {
        let pad = " ".repeat(indent);
        output.push_str(&format!(
            "{pad}if {} {{\n",
            swift_show_condition(show, context)
        ));
        render_swift_node_expression(
            node,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}}}\n"));
    } else {
        render_swift_node_expression(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
    }
}

fn render_swift_node_expression(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    if should_type_erase_swift_node(node) {
        let pad = " ".repeat(indent);
        output.push_str(&format!("{pad}AnyView(\n"));
        render_swift_node_body(
            node,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad})\n"));
    } else {
        render_swift_node_body(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
    }
}

fn should_type_erase_swift_node(node: &ViewNode) -> bool {
    match node {
        ViewNode::Scope { .. } | ViewNode::Children => false,
        ViewNode::Alert { props } if props.visible.is_some() => false,
        _ => true,
    }
}

fn render_swift_node_body(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    match node {
        ViewNode::Scope { .. }
        | ViewNode::Each { .. }
        | ViewNode::Box { .. }
        | ViewNode::Section { .. }
        | ViewNode::Flex { .. }
        | ViewNode::Grid { .. }
        | ViewNode::Card { .. }
        | ViewNode::Children => render_swift_structure_node(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Button { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::Fab { .. }
        | ViewNode::Input { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
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
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. } => render_swift_form_node(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Accordion { .. }
        | ViewNode::Carousel { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. }
        | ViewNode::Table { .. } => render_swift_media_data_node(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::Marquee { .. }
        | ViewNode::TypeWriter { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Collapsible { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. } => render_swift_rich_display_node(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Divider { .. }
        | ViewNode::Title { .. }
        | ViewNode::Text { .. }
        | ViewNode::Alert { .. }
        | ViewNode::Svg { .. } => render_swift_text_svg_alert_node(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::AppBar { .. }
        | ViewNode::Footer { .. }
        | ViewNode::BottomBar { .. }
        | ViewNode::SideNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::NavMenu { .. }
        | ViewNode::Scaffold { .. }
        | ViewNode::Tabs { .. }
        | ViewNode::Drawer { .. } => render_swift_navigation_shell_node(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Avatar { .. }
        | ViewNode::Badge { .. }
        | ViewNode::Chip { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::Modal { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Tooltip { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Dropdown { .. }
        | ViewNode::Command { .. } => render_swift_overlay_node(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
    }
}
