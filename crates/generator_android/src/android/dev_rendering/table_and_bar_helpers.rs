fn render_dev_android_table(props: &TableProps, view: &str, data_path: &str, output: &mut String) {
    let fields = java_string_array(props.columns.iter().map(|column| column.field.as_str()));
    let labels = java_string_array(props.columns.iter().map(|column| column.label.as_str()));
    let alignments = java_int_array(
        props
            .columns
            .iter()
            .map(|column| dev_table_align(column.align).to_string()),
    );
    let widths =
        java_nullable_string_array(props.columns.iter().map(|column| column.width.as_deref()));
    output.push_str(&format!(
        "        LinearLayout {view} = doweTable(\"{}\", {fields}, {labels}, {alignments}, {widths}, {}, {}, {}, {}, \"{}\", \"{}\", {}, {}, {});\n",
        escape_java(data_path),
        dev_table_size(props.size),
        props.striped,
        props.bordered,
        props.dividers,
        escape_java(&props.empty_title),
        escape_java(&props.empty_description),
        dev_card_variant_container(&props.style),
        dev_card_variant_content(&props.style),
        dev_card_border(&props.style)
    ));
}

fn dev_table_size(value: TableSize) -> &'static str {
    match value {
        TableSize::Sm => "0",
        TableSize::Md => "1",
        TableSize::Lg => "2",
    }
}

fn dev_table_align(value: TableColumnAlign) -> &'static str {
    match value {
        TableColumnAlign::Start => "Gravity.START",
        TableColumnAlign::Center => "Gravity.CENTER",
        TableColumnAlign::End => "Gravity.END",
    }
}

fn render_dev_android_bar_region(
    children: &[ViewNode],
    parent: &str,
    gravity: &str,
    weighted: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    if children.is_empty() {
        return;
    }
    let view = next_dev_view(counter);
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL | {gravity});\n        {view}.setPadding(doweDp(8), doweDp(8), doweDp(8), doweDp(8));\n"
    ));
    if weighted {
        output.push_str(&format!(
            "        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n"
        ));
    } else {
        output.push_str(&format!(
            "        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
        ));
    }
    output.push_str(&format!("        {parent}.addView({view});\n"));
    for child in children {
        render_dev_android_node(
            child,
            &view,
            Some("8"),
            true,
            counter,
            output,
            inherited_font,
            inherited_color.clone(),
            context,
            children_method,
        );
    }
}

fn render_dev_android_bar_spacer(parent: &str, counter: &mut usize, output: &mut String) {
    let view = next_dev_view(counter);
    output.push_str(&format!(
        "        View {view} = new View(this);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, 0, 1f));\n        {parent}.addView({view});\n"
    ));
}
