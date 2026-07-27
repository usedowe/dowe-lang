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
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => {
            render_dev_android_drawer(
                props,
                header,
                body,
                footer,
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
            let (size, text_size) = match props.size {
                ButtonSize::Xs => (24, 12),
                ButtonSize::Sm => (32, 14),
                ButtonSize::Md => (40, 16),
                ButtonSize::Lg => (48, 18),
                ButtonSize::Xl => (64, 24),
            };
            let view = next_dev_view(counter);
            if let Some(source) = props.src.as_deref() {
                output.push_str(&format!(
                    "        FrameLayout {view} = doweAvatarImage(\"{}\", \"{}\", {}, {}, {}, {text_size}f, {});\n",
                    escape_java(source),
                    escape_java(&props.alt),
                    dev_text_expression(&label, None, context),
                    dev_variant_container(&props.style),
                    dev_variant_content(&props.style),
                    dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                ));
            } else {
                output.push_str(&format!(
                    "        TextView {view} = doweText({}, {}, {text_size}f, 600, 0f, 1.2f, {});\n        {view}.setGravity(Gravity.CENTER);\n",
                    dev_text_expression(&label, None, context),
                    dev_variant_content(&props.style),
                    dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                ));
            }
            let mut avatar_style = props.style.style.clone();
            avatar_style.shadow = None;
            avatar_style.shadow_color = None;
            avatar_style.rounded = None;
            apply_dev_android_style(&avatar_style, &view, false, output);
            output.push_str(&format!(
                "        {view}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({size}), doweDp({size})));\n        {view}.setBackground(doweBackground({}, 999f));\n        doweRound({view}, 999f);\n",
                dev_variant_container(&props.style)
            ));
            apply_dev_android_shadow_with_radius(
                &props.style.style,
                &view,
                "999f",
                output,
            );
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
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
                            "        View {view} = new View(this);\n        {view}.setMinimumHeight(doweDp(16));\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS));\n",
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
            render_dev_android_modal(
                props,
                header,
                body,
                footer,
                counter,
                output,
                inherited_font,
                context,
                children_method,
            );
        }
        ViewNode::AlertDialog { props } => {
            render_dev_android_alert_dialog(props, counter, output, inherited_font, context);
        }
        ViewNode::Tooltip { props, children } => {
            let current_font = props.style.style.font.as_ref().or(inherited_font);
            for child in children {
                render_dev_android_node(
                    child,
                    parent,
                    parent_gap,
                    parent_horizontal,
                    counter,
                    output,
                    current_font,
                    inherited_color.clone(),
                    context,
                    children_method,
                );
            }
        }
        ViewNode::Toast { props } => {
            render_dev_android_toast(props, counter, output, inherited_font, context);
        }
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            entries,
            footer,
        } => {
            render_dev_android_dropdown(
                props,
                trigger,
                header,
                entries,
                footer,
                parent,
                parent_gap,
                parent_horizontal,
                counter,
                output,
                inherited_font,
                inherited_color.clone(),
                context,
                children_method,
            );
        }
        ViewNode::Command { props, entries } => {
            render_dev_android_command(props, entries, counter, output, inherited_font, context);
        }
        ViewNode::Children => {
            if let Some(method) = children_method {
                output.push_str(&format!("        {method}({parent});\n"));
            }
        }
        _ => {}
    }
}
