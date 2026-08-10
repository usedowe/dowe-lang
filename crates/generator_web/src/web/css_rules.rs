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

#[derive(Debug, Clone, PartialEq, Eq)]
struct CssRuleFragment {
    breakpoint: Breakpoint,
    content: String,
}

fn push_css_rule_fragment(rules: &mut Vec<CssRuleFragment>, fragment: String) {
    for breakpoint in [
        Breakpoint::Xs,
        Breakpoint::Sm,
        Breakpoint::Md,
        Breakpoint::Lg,
        Breakpoint::Xl,
    ] {
        let prefix = format!("@media (min-width:{}px){{", breakpoint.min_width());
        if let Some(content) = fragment
            .strip_prefix(&prefix)
            .and_then(|content| content.strip_suffix('}'))
        {
            rules.push(CssRuleFragment {
                breakpoint,
                content: content.to_string(),
            });
            return;
        }
    }
    rules.push(CssRuleFragment {
        breakpoint: Breakpoint::Xs,
        content: fragment,
    });
}

fn append_css_rule_fragments(css: &mut String, rules: &mut [CssRuleFragment]) {
    rules.sort_by_key(|rule| rule.breakpoint);
    for breakpoint in [
        Breakpoint::Xs,
        Breakpoint::Sm,
        Breakpoint::Md,
        Breakpoint::Lg,
        Breakpoint::Xl,
    ] {
        let has_rules = rules.iter().any(|rule| rule.breakpoint == breakpoint);
        if !has_rules {
            continue;
        }
        if breakpoint != Breakpoint::Xs {
            css.push_str(&format!(
                "@media (min-width:{}px){{",
                breakpoint.min_width()
            ));
        }
        for rule in rules.iter().filter(|rule| rule.breakpoint == breakpoint) {
            css.push_str(&rule.content);
        }
        if breakpoint != Breakpoint::Xs {
            css.push('}');
        }
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
    if class_name == "flex-wrap" {
        return Some("flex-wrap:wrap;".to_string());
    }
    if let Some(value) = class_name.strip_prefix("position-")
        && let Some(position) = BoxPosition::from_name(value)
    {
        return Some(format!("position:{};", position.as_str()));
    }
    if matches!(
        class_name,
        "box"
            | "flex"
            | "grid"
            | "card"
            | "button"
            | "theme-toggle"
            | "theme-toggle-icon"
            | "theme-icon"
            | "theme-icon-moon"
            | "theme-icon-sun"
            | "fab-container"
            | "fab-trigger"
            | "fab-actions"
            | "fab-action"
            | "fab-action-label"
            | "fab-action-link"
            | "fab-action-button"
            | "fab-icon"
            | "fab-icon-svg"
            | "control"
            | "input"
            | "slider-wrapper"
            | "slider-info"
            | "slider"
            | "dropzone"
            | "dropzone-input"
            | "dropzone-content"
            | "dropzone-icon"
            | "dropzone-placeholder"
            | "dropzone-files"
            | "dropzone-file"
            | "dropzone-file-preview"
            | "dropzone-file-image"
            | "dropzone-file-icon"
            | "dropzone-file-info"
            | "dropzone-file-name"
            | "dropzone-file-size"
            | "dropzone-file-remove"
            | "combo-box"
            | "combo-box-control"
            | "combo-box-value"
            | "combo-box-clear"
            | "combo-box-popover"
            | "combo-box-search-wrap"
            | "combo-box-search"
            | "combo-box-search-icon"
            | "combo-box-options"
            | "combo-box-option"
            | "combo-box-option-avatar"
            | "combo-box-option-icon"
            | "combo-box-option-copy"
            | "combo-box-option-label"
            | "combo-box-option-description"
            | "combo-box-empty"
            | "combo-box-loading"
            | "csv-field"
            | "csv-field-button"
            | "csv-field-icon"
            | "csv-field-summary"
            | "csv-field-preview"
            | "csv-field-preview-title"
            | "csv-field-preview-table"
            | "csv-field-modal"
            | "csv-field-dialog"
            | "csv-field-title"
            | "csv-field-instructions"
            | "csv-field-columns"
            | "csv-field-column"
            | "csv-field-select"
            | "csv-field-error"
            | "csv-field-actions"
            | "csv-field-action"
            | "drag-drop"
            | "drag-drop-group"
            | "drag-drop-group-title"
            | "drag-drop-list"
            | "drag-drop-empty"
            | "drag-drop-item"
            | "drag-drop-handle"
            | "drag-drop-item-copy"
            | "drag-drop-item-label"
            | "drag-drop-item-description"
            | "editor"
            | "editor-toolbar"
            | "editor-toolbar-button"
            | "editor-content"
            | "image-cropper"
            | "image-cropper-trigger"
            | "image-cropper-image"
            | "image-cropper-empty-icon"
            | "image-cropper-label"
            | "image-cropper-actions"
            | "image-cropper-action"
            | "image-cropper-modal"
            | "image-cropper-dialog"
            | "image-cropper-stage"
            | "image-cropper-canvas"
            | "image-cropper-box"
            | "image-cropper-modal-actions"
            | "password"
            | "password-input"
            | "password-toggle"
            | "password-strength"
            | "password-strength-bars"
            | "password-strength-bar"
            | "password-strength-label"
            | "phone"
            | "phone-country-trigger"
            | "phone-flag"
            | "phone-dial"
            | "phone-input"
            | "phone-popover"
            | "phone-search-wrap"
            | "phone-search"
            | "phone-search-icon"
            | "phone-countries"
            | "phone-country"
            | "phone-country-name"
            | "phone-empty"
            | "phone-loading"
            | "pin"
            | "pin-cells"
            | "pin-cell"
            | "textarea-field"
            | "textarea-control"
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
            | "color-control-shell"
            | "color-control-trigger"
            | "color-input"
            | "color-field-swatch"
            | "color-field-value"
            | "color-picker-popover"
            | "color-picker-canvas"
            | "color-picker-cursor"
            | "color-picker-hue"
            | "color-picker-slider-thumb"
            | "color-picker-preview"
            | "color-picker-preview-swatch"
            | "color-picker-preview-color"
            | "color-picker-preview-info"
            | "color-picker-preview-hex"
            | "color-picker-preview-foreground"
            | "color-picker-values"
            | "color-picker-value-code"
            | "date-field"
            | "date-control-shell"
            | "date-control-trigger"
            | "date-control-value"
            | "date-popover"
            | "date-picker-header"
            | "date-picker-month"
            | "date-picker-nav"
            | "date-picker-weekdays"
            | "date-picker-days"
            | "weekday"
            | "date-picker-day-button"
            | "date-picker-empty-day"
            | "date-range-field"
            | "date-range-popover"
            | "date-range-calendars"
            | "date-range-calendar"
            | "date-range-spacer"
            | "radio-group"
            | "radio-item"
            | "radio"
            | "toggle"
            | "toggle-input"
            | "toggle-label-left"
            | "toggle-label-right"
            | "arc-chart-container"
            | "area-chart-container"
            | "bar-chart-container"
            | "line-chart-container"
            | "pie-chart-container"
            | "dowe-chart-viewport"
            | "dowe-chart-svg"
            | "dowe-chart-loading"
            | "dowe-chart-empty"
            | "dowe-chart-legend"
            | "dowe-chart-legend-item"
            | "dowe-chart-legend-color"
            | "dowe-chart-axis-line"
            | "dowe-chart-axis-label"
            | "dowe-chart-grid-line"
            | "dowe-chart-line"
            | "dowe-chart-area"
            | "dowe-chart-point"
            | "dowe-chart-bar"
            | "dowe-chart-slice"
            | "dowe-chart-arc"
            | "dowe-chart-center-value"
            | "dowe-chart-center-label"
            | "table-wrapper"
            | "table-container"
            | "table"
            | "appbar"
            | "footer"
            | "bottombar"
            | "sidenav"
            | "railnav"
            | "railnav-item"
            | "railnav-icon"
            | "railnav-label"
            | "railnav-divider"
            | "railnav-tooltip"
            | "sidebar"
            | "navmenu"
            | "scaffold"
            | "scaffold-body"
            | "scaffold-main"
            | "scaffold-overlays"
            | "tabs"
            | "tabs-list"
            | "tab"
            | "tabs-label"
            | "tabs-wrapper"
            | "tabs-content"
            | "drawer-panel"
            | "drawer"
            | "drawer-header"
            | "drawer-body"
            | "drawer-footer"
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
    for prefix in [
        "p", "px", "py", "pl", "pr", "pt", "pb", "top", "right", "bottom", "left", "gap", "w", "h",
    ] {
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
                "top" => format!("top:{rem};"),
                "right" => format!("right:{rem};"),
                "bottom" => format!("bottom:{rem};"),
                "left" => format!("left:{rem};"),
                "gap" => format!("gap:{rem};"),
                "w" => format!("width:{rem};"),
                "h" => format!("height:{rem};"),
                _ => String::new(),
            });
        }
    }
    if let Some(suffix) = class_name.strip_prefix("vh-")
        && let Some(rem) = scale_suffix_rem(suffix)
    {
        return Some(format!("height:calc(100vh - {rem});"));
    }
    for prefix in ["min-w", "min-h", "max-w", "max-h"] {
        if let Some(suffix) = class_name.strip_prefix(&format!("{prefix}-"))
            && let Some(rem) = scale_suffix_rem(suffix)
        {
            return Some(match prefix {
                "min-w" => format!("min-width:{rem};"),
                "min-h" => format!("min-height:{rem};"),
                "max-w" => format!("max-width:{rem};"),
                "max-h" => format!("max-height:{rem};"),
                _ => String::new(),
            });
        }
    }
    if let Some(suffix) = class_name.strip_prefix("min-h-vh-")
        && let Some(rem) = scale_suffix_rem(suffix)
    {
        return Some(format!("min-height:calc(100vh - {rem});"));
    }
    if let Some(suffix) = class_name.strip_prefix("max-h-vh-")
        && let Some(rem) = scale_suffix_rem(suffix)
    {
        return Some(format!("max-height:calc(100vh - {rem});"));
    }
    match class_name {
        "w-full" => return Some("width:100%;".to_string()),
        "h-full" => return Some("height:100%;".to_string()),
        "min-w-full" => return Some("min-width:100%;".to_string()),
        "min-h-full" => return Some("min-height:100%;".to_string()),
        "max-w-full" => return Some("max-width:100%;".to_string()),
        "max-h-full" => return Some("max-height:100%;".to_string()),
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
    if let Some(value) = class_name.strip_prefix("border-color-")
        && let Some(family) = ColorFamily::from_name(value)
    {
        return Some(format!("border-color:var(--dowe-{});", family.as_str()));
    }
    if let Some(value) = class_name.strip_prefix("shadow-")
        && let Some(size) = ShadowSize::from_name(value)
    {
        return Some(format!("box-shadow:{};", shadow_value(size)));
    }
    if let Some(value) = class_name.strip_prefix("shadow-color-")
        && let Some(family) = ColorFamily::from_name(value)
    {
        return Some(format!(
            "--dowe-shadow-color:color-mix(in srgb,var(--dowe-{}) 28%,transparent);",
            family.as_str()
        ));
    }
    if let Some(value) = class_name.strip_prefix("justify-")
        && let Some(justify) = Justify::from_name(value)
    {
        return Some(format!("justify-content:{};", justify_css(justify)));
    }
    if let Some(value) = class_name.strip_prefix("direction-")
        && let Some(direction) = FlexDirection::from_name(value)
    {
        return Some(format!("flex-direction:{};", direction.as_str()));
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
        return Some(format!(
            "justify-items:{css_val};justify-content:{css_val};"
        ));
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

fn shadow_value(value: ShadowSize) -> &'static str {
    match value {
        ShadowSize::Xs => "0 1px 2px var(--dowe-shadow-color,rgba(15,23,42,.12))",
        ShadowSize::Sm => "0 4px 12px var(--dowe-shadow-color,rgba(15,23,42,.14))",
        ShadowSize::Md => "0 10px 24px var(--dowe-shadow-color,rgba(15,23,42,.16))",
        ShadowSize::Lg => "0 18px 44px var(--dowe-shadow-color,rgba(15,23,42,.18))",
        ShadowSize::Xl => "0 28px 70px var(--dowe-shadow-color,rgba(15,23,42,.22))",
    }
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
        let (surface, content, accent) = match family {
            ColorFamily::Background => ("background", on, on),
            ColorFamily::Surface => ("surface", on, on),
            _ => ("background", color, color),
        };
        css.push_str(&format!(
            ".control.is-outlined.is-{name}{{background-color:var(--dowe-{surface});color:var(--dowe-{content});border:1px solid rgba(127,127,127,0.36);}}.control.is-outlined.is-{name}:focus-within{{border-color:var(--dowe-{accent});box-shadow:0 0 0 3px rgba(127,127,127,0.12);}}"
        ));
        return;
    }
    if base == "control" && variant == ComponentVariant::Line {
        css.push_str(&format!(
            ".control.is-line.is-{name}{{background-color:transparent;color:var(--dowe-{color});border:0;border-bottom:1px solid rgba(127,127,127,0.42);border-radius:0;}}.control.is-line.is-{name}:focus-within{{border-bottom-color:var(--dowe-{color});box-shadow:0 1px 0 0 var(--dowe-{color});}}"
        ));
        return;
    }
    if base == "table" && variant == ComponentVariant::Outlined {
        let content = if matches!(family, ColorFamily::Background | ColorFamily::Surface) {
            on
        } else {
            color
        };
        css.push_str(&format!(
            ".table.is-outlined.is-{name}{{background-color:transparent;color:var(--dowe-{content});border:1px solid var(--dowe-{content});}}"
        ));
        return;
    }
    if matches!(base, "accordion" | "collapsible") && variant == ComponentVariant::Outlined {
        let (surface, content) = if family == ColorFamily::Background {
            ("background", "onBackground")
        } else {
            ("surface", "onSurface")
        };
        css.push_str(&format!(
            ".{base}.is-outlined.is-{name}{{background-color:var(--dowe-{surface});color:var(--dowe-{content});border:1px solid var(--dowe-{color});}}"
        ));
        return;
    }
    if base == "toggle-group-item" {
        match variant {
            ComponentVariant::Solid => css.push_str(&format!(
                ".toggle-group-item.is-active.is-solid.is-{name}{{background-color:var(--dowe-{color});color:var(--dowe-{on});box-shadow:0 1px 6px rgba(15,23,42,.14);}}"
            )),
            ComponentVariant::Soft => css.push_str(&format!(
                ".toggle-group-item.is-active.is-soft.is-{name}{{background-color:var(--dowe-{soft});color:var(--dowe-{on_soft});}}"
            )),
            ComponentVariant::Outlined => css.push_str(&format!(
                ".toggle-group-item.is-active.is-outlined.is-{name}{{background-color:transparent;color:var(--dowe-{color});box-shadow:inset 0 0 0 1px var(--dowe-{color});}}"
            )),
            ComponentVariant::Line => css.push_str(&format!(
                ".toggle-group-item.is-active.is-line.is-{name}{{background-color:transparent;color:var(--dowe-{color});box-shadow:inset 0 -2px 0 var(--dowe-{color});}}"
            )),
            ComponentVariant::Ghost => css.push_str(&format!(
                ".toggle-group-item.is-active.is-ghost.is-{name}{{background-color:transparent;color:var(--dowe-{color});}}"
            )),
        }
        return;
    }
    if base == "sidenav" {
        let accent = family.as_str();
        let variant = variant.as_str();
        let (hover_background, active_background, active_content, active_border) = match variant {
            "solid" => (
                format!("color-mix(in srgb,var(--dowe-{color}) 20%,transparent)"),
                format!("var(--dowe-{color})"),
                on,
                format!("var(--dowe-{color})"),
            ),
            "soft" => (
                format!("color-mix(in srgb,var(--dowe-{soft}) 50%,transparent)"),
                format!("var(--dowe-{soft})"),
                on_soft,
                "transparent".to_string(),
            ),
            "outlined" | "line" => (
                format!("color-mix(in srgb,var(--dowe-{soft}) 50%,transparent)"),
                "transparent".to_string(),
                accent,
                format!("var(--dowe-{accent})"),
            ),
            _ => (
                "transparent".to_string(),
                "transparent".to_string(),
                accent,
                "transparent".to_string(),
            ),
        };
        css.push_str(&format!(
            ".sidenav.is-{variant}.is-{name} .sidenav-header:hover,.sidenav.is-{variant}.is-{name} .sidenav-header.is-active{{background-color:transparent;color:var(--dowe-{accent});}}.sidenav.is-{variant}.is-{name} .sidenav-entry:hover{{background-color:{hover_background};color:var(--dowe-{accent});}}.sidenav.is-{variant}.is-{name} .sidenav-entry.is-active{{background-color:{active_background};color:var(--dowe-{active_content});border-color:{active_border};font-weight:600;}}"
        ));
        return;
    }
    if base == "railnav" {
        let accent = family.as_str();
        let variant = variant.as_str();
        let (hover_background, active_background, active_content, active_border) = match variant {
            "solid" => (
                format!("color-mix(in srgb,var(--dowe-{color}) 20%,transparent)"),
                format!("var(--dowe-{color})"),
                on,
                format!("var(--dowe-{color})"),
            ),
            "soft" => (
                format!("color-mix(in srgb,var(--dowe-{soft}) 50%,transparent)"),
                format!("var(--dowe-{soft})"),
                on_soft,
                "transparent".to_string(),
            ),
            "outlined" | "line" => (
                format!("color-mix(in srgb,var(--dowe-{soft}) 50%,transparent)"),
                "transparent".to_string(),
                accent,
                format!("var(--dowe-{accent})"),
            ),
            _ => (
                "transparent".to_string(),
                "transparent".to_string(),
                accent,
                "transparent".to_string(),
            ),
        };
        css.push_str(&format!(
            ".railnav.is-{variant}.is-{name} .railnav-item:hover,.railnav.is-{variant}.is-{name} .railnav-item:focus-visible{{background-color:{hover_background};color:var(--dowe-{accent});}}.railnav.is-{variant}.is-{name} .railnav-item.is-active{{background-color:{active_background};color:var(--dowe-{active_content});border-color:{active_border};font-weight:600;}}"
        ));
        return;
    }
    if base == "navmenu" {
        let (background, content, border) = match variant {
            ComponentVariant::Solid => (color, on, color),
            ComponentVariant::Soft => (soft, on_soft, soft),
            ComponentVariant::Outlined => (
                "transparent",
                nav_active_content_token(family, variant),
                nav_active_content_token(family, variant),
            ),
            ComponentVariant::Line => (
                "transparent",
                nav_active_content_token(family, variant),
                nav_active_content_token(family, variant),
            ),
            ComponentVariant::Ghost => (
                "transparent",
                nav_active_content_token(family, variant),
                "transparent",
            ),
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
            let (surface, content) = if matches!(base, "card" | "modal" | "toast") {
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
        ComponentVariant::Line => {
            css.push_str(&format!(
                ".{base}.is-line.is-{name}{{background-color:transparent;color:var(--dowe-{color});border-color:transparent;border-bottom:1px solid var(--dowe-{color});border-radius:0;}}"
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
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            family.as_str()
        }
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
            ".tabs-list.is-solid.is-{name}{{border-radius:var(--dowe-radius);background-color:var(--dowe-{soft});color:var(--dowe-{on_soft});}}.tabs-list.is-solid.is-{name} .tab{{border-radius:var(--dowe-radius);}}.tabs-list.is-solid.is-{name} .tab.on-active{{background-color:var(--dowe-{active_background});color:var(--dowe-{active_content});}}"
        )),
        TabsVariant::Outlined => css.push_str(&format!(
            ".tabs-list.is-outlined.is-{name}{{border:1px solid var(--dowe-muted);border-radius:var(--dowe-radius);}}.tabs-list.is-outlined.is-{name} .tab{{border-radius:var(--dowe-radius);}}.tabs-list.is-outlined.is-{name} .tab.on-active{{background-color:var(--dowe-{active_background});color:var(--dowe-{active_content});}}"
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
        TabsVariant::Stepper => css.push_str(&format!(
            ".tabs-list.is-stepper.is-{name}{{gap:0;padding:0;overflow-x:auto;scroll-snap-type:x proximity;}}.tabs-list.is-stepper.is-{name} .tab{{gap:0.625rem;padding:0.5rem 0;scroll-snap-align:start;color:var(--dowe-muted);}}.tabs-list.is-stepper.is-{name} .tab:not(:last-child)::after{{content:\"\";display:block;width:2rem;height:2px;margin-inline:0.5rem;background:var(--dowe-softMuted);}}.tabs-list.is-stepper.is-{name} .tab.on-active{{color:var(--dowe-{accent});}}.tabs-list.is-stepper.is-{name} .step-indicator{{display:inline-grid;place-items:center;flex:0 0 auto;width:2rem;height:2rem;border:2px solid var(--dowe-softMuted);border-radius:9999px;background:var(--dowe-background);color:var(--dowe-muted);font-weight:700;}}.tabs-list.is-stepper.is-{name} .tab.on-active .step-indicator{{border-color:var(--dowe-{accent});background:var(--dowe-{active_background});color:var(--dowe-{active_content});}}.tabs.is-start .tabs-list.is-stepper.is-{name} .tab{{width:100%;}}.tabs.is-start .tabs-list.is-stepper.is-{name} .tab:not(:last-child)::after{{position:absolute;top:2.5rem;left:0.9375rem;width:2px;height:1.5rem;margin:0;background:var(--dowe-softMuted);}}"
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
        "lg" => "calc(var(--dowe-radius) * 1.5)",
        "xl" => "calc(var(--dowe-radius) * 2.25)",
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
