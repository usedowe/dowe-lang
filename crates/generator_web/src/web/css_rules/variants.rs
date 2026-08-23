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
    let text = text_token(family);
    let title = title_token(family);
    let soft = soft_token(family);
    let soft_text = soft_text_token(family);
    let soft_title = soft_title_token(family);
    let (surface, surface_text) = if family == ColorFamily::Background {
        ("background", "backgroundText")
    } else {
        ("surface", "surfaceText")
    };
    if base == "control" && variant == ComponentVariant::Outlined {
        let (surface, content, accent) = match family {
            ColorFamily::Background => ("background", text, text),
            ColorFamily::Surface => ("surface", text, text),
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
            text
        } else {
            color
        };
        css.push_str(&format!(
            ".table.is-outlined.is-{name}{{background-color:transparent;color:var(--dowe-{content});border:1px solid var(--dowe-{content});}}"
        ));
        return;
    }
    if base == "accordion" {
        let (surface, content, content_title) = if family == ColorFamily::Background {
            ("background", "backgroundText", "backgroundTitle")
        } else {
            ("surface", "surfaceText", "surfaceTitle")
        };
        let (background, content, border, item_background, item_border, item_radius, padding, gap) =
            match variant {
                ComponentVariant::Solid => (
                    format!("var(--dowe-{color})"),
                    text,
                    "transparent".to_string(),
                    "transparent".to_string(),
                    format!("1px solid color-mix(in srgb,var(--dowe-{text}) 24%,transparent)"),
                    "calc(var(--dowe-radius) * .85)".to_string(),
                    ".25rem",
                    ".75rem",
                ),
                ComponentVariant::Soft => (
                    format!("var(--dowe-{soft})"),
                    soft_text,
                    "transparent".to_string(),
                    "var(--dowe-surface)".to_string(),
                    format!("1px solid color-mix(in srgb,var(--dowe-{soft_text}) 16%,transparent)"),
                    "calc(var(--dowe-radius) * .85)".to_string(),
                    ".25rem",
                    ".75rem",
                ),
                ComponentVariant::Outlined => (
                    format!("var(--dowe-{surface})"),
                    content,
                    format!("var(--dowe-{color})"),
                    format!("var(--dowe-{surface})"),
                    format!("1px solid color-mix(in srgb,var(--dowe-{color}) 24%,transparent)"),
                    "calc(var(--dowe-radius) * .85)".to_string(),
                    ".25rem",
                    ".75rem",
                ),
                ComponentVariant::Line => (
                    "transparent".to_string(),
                    color,
                    "transparent".to_string(),
                    "transparent".to_string(),
                    format!(
                        "0;border-bottom:1px solid color-mix(in srgb,var(--dowe-{color}) 24%,transparent)"
                    ),
                    "0".to_string(),
                    "0",
                    "0",
                ),
                ComponentVariant::Ghost => (
                    "transparent".to_string(),
                    if matches!(family, ColorFamily::Background | ColorFamily::Surface) {
                        text
                    } else {
                        color
                    },
                    "transparent".to_string(),
                    "transparent".to_string(),
                    format!(
                        "0;border-bottom:1px solid color-mix(in srgb,var(--dowe-{content}) 22%,transparent)"
                    ),
                    "0".to_string(),
                    "0",
                    "0",
                ),
            };
        css.push_str(&format!(
            ".accordion.is-{variant}.is-{name}{{--dowe-content-text:var(--dowe-{content});--dowe-content-title:var(--dowe-{content_title});background-color:{background};color:var(--dowe-{content});border:1px solid {border};padding:{padding};gap:{gap};}}.accordion.is-{variant}.is-{name} .accordion-item{{background-color:{item_background};border:{item_border};border-radius:{item_radius};}}.accordion.is-{variant}.is-{name} .accordion-header:hover,.accordion.is-{variant}.is-{name} .accordion-header:focus-visible{{background-color:color-mix(in srgb,currentColor 8%,transparent);}}",
            variant = variant.as_str()
        ));
        return;
    }
    if base == "collapsible" && variant == ComponentVariant::Outlined {
        let (surface, content, content_title) = if family == ColorFamily::Background {
            ("background", "backgroundText", "backgroundTitle")
        } else {
            ("surface", "surfaceText", "surfaceTitle")
        };
        css.push_str(&format!(
            ".{base}.is-outlined.is-{name}{{--dowe-content-text:var(--dowe-{content});--dowe-content-title:var(--dowe-{content_title});background-color:var(--dowe-{surface});color:var(--dowe-{content});border:1px solid var(--dowe-{color});}}"
        ));
        return;
    }
    if base == "toggle-group" {
        match variant {
            ComponentVariant::Solid | ComponentVariant::Soft => css.push_str(&format!(
                ".toggle-group.is-{variant}.is-{name}{{--dowe-content-text:var(--dowe-{text});--dowe-content-title:var(--dowe-{title});background-color:var(--dowe-{color});color:var(--dowe-{text});border-color:transparent;}}",
                variant = variant.as_str()
            )),
            ComponentVariant::Outlined => css.push_str(&format!(
                ".toggle-group.is-outlined.is-{name}{{--dowe-content-text:var(--dowe-{surface_text});--dowe-content-title:var(--dowe-{surface_text});background-color:var(--dowe-{surface});color:var(--dowe-{surface_text});border:1px solid var(--dowe-{color});}}"
            )),
            ComponentVariant::Line | ComponentVariant::Ghost => {
                let content = if matches!(family, ColorFamily::Background | ColorFamily::Surface) {
                    text
                } else {
                    color
                };
                css.push_str(&format!(
                    ".toggle-group.is-{variant}.is-{name}{{--dowe-content-text:var(--dowe-{content});--dowe-content-title:var(--dowe-{content});background-color:transparent;color:var(--dowe-{content});border-color:transparent;}}",
                    variant = variant.as_str()
                ));
            }
        }
        return;
    }
    if base == "toggle-group-item" {
        match variant {
            ComponentVariant::Solid | ComponentVariant::Soft => css.push_str(&format!(
                ".toggle-group-item.is-active.is-solid.is-{name},.toggle-group-item.is-active.is-soft.is-{name}{{background-color:var(--dowe-{text});color:var(--dowe-{color});}}"
            )),
            ComponentVariant::Outlined => css.push_str(&format!(
                ".toggle-group-item.is-active.is-outlined.is-{name}{{background-color:var(--dowe-{surface_text});color:var(--dowe-{surface});}}"
            )),
            ComponentVariant::Line | ComponentVariant::Ghost => css.push_str(&format!(
                ".toggle-group-item.is-active.is-line.is-{name},.toggle-group-item.is-active.is-ghost.is-{name}{{background-color:var(--dowe-{color});color:var(--dowe-{text});}}"
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
                text,
                format!("var(--dowe-{color})"),
            ),
            "soft" => (
                format!("color-mix(in srgb,var(--dowe-{soft}) 50%,transparent)"),
                format!("var(--dowe-{soft})"),
                soft_text,
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
            ".sidenav.is-{variant}.is-{name} .sidenav-entry:hover{{background-color:{hover_background};color:var(--dowe-{accent});}}.sidenav.is-{variant}.is-{name} .sidenav-entry.is-active{{background-color:{active_background};color:var(--dowe-{active_content});border-color:{active_border};font-weight:600;}}"
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
                text,
                format!("var(--dowe-{color})"),
            ),
            "soft" => (
                format!("color-mix(in srgb,var(--dowe-{soft}) 50%,transparent)"),
                format!("var(--dowe-{soft})"),
                soft_text,
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
        let (hover_background, active_background, active_content, active_border) = match variant {
            ComponentVariant::Solid => (format!("color-mix(in srgb,var(--dowe-{color}) 20%,transparent)"), format!("var(--dowe-{color})"), text, format!("var(--dowe-{color})")),
            ComponentVariant::Soft => (format!("color-mix(in srgb,var(--dowe-{soft}) 50%,transparent)"), format!("var(--dowe-{soft})"), soft_text, "transparent".to_string()),
            ComponentVariant::Outlined | ComponentVariant::Line => (format!("color-mix(in srgb,var(--dowe-{soft}) 50%,transparent)"), "transparent".to_string(), color, format!("var(--dowe-{color})")),
            ComponentVariant::Ghost => ("transparent".to_string(), "transparent".to_string(), color, "transparent".to_string()),
        };
        css.push_str(&format!(
            ".navmenu.is-{variant}.is-{name} .navmenu-item:hover{{background-color:{hover_background};color:var(--dowe-{color});}}.navmenu.is-{variant}.is-{name} .navmenu-item.is-active,.navmenu.is-{variant}.is-{name} .navmenu-item.is-open{{background-color:{active_background};color:var(--dowe-{active_content});border-color:{active_border};font-weight:600;}}",
            variant = variant.as_str()
        ));
        return;
    }
    if base == "media" {
        let (button_background, button_content) = match variant {
            ComponentVariant::Solid => (format!("var(--dowe-{soft})"), color),
            ComponentVariant::Soft => (format!("var(--dowe-{color})"), text),
            ComponentVariant::Outlined => ("transparent".to_string(), color),
            ComponentVariant::Line | ComponentVariant::Ghost => ("transparent".to_string(), color),
        };
        css.push_str(&format!(
            ".media.is-{variant}.is-{name} .media-button{{background-color:{button_background};color:var(--dowe-{button_content});}}",
            variant = variant.as_str()
        ));
    }
    match variant {
        ComponentVariant::Solid => css.push_str(&format!(
            ".{base}.is-solid.is-{name}{{--dowe-content-text:var(--dowe-{text});--dowe-content-title:var(--dowe-{title});background-color:var(--dowe-{color});color:var(--dowe-{text});border-color:var(--dowe-{color});}}"
        )),
        ComponentVariant::Soft => css.push_str(&format!(
            ".{base}.is-soft.is-{name}{{--dowe-content-text:var(--dowe-{soft_text});--dowe-content-title:var(--dowe-{soft_title});background-color:var(--dowe-{soft});color:var(--dowe-{soft_text});border-color:var(--dowe-{soft});}}"
        )),
        ComponentVariant::Outlined => {
            let (surface, content, content_title) = if matches!(base, "card" | "modal" | "toast") {
                if family == ColorFamily::Background {
                    ("var(--dowe-background)", "backgroundText", "backgroundTitle")
                } else {
                    ("var(--dowe-surface)", "surfaceText", "surfaceTitle")
                }
            } else {
                ("transparent", color, color)
            };
            css.push_str(&format!(
                ".{base}.is-outlined.is-{name}{{--dowe-content-text:var(--dowe-{content});--dowe-content-title:var(--dowe-{content_title});background-color:{surface};color:var(--dowe-{content});border:1px solid var(--dowe-{color});}}"
            ));
        }
        ComponentVariant::Line => {
            css.push_str(&format!(
                ".{base}.is-line.is-{name}{{background-color:transparent;color:var(--dowe-{color});border-color:transparent;border-bottom:1px solid var(--dowe-{color});border-radius:0;}}"
            ));
        }
        ComponentVariant::Ghost => {
            let content = if matches!(family, ColorFamily::Background | ColorFamily::Surface) {
                text
            } else {
                color
            };
            css.push_str(&format!(
                ".{base}.is-ghost.is-{name}{{--dowe-content-text:var(--dowe-{content});--dowe-content-title:var(--dowe-{content});background-color:transparent;color:var(--dowe-{content});border-color:transparent;}}"
            ));
        }
    }
}

fn append_tabs_variant_css(css: &mut String, family: ColorFamily, variant: TabsVariant) {
    let name = family.as_str();
    let soft = soft_token(family);
    let soft_text = soft_text_token(family);
    let active_background = tabs_active_background(family);
    let active_content = tabs_active_content(family);
    let accent = tabs_accent(family);
    match variant {
        TabsVariant::Solid => css.push_str(&format!(
            ".tabs-list.is-solid.is-{name}{{border-radius:var(--dowe-radius);background-color:var(--dowe-{soft});color:var(--dowe-{soft_text});}}.tabs-list.is-solid.is-{name} .tab{{border-radius:var(--dowe-radius);}}.tabs-list.is-solid.is-{name} .tab.on-active{{background-color:var(--dowe-{active_background});color:var(--dowe-{active_content});}}"
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
            ".tabs-list.is-pills.is-{name}{{border-radius:9999px;background-color:var(--dowe-{soft});color:var(--dowe-{soft_text});}}.tabs-list.is-pills.is-{name} .tab{{border-radius:9999px;}}.tabs-list.is-pills.is-{name} .tab.on-active{{background-color:var(--dowe-{active_background});color:var(--dowe-{active_content});}}"
        )),
        TabsVariant::Stepper => css.push_str(&format!(
            ".tabs-list.is-stepper.is-{name}{{gap:0;padding:0;overflow-x:auto;scroll-snap-type:x proximity;}}.tabs-list.is-stepper.is-{name} .tab{{gap:0.625rem;padding:0.5rem 0;scroll-snap-align:start;color:var(--dowe-muted);}}.tabs-list.is-stepper.is-{name} .tab:not(:last-child)::after{{content:\"\";display:block;width:2rem;height:2px;margin-inline:0.5rem;background:var(--dowe-muted);}}.tabs-list.is-stepper.is-{name} .tab.on-active{{color:var(--dowe-{accent});}}.tabs-list.is-stepper.is-{name} .step-indicator{{display:inline-grid;place-items:center;flex:0 0 auto;width:2rem;height:2rem;border:2px solid var(--dowe-muted);border-radius:9999px;background:var(--dowe-background);color:var(--dowe-muted);font-weight:700;}}.tabs-list.is-stepper.is-{name} .tab.on-active .step-indicator{{border-color:var(--dowe-{accent});background:var(--dowe-{active_background});color:var(--dowe-{active_content});}}.tabs.is-start .tabs-list.is-stepper.is-{name} .tab{{width:100%;}}.tabs.is-start .tabs-list.is-stepper.is-{name} .tab:not(:last-child)::after{{position:absolute;top:2.5rem;left:0.9375rem;width:2px;height:1.5rem;margin:0;background:var(--dowe-muted);}}"
        )),
    }
}

fn tabs_active_background(family: ColorFamily) -> &'static str {
    if family == ColorFamily::Muted {
        text_token(family)
    } else {
        family.as_str()
    }
}

fn tabs_active_content(family: ColorFamily) -> &'static str {
    if family == ColorFamily::Muted {
        family.as_str()
    } else {
        text_token(family)
    }
}

fn tabs_accent(family: ColorFamily) -> &'static str {
    match family {
        ColorFamily::Muted | ColorFamily::Background | ColorFamily::Surface => text_token(family),
        _ => family.as_str(),
    }
}
