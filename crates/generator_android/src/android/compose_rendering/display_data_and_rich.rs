#[allow(unused_variables)]
fn render_compose_display_tree(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Tree { props } = node else { return; };
    let pad = " ".repeat(indent);
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                table_variant_content(&props.style)
            } else {
                "null"
            };
            output.push_str(&format!(
                "{pad}DoweTree(state = state, dataPath = {}, bindPath = {}, defaultOpen = {}, emptyLabel = {}, ariaLabel = {}, onSelect = {}, modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {border}, radius = {})\n",
                compose_string_literal(&context.signal_path(&props.data)),
                props.bind.as_deref().map(|path| compose_string_literal(&context.signal_path(path))).unwrap_or_else(|| "null".to_string()),
                props.default_open,
                compose_string_literal(&props.empty_label),
                compose_string_literal(&props.aria_label),
                props.on_select.as_deref().and_then(|value| context.action_id(value)).map(|value| compose_string_literal(&value)).unwrap_or_else(|| "null".to_string()),
                modifier_for_style(&props.style.style),
                table_variant_container(&props.style),
                table_variant_content(&props.style),
                compose_card_radius(&props.style.style),
            ));
}

#[allow(unused_variables)]
fn render_compose_display_avatar_group(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::AvatarGroup { props, items } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_avatar_group(props, items, indent, output, context)
}

#[allow(unused_variables)]
fn render_compose_display_chat_box(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::ChatBox { props } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_chat_box(props, indent, output, context);
}

#[allow(unused_variables)]
fn render_compose_display_empty(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Empty { props } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_empty(props, indent, output, context);
}

#[allow(unused_variables)]
fn render_compose_display_marquee(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Marquee { props, children } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_marquee(
                props,
                children,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
}

#[allow(unused_variables)]
fn render_compose_display_type_writer(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::TypeWriter { props, items } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_type_writer(props, items, indent, output)
}

#[allow(unused_variables)]
fn render_compose_display_rich_text(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::RichText { props, marks } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_rich_text(props, marks, indent, output, inherited_font, default_family);
}

#[allow(unused_variables)]
fn render_compose_display_record(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Record { props } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_record(props, indent, output, context);
}

#[allow(unused_variables)]
fn render_compose_display_toggle_group(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::ToggleGroup { props, items } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_toggle_group(props, items, indent, output, context)
}

#[allow(unused_variables)]
fn render_compose_display_collapsible(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Collapsible { props, children } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_collapsible(
                props,
                children,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
}

