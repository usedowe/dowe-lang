fn render_dev_android_flow_node(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    match node {
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => {
            let context = context.with_scope(signals, actions);
            for child in children {
                render_dev_android_node(
                    child,
                    parent,
                    parent_gap,
                    parent_horizontal,
                    counter,
                    output,
                    inherited_font,
                    inherited_color.clone(),
                    &context,
                    children_method,
                );
            }
        }
        ViewNode::Each {
            item,
            collection,
            children,
            ..
        } => {
            let row = format!("row{}", *counter);
            *counter += 1;
            output.push_str(&format!(
                "        for (Map<String, Object> {row} : doweRows(\"{}\")) {{\n",
                escape_java(&context.signal_path(collection))
            ));
            let context = context.with_item(item, row);
            for child in children {
                render_dev_android_node(
                    child,
                    parent,
                    parent_gap,
                    parent_horizontal,
                    counter,
                    output,
                    inherited_font,
                    inherited_color.clone(),
                    &context,
                    children_method,
                );
            }
            output.push_str("        }\n");
        }
        ViewNode::Box { props, children } => {
            let current_font = props.font.as_ref().or(inherited_font);
            let current_color = dev_inherited_color(props, inherited_color.as_deref());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            apply_dev_android_style(props, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    current_color.clone(),
                    context,
                    children_method,
                );
            }
        }
        ViewNode::Section { props, children } => {
            let current_font = props.font.as_ref().or(inherited_font);
            let current_color = dev_inherited_color(props, inherited_color.as_deref());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            apply_dev_android_style(props, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    current_color.clone(),
                    context,
                    children_method,
                );
            }
        }
        ViewNode::Flex { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let current_color = dev_inherited_color(&props.style, inherited_color.as_deref());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        DoweFlexLayout {view} = doweFlex({}, {}, {});\n",
                dev_flex_justify(props.justify.as_ref()),
                dev_flex_align(props.align.as_ref()),
                dev_optional_gap(props.gap.as_ref(), true).unwrap_or_else(|| "null".to_string())
            ));
            apply_dev_android_style(&props.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    true,
                    counter,
                    output,
                    current_font,
                    current_color.clone(),
                    context,
                    children_method,
                );
            }
        }
        ViewNode::Grid { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let current_color = dev_inherited_color(&props.style, inherited_color.as_deref());
            let view = next_dev_view(counter);
            let columns = dev_grid_columns(props.columns.as_ref());
            let row_gap =
                dev_optional_gap(props.gap.as_ref(), false).unwrap_or_else(|| "null".to_string());
            let column_gap =
                dev_optional_gap(props.gap.as_ref(), true).unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        DoweGridLayout {view} = doweGrid({columns}, {row_gap}, {column_gap});\n"
            ));
            apply_dev_android_style(&props.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    current_color.clone(),
                    context,
                    children_method,
                );
            }
        }
        ViewNode::Card { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let current_color = Some(dev_card_variant_content(props).to_string());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweCard({}, {});\n",
                dev_card_variant_container(props),
                dev_card_border(props)
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    current_color.clone(),
                    context,
                    children_method,
                );
            }
        }
        ViewNode::Button { props, children } => {
            let view = next_dev_view(counter);
            let text = collect_joined_text(children);
            output.push_str(&format!(
                            "        Button {view} = new Button(this);\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        {view}.setText(\"{}\");\n        {view}.setAllCaps(false);\n        {view}.setTypeface(Typeface.create({}, android.graphics.Typeface.NORMAL));\n        {view}.setTextSize({});\n        {view}.setIncludeFontPadding(false);\n        {view}.setGravity(Gravity.CENTER);\n        {view}.setMinWidth(0);\n        {view}.setMinimumWidth(0);\n        {view}.setMinHeight(0);\n        {view}.setMinimumHeight(0);\n        {view}.setTextColor({});\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_UI));\n",
                            escape_java(&text),
                            dev_font_value(props.style.font.as_ref().or(inherited_font)),
                            dev_text_size_expr(false, INPUT_TEXT_SIZE),
                            dev_variant_content(props),
                            dev_variant_container(props)
                        ));
            let action = props
                .element
                .on_click
                .as_deref()
                .and_then(|name| context.action_id(name))
                .map(|id| {
                    let item = context.active_item().unwrap_or("null");
                    format!("doweRunAction(\"{}\", {item})", escape_java(id))
                })
                .or_else(|| dev_android_navigation_action(props.navigation.as_ref()));
            if let Some(action) = action {
                output.push_str(&format!(
                    "        {view}.setOnClickListener(v -> {action});\n"
                ));
            }
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        _ => {}
    }
}
