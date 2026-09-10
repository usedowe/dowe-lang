fn render_dev_android_basic_display_node(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    _inherited_font: Option<&ResponsiveValue<FontFamily>>,
    _inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    _children_method: Option<&str>,
) -> bool {
    if !matches!(
        node,
        ViewNode::Audio { .. }
            | ViewNode::Camera { .. }
            | ViewNode::Microphone { .. }
            | ViewNode::Image { .. }
    ) {
        return false;
    }
    match node {
        ViewNode::Audio { props } => {
            let play = solar_control_icon("play").expect("bundled Audio play icon");
            let pause = solar_control_icon("pause").expect("bundled Audio pause icon");
            let content_color = dev_card_variant_content(&props.style);
            let mut button_style = props.style.clone();
            button_style.variant = Some(
                if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Solid
                {
                    ComponentVariant::Solid
                } else {
                    ComponentVariant::Solid
                },
            );
            let button_background = dev_variant_container(&button_style);
            let button_content = dev_variant_content(&button_style);
            let play_view =
                render_dev_android_icon_view(&play, counter, output, Some(button_content));
            let pause_view =
                render_dev_android_icon_view(&pause, counter, output, Some(button_content));
            let view = next_dev_view(counter);
            let subtitle = props
                .subtitle
                .as_deref()
                .map(escape_java)
                .map(|value| format!("\"{value}\""))
                .unwrap_or_else(|| "null".to_string());
            let avatar = props
                .avatar_src
                .as_deref()
                .map(escape_java)
                .map(|value| format!("\"{value}\""))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        LinearLayout {view} = doweAudio(\"{}\", {}, {}, {}, {}, {}, {}, {}, {play_view}, {pause_view});\n",
                escape_java(&props.src),
                subtitle,
                avatar,
                dev_card_variant_container(&props.style),
                content_color,
                button_background,
                button_content,
                dev_card_border(&props.style),
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Camera { props } => {
            let view = next_dev_view(counter);
            let on_start = props
                .on_start
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let on_capture = props
                .on_capture
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let on_error = props
                .on_error
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            output.push_str(&format!(
                "        Button {view} = new Button(this);\n        {view}.setText(\"{}\");\n        {view}.setAllCaps(false);\n        {view}.setEnabled({});\n        {view}.setOnClickListener(target -> doweOpenCamera(\"{}\", {on_start}, {on_capture}, {on_error}));\n",
                escape_java(&props.label),
                !props.disabled,
                props.facing.as_str(),
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Microphone { props } => {
            let view = next_dev_view(counter);
            let start = props
                .on_start
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let stop = props
                .on_stop
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let error = props
                .on_error
                .as_deref()
                .and_then(|value| context.action_id(value))
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string());
            let duration = props
                .max_duration
                .map(|value| value.to_string())
                .unwrap_or_else(|| "0".to_string());
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(true);\n        Button {view}Start = new Button(this);\n        {view}Start.setText(\"{}\");\n        {view}Start.setAllCaps(false);\n        {view}Start.setEnabled({});\n        {view}Start.setOnClickListener(target -> doweStartMicrophone({start}, {stop}, {error}, {duration}));\n        doweAdd({view}, {view}Start);\n",
                escape_java(&props.label),
                !props.disabled,
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        ViewNode::Image { props } => {
            let view = next_dev_view(counter);
            let source = props
                .reactive_src
                .as_deref()
                .map(|path| dev_text_expression(path, None, context))
                .unwrap_or_else(|| format!("\"{}\"", escape_java(&props.src)));
            output.push_str(&format!(
                                        "        FrameLayout {view} = doweImage({source}, \"{}\", \"{}\", \"{}\", {}, {});\n",
                                        escape_java(&props.alt),
                                        props.aspect.as_str(),
                                        props.object_fit.as_str(),
                                        dev_card_variant_container(&props.style),
                                        dev_card_border(&props.style)
                                    ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
        }
        _ => {}
    }
    true
}
