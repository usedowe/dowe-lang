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

fn apply_dev_android_flex_item(props: &StyleProps, parent: &str, view: &str, output: &mut String) {
    let Some(value) = props.flex.as_ref() else {
        return;
    };
    output.push_str(&format!(
        "        Integer {view}Flex = {};\n        doweApplyFlexItem({parent}, {view}, {view}Flex);\n",
        dev_flex_item(value)
    ));
}

fn dev_flex_item(value: &ResponsiveValue<FlexItem>) -> String {
    dev_responsive_value(value, |value| match value {
        FlexItem::Initial => "DOWE_FLEX_INITIAL".to_string(),
        FlexItem::Auto => "DOWE_FLEX_AUTO".to_string(),
        FlexItem::None => "DOWE_FLEX_NONE".to_string(),
        FlexItem::Fill => "DOWE_FLEX_FILL".to_string(),
    })
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
        Justify::Stretch | Justify::Normal => "DOWE_JUSTIFY_START".to_string(),
        Justify::EndSafe => "DOWE_JUSTIFY_END".to_string(),
        Justify::CenterSafe => "DOWE_JUSTIFY_CENTER".to_string(),
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
        Align::BaselineLast => "DOWE_ALIGN_BASELINE".to_string(),
        Align::EndSafe => "DOWE_ALIGN_END".to_string(),
        Align::CenterSafe => "DOWE_ALIGN_CENTER".to_string(),
    })
}

fn dev_text_align(value: &ResponsiveValue<TextAlign>) -> String {
    dev_responsive_value(value, |value| match value {
        TextAlign::Start => "Gravity.TOP | Gravity.START".to_string(),
        TextAlign::Center => "Gravity.TOP | Gravity.CENTER_HORIZONTAL".to_string(),
        TextAlign::End => "Gravity.TOP | Gravity.END".to_string(),
        TextAlign::Justify => "Gravity.TOP | Gravity.START".to_string(),
    })
}

fn dev_text_justify(value: &ResponsiveValue<TextAlign>) -> String {
    dev_responsive_value(value, |value| match value {
        TextAlign::Justify => "1".to_string(),
        TextAlign::Start | TextAlign::Center | TextAlign::End => "0".to_string(),
    })
}

fn apply_dev_text_alignment(
    props: &TextProps,
    view: &str,
    parent_horizontal: bool,
    output: &mut String,
) {
    let Some(value) = props.align.as_ref() else {
        return;
    };
    if props.style.sizing.w.is_none() && !parent_horizontal {
        output.push_str(&format!(
            "        {view}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n"
        ));
    }
    output.push_str(&format!(
        "        {view}.setGravity({});\n",
        dev_text_align(value)
    ));
    output.push_str(&format!(
        "        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O && {} == 1) {{ {view}.setJustificationMode(android.text.Layout.JUSTIFICATION_MODE_INTER_WORD); }}\n",
        dev_text_justify(value)
    ));
}

fn dev_grid_tracks(value: Option<&ResponsiveValue<GridTracks>>) -> String {
    value
        .map(|value| {
            dev_responsive_tracks(value, |value| match value {
                GridTracks::Count(count) => {
                    format!("new float[]{{{}}}", vec!["1f"; *count as usize].join(", "))
                }
                GridTracks::Fractions(weights) => format!(
                    "new float[]{{{}}}",
                    weights
                        .iter()
                        .map(|weight| format!("{weight}f"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                GridTracks::Auto => "new float[]{1f}".to_string(),
            })
        })
        .unwrap_or_else(|| "new float[]{1f}".to_string())
}

fn dev_responsive_tracks<T, F>(value: &ResponsiveValue<T>, map: F) -> String
where
    F: Fn(&T) -> String,
{
    format!(
        "doweResponsiveTracks(viewportWidth, {})",
        dev_responsive_args(value, map)
    )
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
    format!("{text}|{title}")
}

fn dev_inherited_text_color(value: Option<&str>) -> Option<&str> {
    value.map(|value| value.split_once('|').map_or(value, |colors| colors.0))
}

fn dev_inherited_content_color(props: &StyleProps, inherited_color: Option<&str>) -> String {
    dev_inherited_color(props, inherited_color)
        .as_deref()
        .and_then(|value| dev_inherited_text_color(Some(value)))
        .unwrap_or("DOWE_BACKGROUND_TEXT")
        .to_string()
}

fn dev_inherited_title_color(value: Option<&str>) -> Option<&str> {
    value.map(|value| value.split_once('|').map_or(value, |colors| colors.1))
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
        SvgPathFill::CurrentColor
        | SvgPathFill::Fill { color: None, .. }
        | SvgPathFill::Stroke { color: None, .. } => "true",
        SvgPathFill::None
        | SvgPathFill::Color(_)
        | SvgPathFill::RawFill { .. }
        | SvgPathFill::RawStroke { .. }
        | SvgPathFill::LiteralFill { .. }
        | SvgPathFill::LiteralStroke { .. }
        | SvgPathFill::Fill { color: Some(_), .. }
        | SvgPathFill::Stroke { color: Some(_), .. } => "false",
    }
}

fn dev_svg_path_color(fill: SvgPathFill) -> String {
    match fill {
        SvgPathFill::None | SvgPathFill::CurrentColor => "null".to_string(),
        SvgPathFill::RawFill { color, .. } | SvgPathFill::RawStroke { color, .. } => {
            android_java_color_literal(color)
        }
        SvgPathFill::LiteralFill {
            red, green, blue, ..
        }
        | SvgPathFill::LiteralStroke {
            red, green, blue, ..
        } => {
            format!("Color.rgb({red}, {green}, {blue})")
        }
        SvgPathFill::Color(token)
        | SvgPathFill::Fill {
            color: Some(token), ..
        }
        | SvgPathFill::Stroke {
            color: Some(token), ..
        } => java_color(token).to_string(),
        SvgPathFill::Fill { color: None, .. } | SvgPathFill::Stroke { color: None, .. } => {
            "null".to_string()
        }
    }
}

fn dev_svg_path_details(fill: SvgPathFill) -> String {
    match fill {
        SvgPathFill::RawFill {
            opacity, even_odd, ..
        }
        | SvgPathFill::LiteralFill {
            opacity, even_odd, ..
        }
        | SvgPathFill::Fill {
            opacity, even_odd, ..
        } => {
            format!("false, {opacity}, 0f, {even_odd}, \"butt\", \"miter\"")
        }
        SvgPathFill::RawStroke {
            opacity,
            width,
            line_cap,
            line_join,
            ..
        }
        | SvgPathFill::LiteralStroke {
            opacity,
            width,
            line_cap,
            line_join,
            ..
        }
        | SvgPathFill::Stroke {
            opacity,
            width,
            line_cap,
            line_join,
            ..
        } => {
            format!(
                "true, {opacity}, {}f, false, \"{}\", \"{}\"",
                width as f32 / 100.0,
                match line_cap {
                    SvgLineCap::Butt => "butt",
                    SvgLineCap::Round => "round",
                    SvgLineCap::Square => "square",
                },
                match line_join {
                    SvgLineJoin::Miter => "miter",
                    SvgLineJoin::Round => "round",
                    SvgLineJoin::Bevel => "bevel",
                }
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
    let segments = text_template_segments(value);
    if segments.iter().all(|(_, binding)| binding.is_none()) {
        return format!("\"{}\"", escape_java(value));
    }
    segments
        .into_iter()
        .map(|(literal, binding)| match binding {
            Some(binding) => match context.dynamic_path(&binding) {
                Some(path) => context
                    .item_value(&binding)
                    .map(|item| format!("doweTextValue(\"{}\", {item})", escape_java(&path)))
                    .unwrap_or_else(|| format!("doweTextValue(\"{}\", null)", escape_java(&path))),
                None => format!("\"{{{}}}\"", escape_java(&binding)),
            },
            None => format!("\"{}\"", escape_java(&literal)),
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

fn dev_localized_literal(value: &str, i18n: Option<&str>) -> String {
    i18n.map(|key| format!("getString(R.string.{})", translation_resource_name(key)))
        .unwrap_or_else(|| format!("\"{}\"", escape_java(value)))
}

fn next_dev_view(counter: &mut usize) -> String {
    let value = format!("view{}", *counter);
    *counter += 1;
    value
}

fn dev_android_style_tag(property: &str) -> &'static str {
    match property {
        "p" => "DOWE_STYLE_P_TAG",
        "px" => "DOWE_STYLE_PX_TAG",
        "py" => "DOWE_STYLE_PY_TAG",
        "pl" => "DOWE_STYLE_PL_TAG",
        "pr" => "DOWE_STYLE_PR_TAG",
        "pt" => "DOWE_STYLE_PT_TAG",
        "pb" => "DOWE_STYLE_PB_TAG",
        "w" => "DOWE_STYLE_W_TAG",
        "h" => "DOWE_STYLE_H_TAG",
        "minW" => "DOWE_STYLE_MIN_W_TAG",
        "minH" => "DOWE_STYLE_MIN_H_TAG",
        "maxW" => "DOWE_STYLE_MAX_W_TAG",
        "maxH" => "DOWE_STYLE_MAX_H_TAG",
        "border" => "DOWE_STYLE_BORDER_TAG",
        "rounded" => "DOWE_STYLE_ROUNDED_TAG",
        "bg" => "DOWE_STYLE_BG_TAG",
        _ => "DOWE_STYLE_COLOR_TAG",
    }
}

