fn dev_show_condition(show: &VisibilityCondition, context: &ComposeReactiveContext) -> String {
    match show {
        VisibilityCondition::Static(value) => {
            format!("doweShow({})", dev_bool_value(value))
        }
        VisibilityCondition::Signal(path) => {
            let item = context.item_value(path).unwrap_or("null");
            let path = context
                .item_path(path)
                .unwrap_or_else(|| context.signal_path(path));
            format!("doweBool(\"{}\", {item})", escape_java(&path))
        }
    }
}

fn dev_scheme_color(props: &VariantProps) -> &'static str {
    java_color(family_color(props.color.unwrap_or(ColorFamily::Primary)))
}

fn dev_bound_text(
    props: &VariantProps,
    fallback: &str,
    context: &ComposeReactiveContext,
) -> String {
    dev_optional_bound_text(props.element.bind.as_deref(), fallback, context)
}

fn dev_optional_bound_text(
    path: Option<&str>,
    fallback: &str,
    context: &ComposeReactiveContext,
) -> String {
    path.map(|path| {
        format!(
            "doweTextValue(\"{}\", null)",
            escape_java(&context.signal_path(path))
        )
    })
    .unwrap_or_else(|| format!("\"{}\"", escape_java(fallback)))
}

fn dev_bound_bool(
    props: &VariantProps,
    fallback: bool,
    context: &ComposeReactiveContext,
) -> String {
    props
        .element
        .bind
        .as_deref()
        .map(|path| format!("doweBool(\"{}\")", escape_java(&context.signal_path(path))))
        .unwrap_or_else(|| fallback.to_string())
}

fn render_dev_android_variant_label(
    value: &str,
    props: &VariantProps,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let view = next_dev_view(counter);
    output.push_str(&format!(
        "        TextView {view} = doweText({}, {}, 14f, 500, 0f, 1.2f, {});\n        {view}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_UI));\n",
        dev_text_expression(value, None, context),
        dev_variant_content(props),
        dev_font_value(props.style.font.as_ref().or(inherited_font)),
        dev_variant_container(props)
    ));
    apply_dev_android_style(&props.style, &view, false, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}

fn render_compose_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    default_family: FontFamily,
) {
    render_compose_node_in_flow(
        node,
        indent,
        output,
        ComposeFlow::Block,
        None,
        default_family,
        &ComposeReactiveContext::default(),
    );
}
