fn render_dev_android_base_flow_node(
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
) -> bool {
    if !matches!(node, ViewNode::Splash { .. } | ViewNode::Scope { .. } | ViewNode::Each { .. } | ViewNode::Box { .. } | ViewNode::Section { .. } | ViewNode::Flex { .. } | ViewNode::Grid { .. }) {
        return false;
    }
    match node {
        ViewNode::Splash {
            binding,
            content,
            children,
            ..
        } => {
            output.push_str(&format!(
                "        if (doweBool(\"{}\")) {{\n",
                escape_java(&context.signal_path(binding))
            ));
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
                    context,
                    children_method,
                );
            }
            output.push_str("        } else {\n");
            for child in content {
                render_dev_android_node(
                    child,
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
            output.push_str("        }\n");
        }
        ViewNode::Scope {
            constants,
            signals,
            actions,
            children,
        } => {
            let context = context.with_scope(constants, signals, actions);
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
            if props.position().mode == BoxPosition::Fixed {
                render_dev_android_fixed_box(
                    props,
                    children,
                    counter,
                    output,
                    inherited_font,
                    inherited_color,
                    context,
                    children_method,
                );
            } else if props.position().mode == BoxPosition::Relative
                && (props.cover.is_some()
                    || children.iter().any(|child| {
                        matches!(child, ViewNode::Box { props, .. } if props.position().mode == BoxPosition::Absolute)
                    }))
            {
                render_dev_android_relative_box(
                    props,
                    children,
                    parent,
                    parent_gap,
                    parent_horizontal,
                    counter,
                    output,
                    inherited_font,
                    inherited_color,
                    context,
                    children_method,
                );
            } else if props.position().mode != BoxPosition::Absolute {
                let current_font = props.font.as_ref().or(inherited_font);
                let current_color = dev_inherited_color(props, inherited_color.as_deref());
                let view = next_dev_view(counter);
                output.push_str(&format!(
                    "        LinearLayout {view} = doweContainer(false);\n"
                ));
                apply_dev_android_style(props, &view, true, output);
                if props.center_x.is_some() || props.center_y.is_some() {
                    let x = props.center_x.as_ref().map(dev_bool_value).unwrap_or_else(|| "false".to_string());
                    let y = props.center_y.as_ref().map(dev_bool_value).unwrap_or_else(|| "false".to_string());
                    output.push_str(&format!("        {view}.setGravity((Boolean.TRUE.equals({y}) ? Gravity.CENTER_VERTICAL : Gravity.TOP) | (Boolean.TRUE.equals({x}) ? Gravity.CENTER_HORIZONTAL : Gravity.START));\n"));
                }
                apply_dev_android_click(props, &view, context, output);
                apply_dev_android_inline_width(props, &view, parent_horizontal, output);
                apply_dev_android_flex_item(props, parent, &view, output);
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
        }
        ViewNode::Section { props, children } => {
            let current_font = props.font.as_ref().or(inherited_font);
            let current_color = dev_inherited_color(props, inherited_color.as_deref());
            let view = next_dev_view(counter);
            let body = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n"
            ));
            let mut outer_props = props.clone();
            outer_props.spacing = Default::default();
            apply_dev_android_style(&outer_props, &view, true, output);
            apply_dev_android_inline_width(props, &view, parent_horizontal, output);
            let exact_height = props
                .sizing
                .h
                .as_ref()
                .and_then(dev_section_exact_height)
                .or_else(|| {
                    props
                        .sizing
                        .min_h
                        .as_ref()
                        .and_then(dev_section_exact_height)
                });
            if let Some(exact_height) = exact_height {
                output.push_str(&format!(
                    "        LinearLayout.LayoutParams {view}Params = (LinearLayout.LayoutParams) {view}.getLayoutParams();\n        {view}Params.height = {exact_height};\n        {view}.setLayoutParams({view}Params);\n"
                ));
            } else if props.sizing.h.is_some() || props.sizing.min_h.is_some() {
                output.push_str(&format!(
                    "        LinearLayout.LayoutParams {view}Params = (LinearLayout.LayoutParams) {view}.getLayoutParams();\n        {view}Params.height = ViewGroup.LayoutParams.MATCH_PARENT;\n        {view}.setLayoutParams({view}Params);\n"
                ));
            }
            apply_dev_android_flex_item(props, parent, &view, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            let body_constructor = if props.boxed {
                "doweBoxedContainer(1536)"
            } else {
                "doweContainer(false)"
            };
            output.push_str(&format!(
                "        LinearLayout {body} = {body_constructor};\n"
            ));
            let mut body_props = StyleProps::default();
            body_props.spacing = dowe_components::section_content_spacing(&props.spacing);
            apply_dev_android_style(&body_props, &body, false, output);
            if props.sizing.h.is_some() || props.sizing.min_h.is_some() {
                output.push_str(&format!(
                    "        LinearLayout.LayoutParams {body}Params = (LinearLayout.LayoutParams) {body}.getLayoutParams();\n        {body}Params.height = ViewGroup.LayoutParams.MATCH_PARENT;\n        {body}.setLayoutParams({body}Params);\n"
                ));
            }
            output.push_str(&dev_add(&view, &body, None, false));
            let section_gap = dev_optional_gap(props.gap.as_ref(), false);
            if props.center_x.is_some() || props.center_y.is_some() {
                let x = props.center_x.as_ref().map(dev_bool_value).unwrap_or_else(|| "false".to_string());
                let y = props.center_y.as_ref().map(dev_bool_value).unwrap_or_else(|| "false".to_string());
                output.push_str(&format!(
                    "        {body}.setGravity((Boolean.TRUE.equals({y}) ? Gravity.CENTER_VERTICAL : Gravity.TOP) | (Boolean.TRUE.equals({x}) ? Gravity.CENTER_HORIZONTAL : Gravity.START));\n"
                ));
            }
            for child in children {
                render_dev_android_node(
                    child,
                    &body,
                    section_gap.as_deref(),
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
                "        DoweFlexLayout {view} = doweFlex({}, {}, {}, {}, {});\n",
                dev_flex_direction(&props.direction),
                props.wrap,
                dev_flex_justify(props.justify.as_ref()),
                dev_flex_align(props.align.as_ref()),
                dev_optional_gap(props.gap.as_ref(), true).unwrap_or_else(|| "null".to_string())
            ));
            apply_dev_android_style(&props.style, &view, true, output);
            apply_dev_android_inline_width(&props.style, &view, parent_horizontal, output);
            apply_dev_android_flex_item(&props.style, parent, &view, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            for child in children {
                render_dev_android_node(
                    child,
                    &view,
                    None,
                    dev_flex_has_row(&props.direction),
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
            let tracks = dev_grid_tracks(props.columns.as_ref());
            let row_gap =
                dev_optional_gap(props.gap.as_ref(), false).unwrap_or_else(|| "null".to_string());
            let column_gap =
                dev_optional_gap(props.gap.as_ref(), true).unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        DoweGridLayout {view} = doweGrid({tracks}, {row_gap}, {column_gap});\n"
            ));
            apply_dev_android_style_with_shadow_radius(
                &props.style,
                &view,
                true,
                Some("0f"),
                output,
            );
            apply_dev_android_inline_width(&props.style, &view, parent_horizontal, output);
            apply_dev_android_flex_item(&props.style, parent, &view, output);
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
        _ => {}
    }
    true
}
