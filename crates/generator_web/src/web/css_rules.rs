fn push_custom_rule(rules: &mut Vec<String>, breakpoint: Breakpoint, rule: &str) {
    let rule = if breakpoint == Breakpoint::Xs {
        rule.to_string()
    } else {
        format!("@media (min-width:{}px){{{rule}}}", breakpoint.min_width())
    };
    if !rules.contains(&rule) {
        rules.push(rule);
    }
}

fn responsive_custom_class(breakpoint: Breakpoint, base: &str) -> String {
    if breakpoint == Breakpoint::Xs {
        base.to_string()
    } else {
        format!("{}:{base}", breakpoint.as_str())
    }
}

fn push_variant_rule(
    variants: &mut Vec<(&'static str, ColorFamily, ComponentVariant)>,
    base: &'static str,
    props: &VariantProps,
) {
    let rule = (
        base,
        props.color.unwrap_or(ColorFamily::Primary),
        props.variant.unwrap_or(ComponentVariant::Solid),
    );
    if !variants.contains(&rule) {
        variants.push(rule);
    }
}

fn append_class_css(css: &mut String, class_name: &str) {
    if let Some((breakpoint, base)) = responsive_class(class_name) {
        if let Some(body) = class_body(base) {
            css.push_str(&format!(
                "@media (min-width:{}px){{",
                breakpoint.min_width()
            ));
            append_responsive_rule(css, breakpoint, base, &body);
            css.push('}');
        }
    } else if let Some(body) = class_body(class_name) {
        append_rule(css, class_name, &body);
    }
}

fn responsive_class(class_name: &str) -> Option<(Breakpoint, &str)> {
    let (prefix, base) = class_name.split_once(':')?;
    let breakpoint = Breakpoint::from_name(prefix)?;
    Some((breakpoint, base))
}

fn class_body(class_name: &str) -> Option<String> {
    if matches!(
        class_name,
        "box"
            | "flex"
            | "grid"
            | "card"
            | "button"
            | "control"
            | "input"
            | "svg"
            | "video"
            | "media"
            | "media-button"
            | "media-content"
            | "media-waveform"
            | "media-bars"
            | "media-bar"
            | "media-footer"
            | "media-time"
            | "media-subtitle"
            | "media-avatar"
            | "image"
            | "image-element"
            | "image-controls"
            | "image-actions"
            | "image-action"
            | "accordion"
            | "accordion-item"
            | "accordion-header"
            | "accordion-start"
            | "accordion-label"
            | "accordion-end"
            | "accordion-arrow"
            | "accordion-content"
            | "carousel"
            | "carousel-header"
            | "carousel-title"
            | "carousel-viewport"
            | "carousel-container"
            | "carousel-slide"
            | "carousel-controls"
            | "carousel-control"
            | "carousel-indicators"
            | "carousel-indicator"
            | "carousel-counter"
            | "carousel-nav"
            | "checkbox"
            | "checkbox-input"
            | "label-md"
            | "label"
            | "color-field"
            | "color-input"
            | "color-field-display"
            | "color-field-swatch"
            | "color-field-value"
            | "color-picker-values"
            | "color-picker-value-code"
            | "date-field"
            | "date-input"
            | "date-range-field"
            | "date-range-inputs"
            | "date-range-separator"
            | "radio-group"
            | "radio-item"
            | "radio"
            | "toggle"
            | "toggle-input"
            | "toggle-label-left"
            | "toggle-label-right"
            | "table-wrapper"
            | "table-container"
            | "table"
            | "appbar"
            | "footer"
            | "bottombar"
            | "sidenav"
            | "sidebar"
            | "navmenu"
            | "scaffold"
            | "scaffold-body"
            | "scaffold-main"
            | "tabs"
            | "tabs-list"
            | "tab"
            | "tabs-label"
            | "tabs-wrapper"
            | "tabs-content"
            | "drawer-panel"
            | "drawer"
            | "avatar"
            | "avatar-image"
            | "avatar-icon"
            | "avatar-name"
            | "avatar-status"
            | "avatar-indicator"
            | "badge"
            | "badge-content"
            | "badge-text"
            | "chip"
            | "chip-label"
            | "chip-icon"
            | "chip-close"
            | "skeleton"
            | "modal-dialog"
            | "modal-overlay"
            | "modal"
            | "modal-header"
            | "modal-body"
            | "modal-footer"
            | "modal-close"
            | "alert-dialog-title"
            | "alert-dialog-description"
            | "alert-dialog-actions"
            | "tooltip"
            | "tooltip-popover"
            | "tooltip-arrow"
            | "toast"
            | "toast-content"
            | "toast-title"
            | "toast-description"
            | "toast-close"
            | "dropdown"
            | "dropdown-trigger"
            | "dropdown-popover"
            | "dropdown-options"
            | "dropdown-divider"
            | "dropdown-item"
            | "dropdown-item-icon"
            | "dropdown-item-content"
            | "dropdown-item-label"
            | "dropdown-item-description"
            | "command-dialog"
            | "command"
            | "command-header"
            | "command-input"
            | "command-kbd"
            | "command-results"
            | "command-empty"
            | "command-group"
            | "command-group-label"
            | "command-group-icon"
            | "command-group-items"
            | "command-shortcuts"
            | "command-item"
            | "command-item-icon"
            | "command-item-content"
            | "command-item-label"
            | "command-item-description"
    ) {
        return Some(String::new());
    }
    if let Some(value) = class_name.strip_prefix("avatar-")
        && ButtonSize::from_name(value).is_some()
    {
        return Some(String::new());
    }
    if let Some(value) = class_name.strip_prefix("chip-")
        && ButtonSize::from_name(value).is_some()
    {
        return Some(String::new());
    }
    if let Some(value) = class_name.strip_prefix("font-")
        && FontFamily::from_name(value).is_some()
    {
        return Some(format!("font-family:var(--dowe-font-{value});"));
    }
    if let Some(token) = class_name.strip_prefix("bg-") {
        return Some(format!("background-color:var(--dowe-{token});"));
    }
    if let Some(token) = class_name.strip_prefix("color-") {
        return Some(format!("color:var(--dowe-{token});"));
    }
    if let Some(value) = class_name.strip_prefix("animate-")
        && let Some(animation) = animation_css(value)
    {
        return Some(animation);
    }
    if let Some(value) = class_name.strip_prefix("background-")
        && let Some(background) = SectionBackground::from_name(value)
    {
        return Some(section_background_css(background));
    }
    if let Some(value) = class_name.strip_prefix("gap-px-")
        && value.parse::<u16>().is_ok()
    {
        return Some(format!("gap:{value}px;"));
    }
    if let Some(value) = class_name.strip_prefix("button-")
        && let Some(size) = ButtonSize::from_name(value)
    {
        return Some(button_size_css(size));
    }
    for prefix in ["p", "px", "py", "pl", "pr", "pt", "pb", "gap", "w", "h"] {
        if let Some(suffix) = class_name.strip_prefix(&format!("{prefix}-"))
            && let Some(rem) = scale_suffix_rem(suffix)
        {
            return Some(match prefix {
                "p" => format!("padding:{rem};"),
                "px" => format!("padding-left:{rem};padding-right:{rem};"),
                "py" => format!("padding-top:{rem};padding-bottom:{rem};"),
                "pl" => format!("padding-left:{rem};"),
                "pr" => format!("padding-right:{rem};"),
                "pt" => format!("padding-top:{rem};"),
                "pb" => format!("padding-bottom:{rem};"),
                "gap" => format!("gap:{rem};"),
                "w" => format!("width:{rem};"),
                "h" => format!("height:{rem};"),
                _ => String::new(),
            });
        }
    }
    for prefix in ["min-w", "min-h"] {
        if let Some(suffix) = class_name.strip_prefix(&format!("{prefix}-"))
            && let Some(rem) = scale_suffix_rem(suffix)
        {
            return Some(match prefix {
                "min-w" => format!("min-width:{rem};"),
                "min-h" => format!("min-height:{rem};"),
                _ => String::new(),
            });
        }
    }
    match class_name {
        "w-full" => return Some("width:100%;".to_string()),
        "h-full" => return Some("height:100%;".to_string()),
        "min-w-full" => return Some("min-width:100%;".to_string()),
        "min-h-full" => return Some("min-height:100%;".to_string()),
        _ => {}
    }
    if let Some(value) = class_name.strip_prefix("rounded-") {
        return Some(format!("border-radius:{};", rounded_value(value)));
    }
    if let Some(value) = class_name.strip_prefix("border-")
        && matches!(value, "1" | "2" | "3" | "4")
    {
        return Some(format!("border-width:{value}px;border-style:solid;"));
    }
    if let Some(value) = class_name.strip_prefix("justify-")
        && let Some(justify) = Justify::from_name(value)
    {
        return Some(format!("justify-content:{};", justify_css(justify)));
    }
    if let Some(value) = class_name.strip_prefix("align-")
        && let Some(align) = Align::from_name(value)
    {
        return Some(format!("align-items:{};", align_css(align)));
    }
    if let Some(value) = class_name.strip_prefix("grid-cols-")
        && let Ok(count) = value.parse::<u16>()
        && count > 0
    {
        return Some(format!(
            "grid-template-columns:repeat({count},minmax(0,1fr));"
        ));
    }
    if let Some(value) = class_name.strip_prefix("grid-rows-") {
        if value == "auto" {
            return Some("grid-auto-rows:auto;".to_string());
        }
        if let Ok(count) = value.parse::<u16>()
            && count > 0
        {
            return Some(format!("grid-template-rows:repeat({count},minmax(0,1fr));"));
        }
    }
    if let Some(value) = class_name.strip_prefix("grid-justify-")
        && let Some(align) = GridAlignment::from_name(value)
    {
        let css_val = grid_alignment_css(align);
        return Some(format!("justify-items:{css_val};justify-content:{css_val};"));
    }
    if let Some(value) = class_name.strip_prefix("grid-align-")
        && let Some(align) = GridAlignment::from_name(value)
    {
        return Some(format!("align-items:{};", grid_alignment_css(align)));
    }
    if let Some(value) = class_name.strip_prefix("col-span-")
        && let Ok(span) = value.parse::<u16>()
        && span > 0
    {
        return Some(format!("grid-column:span {span} / span {span};"));
    }
    if let Some(value) = class_name.strip_prefix("row-span-")
        && let Ok(span) = value.parse::<u16>()
        && span > 0
    {
        return Some(format!("grid-row:span {span} / span {span};"));
    }
    if let Some(value) = class_name.strip_prefix("text-")
        && let Some(size) = TextSize::from_name(value)
    {
        return Some(format!(
            "--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));font-size:{};line-height:{};font-weight:400;margin:0;",
            text_size_css(size),
            text_line_css(size)
        ));
    }
    if let Some(value) = class_name.strip_prefix("title-")
        && let Some(size) = TextSize::from_name(value)
    {
        return Some(format!(
            "--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));font-size:{};line-height:{};font-weight:{};letter-spacing:{};margin:0;",
            title_text_size_css(size),
            title_text_line_css(size),
            title_text_weight_css(size),
            title_text_spacing_css(size)
        ));
    }
    if let Some(value) = class_name.strip_prefix("weight-")
        && let Some(weight) = TextWeight::from_name(value)
    {
        return Some(format!("font-weight:{};", text_weight_css(weight)));
    }
    if let Some(value) = class_name.strip_prefix("tracking-")
        && let Some(spacing) = TextSpacing::from_name(value)
    {
        return Some(format!("letter-spacing:{};", text_spacing_css(spacing)));
    }
    None
}

fn scale_suffix_rem(value: &str) -> Option<String> {
    let half_steps = if let Some((whole, half)) = value.split_once('.') {
        if half != "5" {
            return None;
        }
        whole.parse::<u16>().ok()?.checked_mul(2)?.checked_add(1)?
    } else {
        value.parse::<u16>().ok()?.checked_mul(2)?
    };
    Some(scale_rem(dowe_components::ScaleValue::from_half_steps(
        half_steps,
    )))
}

fn button_size_css(value: ButtonSize) -> String {
    format!(
        "padding:{} {};min-height:{};",
        scale_rem(value.padding_y()),
        scale_rem(value.padding_x()),
        scale_rem(value.min_height())
    )
}

fn append_single_variant_css(
    css: &mut String,
    base: &str,
    family: ColorFamily,
    variant: ComponentVariant,
) {
    let name = family.as_str();
    let color = name;
    let on = on_token(family);
    let soft = soft_token(family);
    let on_soft = on_soft_token(family);
    if base == "control" && variant == ComponentVariant::Outlined {
        css.push_str(&format!(
            ".control.is-outlined.is-{name}{{background-color:var(--dowe-background);color:var(--dowe-{color});border:1px solid rgba(127,127,127,0.36);}}.control.is-outlined.is-{name}:focus-within{{border-color:var(--dowe-{color});box-shadow:0 0 0 3px rgba(127,127,127,0.12);}}"
        ));
        return;
    }
    if matches!(base, "sidenav" | "sidebar") {
        let (background, content, border) = match variant {
            ComponentVariant::Solid => (color, on, color),
            ComponentVariant::Soft => (soft, on_soft, soft),
            ComponentVariant::Outlined => ("transparent", nav_active_content_token(family, variant), nav_active_content_token(family, variant)),
            ComponentVariant::Ghost => ("transparent", nav_active_content_token(family, variant), "transparent"),
        };
        css.push_str(&format!(
            ".{base}.is-{variant}.is-{name} .{base}-entry:hover,.{base}.is-{variant}.is-{name} .{base}-header:hover{{background-color:var(--dowe-{soft});color:var(--dowe-{on_soft});}}.{base}.is-{variant}.is-{name} .{base}-entry.is-active{{background-color:{};color:var(--dowe-{content});border-color:{};}}",
            if background == "transparent" {
                "transparent".to_string()
            } else {
                format!("var(--dowe-{background})")
            },
            if border == "transparent" {
                "transparent".to_string()
            } else {
                format!("var(--dowe-{border})")
            },
            variant = variant.as_str()
        ));
        return;
    }
    if base == "navmenu" {
        let (background, content, border) = match variant {
            ComponentVariant::Solid => (color, on, color),
            ComponentVariant::Soft => (soft, on_soft, soft),
            ComponentVariant::Outlined => ("transparent", nav_active_content_token(family, variant), nav_active_content_token(family, variant)),
            ComponentVariant::Ghost => ("transparent", nav_active_content_token(family, variant), "transparent"),
        };
        css.push_str(&format!(
            ".navmenu.is-{variant}.is-{name} .navmenu-item:hover{{background-color:var(--dowe-{soft});color:var(--dowe-{on_soft});}}.navmenu.is-{variant}.is-{name} .navmenu-item.is-active,.navmenu.is-{variant}.is-{name} .navmenu-item.is-open{{background-color:{};color:var(--dowe-{content});border-color:{};}}",
            if background == "transparent" {
                "transparent".to_string()
            } else {
                format!("var(--dowe-{background})")
            },
            if border == "transparent" {
                "transparent".to_string()
            } else {
                format!("var(--dowe-{border})")
            },
            variant = variant.as_str()
        ));
        return;
    }
    match variant {
        ComponentVariant::Solid => css.push_str(&format!(
            ".{base}.is-solid.is-{name}{{background-color:var(--dowe-{color});color:var(--dowe-{on});border-color:var(--dowe-{color});}}"
        )),
        ComponentVariant::Soft => css.push_str(&format!(
            ".{base}.is-soft.is-{name}{{background-color:var(--dowe-{soft});color:var(--dowe-{on_soft});border-color:var(--dowe-{soft});}}"
        )),
        ComponentVariant::Outlined => {
            let (surface, content) = if base == "card" {
                if family == ColorFamily::Background {
                    ("var(--dowe-background)", "onBackground")
                } else {
                    ("var(--dowe-surface)", "onSurface")
                }
            } else {
                ("transparent", color)
            };
            css.push_str(&format!(
                ".{base}.is-outlined.is-{name}{{background-color:{surface};color:var(--dowe-{content});border:1px solid var(--dowe-{color});}}"
            ));
        }
        ComponentVariant::Ghost => {
            let content = if matches!(family, ColorFamily::Background | ColorFamily::Surface) {
                on
            } else {
                color
            };
            css.push_str(&format!(
                ".{base}.is-ghost.is-{name}{{background-color:transparent;color:var(--dowe-{content});border-color:transparent;}}"
            ));
        }
    }
}

fn nav_active_content_token(family: ColorFamily, variant: ComponentVariant) -> &'static str {
    match variant {
        ComponentVariant::Solid => on_token(family),
        ComponentVariant::Soft => on_soft_token(family),
        ComponentVariant::Outlined | ComponentVariant::Ghost
            if matches!(family, ColorFamily::Background | ColorFamily::Surface) =>
        {
            on_token(family)
        }
        ComponentVariant::Outlined | ComponentVariant::Ghost => family.as_str(),
    }
}

fn append_tabs_variant_css(css: &mut String, family: ColorFamily, variant: TabsVariant) {
    let name = family.as_str();
    let soft = soft_token(family);
    let on_soft = on_soft_token(family);
    let active_background = tabs_active_background(family);
    let active_content = tabs_active_content(family);
    let accent = tabs_accent(family);
    match variant {
        TabsVariant::Solid => css.push_str(&format!(
            ".tabs-list.is-solid.is-{name}{{border-radius:var(--dowe-radiusBox);background-color:var(--dowe-{soft});color:var(--dowe-{on_soft});}}.tabs-list.is-solid.is-{name} .tab{{border-radius:var(--dowe-radiusUi);}}.tabs-list.is-solid.is-{name} .tab.on-active{{background-color:var(--dowe-{active_background});color:var(--dowe-{active_content});}}"
        )),
        TabsVariant::Outlined => css.push_str(&format!(
            ".tabs-list.is-outlined.is-{name}{{border:1px solid var(--dowe-muted);border-radius:var(--dowe-radiusBox);}}.tabs-list.is-outlined.is-{name} .tab{{border-radius:var(--dowe-radiusUi);}}.tabs-list.is-outlined.is-{name} .tab.on-active{{background-color:var(--dowe-{active_background});color:var(--dowe-{active_content});}}"
        )),
        TabsVariant::Line => css.push_str(&format!(
            ".tabs-list.is-line.is-{name}{{gap:1rem;padding-inline:0;}}.tabs-list.is-line.is-{name} .tab{{border-bottom:2px solid transparent;padding-inline:0.25rem;}}.tabs-list.is-line.is-{name} .tab.on-active{{color:var(--dowe-{accent});border-bottom-color:var(--dowe-{accent});}}.tabs.is-start .tabs-list.is-line.is-{name} .tab,.tabs.is-end .tabs-list.is-line.is-{name} .tab{{padding-inline:1rem;border-bottom:0;}}.tabs.is-start .tabs-list.is-line.is-{name} .tab.on-active{{border-left:2px solid var(--dowe-{accent});}}.tabs.is-end .tabs-list.is-line.is-{name} .tab.on-active{{border-right:2px solid var(--dowe-{accent});}}"
        )),
        TabsVariant::Ghost => css.push_str(&format!(
            ".tabs-list.is-ghost.is-{name} .tab.on-active{{color:var(--dowe-{accent});}}"
        )),
        TabsVariant::Pills => css.push_str(&format!(
            ".tabs-list.is-pills.is-{name}{{border-radius:9999px;background-color:var(--dowe-{soft});color:var(--dowe-{on_soft});}}.tabs-list.is-pills.is-{name} .tab{{border-radius:9999px;}}.tabs-list.is-pills.is-{name} .tab.on-active{{background-color:var(--dowe-{active_background});color:var(--dowe-{active_content});}}"
        )),
    }
}

fn tabs_active_background(family: ColorFamily) -> &'static str {
    if family == ColorFamily::Muted {
        on_token(family)
    } else {
        family.as_str()
    }
}

fn tabs_active_content(family: ColorFamily) -> &'static str {
    if family == ColorFamily::Muted {
        family.as_str()
    } else {
        on_token(family)
    }
}

fn tabs_accent(family: ColorFamily) -> &'static str {
    match family {
        ColorFamily::Muted | ColorFamily::Background | ColorFamily::Surface => on_token(family),
        _ => family.as_str(),
    }
}

fn scale_rem(value: dowe_components::ScaleValue) -> String {
    if value.0 == 0 {
        return "0rem".to_string();
    }

    let rem = value.0 as f32 / 8.0;
    let mut output = format!("{rem:.3}");
    while output.contains('.') && output.ends_with('0') {
        output.pop();
    }
    if output.ends_with('.') {
        output.pop();
    }
    format!("{output}rem")
}

fn rounded_value(value: &str) -> &'static str {
    match value {
        "xs" => "calc(var(--dowe-radius) * 0.5)",
        "sm" => "calc(var(--dowe-radius) * 0.75)",
        "md" => "var(--dowe-radius)",
        "lg" => "var(--dowe-radiusBox)",
        "xl" => "calc(var(--dowe-radiusBox) * 1.5)",
        "full" => "9999px",
        _ => "var(--dowe-radius)",
    }
}

fn animation_css(value: &str) -> Option<String> {
    let name = match value {
        "fade-in" => "dowe-fade-in",
        "slide-up" => "dowe-slide-up",
        "slide-down" => "dowe-slide-down",
        "slide-left" => "dowe-slide-left",
        "slide-right" => "dowe-slide-right",
        "scale-in" => "dowe-scale-in",
        "none" => return Some("animation:none;".to_string()),
        _ => return None,
    };
    Some(format!("animation:{name} 220ms ease-out both;"))
}

fn justify_css(value: Justify) -> &'static str {
    match value {
        Justify::Start => "flex-start",
        Justify::Center => "center",
        Justify::End => "flex-end",
        Justify::Between => "space-between",
        Justify::Around => "space-around",
        Justify::Evenly => "space-evenly",
    }
}

fn align_css(value: Align) -> &'static str {
    match value {
        Align::Start => "flex-start",
        Align::Center => "center",
        Align::End => "flex-end",
        Align::Stretch => "stretch",
        Align::Baseline => "baseline",
    }
}

fn grid_alignment_css(value: GridAlignment) -> &'static str {
    match value {
        GridAlignment::Start => "start",
        GridAlignment::Center => "center",
        GridAlignment::End => "end",
        GridAlignment::Stretch => "stretch",
    }
}

fn gap_size_css(value: &GapSize) -> String {
    match value {
        GapSize::Scale(value) => scale_rem(*value),
        GapSize::Px(value) => format!("{value}px"),
    }
}

fn cover_suffix(value: &CoverSource) -> String {
    short_id("cover", &value.0)
}

fn overlay_suffix(value: &OverlayPaint) -> String {
    short_id("overlay", &overlay_key(value))
}

fn overlay_key(value: &OverlayPaint) -> String {
    match value {
        OverlayPaint::BlackOpacity(value) => format!("black-{value}"),
        OverlayPaint::Color(value) => format!("color-{}", value.as_str()),
        OverlayPaint::Rgba(value) => format!("rgba-{value}"),
        OverlayPaint::LinearGradient(value) => format!("gradient-{value}"),
    }
}

fn section_background_css(value: SectionBackground) -> String {
    match value {
        SectionBackground::Soft => "background-image:linear-gradient(135deg,var(--dowe-surface),var(--dowe-background));".to_string(),
        SectionBackground::Aurora => "background-image:linear-gradient(135deg,var(--dowe-softPrimary),var(--dowe-softSecondary),var(--dowe-softTertiary));".to_string(),
        SectionBackground::Sunrise => "background-image:linear-gradient(135deg,var(--dowe-softWarning),var(--dowe-softDanger),var(--dowe-surface));".to_string(),
        SectionBackground::Ocean => "background-image:linear-gradient(135deg,var(--dowe-softInfo),var(--dowe-softPrimary),var(--dowe-softTertiary));".to_string(),
        SectionBackground::Meadow => "background-image:linear-gradient(135deg,var(--dowe-softSuccess),var(--dowe-softTertiary),var(--dowe-surface));".to_string(),
        SectionBackground::Slate => "background-image:linear-gradient(135deg,var(--dowe-softMuted),var(--dowe-surface),var(--dowe-background));".to_string(),
    }
}

fn overlay_css(value: &OverlayPaint) -> String {
    match value {
        OverlayPaint::BlackOpacity(value) => format!("rgba(0,0,0,{value})"),
        OverlayPaint::Color(value) => format!("var(--dowe-{})", value.as_str()),
        OverlayPaint::Rgba(value) | OverlayPaint::LinearGradient(value) => value.clone(),
    }
}

fn escape_css_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn text_size_css(value: TextSize) -> String {
    fluid_text_size_css(text_typography(false, value).font_size)
}

fn fluid_text_size_css(value: dowe_components::FluidTextSize) -> String {
    format!(
        "clamp({}rem, {}rem + {}vw, {}rem)",
        points_to_rem(value.min),
        points_to_rem(value.preferred_base),
        value.preferred_viewport,
        points_to_rem(value.max)
    )
}

fn points_to_rem(value: &str) -> String {
    let rem = value.parse::<f64>().expect("text metric") / 16.0;
    let mut output = format!("{rem:.4}");
    while output.contains('.') && output.ends_with('0') {
        output.pop();
    }
    if output.ends_with('.') {
        output.pop();
    }
    output
}
