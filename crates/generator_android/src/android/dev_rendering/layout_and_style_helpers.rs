fn dev_add(parent: &str, view: &str, gap: Option<&str>, horizontal: bool) -> String {
    match gap {
        Some(gap) => format!(
            "        doweAdd({parent}, {view}, {gap}, {});\n",
            if horizontal { "true" } else { "false" }
        ),
        None => format!("        doweAdd({parent}, {view});\n"),
    }
}

fn apply_dev_android_inline_width(
    props: &StyleProps,
    view: &str,
    parent_horizontal: bool,
    output: &mut String,
) {
    if parent_horizontal && props.sizing.w.is_none() {
        output.push_str(&format!("        doweWrapContentWidth({view});\n"));
    }
}

fn dev_optional_gap(value: Option<&ResponsiveValue<GapValue>>, horizontal: bool) -> Option<String> {
    value.map(|value| dev_responsive_value(value, |value| dev_gap_expr(value, horizontal)))
}

fn dev_flex_justify(value: Option<&ResponsiveValue<Justify>>) -> String {
    value
        .map(dev_flex_justify_value)
        .unwrap_or_else(|| "null".to_string())
}

fn dev_flex_direction(value: &ResponsiveValue<FlexDirection>) -> String {
    dev_responsive_value(value, |value| match value {
        FlexDirection::Row => "DOWE_DIRECTION_ROW".to_string(),
        FlexDirection::Column => "DOWE_DIRECTION_COLUMN".to_string(),
    })
}

fn dev_flex_has_row(value: &ResponsiveValue<FlexDirection>) -> bool {
    value
        .entries
        .iter()
        .any(|entry| entry.value == FlexDirection::Row)
}

fn dev_flex_justify_value(value: &ResponsiveValue<Justify>) -> String {
    dev_responsive_value(value, |value| match value {
        Justify::Start => "DOWE_JUSTIFY_START".to_string(),
        Justify::Center => "DOWE_JUSTIFY_CENTER".to_string(),
        Justify::End => "DOWE_JUSTIFY_END".to_string(),
        Justify::Between => "DOWE_JUSTIFY_BETWEEN".to_string(),
        Justify::Around => "DOWE_JUSTIFY_AROUND".to_string(),
        Justify::Evenly => "DOWE_JUSTIFY_EVENLY".to_string(),
    })
}

fn dev_flex_align(value: Option<&ResponsiveValue<Align>>) -> String {
    value
        .map(dev_flex_align_value)
        .unwrap_or_else(|| "null".to_string())
}

fn dev_flex_align_value(value: &ResponsiveValue<Align>) -> String {
    dev_responsive_value(value, |value| match value {
        Align::Start => "DOWE_ALIGN_START".to_string(),
        Align::Center => "DOWE_ALIGN_CENTER".to_string(),
        Align::End => "DOWE_ALIGN_END".to_string(),
        Align::Stretch => "DOWE_ALIGN_STRETCH".to_string(),
        Align::Baseline => "DOWE_ALIGN_BASELINE".to_string(),
    })
}

fn dev_grid_columns(value: Option<&ResponsiveValue<GridTracks>>) -> String {
    value
        .map(|value| dev_responsive_value(value, |value| value.count().unwrap_or(1).to_string()))
        .unwrap_or_else(|| "1".to_string())
}

fn dev_inherited_color(props: &StyleProps, inherited_color: Option<&str>) -> Option<String> {
    props
        .text
        .as_ref()
        .map(dev_color_value)
        .map(|color| dev_content_colors(&color, &color))
        .or_else(|| inherited_color.map(str::to_string))
}

fn dev_content_colors(text: &str, title: &str) -> String {
    format!("{text}\u{1f}{title}")
}

fn dev_inherited_text_color(value: Option<&str>) -> Option<&str> {
    value.map(|value| value.split_once('\u{1f}').map_or(value, |colors| colors.0))
}

fn dev_inherited_title_color(value: Option<&str>) -> Option<&str> {
    value.map(|value| value.split_once('\u{1f}').map_or(value, |colors| colors.1))
}

fn dev_svg_color(props: &StyleProps, inherited_color: Option<&str>) -> String {
    let fallback = dev_inherited_text_color(inherited_color).unwrap_or("DOWE_BACKGROUND_TEXT");
    props
        .text
        .as_ref()
        .map(dev_color_value)
        .map(|value| format!("doweColor({value}, {fallback})"))
        .unwrap_or_else(|| fallback.to_string())
}

fn dev_svg_path_current_color(fill: SvgPathFill) -> &'static str {
    match fill {
        SvgPathFill::CurrentColor | SvgPathFill::Fill { color: None, .. } | SvgPathFill::Stroke { color: None, .. } => "true",
        SvgPathFill::None | SvgPathFill::Color(_) | SvgPathFill::RawFill { .. } | SvgPathFill::RawStroke { .. } | SvgPathFill::LiteralFill { .. } | SvgPathFill::LiteralStroke { .. } | SvgPathFill::Fill { color: Some(_), .. } | SvgPathFill::Stroke { color: Some(_), .. } => "false",
    }
}

fn dev_svg_path_color(fill: SvgPathFill) -> String {
    match fill {
        SvgPathFill::None | SvgPathFill::CurrentColor => "null".to_string(),
        SvgPathFill::RawFill { color, .. } | SvgPathFill::RawStroke { color, .. } => android_java_color_literal(color),
        SvgPathFill::LiteralFill { red, green, blue, .. }
        | SvgPathFill::LiteralStroke { red, green, blue, .. } => {
            format!("Color.rgb({red}, {green}, {blue})")
        }
        SvgPathFill::Color(token) | SvgPathFill::Fill { color: Some(token), .. } | SvgPathFill::Stroke { color: Some(token), .. } => java_color(token).to_string(),
        SvgPathFill::Fill { color: None, .. } | SvgPathFill::Stroke { color: None, .. } => "null".to_string(),
    }
}

fn dev_svg_path_details(fill: SvgPathFill) -> String {
    match fill {
        SvgPathFill::RawFill { opacity, even_odd, .. } | SvgPathFill::LiteralFill { opacity, even_odd, .. } | SvgPathFill::Fill { opacity, even_odd, .. } => {
            format!("false, {opacity}, 0f, {even_odd}, \"butt\", \"miter\"")
        }
        SvgPathFill::RawStroke { opacity, width, line_cap, line_join, .. } | SvgPathFill::LiteralStroke { opacity, width, line_cap, line_join, .. } | SvgPathFill::Stroke { opacity, width, line_cap, line_join, .. } => {
            format!(
                "true, {opacity}, {}f, false, \"{}\", \"{}\"",
                width as f32 / 100.0,
                match line_cap { SvgLineCap::Butt => "butt", SvgLineCap::Round => "round", SvgLineCap::Square => "square" },
                match line_join { SvgLineJoin::Miter => "miter", SvgLineJoin::Round => "round", SvgLineJoin::Bevel => "bevel" }
            )
        }
        _ => "false, 255, 0f, false, \"butt\", \"miter\"".to_string(),
    }
}

fn dev_svg_path_transform(transform: Option<&SvgTransform>) -> String {
    transform
        .map(|value| {
            format!(
                "new float[] {{{}f, {}f, {}f, {}f, {}f, {}f}}",
                value.a, value.b, value.c, value.d, value.e, value.f
            )
        })
        .unwrap_or_else(|| "null".to_string())
}

fn dev_gap_expr(value: &GapValue, horizontal: bool) -> String {
    match value {
        GapValue::Single(value) => dev_gap_size(value),
        GapValue::Pair(row, column) => {
            if horizontal {
                dev_gap_size(column)
            } else {
                dev_gap_size(row)
            }
        }
    }
}

fn dev_gap_size(value: &GapSize) -> String {
    match value {
        GapSize::Scale(value) => value.native_units().to_string(),
        GapSize::Px(value) => value.to_string(),
    }
}

fn dev_android_navigation_action(action: Option<&NavigationAction>) -> Option<String> {
    match action {
        Some(NavigationAction::Internal {
            path,
            fragment,
            operation,
        }) => Some(format!(
            "doweNavigate(\"{}\", \"{}\", {})",
            operation.as_str(),
            escape_java(path),
            fragment
                .as_ref()
                .map(|value| format!("\"{}\"", escape_java(value)))
                .unwrap_or_else(|| "null".to_string())
        )),
        Some(NavigationAction::Section {
            fragment,
            operation,
        }) => Some(format!(
            "doweNavigate(\"{}\", currentPath, \"{}\")",
            operation.as_str(),
            escape_java(fragment)
        )),
        Some(NavigationAction::External {
            url,
            native_external_mode,
            ..
        }) => Some(format!(
            "doweOpenExternal(\"{}\", \"{}\")",
            native_external_mode.as_str(),
            escape_java(url)
        )),
        Some(NavigationAction::Back) => Some("doweBack()".to_string()),
        None => None,
    }
}

fn dev_text_expression(
    value: &str,
    i18n: Option<&str>,
    context: &ComposeReactiveContext,
) -> String {
    if let Some(key) = i18n {
        return format!("getString(R.string.{})", translation_resource_name(key));
    }
    match context.dynamic_path(value) {
        Some(path) => context
            .item_value(value)
            .map(|item| format!("doweTextValue(\"{}\", {item})", escape_java(&path)))
            .unwrap_or_else(|| format!("doweTextValue(\"{}\", null)", escape_java(&path))),
        None => format!("\"{}\"", escape_java(value)),
    }
}

fn dev_visible_text_expression(
    value: &str,
    i18n: Option<&str>,
    context: &ComposeReactiveContext,
) -> String {
    if let Some(key) = i18n {
        return format!("getString(R.string.{})", translation_resource_name(key));
    }
    let Some(binding) = text_binding_path(value) else {
        return format!("\"{}\"", escape_java(value));
    };
    match context.dynamic_path(binding) {
        Some(path) => context
            .item_value(binding)
            .map(|item| format!("doweTextValue(\"{}\", {item})", escape_java(&path)))
            .unwrap_or_else(|| format!("doweTextValue(\"{}\", null)", escape_java(&path))),
        None => format!("\"{}\"", escape_java(value)),
    }
}

fn dev_localized_literal(value: &str, i18n: Option<&str>) -> String {
    i18n
        .map(|key| format!("getString(R.string.{})", translation_resource_name(key)))
        .unwrap_or_else(|| format!("\"{}\"", escape_java(value)))
}

fn next_dev_view(counter: &mut usize) -> String {
    let value = format!("view{}", *counter);
    *counter += 1;
    value
}

fn apply_dev_android_style(
    props: &StyleProps,
    view: &str,
    include_background: bool,
    output: &mut String,
) {
    if let Some(id) = props.element.id.as_ref() {
        output.push_str(&format!(
            "        doweRegisterSection(\"{}\", {view});\n",
            escape_java(id)
        ));
    }

    let styled_background =
        include_background && props.background.is_none() && props.border.is_some();

    if include_background && !styled_background && let Some(value) = props.bg.as_ref() {
        output.push_str(&format!(
            "        Integer {view}Background = {};\n        if ({view}Background != null) {{\n            {view}.setBackgroundColor({view}Background);\n        }}\n",
            dev_color_value(value)
        ));
    }

    if include_background && let Some(value) = props.background.as_ref() {
        output.push_str(&format!(
            "        String {view}SectionBackground = {};\n        if ({view}SectionBackground != null) {{\n            {view}.setBackground(doweSectionBackground({view}SectionBackground));\n        }}\n",
            dev_section_background_value(value)
        ));
    }

    if styled_background {
        let background = props
            .bg
            .as_ref()
            .map(|value| format!("doweColor({}, Color.TRANSPARENT)", dev_color_value(value)))
            .unwrap_or_else(|| "Color.TRANSPARENT".to_string());
        let border = props
            .border
            .as_ref()
            .map(dev_border_value)
            .unwrap_or_else(|| "null".to_string());
        output.push_str(&format!(
            "        {view}.setBackground(doweStyledBackground({background}, DOWE_BACKGROUND_TEXT, {border}, {}));\n",
            dev_style_radius(props)
        ));
    }

    if props.spacing.p.is_some()
        || props.spacing.px.is_some()
        || props.spacing.py.is_some()
        || props.spacing.pl.is_some()
        || props.spacing.pr.is_some()
        || props.spacing.pt.is_some()
        || props.spacing.pb.is_some()
    {
        output.push_str(&format!(
            "        int {view}Left = 0;\n        int {view}Top = 0;\n        int {view}Right = 0;\n        int {view}Bottom = 0;\n"
        ));
        write_dev_android_padding(
            props.spacing.p.as_ref(),
            view,
            "Padding",
            DevPaddingEdges::All,
            output,
        );
        write_dev_android_padding(
            props.spacing.px.as_ref(),
            view,
            "PaddingX",
            DevPaddingEdges::Horizontal,
            output,
        );
        write_dev_android_padding(
            props.spacing.py.as_ref(),
            view,
            "PaddingY",
            DevPaddingEdges::Vertical,
            output,
        );
        write_dev_android_padding(
            props.spacing.pl.as_ref(),
            view,
            "PaddingLeft",
            DevPaddingEdges::Left,
            output,
        );
        write_dev_android_padding(
            props.spacing.pr.as_ref(),
            view,
            "PaddingRight",
            DevPaddingEdges::Right,
            output,
        );
        write_dev_android_padding(
            props.spacing.pt.as_ref(),
            view,
            "PaddingTop",
            DevPaddingEdges::Top,
            output,
        );
        write_dev_android_padding(
            props.spacing.pb.as_ref(),
            view,
            "PaddingBottom",
            DevPaddingEdges::Bottom,
            output,
        );
        output.push_str(&format!(
            "        {view}.setPadding({view}Left, {view}Top, {view}Right, {view}Bottom);\n"
        ));
    }

    if props.sizing.w.is_some() || props.sizing.h.is_some() {
        output.push_str(&format!(
            "        Integer {view}Width = {};\n        Integer {view}Height = {};\n        {view}.setLayoutParams(new LinearLayout.LayoutParams(doweDimension({view}Width), doweDimension({view}Height)));\n",
            dev_optional_size(props.sizing.w.as_ref()),
            dev_optional_size(props.sizing.h.as_ref())
        ));
    }

    if let Some(value) = props.sizing.min_w.as_ref() {
        output.push_str(&format!(
            "        Integer {view}MinWidth = {};\n        if ({view}MinWidth != null && {view}MinWidth != ViewGroup.LayoutParams.MATCH_PARENT) {{\n            {view}.setMinimumWidth(doweDp({view}MinWidth));\n        }}\n",
            dev_size_value(value)
        ));
    }
    if let Some(value) = props.sizing.min_h.as_ref() {
        output.push_str(&format!(
            "        Integer {view}MinHeight = {};\n        if ({view}MinHeight != null && {view}MinHeight != ViewGroup.LayoutParams.MATCH_PARENT) {{\n            {view}.setMinimumHeight(doweDp({view}MinHeight));\n        }}\n",
            dev_size_value(value)
        ));
    }
    if props.sizing.max_w.is_some() || props.sizing.max_h.is_some() {
        output.push_str(&format!(
            "        doweConstrain({view}, {}, {});\n",
            dev_optional_size(props.sizing.max_w.as_ref()),
            dev_optional_size(props.sizing.max_h.as_ref())
        ));
    }

    apply_dev_android_shadow(props, view, output);

    if let Some(value) = props.rounded.as_ref() {
        output.push_str(&format!(
            "        doweRound({view}, {});\n",
            dev_rounded_value(value)
        ));
    }

    let motion = props.motion();
    if let Some(value) = motion.rotate.as_ref() {
        output.push_str(&format!(
            "        {view}.setRotation({});\n",
            dev_responsive_float_value(value, |value| format!("{}f", value.degrees()))
        ));
    }
    if let Some(value) = motion.scale.as_ref() {
        let scale = dev_responsive_float_value(value, |value| format!("{}f", value.factor()));
        output.push_str(&format!(
            "        {view}.setScaleX({scale});\n        {view}.setScaleY({scale});\n"
        ));
    }
    if let Some(value) = motion.translate_x.as_ref() {
        output.push_str(&format!(
            "        {view}.setTranslationX(doweDp({}));\n",
            dev_responsive_value(value, |value| value.native_units().to_string())
        ));
    }
    if let Some(value) = motion.translate_y.as_ref() {
        output.push_str(&format!(
            "        {view}.setTranslationY(doweDp({}));\n",
            dev_responsive_value(value, |value| value.native_units().to_string())
        ));
    }
    if let Some(gesture) = motion.gesture
        && gesture != ViewGesture::None
    {
        output.push_str(&format!(
            "        doweGesture({view}, \"{}\", \"{}\");\n",
            gesture.as_str(),
            motion.transition.unwrap_or(ViewTransition::Smooth).as_str()
        ));
    }

    if let Some(animation) = props.animation() {
        output.push_str(&format!(
            "        doweAnimate({view}, \"{}\");\n",
            animation.as_str()
        ));
    }
}

fn apply_dev_android_click(
    props: &StyleProps,
    view: &str,
    context: &ComposeReactiveContext,
    output: &mut String,
) {
    if let Some(action) = props
        .element
        .on_click
        .as_deref()
        .and_then(|name| context.action_id(name))
    {
        output.push_str(&format!(
            "        {view}.setOnClickListener(v -> doweRunAction(\"{}\", null));\n",
            escape_java(action)
        ));
    }
}

fn apply_dev_android_shadow(props: &StyleProps, view: &str, output: &mut String) {
    apply_dev_android_shadow_with_radius(props, view, &dev_style_radius(props), output);
}

fn apply_dev_android_shadow_with_radius(
    props: &StyleProps,
    view: &str,
    corner_radius: &str,
    output: &mut String,
) {
    if let Some(value) = props.shadow.as_ref() {
        let color = props
            .shadow_color
            .map(family_color)
            .map(java_color)
            .unwrap_or("Color.BLACK");
        let alpha = if props.shadow_color.is_some() {
            "0.28f"
        } else {
            "null"
        };
        output.push_str(&format!(
            "        doweShadow({view}, {}, {color}, {}, {alpha});\n",
            dev_responsive_value(value, |value| shadow_dp(*value).to_string()),
            corner_radius
        ));
    }
}

enum DevPaddingEdges {
    All,
    Horizontal,
    Vertical,
    Left,
    Right,
    Top,
    Bottom,
}

fn write_dev_android_padding(
    value: Option<&ResponsiveValue<ScaleValue>>,
    view: &str,
    suffix: &str,
    edges: DevPaddingEdges,
    output: &mut String,
) {
    let Some(value) = value else {
        return;
    };
    let name = format!("{view}{suffix}");
    output.push_str(&format!(
        "        Integer {name} = {};\n        if ({name} != null) {{\n",
        dev_scale_value(value)
    ));
    match edges {
        DevPaddingEdges::All => output.push_str(&format!(
            "            int value = doweDp({name});\n            {view}Left = value;\n            {view}Top = value;\n            {view}Right = value;\n            {view}Bottom = value;\n"
        )),
        DevPaddingEdges::Horizontal => output.push_str(&format!(
            "            int value = doweDp({name});\n            {view}Left = value;\n            {view}Right = value;\n"
        )),
        DevPaddingEdges::Vertical => output.push_str(&format!(
            "            int value = doweDp({name});\n            {view}Top = value;\n            {view}Bottom = value;\n"
        )),
        DevPaddingEdges::Left => {
            output.push_str(&format!("            {view}Left = doweDp({name});\n"));
        }
        DevPaddingEdges::Right => {
            output.push_str(&format!("            {view}Right = doweDp({name});\n"));
        }
        DevPaddingEdges::Top => {
            output.push_str(&format!("            {view}Top = doweDp({name});\n"));
        }
        DevPaddingEdges::Bottom => {
            output.push_str(&format!("            {view}Bottom = doweDp({name});\n"));
        }
    }
    output.push_str("        }\n");
}
