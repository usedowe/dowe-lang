fn render_dev_android_overlay_node(
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
        ViewNode::Drawer { props, children } => {
            render_dev_android_drawer(
                props,
                children,
                counter,
                output,
                inherited_font,
                context,
                children_method,
            );
        }
        ViewNode::Avatar { props, .. } => {
            let label = props
                .name
                .as_deref()
                .or(Some(props.alt.as_str()))
                .and_then(|value| value.chars().next())
                .map(|value| value.to_uppercase().to_string())
                .unwrap_or_else(|| "A".to_string());
            render_dev_android_variant_label(
                &label,
                &props.style,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
            );
        }
        ViewNode::Badge { props, children } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let current_color = Some(dev_variant_content(&props.style).to_string());
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
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
            render_dev_android_variant_label(
                &props.text,
                &props.style,
                &view,
                None,
                false,
                counter,
                output,
                current_font,
                context,
            );
        }
        ViewNode::Chip { props, value, .. } => {
            render_dev_android_variant_label(
                value,
                &props.style,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
            );
        }
        ViewNode::Skeleton { props } => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                            "        View {view} = new View(this);\n        {view}.setMinimumHeight(doweDp(16));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_UI));\n",
                            java_color(ColorToken::Muted)
                        ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            let path = escape_java(&context.signal_path(&props.open));
            output.push_str(&format!("        if (doweBool(\"{path}\")) {{\n"));
            let view = next_dev_view(counter);
            output.push_str(&format!(
                            "        LinearLayout {view} = doweContainer(false);\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_BOX));\n",
                            dev_variant_container(&props.style)
                        ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in header.iter().chain(body).chain(footer) {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    props.style.style.font.as_ref().or(inherited_font),
                    Some(dev_variant_content(&props.style).to_string()),
                    context,
                    children_method,
                );
            }
            output.push_str("        }\n");
        }
        ViewNode::AlertDialog { props } => {
            let path = escape_java(&context.signal_path(&props.open));
            output.push_str(&format!("        if (doweBool(\"{path}\")) {{\n"));
            render_dev_android_variant_label(
                &format!("{} {}", props.title, props.description),
                &props.style,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
            );
            output.push_str("        }\n");
        }
        ViewNode::Tooltip { props, children } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
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
                    inherited_color.clone(),
                    context,
                    children_method,
                );
            }
            render_dev_android_variant_label(
                &props.label,
                &props.style,
                &view,
                None,
                false,
                counter,
                output,
                current_font,
                context,
            );
        }
        ViewNode::Toast { props } => {
            let visible = props
                .source
                .as_deref()
                .map(|source| {
                    format!(
                        "doweBool(\"{}.visible\")",
                        escape_java(&context.signal_path(source))
                    )
                })
                .unwrap_or_else(|| "true".to_string());
            output.push_str(&format!("        if ({visible}) {{\n"));
            render_dev_android_variant_label(
                props.title.as_deref().unwrap_or(&props.description),
                &props.style,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                context,
            );
            output.push_str("        }\n");
        }
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            entries,
            footer,
        } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            let view = next_dev_view(counter);
            output.push_str(&format!(
                            "        LinearLayout {view} = doweContainer(false);\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_BOX));\n",
                            dev_variant_container(&props.style)
                        ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in trigger.iter().chain(header).chain(footer) {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    false,
                    counter,
                    output,
                    current_font,
                    Some(dev_variant_content(&props.style).to_string()),
                    context,
                    children_method,
                );
            }
            for entry in entries {
                render_dev_android_overlay_entry(
                    entry,
                    &props.style,
                    &view,
                    counter,
                    output,
                    current_font,
                    context,
                );
            }
        }
        ViewNode::Command { props, entries } => {
            if let Some(open) = props.open.as_deref() {
                output.push_str(&format!(
                    "        if (doweBool(\"{}\")) {{\n",
                    escape_java(&context.signal_path(open))
                ));
            }
            let view = next_dev_view(counter);
            output.push_str(&format!(
                            "        LinearLayout {view} = doweContainer(false);\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS_BOX));\n",
                            dev_variant_container(&props.style)
                        ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            render_dev_android_variant_label(
                &props.placeholder,
                &props.style,
                &view,
                None,
                false,
                counter,
                output,
                props.style.style.font.as_ref().or(inherited_font),
                context,
            );
            for entry in entries {
                render_dev_android_command_entry(
                    entry,
                    &props.style,
                    &view,
                    counter,
                    output,
                    props.style.style.font.as_ref().or(inherited_font),
                    context,
                );
            }
            if props.open.is_some() {
                output.push_str("        }\n");
            }
        }
        ViewNode::Children => {
            if let Some(method) = children_method {
                output.push_str(&format!("        {method}({parent});\n"));
            }
        }
        _ => {}
    }
}
