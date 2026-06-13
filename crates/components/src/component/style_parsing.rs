fn parse_style_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
    mode: StylePropMode,
) -> ComponentResult<StyleProps> {
    let mut style = StyleProps::default();

    for prop in props {
        match prop.name.as_str() {
            "id" => style.element.id = Some(parse_id_prop(&prop.name, &prop.value)?),
            "show" => style.element.show = Some(parse_show_prop(&prop.name, &prop.value)?),
            "font" => {
                let font = parse_font_prop(&prop.name, &prop.value)?;
                style.element.font = Some(font.clone());
                style.font = Some(font);
            }
            "bind"
                if matches!(
                    component,
                    BuiltinComponent::Input
                        | BuiltinComponent::Select
                        | BuiltinComponent::Slider
                        | BuiltinComponent::Checkbox
                        | BuiltinComponent::Color
                        | BuiltinComponent::Date
                        | BuiltinComponent::RadioGroup
                        | BuiltinComponent::Toggle
                ) =>
            {
                style.element.bind = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "onClick" if matches!(component, BuiltinComponent::Button | BuiltinComponent::Avatar | BuiltinComponent::Fab | BuiltinComponent::Empty) => {
                style.element.on_click = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "bg" if style_accepts_colors(mode) => {
                style.bg = Some(parse_color_prop(&prop.name, &prop.value)?)
            }
            "color" if style_accepts_colors(mode) => {
                style.text = Some(parse_color_prop(&prop.name, &prop.value)?)
            }
            "cover" if style_accepts_cover(mode) => {
                style.cover = Some(parse_cover_prop(&prop.name, &prop.value)?)
            }
            "overlay" if style_accepts_cover(mode) => {
                style.overlay = Some(parse_overlay_prop(&prop.name, &prop.value)?)
            }
            "background" if style_accepts_background(mode) => {
                style.background = Some(parse_background_prop(&prop.name, &prop.value)?)
            }
            "animation" if style_accepts_animation(mode) => {
                style.animation = Some(parse_animation_prop(&prop.name, &prop.value)?)
            }
            "colSpan" if style_accepts_grid_item(mode) => {
                style.grid_item.col_span = Some(parse_span_prop(&prop.name, &prop.value)?)
            }
            "rowSpan" if style_accepts_grid_item(mode) => {
                style.grid_item.row_span = Some(parse_span_prop(&prop.name, &prop.value)?)
            }
            "p" => style.spacing.p = Some(parse_scale_prop(&prop.name, &prop.value)?),
            "px" => style.spacing.px = Some(parse_scale_prop(&prop.name, &prop.value)?),
            "py" => style.spacing.py = Some(parse_scale_prop(&prop.name, &prop.value)?),
            "pl" => style.spacing.pl = Some(parse_scale_prop(&prop.name, &prop.value)?),
            "pr" => style.spacing.pr = Some(parse_scale_prop(&prop.name, &prop.value)?),
            "pt" => style.spacing.pt = Some(parse_scale_prop(&prop.name, &prop.value)?),
            "pb" => style.spacing.pb = Some(parse_scale_prop(&prop.name, &prop.value)?),
            "w" => style.sizing.w = Some(parse_size_prop(&prop.name, &prop.value)?),
            "h" => style.sizing.h = Some(parse_size_prop(&prop.name, &prop.value)?),
            "minW" => style.sizing.min_w = Some(parse_size_prop(&prop.name, &prop.value)?),
            "minH" => style.sizing.min_h = Some(parse_size_prop(&prop.name, &prop.value)?),
            "rounded" => style.rounded = Some(parse_rounded_prop(&prop.name, &prop.value)?),
            "border" => style.border = Some(parse_border_prop(&prop.name, &prop.value)?),
            _ => return Err(ComponentError::unknown_prop(component, &prop.name)),
        }
    }

    if style.cover.is_some() && style.background.is_some() {
        return Err(ComponentError::invalid_prop_combination(format!(
            "`cover` and `background` cannot be used together on `{}`",
            component.as_str()
        )));
    }

    if style.overlay.is_some() && style.cover.is_none() {
        return Err(ComponentError::invalid_prop_combination(format!(
            "`overlay` requires `cover` on `{}`",
            component.as_str()
        )));
    }

    Ok(style)
}

fn style_accepts_colors(mode: StylePropMode) -> bool {
    matches!(
        mode,
        StylePropMode::Box | StylePropMode::Section | StylePropMode::Text
    )
}

fn style_accepts_cover(mode: StylePropMode) -> bool {
    matches!(
        mode,
        StylePropMode::Box | StylePropMode::Section | StylePropMode::Card
    )
}

fn style_accepts_background(mode: StylePropMode) -> bool {
    matches!(mode, StylePropMode::Section)
}

fn style_accepts_grid_item(mode: StylePropMode) -> bool {
    matches!(
        mode,
        StylePropMode::Box | StylePropMode::Section | StylePropMode::Card
    )
}

fn style_accepts_animation(mode: StylePropMode) -> bool {
    matches!(
        mode,
        StylePropMode::Box | StylePropMode::Section | StylePropMode::Card
    )
}

fn parse_layout_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<LayoutProps> {
    let mut layout = LayoutProps::default();
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "justify" => layout.justify = Some(parse_justify_prop(&prop.name, &prop.value)?),
            "align" => layout.align = Some(parse_align_prop(&prop.name, &prop.value)?),
            "gap" => layout.gap = Some(parse_gap_prop(&prop.name, &prop.value, false)?),
            _ => style_props.push(prop.clone()),
        }
    }

    layout.style = parse_style_props(component, &style_props, StylePropMode::Layout)?;
    Ok(layout)
}

fn parse_grid_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<GridProps> {
    let mut grid = GridProps::default();
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "columns" => {
                grid.columns = Some(parse_grid_tracks_prop(&prop.name, &prop.value, false)?)
            }
            "rows" => grid.rows = Some(parse_grid_tracks_prop(&prop.name, &prop.value, true)?),
            "justify" => grid.justify = Some(parse_grid_alignment_prop(&prop.name, &prop.value)?),
            "align" => grid.align = Some(parse_grid_alignment_prop(&prop.name, &prop.value)?),
            "gap" => grid.gap = Some(parse_gap_prop(&prop.name, &prop.value, true)?),
            _ => style_props.push(prop.clone()),
        }
    }

    grid.style = parse_style_props(component, &style_props, StylePropMode::Grid)?;
    Ok(grid)
}

fn parse_variant_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<VariantProps> {
    let mut variant_props = VariantProps::default();
    let mut style_props = Vec::new();
    let mut href = None;
    let mut navigate = None;
    let mut history = None;
    let mut target = None;
    let mut external_mode = None;

    for prop in props {
        match prop.name.as_str() {
            "variant" => variant_props.variant = Some(parse_variant_prop(&prop.name, &prop.value)?),
            "scheme" => {
                variant_props.color = Some(parse_family_prop(component, &prop.name, &prop.value)?)
            }
            "size"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::Chip
                        | BuiltinComponent::AvatarGroup
                        | BuiltinComponent::ToggleTheme
                        | BuiltinComponent::Fab
                ) =>
            {
                variant_props.size = Some(parse_button_size_prop(&prop.name, &prop.value)?)
            }
            "label"
                if matches!(
                    component,
                    BuiltinComponent::Input
                        | BuiltinComponent::Select
                        | BuiltinComponent::Checkbox
                        | BuiltinComponent::Color
                        | BuiltinComponent::Date
                        | BuiltinComponent::DateRange
                        | BuiltinComponent::RadioGroup
                        | BuiltinComponent::Toggle
                        | BuiltinComponent::Slider
                        | BuiltinComponent::Dropzone
                        | BuiltinComponent::Fab
                ) =>
            {
                variant_props.label = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "placeholder"
                if matches!(
                    component,
                    BuiltinComponent::Input
                        | BuiltinComponent::Select
                        | BuiltinComponent::Color
                        | BuiltinComponent::Date
                        | BuiltinComponent::DateRange
                        | BuiltinComponent::Dropzone
                ) =>
            {
                variant_props.placeholder = Some(parse_static_string(&prop.name, &prop.value)?)
            }
            "labelFloating"
                if matches!(
                    component,
                    BuiltinComponent::Input
                        | BuiltinComponent::Select
                        | BuiltinComponent::Color
                        | BuiltinComponent::Date
                        | BuiltinComponent::DateRange
                ) =>
            {
                variant_props.label_floating = parse_static_bool(&prop.name, &prop.value)?
            }
            "href" if matches!(component, BuiltinComponent::Button | BuiltinComponent::Avatar | BuiltinComponent::Empty) => {
                href = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "navigate" if matches!(component, BuiltinComponent::Button | BuiltinComponent::Avatar | BuiltinComponent::Empty) => {
                navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?)
            }
            "history" if matches!(component, BuiltinComponent::Button | BuiltinComponent::Avatar | BuiltinComponent::Empty) => {
                history = Some(parse_history_prop(&prop.name, &prop.value)?)
            }
            "target" if matches!(component, BuiltinComponent::Button | BuiltinComponent::Avatar | BuiltinComponent::Empty) => {
                target = Some(parse_web_target(&prop.name, &prop.value)?)
            }
            "externalMode" if matches!(component, BuiltinComponent::Button | BuiltinComponent::Avatar | BuiltinComponent::Empty) => {
                external_mode = Some(parse_native_external_mode(&prop.name, &prop.value)?)
            }
            _ => style_props.push(prop.clone()),
        }
    }

    variant_props.style = parse_style_props(
        component,
        &style_props,
        if component == BuiltinComponent::Card {
            StylePropMode::Card
        } else {
            StylePropMode::Variant
        },
    )?;
    if component == BuiltinComponent::Button {
        normalize_button_visual_props(&mut variant_props);
    }
    variant_props.element = variant_props.style.element.clone();
    variant_props.navigation =
        parse_navigation_props(component, href, navigate, history, target, external_mode)?;
    Ok(variant_props)
}

fn parse_bar_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<BarProps> {
    let mut bar = BarProps::default();
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "bordered" => bar.bordered = parse_static_bool(&prop.name, &prop.value)?,
            "blurred" => bar.blurred = parse_static_bool(&prop.name, &prop.value)?,
            "boxed" => bar.boxed = parse_static_bool(&prop.name, &prop.value)?,
            "floating"
                if matches!(
                    component,
                    BuiltinComponent::AppBar | BuiltinComponent::BottomBar
                ) =>
            {
                bar.floating = parse_static_bool(&prop.name, &prop.value)?
            }
            _ => style_props.push(prop.clone()),
        }
    }

    let mut style = parse_variant_props(component, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Solid);
    style.color.get_or_insert(ColorFamily::Surface);
    bar.style = style;
    Ok(bar)
}

fn parse_tabs_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<TabsProps> {
    let mut variant = TabsVariant::Solid;
    let mut color = ColorFamily::Muted;
    let mut position = TabsPosition::Top;
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "variant" => {
                variant = parse_tabs_variant_prop(&prop.name, &prop.value)?;
            }
            "scheme" => {
                color = parse_family_prop(component, &prop.name, &prop.value)?;
            }
            "position" => {
                position = parse_tabs_position_prop(&prop.name, &prop.value)?;
            }
            "color" => {
                return Err(ComponentError::new(
                    "unknown prop `color` on `Tabs`; use `scheme` for visual family",
                ));
            }
            _ => style_props.push(prop.clone()),
        }
    }

    Ok(TabsProps {
        style: parse_style_props(component, &style_props, StylePropMode::Variant)?,
        variant,
        color,
        position,
    })
}

fn normalize_button_visual_props(props: &mut VariantProps) {
    props.variant.get_or_insert(ComponentVariant::Solid);
    props.color.get_or_insert(ColorFamily::Primary);
    let size = *props.size.get_or_insert(ButtonSize::Md);
    apply_button_size_defaults(&mut props.style, size);
}

fn apply_button_size_defaults(style: &mut StyleProps, size: ButtonSize) {
    if style.spacing.p.is_none() {
        apply_horizontal_button_padding(&mut style.spacing, size.padding_x());
        apply_vertical_button_padding(&mut style.spacing, size.padding_y());
    }

    if style.sizing.h.is_none() && style.sizing.min_h.is_none() {
        style.sizing.min_h = Some(ResponsiveValue::scalar(SizeValue::Scale(size.min_height())));
    }
}

fn apply_horizontal_button_padding(spacing: &mut SpacingProps, value: ScaleValue) {
    if spacing.px.is_some() {
        return;
    }

    match (spacing.pl.is_some(), spacing.pr.is_some()) {
        (false, false) => spacing.px = Some(ResponsiveValue::scalar(value)),
        (false, true) => spacing.pl = Some(ResponsiveValue::scalar(value)),
        (true, false) => spacing.pr = Some(ResponsiveValue::scalar(value)),
        (true, true) => {}
    }
}

fn apply_vertical_button_padding(spacing: &mut SpacingProps, value: ScaleValue) {
    if spacing.py.is_some() {
        return;
    }

    match (spacing.pt.is_some(), spacing.pb.is_some()) {
        (false, false) => spacing.py = Some(ResponsiveValue::scalar(value)),
        (false, true) => spacing.pt = Some(ResponsiveValue::scalar(value)),
        (true, false) => spacing.pb = Some(ResponsiveValue::scalar(value)),
        (true, true) => {}
    }
}

fn parse_text_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<TextProps> {
    let mut text = TextProps::default();
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "size" => text.size = Some(parse_text_size_prop(&prop.name, &prop.value)?),
            "weight" => text.weight = Some(parse_text_weight_prop(&prop.name, &prop.value)?),
            "spacing" => {
                text.letter_spacing = Some(parse_text_spacing_prop(&prop.name, &prop.value)?)
            }
            "i18n" => text.i18n = Some(parse_i18n_key_prop(&prop.name, &prop.value)?),
            _ => style_props.push(prop.clone()),
        }
    }

    text.style = parse_style_props(component, &style_props, StylePropMode::Text)?;

    Ok(text)
}

fn parse_svg_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<SvgProps> {
    let mut style = StyleProps::default();
    let mut view_box = None;

    for prop in props {
        match prop.name.as_str() {
            "id" => style.element.id = Some(parse_id_prop(&prop.name, &prop.value)?),
            "show" => style.element.show = Some(parse_show_prop(&prop.name, &prop.value)?),
            "viewBox" => view_box = Some(parse_svg_view_box(&prop.name, &prop.value)?),
            "color" => style.text = Some(parse_color_prop(&prop.name, &prop.value)?),
            "w" => style.sizing.w = Some(parse_size_prop(&prop.name, &prop.value)?),
            "h" => style.sizing.h = Some(parse_size_prop(&prop.name, &prop.value)?),
            _ => return Err(ComponentError::unknown_prop(component, &prop.name)),
        }
    }

    if style.sizing.w.is_none() {
        style.sizing.w = Some(ResponsiveValue::scalar(SizeValue::Scale(
            ScaleValue::from_half_steps(12),
        )));
    }
    if style.sizing.h.is_none() {
        style.sizing.h = Some(ResponsiveValue::scalar(SizeValue::Scale(
            ScaleValue::from_half_steps(12),
        )));
    }

    Ok(SvgProps {
        style,
        view_box: view_box
            .ok_or_else(|| ComponentError::invalid_prop("viewBox", "four numbers"))?,
    })
}

fn parse_svg_view_box(name: &str, value: &PropValue) -> ComponentResult<SvgViewBox> {
    let value = parse_required_string(name, value)?;
    let parts = value
        .split(|value: char| value.is_whitespace() || value == ',')
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let [min_x, min_y, width, height] = parts.as_slice() else {
        return Err(ComponentError::invalid_prop(name, "four numbers"));
    };
    if !is_svg_number(min_x)
        || !is_svg_number(min_y)
        || !is_positive_svg_number(width)
        || !is_positive_svg_number(height)
    {
        return Err(ComponentError::invalid_prop(
            name,
            "four numbers with positive width and height",
        ));
    }
    Ok(SvgViewBox {
        min_x: normalize_svg_number(min_x),
        min_y: normalize_svg_number(min_y),
        width: normalize_svg_number(width),
        height: normalize_svg_number(height),
    })
}

fn parse_svg_path_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<SvgPath> {
    let mut data = None;
    let mut fill = None;

    for prop in props {
        match prop.name.as_str() {
            "d" => data = Some(parse_svg_path_data(&prop.name, &prop.value)?),
            "fill" => fill = Some(parse_svg_path_fill(&prop.name, &prop.value)?),
            _ => return Err(ComponentError::unknown_prop(component, &prop.name)),
        }
    }

    Ok(SvgPath {
        data: data.ok_or_else(|| ComponentError::invalid_prop("d", "static SVG path data"))?,
        fill: fill.unwrap_or(SvgPathFill::CurrentColor),
    })
}

fn parse_svg_path_fill(name: &str, value: &PropValue) -> ComponentResult<SvgPathFill> {
    let value = parse_required_string(name, value)?;
    match value.as_str() {
        "none" => Ok(SvgPathFill::None),
        "currentColor" => Ok(SvgPathFill::CurrentColor),
        _ => ColorToken::from_name(&value)
            .map(SvgPathFill::Color)
            .ok_or_else(|| ComponentError::invalid_prop(name, "currentColor, none or color token")),
    }
}

fn parse_svg_path_data(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if value.chars().all(is_svg_path_character) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "portable SVG path data"))
    }
}

fn is_svg_number(value: &str) -> bool {
    value.parse::<f32>().is_ok()
}

fn is_positive_svg_number(value: &str) -> bool {
    value.parse::<f32>().ok().is_some_and(|value| value > 0.0)
}

fn normalize_svg_number(value: &str) -> String {
    let mut output = value.trim().to_string();
    if output.ends_with(".0") {
        output.truncate(output.len() - 2);
    }
    output
}

fn is_svg_path_character(value: char) -> bool {
    value.is_ascii_digit()
        || value.is_ascii_whitespace()
        || matches!(
            value,
            'M' | 'm'
                | 'Z'
                | 'z'
                | 'L'
                | 'l'
                | 'H'
                | 'h'
                | 'V'
                | 'v'
                | 'C'
                | 'c'
                | 'S'
                | 's'
                | 'Q'
                | 'q'
                | 'T'
                | 't'
                | 'A'
                | 'a'
                | 'E'
                | 'e'
                | '.'
                | ','
                | '-'
                | '+'
        )
}

fn parse_color_prop(name: &str, value: &PropValue) -> ComponentResult<ResponsiveValue<ColorToken>> {
    parse_responsive(name, value, "color token", |scalar| match scalar {
        PropScalar::String(value) => ColorToken::from_name(value),
        PropScalar::Number(_) | PropScalar::Boolean(_) => None,
    })
}

fn parse_font_prop(name: &str, value: &PropValue) -> ComponentResult<ResponsiveValue<FontFamily>> {
    parse_responsive(
        name,
        value,
        "system, inter, roboto, montserrat, lato, poppins, manrope, quicksand or lora",
        |scalar| match scalar {
            PropScalar::String(value) => FontFamily::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_cover_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<CoverSource>> {
    parse_responsive(
        name,
        value,
        "asset path or https URL",
        |scalar| match scalar {
            PropScalar::String(value) => parse_cover_source(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_overlay_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<OverlayPaint>> {
    parse_responsive(
        name,
        value,
        "boolean, opacity from 0 to 1, color token, rgba or linear-gradient",
        |scalar| match scalar {
            PropScalar::Boolean(true) => Some(OverlayPaint::BlackOpacity("0.4".to_string())),
            PropScalar::Boolean(false) => None,
            PropScalar::Number(value) => parse_overlay_opacity(value),
            PropScalar::String(value) => parse_overlay_string(value),
        },
    )
}

fn parse_background_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<SectionBackground>> {
    parse_responsive(
        name,
        value,
        "soft, aurora, sunrise, ocean, meadow or slate",
        |scalar| match scalar {
            PropScalar::String(value) => SectionBackground::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_cover_source(value: &str) -> Option<CoverSource> {
    if value.starts_with("https://") {
        let host = value
            .strip_prefix("https://")?
            .split(['/', '#', '?'])
            .next()
            .filter(|host| !host.is_empty())?;
        if host.chars().any(|value| value.is_control() || value == ' ') {
            return None;
        }
        return Some(CoverSource(value.to_string()));
    }

    if value.starts_with("//")
        || value.starts_with("javascript:")
        || value.starts_with("data:")
        || value.starts_with("file:")
        || value.contains("://")
        || value.is_empty()
    {
        return None;
    }

    Some(CoverSource(value.to_string()))
}

fn parse_overlay_opacity(value: &str) -> Option<OverlayPaint> {
    let parsed = value.parse::<f32>().ok()?;
    if !(0.0..=1.0).contains(&parsed) {
        return None;
    }
    Some(OverlayPaint::BlackOpacity(normalize_decimal(value)))
}

fn parse_overlay_string(value: &str) -> Option<OverlayPaint> {
    if let Some(token) = ColorToken::from_name(value) {
        return Some(OverlayPaint::Color(token));
    }
    if is_valid_rgba(value) {
        return Some(OverlayPaint::Rgba(value.to_string()));
    }
    if is_valid_linear_gradient(value) {
        return Some(OverlayPaint::LinearGradient(value.to_string()));
    }
    None
}

fn normalize_decimal(value: &str) -> String {
    let mut output = value.trim().trim_end_matches('0').to_string();
    if output.is_empty() {
        return "0".to_string();
    }
    if output.ends_with('.') {
        output.push('0');
    }
    if output == "0." {
        "0".to_string()
    } else {
        output
    }
}
