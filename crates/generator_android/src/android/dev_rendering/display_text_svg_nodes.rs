fn render_dev_android_display_text_svg_node(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    _children_method: Option<&str>,
) {
    match node {
        ViewNode::Divider { props } => {
            let view = next_dev_view(counter);
            let (default_width, default_height) = match props.orientation {
                DividerOrientation::Horizontal => {
                    ("ViewGroup.LayoutParams.MATCH_PARENT", "doweDp(1)")
                }
                DividerOrientation::Vertical => {
                    ("doweDp(1)", "ViewGroup.LayoutParams.MATCH_PARENT")
                }
            };
            output.push_str(&format!(
                                        "        View {view} = new View(this);\n        {view}.setBackgroundColor({});\n        {view}.setLayoutParams(new LinearLayout.LayoutParams({default_width}, {default_height}));\n",
                                        java_color(family_color(props.color))
                                    ));
            apply_dev_android_style(&props.style, &view, false, output);
            if props.style.sizing.w.is_none() || props.style.sizing.h.is_none() {
                output.push_str(&format!(
                                            "        LinearLayout.LayoutParams {view}Params = (LinearLayout.LayoutParams) {view}.getLayoutParams();\n"
                                        ));
                if props.style.sizing.w.is_none() {
                    output.push_str(&format!("        {view}Params.width = {default_width};\n"));
                }
                if props.style.sizing.h.is_none() {
                    output.push_str(&format!(
                        "        {view}Params.height = {default_height};\n"
                    ));
                }
                output.push_str(&format!("        {view}.setLayoutParams({view}Params);\n"));
            }
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Title { props, value } => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        TextView {view} = doweText({}, {}, {}, {}, {}, {}, {});\n",
                dev_text_expression(value, props.i18n.as_deref(), context),
                dev_text_color(props, inherited_color.as_deref()),
                dev_text_size(true, props),
                dev_text_weight(true, props),
                dev_text_spacing(true, props),
                dev_text_line_height(true, props),
                dev_font_value(props.style.font.as_ref().or(inherited_font))
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Text { props, value } => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        TextView {view} = doweText({}, {}, {}, {}, {}, {}, {});\n",
                dev_text_expression(value, props.i18n.as_deref(), context),
                dev_text_color(props, inherited_color.as_deref()),
                dev_text_size(false, props),
                dev_text_weight(false, props),
                dev_text_spacing(false, props),
                dev_text_line_height(false, props),
                dev_font_value(props.style.font.as_ref().or(inherited_font))
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Alert { props } => {
            let view = next_dev_view(counter);
            if let Some(visible) = props.visible.as_deref() {
                output.push_str(&format!(
                    "        if (doweBool(\"{}\")) {{\n",
                    escape_java(&context.signal_path(visible))
                ));
            }
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                dev_variant_content(&props.style)
            } else {
                "null"
            };
            output.push_str(&format!(
                                        "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n        {view}.setPadding(doweDp(14), doweDp(10), doweDp(14), doweDp(10));\n        {view}.setBackground(doweInputBackground({}, {border}, DOWE_RADIUS_UI));\n        TextView {view}Text = doweText({}, {}, 14f, 400, 0f, 1.2f, {});\n        {view}Text.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n        doweAdd({view}, {view}Text);\n",
                                        dev_variant_container(&props.style),
                                        dev_text_expression(&props.message, None, context),
                                        dev_variant_content(&props.style),
                                        dev_font_value(props.style.style.font.as_ref().or(inherited_font))
                                    ));
            if let Some(action) = props
                .on_close
                .as_deref()
                .and_then(|name| context.action_id(name))
            {
                let close = next_dev_view(counter);
                output.push_str(&format!(
                                            "        Button {close} = new Button(this);\n        {close}.setText(\"x\");\n        {close}.setOnClickListener(v -> doweRunAction(\"{}\", null));\n",
                                            escape_java(action)
                                        ));
                output.push_str(&format!("        doweAdd({view}, {close});\n"));
            }
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            if props.visible.is_some() {
                output.push_str("        }\n");
            }
        }
        ViewNode::Svg { props, paths } => {
            let view = next_dev_view(counter);
            let paths_name = format!("{view}Paths");
            output.push_str(&format!(
                "        ArrayList<DoweSvgPathEntry> {paths_name} = new ArrayList<>();\n"
            ));
            for path in paths {
                output.push_str(&format!(
                    "        {paths_name}.add(new DoweSvgPathEntry(\"{}\", {}, {}));\n",
                    escape_java(&path.data),
                    dev_svg_path_current_color(path.fill),
                    dev_svg_path_color(path.fill)
                ));
            }
            output.push_str(&format!(
                                        "        DoweSvgView {view} = new DoweSvgView(this, {}f, {}f, {}f, {}f, {}, {paths_name});\n",
                                        props.view_box.min_x,
                                        props.view_box.min_y,
                                        props.view_box.width,
                                        props.view_box.height,
                                        dev_svg_color(&props.style, inherited_color.as_deref())
                                    ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        _ => {}
    }
}
