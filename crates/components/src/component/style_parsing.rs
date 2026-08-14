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
                        | BuiltinComponent::ComboBox
                        | BuiltinComponent::Editor
                        | BuiltinComponent::ImageCropper
                        | BuiltinComponent::Password
                        | BuiltinComponent::Phone
                        | BuiltinComponent::Pin
                        | BuiltinComponent::Textarea
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
            "onClick"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::IconButton
                        | BuiltinComponent::Avatar
                        | BuiltinComponent::Fab
                        | BuiltinComponent::Empty
                        | BuiltinComponent::Box
                        | BuiltinComponent::Card
                        | BuiltinComponent::Chip
                ) =>
            {
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
            "boxed" if matches!(mode, StylePropMode::Section) => {
                style.boxed = parse_static_bool(&prop.name, &prop.value)?
            }
            "animation" if style_accepts_animation(mode) => {
                style.set_animation(Some(parse_animation_prop(&prop.name, &prop.value)?))
            }
            "rotate" => {
                style.motion_mut().rotate = Some(parse_rotation_prop(&prop.name, &prop.value)?)
            }
            "scale" => {
                style.motion_mut().scale = Some(parse_view_scale_prop(&prop.name, &prop.value)?)
            }
            "translateX" => {
                style.motion_mut().translate_x =
                    Some(parse_translation_prop(&prop.name, &prop.value)?)
            }
            "translateY" => {
                style.motion_mut().translate_y =
                    Some(parse_translation_prop(&prop.name, &prop.value)?)
            }
            "transition" => {
                style.motion_mut().transition =
                    Some(parse_transition_prop(&prop.name, &prop.value)?)
            }
            "gesture" => {
                style.motion_mut().gesture = Some(parse_gesture_prop(&prop.name, &prop.value)?)
            }
            "colSpan" if style_accepts_grid_item(mode) => {
                style.grid_item_mut().col_span = Some(parse_span_prop(&prop.name, &prop.value)?)
            }
            "rowSpan" if style_accepts_grid_item(mode) => {
                style.grid_item_mut().row_span = Some(parse_span_prop(&prop.name, &prop.value)?)
            }
            "position" if matches!(mode, StylePropMode::Box) => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                style.position_mut().mode = BoxPosition::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("position", "static, relative, absolute or fixed")
                })?;
            }
            "top" if matches!(mode, StylePropMode::Box) => {
                style.position_mut().top = Some(parse_scale_prop(&prop.name, &prop.value)?)
            }
            "right" if matches!(mode, StylePropMode::Box) => {
                style.position_mut().right = Some(parse_scale_prop(&prop.name, &prop.value)?)
            }
            "bottom" if matches!(mode, StylePropMode::Box) => {
                style.position_mut().bottom = Some(parse_scale_prop(&prop.name, &prop.value)?)
            }
            "left" if matches!(mode, StylePropMode::Box) => {
                style.position_mut().left = Some(parse_scale_prop(&prop.name, &prop.value)?)
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
            "maxW" => style.sizing.max_w = Some(parse_size_prop(&prop.name, &prop.value)?),
            "maxH" => style.sizing.max_h = Some(parse_size_prop(&prop.name, &prop.value)?),
            "rounded" => style.rounded = Some(parse_rounded_prop(&prop.name, &prop.value)?),
            "border" => style.border = Some(parse_border_prop(&prop.name, &prop.value)?),
            "borderColor" => {
                style.border_color = Some(parse_family_prop(component, &prop.name, &prop.value)?)
            }
            "shadow" => style.shadow = Some(parse_shadow_prop(&prop.name, &prop.value)?),
            "shadowColor" => {
                style.shadow_color = Some(parse_family_prop(component, &prop.name, &prop.value)?)
            }
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

    let position = style.position();
    let has_position_offset = position.top.is_some()
        || position.right.is_some()
        || position.bottom.is_some()
        || position.left.is_some();
    if has_position_offset && matches!(position.mode, BoxPosition::Static | BoxPosition::Relative) {
        return Err(ComponentError::invalid_prop_combination(
            "`top`, `right`, `bottom` and `left` require `position:\"absolute\"` or `position:\"fixed\"`",
        ));
    }
    if position.top.is_some() && position.bottom.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "`top` and `bottom` cannot be used together on positioned `Box`",
        ));
    }
    if position.left.is_some() && position.right.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "`left` and `right` cannot be used together on positioned `Box`",
        ));
    }

    Ok(style)
}

fn reactive_reference(value: &PropValue) -> Option<String> {
    let PropValue::String(value) = value else {
        return None;
    };
    value.strip_prefix("@signal:").map(str::to_string)
}

fn style_accepts_colors(mode: StylePropMode) -> bool {
    matches!(
        mode,
        StylePropMode::Box | StylePropMode::Banner | StylePropMode::Section | StylePropMode::Text
    )
}

fn style_accepts_cover(mode: StylePropMode) -> bool {
    matches!(
        mode,
        StylePropMode::Box | StylePropMode::Banner | StylePropMode::Section | StylePropMode::Card
    )
}

fn style_accepts_background(mode: StylePropMode) -> bool {
    matches!(mode, StylePropMode::Section)
}

fn style_accepts_grid_item(mode: StylePropMode) -> bool {
    matches!(
        mode,
        StylePropMode::Box | StylePropMode::Banner | StylePropMode::Section | StylePropMode::Card
    )
}

fn style_accepts_animation(mode: StylePropMode) -> bool {
    let _ = mode;
    true
}

fn parse_layout_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<LayoutProps> {
    let mut layout = LayoutProps::default();
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "direction" => layout.direction = parse_flex_direction_prop(&prop.name, &prop.value)?,
            "wrap" => layout.wrap = parse_static_bool(&prop.name, &prop.value)?,
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
                grid.columns = Some(parse_grid_tracks_prop(
                    &prop.name,
                    &prop.value,
                    false,
                    Some(12),
                )?)
            }
            "rows" => {
                grid.rows = Some(parse_grid_tracks_prop(&prop.name, &prop.value, true, None)?)
            }
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
    let mut form_help_text = None;
    let mut form_error_text = None;

    for prop in props {
        match prop.name.as_str() {
            "variant" if reactive_reference(&prop.value).is_some() => {
                variant_props.reactive.variant = reactive_reference(&prop.value)
            }
            "scheme" if reactive_reference(&prop.value).is_some() => {
                variant_props.reactive.scheme = reactive_reference(&prop.value)
            }
            "size" if reactive_reference(&prop.value).is_some() => {
                variant_props.reactive.size = reactive_reference(&prop.value)
            }
            "rounded" if reactive_reference(&prop.value).is_some() => {
                variant_props.reactive.rounded = reactive_reference(&prop.value)
            }
            "loading" if component == BuiltinComponent::Button => {
                let loading = reactive_reference(&prop.value).ok_or_else(|| {
                    ComponentError::invalid_prop(&prop.name, "boolean Signal or View Store path")
                })?;
                variant_props.reactive.loading = Some(loading);
                variant_props.loading_icon = Some(svg_spinner_control_icon("3-dots-move")?);
            }
            "disabled" if component == BuiltinComponent::Button => {
                let disabled = reactive_reference(&prop.value).ok_or_else(|| {
                    ComponentError::invalid_prop(&prop.name, "boolean Signal or View Store path")
                })?;
                variant_props.reactive.disabled = Some(disabled);
            }
            "i18n" if component == BuiltinComponent::Button => {
                variant_props.i18n = Some(parse_i18n_key_prop(&prop.name, &prop.value)?);
            }
            "iconStart"
                if matches!(
                    component,
                    BuiltinComponent::Button | BuiltinComponent::Input
                ) =>
            {
                let name = parse_static_string(&prop.name, &prop.value)?;
                let (condition, name, comparison) = conditional_icon_value(&name);
                variant_props.reactive.icon_start_when = condition;
                variant_props.reactive.icon_start_comparison = comparison;
                if !name.is_empty() {
                    variant_props.icon_start = Some(solar_control_icon(name)?);
                }
            }
            "iconEnd"
                if matches!(
                    component,
                    BuiltinComponent::Button | BuiltinComponent::Input
                ) =>
            {
                let name = parse_static_string(&prop.name, &prop.value)?;
                let (condition, name, comparison) = conditional_icon_value(&name);
                variant_props.reactive.icon_end_when = condition;
                variant_props.reactive.icon_end_comparison = comparison;
                if !name.is_empty() {
                    variant_props.icon_end = Some(solar_control_icon(name)?);
                }
            }
            "icon" if component == BuiltinComponent::IconButton => {
                let name = parse_static_string(&prop.name, &prop.value)?;
                variant_props.icon_start = Some(solar_control_icon(&name)?);
            }
            "variant" => variant_props.variant = Some(parse_variant_prop(&prop.name, &prop.value)?),
            "scheme" => {
                variant_props.color = Some(parse_family_prop(component, &prop.name, &prop.value)?)
            }
            "size"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::IconButton
                        | BuiltinComponent::Chip
                        | BuiltinComponent::AvatarGroup
                        | BuiltinComponent::ToggleTheme
                        | BuiltinComponent::SelectTheme
                        | BuiltinComponent::Fab
                ) =>
            {
                variant_props.size = Some(parse_button_size_prop(&prop.name, &prop.value)?)
            }
            "size"
                if matches!(
                    component,
                    BuiltinComponent::Input | BuiltinComponent::Select
                ) =>
            {
                variant_props.size = Some(parse_control_size_prop(&prop.name, &prop.value)?)
            }
            "label" if component == BuiltinComponent::IconButton => {
                variant_props.label = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "label"
                if matches!(
                    component,
                    BuiltinComponent::Input
                        | BuiltinComponent::Select
                        | BuiltinComponent::ComboBox
                        | BuiltinComponent::CsvField
                        | BuiltinComponent::DragDrop
                        | BuiltinComponent::Editor
                        | BuiltinComponent::ImageCropper
                        | BuiltinComponent::Password
                        | BuiltinComponent::Phone
                        | BuiltinComponent::Pin
                        | BuiltinComponent::Textarea
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
                        | BuiltinComponent::ComboBox
                        | BuiltinComponent::Editor
                        | BuiltinComponent::ImageCropper
                        | BuiltinComponent::Password
                        | BuiltinComponent::Phone
                        | BuiltinComponent::Textarea
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
                        | BuiltinComponent::ComboBox
                        | BuiltinComponent::Password
                        | BuiltinComponent::Phone
                        | BuiltinComponent::Textarea
                        | BuiltinComponent::Color
                        | BuiltinComponent::Date
                        | BuiltinComponent::DateRange
                ) =>
            {
                variant_props.label_floating = parse_static_bool(&prop.name, &prop.value)?
            }
            "helpText"
                if matches!(component, BuiltinComponent::Input | BuiltinComponent::Select) =>
            {
                form_help_text = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "errorText"
                if matches!(component, BuiltinComponent::Input | BuiltinComponent::Select) =>
            {
                form_error_text = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "href"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::IconButton
                        | BuiltinComponent::Avatar
                        | BuiltinComponent::Empty
                ) =>
            {
                href = Some(parse_required_string(&prop.name, &prop.value)?)
            }
            "navigate"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::IconButton
                        | BuiltinComponent::Avatar
                        | BuiltinComponent::Empty
                ) =>
            {
                navigate = Some(parse_navigation_operation(&prop.name, &prop.value)?)
            }
            "history"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::IconButton
                        | BuiltinComponent::Avatar
                        | BuiltinComponent::Empty
                ) =>
            {
                history = Some(parse_history_prop(&prop.name, &prop.value)?)
            }
            "target"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::IconButton
                        | BuiltinComponent::Avatar
                        | BuiltinComponent::Empty
                ) =>
            {
                target = Some(parse_web_target(&prop.name, &prop.value)?)
            }
            "externalMode"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::IconButton
                        | BuiltinComponent::Avatar
                        | BuiltinComponent::Empty
                ) =>
            {
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
    if component == BuiltinComponent::Card {
        normalize_card_visual_props(&mut variant_props);
    }
    if form_help_text.is_some() || form_error_text.is_some() {
        let validation = variant_props.style.element.form_validation_mut();
        validation.help_text = form_help_text;
        validation.error_text = form_error_text;
    }
    variant_props.element = variant_props.style.element.clone();
    variant_props.navigation =
        parse_navigation_props(component, href, navigate, history, target, external_mode)?;
    if component == BuiltinComponent::IconButton {
        variant_props.icon_only = true;
        if variant_props.icon_start.is_none() {
            return Err(ComponentError::invalid_prop(
                "icon",
                "known quoted Solar icon name",
            ));
        }
        if variant_props.label.as_deref().is_none_or(str::is_empty) {
            return Err(ComponentError::invalid_prop(
                "label",
                "non-empty accessibility label",
            ));
        }
    }
    Ok(variant_props)
}

fn parse_brand_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<BrandProps> {
    let mut style_props = Vec::new();
    let mut href = None;
    let mut label = None;

    for prop in props {
        match prop.name.as_str() {
            "href" => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => style_props.push(prop.clone()),
        }
    }

    Ok(BrandProps {
        style: parse_style_props(component, &style_props, StylePropMode::Layout)?,
        navigation: parse_link_navigation_props(component.as_str(), href, None, None, None)?,
        label,
    })
}

fn parse_banner_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<BannerProps> {
    let mut style_props = Vec::new();
    let mut href = None;
    let mut label = None;

    for prop in props {
        match prop.name.as_str() {
            "href" => href = Some(parse_required_string(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => style_props.push(prop.clone()),
        }
    }

    let href = href.ok_or_else(|| ComponentError::invalid_prop("href", "required https URL"))?;
    let navigation = classify_href(
        &href,
        NavigationOperation::Push,
        WebTarget::Blank,
        NativeExternalMode::System,
    )?;
    if !matches!(navigation, NavigationAction::External { .. }) {
        return Err(ComponentError::invalid_prop("href", "https URL"));
    }

    Ok(BannerProps {
        style: parse_style_props(component, &style_props, StylePropMode::Banner)?,
        navigation,
        label,
    })
}

fn conditional_icon_value(value: &str) -> (Option<String>, &str, Option<ReactiveNumberComparison>) {
    let Some(value) = value.strip_prefix("@conditional-icon:") else {
        return (None, value, None);
    };
    let mut parts = value.split(':');
    let (Some(condition), Some(name)) = (parts.next(), parts.next()) else {
        return (None, value, None);
    };
    let comparison = match (parts.next(), parts.next()) {
        (Some(operator), Some(value)) => Some(ReactiveNumberComparison {
            operator: match operator {
                "gt" => NumberComparisonOperator::GreaterThan,
                "gte" => NumberComparisonOperator::GreaterThanOrEqual,
                "lt" => NumberComparisonOperator::LessThan,
                "lte" => NumberComparisonOperator::LessThanOrEqual,
                _ => return (None, value, None),
            },
            value: value.to_string(),
        }),
        _ => None,
    };
    (Some(condition.to_string()), name, comparison)
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
            "position" if component == BuiltinComponent::AppBar => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                bar.position = BarPosition::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("position", "static, sticky or fixed")
                })?;
            }
            "hideOnScroll" if component == BuiltinComponent::AppBar => {
                bar.hide_on_scroll = parse_static_bool(&prop.name, &prop.value)?
            }
            "dockOnScroll" if component == BuiltinComponent::AppBar => {
                bar.dock_on_scroll = parse_static_bool(&prop.name, &prop.value)?
            }
            _ => style_props.push(prop.clone()),
        }
    }

    if bar.dock_on_scroll && (!bar.floating || bar.position != BarPosition::Fixed) {
        return Err(ComponentError::invalid_prop_combination(
            "`dockOnScroll:true` requires `floating:true` and `position:\"fixed\"` on `AppBar`",
        ));
    }

    let mut style = parse_variant_props(component, &style_props)?;
    if component == BuiltinComponent::Footer {
        let horizontal_padding = ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: ScaleValue::from_half_steps(8),
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: ScaleValue::from_half_steps(12),
            },
        ]);
        let top_padding = ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: ScaleValue::from_half_steps(20),
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: ScaleValue::from_half_steps(32),
            },
        ]);
        let bottom_padding = horizontal_padding.clone();
        style.style.spacing = style
            .style
            .spacing
            .with_horizontal_padding_default(horizontal_padding)
            .with_vertical_padding_defaults(top_padding, bottom_padding);
    }
    bar.style = style;
    Ok(bar)
}

fn parse_tabs_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<TabsProps> {
    let mut variant = TabsVariant::Pills;
    let mut color = ColorFamily::Primary;
    let mut variant_explicit = false;
    let mut color_explicit = false;
    let mut position = TabsPosition::Top;
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "variant" => {
                variant = parse_tabs_variant_prop(&prop.name, &prop.value)?;
                variant_explicit = true;
            }
            "scheme" => {
                color = parse_family_prop(component, &prop.name, &prop.value)?;
                color_explicit = true;
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
        variant_explicit,
        color_explicit,
    })
}

fn parse_stepper_props(props: &[ComponentProp]) -> ComponentResult<TabsProps> {
    let mut color = ColorFamily::Primary;
    let mut color_explicit = false;
    let mut position = TabsPosition::Top;
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "scheme" => {
                color = parse_family_prop(BuiltinComponent::Stepper, &prop.name, &prop.value)?;
                color_explicit = true;
            }
            "orientation" => {
                let orientation = parse_required_string(&prop.name, &prop.value)?;
                position = match orientation.as_str() {
                    "horizontal" => TabsPosition::Top,
                    "vertical" => TabsPosition::Start,
                    _ => {
                        return Err(ComponentError::invalid_prop(
                            "orientation",
                            "horizontal or vertical",
                        ));
                    }
                };
            }
            "color" => {
                return Err(ComponentError::new(
                    "unknown prop `color` on `Stepper`; use `scheme` for visual family",
                ));
            }
            _ => style_props.push(prop.clone()),
        }
    }

    Ok(TabsProps {
        style: parse_style_props(
            BuiltinComponent::Stepper,
            &style_props,
            StylePropMode::Variant,
        )?,
        variant: TabsVariant::Stepper,
        color,
        position,
        variant_explicit: true,
        color_explicit,
    })
}

fn normalize_button_visual_props(props: &mut VariantProps) {
    let size = *props.size.get_or_insert(ButtonSize::Md);
    if props.reactive.size.is_none() {
        apply_button_size_defaults(&mut props.style, size);
    }
}

fn normalize_icon_button_visual_props(props: &mut VariantProps) {
    let size = *props.size.get_or_insert(ButtonSize::Md);
    if props.reactive.size.is_some() {
        return;
    }

    let control_size = ResponsiveValue::scalar(SizeValue::Scale(size.icon_button_control_size()));
    if props.style.sizing.w.is_none() {
        props.style.sizing.w = Some(control_size.clone());
    }
    if props.style.sizing.h.is_none() {
        props.style.sizing.h = Some(control_size);
    }

    if let Some(icon) = props.icon_start.as_mut() {
        let icon_size = ResponsiveValue::scalar(SizeValue::Scale(size.icon_button_icon_size()));
        icon.props.style.sizing.w = Some(icon_size.clone());
        icon.props.style.sizing.h = Some(icon_size);
    }
}

fn normalize_card_visual_props(props: &mut VariantProps) {
    props.style.spacing = props
        .style
        .spacing
        .with_padding_default(ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: ScaleValue::from_half_steps(8),
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Lg,
                value: ScaleValue::from_half_steps(10),
            },
        ]));
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
            "title" if component == BuiltinComponent::RichText => {
                text.title = parse_static_bool(&prop.name, &prop.value)?
            }
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
    let mut data = None;

    for prop in props {
        match prop.name.as_str() {
            "id" => style.element.id = Some(parse_id_prop(&prop.name, &prop.value)?),
            "show" => style.element.show = Some(parse_show_prop(&prop.name, &prop.value)?),
            "viewBox" => view_box = Some(parse_svg_view_box(&prop.name, &prop.value)?),
            "data" => data = Some(parse_required_string(&prop.name, &prop.value)?),
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

    if data.is_some() && view_box.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "Svg data cannot combine with viewBox",
        ));
    }

    Ok(SvgProps {
        style,
        view_box: match (view_box, data.as_ref()) {
            (Some(view_box), _) => view_box,
            (None, Some(_)) => SvgViewBox {
                min_x: "0".to_string(),
                min_y: "0".to_string(),
                width: "24".to_string(),
                height: "24".to_string(),
            },
            (None, None) => {
                return Err(ComponentError::invalid_prop("viewBox", "four numbers"));
            }
        },
        data,
        motion: None,
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
    let mut even_odd = false;
    let mut transform = None;

    for prop in props {
        match prop.name.as_str() {
            "d" => data = Some(parse_svg_path_data(&prop.name, &prop.value)?),
            "fill" => fill = Some(parse_svg_path_fill(&prop.name, &prop.value)?),
            "fillRule" => even_odd = parse_svg_fill_rule(&prop.name, &prop.value)?,
            "transform" => transform = Some(parse_svg_transform(&prop.name, &prop.value)?),
            _ => return Err(ComponentError::unknown_prop(component, &prop.name)),
        }
    }

    Ok(SvgPath {
        data: data.ok_or_else(|| ComponentError::invalid_prop("d", "static SVG path data"))?,
        fill: svg_path_fill_rule(fill.unwrap_or(SvgPathFill::CurrentColor), even_odd),
        transform,
    })
}

fn parse_svg_fill_rule(name: &str, value: &PropValue) -> ComponentResult<bool> {
    match parse_required_string(name, value)?.as_str() {
        "nonzero" => Ok(false),
        "evenodd" => Ok(true),
        _ => Err(ComponentError::invalid_prop(name, "nonzero or evenodd")),
    }
}

fn svg_path_fill_rule(fill: SvgPathFill, even_odd: bool) -> SvgPathFill {
    if !even_odd {
        return fill;
    }
    match fill {
        SvgPathFill::CurrentColor => SvgPathFill::Fill {
            color: None,
            opacity: 255,
            even_odd: true,
        },
        SvgPathFill::Color(color) => SvgPathFill::Fill {
            color: Some(color),
            opacity: 255,
            even_odd: true,
        },
        SvgPathFill::RawFill { color, opacity, .. } => SvgPathFill::RawFill {
            color,
            opacity,
            even_odd: true,
        },
        SvgPathFill::Fill { color, opacity, .. } => SvgPathFill::Fill {
            color,
            opacity,
            even_odd: true,
        },
        SvgPathFill::LiteralFill {
            red,
            green,
            blue,
            opacity,
            ..
        } => SvgPathFill::LiteralFill {
            red,
            green,
            blue,
            opacity,
            even_odd: true,
        },
        _ => fill,
    }
}

fn parse_svg_transform(name: &str, value: &PropValue) -> ComponentResult<SvgTransform> {
    let value = parse_required_string(name, value)?;
    let body = value
        .strip_prefix("matrix(")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| ComponentError::invalid_prop(name, "matrix(a b c d e f)"))?;
    let parts = body
        .split(|value: char| value.is_whitespace() || value == ',')
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let [a, b, c, d, e, f] = parts.as_slice() else {
        return Err(ComponentError::invalid_prop(name, "matrix(a b c d e f)"));
    };
    if !parts.iter().all(|value| is_svg_number(value)) {
        return Err(ComponentError::invalid_prop(
            name,
            "six finite matrix numbers",
        ));
    }
    Ok(SvgTransform {
        a: normalize_svg_number(a),
        b: normalize_svg_number(b),
        c: normalize_svg_number(c),
        d: normalize_svg_number(d),
        e: normalize_svg_number(e),
        f: normalize_svg_number(f),
    })
}

fn parse_svg_path_fill(name: &str, value: &PropValue) -> ComponentResult<SvgPathFill> {
    let value = parse_required_string(name, value)?;
    match value.as_str() {
        "none" => Ok(SvgPathFill::None),
        "currentColor" => Ok(SvgPathFill::CurrentColor),
        _ => {
            if let Some((red, green, blue, opacity)) = parse_svg_hex_fill(&value) {
                return Ok(SvgPathFill::LiteralFill {
                    red,
                    green,
                    blue,
                    opacity,
                    even_odd: false,
                });
            }
            ColorToken::from_name(&value)
                .map(SvgPathFill::Color)
                .ok_or_else(|| {
                    ComponentError::invalid_prop(
                        name,
                        "currentColor, none, hexadecimal color or color token",
                    )
                })
        }
    }
}

fn parse_svg_hex_fill(value: &str) -> Option<(u8, u8, u8, u8)> {
    let hex = value.strip_prefix('#')?;
    let expanded = match hex.len() {
        3 | 4 => hex
            .chars()
            .flat_map(|value| [value, value])
            .collect::<String>(),
        6 | 8 => hex.to_string(),
        _ => return None,
    };
    let red = u8::from_str_radix(&expanded[0..2], 16).ok()?;
    let green = u8::from_str_radix(&expanded[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&expanded[4..6], 16).ok()?;
    let opacity = if expanded.len() == 8 {
        u8::from_str_radix(&expanded[6..8], 16).ok()?
    } else {
        255
    };
    Some((red, green, blue, opacity))
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
    value.parse::<f32>().ok().is_some_and(f32::is_finite)
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
        "system, inter, roboto, montserrat, lato, poppins, manrope, quicksand, lora, syne, jost or puritan",
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
