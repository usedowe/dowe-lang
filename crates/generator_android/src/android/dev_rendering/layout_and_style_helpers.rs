fn dev_add(parent: &str, view: &str, gap: Option<&str>, horizontal: bool) -> String {
    match gap {
        Some(gap) => format!(
            "        doweAdd({parent}, {view}, {gap}, {});\n",
            if horizontal { "true" } else { "false" }
        ),
        None => format!("        doweAdd({parent}, {view});\n"),
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
        .or_else(|| inherited_color.map(str::to_string))
}

fn dev_svg_color(props: &StyleProps, inherited_color: Option<&str>) -> String {
    let fallback = inherited_color.unwrap_or("DOWE_ON_BACKGROUND");
    props
        .text
        .as_ref()
        .map(dev_color_value)
        .map(|value| format!("doweColor({value}, {fallback})"))
        .unwrap_or_else(|| fallback.to_string())
}

fn dev_svg_path_current_color(fill: SvgPathFill) -> &'static str {
    match fill {
        SvgPathFill::CurrentColor => "true",
        SvgPathFill::None | SvgPathFill::Color(_) => "false",
    }
}

fn dev_svg_path_color(fill: SvgPathFill) -> String {
    match fill {
        SvgPathFill::None | SvgPathFill::CurrentColor => "null".to_string(),
        SvgPathFill::Color(token) => java_color(token).to_string(),
    }
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

fn dev_route_method_name(route: &str) -> String {
    format!("render{}", pascal_route(route))
}

fn dev_route_page_method_name(route: &str) -> String {
    format!("{}Page", dev_route_method_name(route))
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

    if include_background && let Some(value) = props.bg.as_ref() {
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

    if props.spacing.p.is_some()
        || props.spacing.px.is_some()
        || props.spacing.py.is_some()
        || props.spacing.pl.is_some()
        || props.spacing.pr.is_some()
        || props.spacing.pt.is_some()
        || props.spacing.pb.is_some()
    {
        output.push_str(&format!(
            "        int {view}Left = 0;\n        int {view}Top = 0;\n        int {view}Right = 0;\n        int {view}Bottom = 0;\n        Integer {view}Padding = {};\n        if ({view}Padding != null) {{\n            int value = doweDp({view}Padding);\n            {view}Left = value;\n            {view}Top = value;\n            {view}Right = value;\n            {view}Bottom = value;\n        }}\n        Integer {view}PaddingX = {};\n        if ({view}PaddingX != null) {{\n            int value = doweDp({view}PaddingX);\n            {view}Left = value;\n            {view}Right = value;\n        }}\n        Integer {view}PaddingY = {};\n        if ({view}PaddingY != null) {{\n            int value = doweDp({view}PaddingY);\n            {view}Top = value;\n            {view}Bottom = value;\n        }}\n        Integer {view}PaddingLeft = {};\n        if ({view}PaddingLeft != null) {{\n            {view}Left = doweDp({view}PaddingLeft);\n        }}\n        Integer {view}PaddingRight = {};\n        if ({view}PaddingRight != null) {{\n            {view}Right = doweDp({view}PaddingRight);\n        }}\n        Integer {view}PaddingTop = {};\n        if ({view}PaddingTop != null) {{\n            {view}Top = doweDp({view}PaddingTop);\n        }}\n        Integer {view}PaddingBottom = {};\n        if ({view}PaddingBottom != null) {{\n            {view}Bottom = doweDp({view}PaddingBottom);\n        }}\n        {view}.setPadding({view}Left, {view}Top, {view}Right, {view}Bottom);\n",
            dev_optional_scale(props.spacing.p.as_ref()),
            dev_optional_scale(props.spacing.px.as_ref()),
            dev_optional_scale(props.spacing.py.as_ref()),
            dev_optional_scale(props.spacing.pl.as_ref()),
            dev_optional_scale(props.spacing.pr.as_ref()),
            dev_optional_scale(props.spacing.pt.as_ref()),
            dev_optional_scale(props.spacing.pb.as_ref())
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

    if let Some(animation) = props.animation {
        output.push_str(&format!(
            "        doweAnimate({view}, \"{}\");\n",
            animation.as_str()
        ));
    }
}
