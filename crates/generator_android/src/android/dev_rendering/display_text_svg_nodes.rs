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
                dev_visible_text_expression(value, props.i18n.as_deref(), context),
                dev_text_color(true, props, inherited_color.as_deref()),
                dev_text_size(true, props),
                dev_text_weight(true, props),
                dev_text_spacing(true, props),
                dev_text_line_height(true, props),
                dev_font_value(props.style.font.as_ref().or(inherited_font))
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            apply_dev_text_alignment(props, &view, parent_horizontal, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Text { props, value } => {
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        TextView {view} = doweText({}, {}, {}, {}, {}, {}, {});\n",
                dev_visible_text_expression(value, props.i18n.as_deref(), context),
                dev_text_color(false, props, inherited_color.as_deref()),
                dev_text_size(false, props),
                dev_text_weight(false, props),
                dev_text_spacing(false, props),
                dev_text_line_height(false, props),
                dev_font_value(props.style.font.as_ref().or(inherited_font))
            ));
            apply_dev_android_style(&props.style, &view, false, output);
            apply_dev_text_alignment(props, &view, parent_horizontal, output);
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
                                        "        LinearLayout {view} = doweContainer(true);\n        {view}.setGravity(Gravity.CENTER_VERTICAL);\n        {view}.setPadding(doweDp(14), doweDp(10), doweDp(14), doweDp(10));\n        {view}.setBackground(doweInputBackground({}, {border}, DOWE_RADIUS));\n        TextView {view}Text = doweText({}, {}, 14f, 400, 0f, 1.2f, {});\n        {view}Text.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n        doweAdd({view}, {view}Text);\n",
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
            if let Some(data) = props.data.as_deref() {
                let payload = if let Some(item) = context.item_value(data) {
                    let path = context.item_path(data).unwrap_or_else(|| data.to_string());
                    format!("doweTextValue(\"{}\", {item})", escape_java(&path))
                } else {
                    format!(
                        "doweTextValue(\"{}\", null)",
                        escape_java(&context.signal_path(data))
                    )
                };
                output.push_str(&format!(
                    "        DoweSvgView {view} = doweRuntimeSvg({payload}, {}, {});\n        if ({view} == null) {view} = new DoweSvgView(this, 0f, 0f, 24f, 24f, {}, new ArrayList<>(), false);\n",
                    dev_svg_color(&props.style, inherited_color.as_deref()),
                    props.is_animated(),
                    dev_svg_color(&props.style, inherited_color.as_deref())
                ));
                apply_dev_android_style(&props.style, &view, false, output);
                output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
                return;
            }
            if let Some(name) = props.icon_name.as_deref() {
                let name_expression = if let Some(item) = context.item_value(name) {
                    let path = context.item_path(name).unwrap_or_else(|| name.to_string());
                    format!(
                        "doweTextValue(\"{}\", {item})",
                        escape_java(&path)
                    )
                } else {
                    format!(
                        "doweTextValue(\"{}\", null)",
                        escape_java(&context.signal_path(name))
                    )
                };
                let fallback = props.icon_fallback.as_deref().unwrap_or_default();
                output.push_str(&format!(
                    "        DoweSvgView {view} = doweRuntimeSvg(doweDynamicIconPayload({name_expression}, \"{}\"), {}, {});\n        if ({view} == null) {view} = new DoweSvgView(this, 0f, 0f, 24f, 24f, {}, new ArrayList<>(), false);\n",
                    escape_java(fallback),
                    dev_dynamic_icon_color(props, inherited_color.as_deref()),
                    props.is_animated(),
                    dev_dynamic_icon_color(props, inherited_color.as_deref())
                ));
                apply_dev_android_style(&props.style, &view, false, output);
                output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
                return;
            }
            let paths_name = format!("{view}Paths");
            output.push_str(&format!(
                "        ArrayList<DoweSvgPathEntry> {paths_name} = new ArrayList<>();\n"
            ));
            for path in paths {
                output.push_str(&format!(
                    "        {paths_name}.add(new DoweSvgPathEntry(\"{}\", {}, {}, {}, {}));\n",
                    escape_java(&path.data),
                    dev_svg_path_current_color(path.fill),
                    dev_svg_path_color(path.fill),
                    dev_svg_path_details(path.fill),
                    dev_svg_path_transform(path.transform.as_ref())
                ));
            }
            output.push_str(&format!(
                                        "        DoweSvgView {view} = new DoweSvgView(this, {}f, {}f, {}f, {}f, {}, {paths_name}, {});\n",
                                        props.view_box.min_x,
                                        props.view_box.min_y,
                                        props.view_box.width,
                                        props.view_box.height,
                                        dev_svg_color(&props.style, inherited_color.as_deref()),
                                        props.is_animated()
                                    ));
            apply_dev_android_style(&props.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        _ => {}
    }
}

fn dev_dynamic_icon_color(props: &SvgProps, inherited_color: Option<&str>) -> String {
    props
        .icon_fill
        .or(props.icon_stroke)
        .map(java_color)
        .map(str::to_string)
        .unwrap_or_else(|| dev_svg_color(&props.style, inherited_color))
}
