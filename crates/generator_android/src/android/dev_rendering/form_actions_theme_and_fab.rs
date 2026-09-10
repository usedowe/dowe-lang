#[allow(unused_variables)]
fn render_dev_android_form_actions_toggle_theme(
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
    let ViewNode::ToggleTheme { props } = node else { return; };
            let view = next_dev_view(counter);
            let light_icon = render_dev_android_icon_view(&props.light_icon, counter, output, Some(&dev_variant_content(&props.style)));
            let dark_icon = render_dev_android_icon_view(&props.dark_icon, counter, output, Some(&dev_variant_content(&props.style)));
            output.push_str(&format!(
                "        FrameLayout {view} = new FrameLayout(this);\n        final boolean[] {view}Dark = new boolean[]{{\"dark\".equals(getSharedPreferences(\"dowe\", 0).getString(\"theme-preference\", \"light\"))}};\n        {view}.setContentDescription({view}Dark[0] ? \"{}\" : \"{}\");\n        {view}.setBackground(doweBackground({}, DOWE_RADIUS));\n        {view}.setOnClickListener(v -> {{ {view}Dark[0] = !{view}Dark[0]; {view}.setContentDescription({view}Dark[0] ? \"{}\" : \"{}\"); doweSetTheme({view}Dark[0] ? \"dark\" : \"light\"); }});\n        {view}.addView({light_icon}, new FrameLayout.LayoutParams(doweDp(18), doweDp(18), Gravity.CENTER));\n        {view}.addView({dark_icon}, new FrameLayout.LayoutParams(doweDp(18), doweDp(18), Gravity.CENTER));\n        {dark_icon}.setVisibility({view}Dark[0] ? View.INVISIBLE : View.VISIBLE);\n        {light_icon}.setVisibility({view}Dark[0] ? View.VISIBLE : View.INVISIBLE);\n",
                escape_java(&props.light_label),
                escape_java(&props.dark_label),
                dev_variant_container(&props.style),
                escape_java(&props.light_label),
                escape_java(&props.dark_label)
            ));
            apply_dev_android_style(&props.style.style, &view, false, output);
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}

#[allow(unused_variables)]
fn render_dev_android_form_actions_select_theme(
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
    let ViewNode::SelectTheme { props } = node else { return; };
            let view = next_dev_view(counter);
            let field = next_dev_view(counter);
            let frame = next_dev_view(counter);
            let labels = props
                .themes
                .iter()
                .map(|theme| theme_display_label(theme))
                .collect::<Vec<_>>();
            let labels = java_string_array(labels.iter().map(String::as_str));
            let values = java_string_array(props.themes.iter().map(String::as_str));
            let descriptions = java_string_array(props.themes.iter().map(|_| ""));
            let content = dev_card_variant_content(&props.style);
            let background = if props.style.variant.unwrap_or(ComponentVariant::Outlined)
                == ComponentVariant::Outlined
            {
                format!(
                    "doweInputBackground({}, {}, DOWE_RADIUS)",
                    dev_card_variant_container(&props.style),
                    content
                )
            } else {
                format!(
                    "doweBackground({}, DOWE_RADIUS)",
                    dev_card_variant_container(&props.style)
                )
            };
            let font = dev_font_value(props.style.style.font.as_ref().or(inherited_font));
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n        TextView {view}Label = doweControlLabel(\"{}\", {content}, {font});\n        doweAdd({view}, {view}Label);\n        String[] {field}Labels = {labels};\n        String[] {field}Values = {values};\n        String[] {field}Descriptions = {descriptions};\n        TextView {field} = doweSelectTrigger(\"{}\", {content}, {font});\n        {field}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n        {field}.setMinimumHeight(doweDp({}));\n        {field}.setPadding(doweDp({}), 0, doweDp(36), 0);\n        {field}.setBackgroundColor(Color.TRANSPARENT);\n        final String[] {field}Selected = new String[]{{getSharedPreferences(\"dowe\", 0).getString(\"theme-preference\", \"{}\")}};\n        FrameLayout {frame} = doweSelectFrame({field}, {content}, {background});\n        doweAdd({view}, {frame}, 4, false);\n        doweBindSelect({field}, null, {field}Labels, {field}Values, {field}Descriptions, {field}Selected, \"{}\", {content}, {font}, null, false, value -> doweSetTheme(value));\n",
                escape_java(&props.label),
                escape_java(&props.placeholder),
                INPUT_MIN_HEIGHT.native_units(),
                INPUT_HORIZONTAL_PADDING.native_units(),
                escape_java(&props.default_theme),
                escape_java(&props.placeholder)
            ));
            apply_dev_android_style(&props.style.style, &view, true, output);
            if parent_horizontal && props.style.style.sizing.w.is_none() {
                output.push_str(&format!(
                    "        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n"
                ));
            }
            output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}

#[allow(unused_variables)]
fn render_dev_android_form_actions_fab(
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
    let ViewNode::Fab { props, actions } = node else { return; };
            let view = next_dev_view(counter);
            output.push_str(&format!(
                "        LinearLayout {view} = doweContainer(false);\n        {view}.setGravity({});\n",
                dev_fab_content_gravity(props.position)
            ));
            let mut action_views = Vec::new();
            for action in actions {
                let item = next_dev_view(counter);
                let label = next_dev_view(counter);
                let action_props = VariantProps {
                    color: Some(action.color),
                    variant: props.style.variant,
                    ..VariantProps::default()
                };
                output.push_str(&format!(
                    "        LinearLayout {item} = doweContainer(true);\n        {item}.setGravity(Gravity.CENTER_VERTICAL);\n        {item}.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));\n        {item}.setBackground(doweBackground({}, 999f));\n        {item}.setClickable(true);\n        {item}.setFocusable(true);\n        TextView {label} = doweText(\"{}\", {}, 14f, 600, 0f, 1.2f, null);\n        doweAdd({item}, {label});\n",
                    dev_variant_container(&action_props),
                    escape_java(&action.label),
                    dev_variant_content(&action_props)
                ));
                let icon = view_icon(action.icon);
                let icon_view = render_dev_android_icon_view(
                    &icon,
                    counter,
                    output,
                    Some(&dev_variant_content(&action_props)),
                );
                output.push_str(&format!(
                    "        doweAdd({item}, {icon_view}, 8, true);\n        doweWrapContentWidth({item});\n        {item}.setVisibility(View.INVISIBLE);\n"
                ));
                if let Some(click) = action
                    .on_click
                    .as_deref()
                    .and_then(|name| context.action_id(name))
                    .map(|id| {
                        let item = context.active_item().unwrap_or("null");
                        format!("doweRunAction(\"{}\", {item})", escape_java(id))
                    })
                    .or_else(|| dev_android_navigation_action(action.navigation.as_ref()))
                {
                    output.push_str(&format!(
                        "        {item}.setOnClickListener(v -> {click});\n"
                    ));
                }
                action_views.push(item);
            }
            let trigger = next_dev_view(counter);
            let trigger_size = dev_fab_size(props.style.size.unwrap_or(ButtonSize::Lg));
            output.push_str(&format!(
                "        FrameLayout {trigger} = new FrameLayout(this);\n        {trigger}.setContentDescription(\"{}\");\n        {trigger}.setClickable(true);\n        {trigger}.setFocusable(true);\n        {trigger}.setBackground(doweBackground({}, 999f));\n",
                escape_java(&props.label),
                dev_variant_container(&props.style)
            ));
            let trigger_icon = view_icon(props.icon);
            let trigger_icon_view = render_dev_android_icon_view(
                &trigger_icon,
                counter,
                output,
                Some(&dev_variant_content(&props.style)),
            );
            output.push_str(&format!(
                "        {trigger}.addView({trigger_icon_view}, new FrameLayout.LayoutParams(doweDp({}), doweDp({}), Gravity.CENTER));\n        {trigger}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({trigger_size}), doweDp({trigger_size})));\n",
                trigger_size / 2,
                trigger_size / 2
            ));
            if let Some(gesture) = props.style.style.motion().gesture
                && gesture != ViewGesture::None
            {
                output.push_str(&format!(
                    "        doweGesture({trigger}, \"{}\", \"{}\");\n",
                    gesture.as_str(),
                    props
                        .style
                        .style
                        .motion()
                        .transition
                        .unwrap_or(ViewTransition::Smooth)
                        .as_str()
                ));
            }
            if action_views.is_empty() {
                if let Some(click) = props
                    .style
                    .element
                    .on_click
                    .as_deref()
                    .and_then(|name| context.action_id(name))
                    .map(|id| {
                        let item = context.active_item().unwrap_or("null");
                        format!("doweRunAction(\"{}\", {item})", escape_java(id))
                    })
                    .or_else(|| dev_android_navigation_action(props.style.navigation.as_ref()))
                {
                    output.push_str(&format!(
                        "        {trigger}.setOnClickListener(v -> {click});\n"
                    ));
                }
            } else {
                let visibility_updates = action_views
                    .iter()
                    .map(|item| {
                        format!("{item}.setVisibility(open ? View.VISIBLE : View.INVISIBLE);")
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                output.push_str(&format!(
                    "        {trigger}.setOnClickListener(v -> {{ boolean open = {}.getVisibility() != View.VISIBLE; {visibility_updates} {trigger_icon_view}.setRotation(open ? 45f : 0f); }});\n",
                    action_views[0],
                ));
            }
            let action_additions = action_views
                .iter()
                .map(|item| format!("        doweAdd({view}, {item}, 12, false);\n"))
                .collect::<String>();
            let trigger_addition = format!("        doweAdd({view}, {trigger}, 12, false);\n");
            if matches!(
                props.position,
                OverlayCornerPosition::TopLeft | OverlayCornerPosition::TopRight
            ) {
                output.push_str(&trigger_addition);
                output.push_str(&action_additions);
            } else {
                output.push_str(&action_additions);
                output.push_str(&trigger_addition);
            }
            if props.fixed {
                let overlay = next_dev_view(counter);
                output.push_str(&format!(
                    "        FrameLayout {overlay} = new FrameLayout(this);\n        {overlay}.setTag(\"dowe-fixed-fab\");\n        {overlay}.setClipChildren(false);\n        {overlay}.setClipToPadding(false);\n        FrameLayout.LayoutParams {view}Params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT, {});\n        {view}Params.setMargins(doweDp({}), doweDp({}), doweDp({}), doweDp({}));\n        {overlay}.addView({view}, {view}Params);\n        ((ViewGroup) scrollView.getParent()).addView({overlay}, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n        doweApplySystemInsets({overlay});\n",
                    dev_fab_layout_gravity(props.position),
                    if matches!(props.position, OverlayCornerPosition::TopLeft | OverlayCornerPosition::BottomLeft) { props.offset_x.native_units() } else { 0 },
                    if matches!(props.position, OverlayCornerPosition::TopLeft | OverlayCornerPosition::TopRight) { props.offset_y.native_units() } else { 0 },
                    if matches!(props.position, OverlayCornerPosition::TopRight | OverlayCornerPosition::BottomRight) { props.offset_x.native_units() } else { 0 },
                    if matches!(props.position, OverlayCornerPosition::BottomLeft | OverlayCornerPosition::BottomRight) { props.offset_y.native_units() } else { 0 }
                ));
            } else {
                let mut container_style = props.style.style.clone();
                if container_style.motion().gesture.is_some() {
                    container_style.motion_mut().gesture = None;
                }
                apply_dev_android_style(&container_style, &view, false, output);
                output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
            }
}

