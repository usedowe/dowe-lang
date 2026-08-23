fn responsive_class(class_name: &str) -> Option<(Breakpoint, &str)> {
    let (prefix, base) = class_name.split_once(':')?;
    let breakpoint = Breakpoint::from_name(prefix)?;
    Some((breakpoint, base))
}

fn class_body(class_name: &str) -> Option<String> {
    if class_name == "flex-wrap" {
        return Some("flex-wrap:wrap;".to_string());
    }
    if let Some(value) = class_name.strip_prefix("flex-")
        && let Some(value) = FlexItem::from_name(value)
    {
        return Some(match value {
            FlexItem::Initial => "flex:0 1 auto;".to_string(),
            FlexItem::Auto => "flex:1 1 auto;".to_string(),
            FlexItem::None => "flex:0 0 auto;".to_string(),
            FlexItem::Fill => unreachable!(),
        });
    }
    if class_name == "flex-1" {
        return Some("flex:1 1 0%;".to_string());
    }
    if let Some(value) = class_name.strip_prefix("position-")
        && let Some(position) = BoxPosition::from_name(value)
    {
        return Some(format!("position:{};", position.as_str()));
    }
    if is_structural_class(class_name) {
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
    for (prefix, css_property) in [("w-pct", "width"), ("min-w-pct", "min-width")] {
        if let Some(value) = class_name.strip_prefix(&format!("{prefix}-"))
            && let Ok(value) = value.parse::<u8>()
            && (10..=100).contains(&value)
            && value % 10 == 0
        {
            return Some(format!("{css_property}:{value}%;"));
        }
    }
    for (prefix, css_property) in [
        ("w", "width"),
        ("min-w", "min-width"),
        ("max-w", "max-width"),
    ] {
        if let Some(suffix) = class_name.strip_prefix(&format!("{prefix}-"))
            && let Some(value) = ContainerSize::from_name(suffix)
        {
            return Some(format!(
                "{css_property}:var(--container-{});",
                value.as_str()
            ));
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
        "h-auto" => return Some("height:auto;".to_string()),
        "min-h-auto" => return Some("min-height:auto;".to_string()),
        "max-h-auto" => return Some("max-height:auto;".to_string()),
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
    if let Some(value) = class_name.strip_prefix("box-center-x-") {
        return Some(format!("align-items:{};", if value == "true" { "center" } else { "flex-start" }));
    }
    if let Some(value) = class_name.strip_prefix("box-center-y-") {
        return Some(format!("justify-content:{};", if value == "true" { "center" } else { "flex-start" }));
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
    if let Some(value) = class_name.strip_prefix("text-align-")
        && let Some(align) = TextAlign::from_name(value)
    {
        return Some(format!("text-align:{};", text_align_css(align)));
    }
    if let Some(value) = class_name.strip_prefix("grid-cols-") {
        if let Ok(count) = value.parse::<u16>()
            && count > 0
        {
            return Some(format!(
                "grid-template-columns:repeat({count},minmax(0,1fr));"
            ));
        }
        if let Some(weights) = value.strip_prefix("fr-") {
            let tracks = weights
                .split('-')
                .filter_map(|weight| weight.parse::<u16>().ok())
                .filter(|weight| *weight > 0)
                .map(|weight| format!("{weight}fr"))
                .collect::<Vec<_>>();
            if !tracks.is_empty() && tracks.len() == weights.split('-').count() {
                return Some(format!("grid-template-columns:{};", tracks.join(" ")));
            }
        }
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
        return Some(format!("justify-items:{css_val};"));
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
