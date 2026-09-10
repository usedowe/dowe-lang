fn render_side_nav_item_html(
    base: &str,
    item: &SideNavItem,
    index: usize,
    context: &ReactiveRenderContext,
) -> String {
    match item {
        SideNavItem::Header(props) => {
            render_side_nav_entry_html(base, props, &format!("{base}-header"), context)
        }
        SideNavItem::Item(props) => {
            render_side_nav_entry_html(base, props, &format!("{base}-entry"), context)
        }
        SideNavItem::Divider => format!(r#"<div class="{base}-divider"></div>"#),
        SideNavItem::Submenu {
            props,
            open,
            bordered,
            items,
        } => {
            let classes = if *open {
                format!("{base}-submenu is-open")
            } else {
                format!("{base}-submenu")
            };
            let classes = if *bordered {
                classes
            } else {
                format!("{classes} is-unbordered")
            };
            let mut html = format!(
                r#"<details class="{classes}" data-dowe-{base}-submenu data-dowe-nav-submenu-key="{index}"{}><summary class="{base}-entry {base}-trigger" aria-expanded="{}">{}{}{}</summary><div class="{base}-submenu-content"><div class="{base}-submenu-content-inner">"#,
                if *open { " open" } else { "" },
                if *open { "true" } else { "false" },
                render_side_nav_icon_html(base, props.icon.as_ref(), context),
                render_side_nav_content_html(base, props),
                render_side_nav_arrow_html(base, context)
            );
            for item in items {
                html.push_str(&render_side_nav_entry_html(
                    base,
                    item,
                    &format!("{base}-entry {base}-subitem"),
                    context,
                ));
            }
            html.push_str("</div></div></details>");
            html
        }
    }
}

fn render_rail_nav_html(
    props: &RailNavProps,
    items: &[RailNavItem],
    context: &ReactiveRenderContext,
) -> String {
    let mut html = format!(
        "<nav{}>",
        attrs(
            rail_nav_classes(props),
            Some(&props.style.element),
            Some(r#" aria-label="Rail navigation""#),
            context,
        )
    );
    for item in items {
        match item {
            RailNavItem::Item(item) => {
                html.push_str(&render_rail_nav_item_html(item, props.show_labels, context))
            }
            RailNavItem::Divider => html.push_str(r#"<div class="railnav-divider"></div>"#),
        }
    }
    html.push_str("</nav>");
    html
}

fn render_rail_nav_item_html(
    props: &RailNavItemProps,
    show_label: bool,
    context: &ReactiveRenderContext,
) -> String {
    let classes = class_attr(vec!["railnav-item".to_string()]);
    let aria = format!(r#" aria-label="{}""#, escape_attr(&props.label));
    let (tag, attributes) = match props.navigation.as_ref() {
        Some(action) => (
            "a",
            format!(
                "{classes}{aria}{}",
                side_nav_navigation_attrs("railnav", action)
            ),
        ),
        None => (
            "button",
            format!(
                r#"{classes}{aria} type="button"{}"#,
                props
                    .on_click
                    .as_deref()
                    .map(|action| format!(
                        r#" data-dowe-click="{}""#,
                        escape_attr(&context.action_id(action))
                    ))
                    .unwrap_or_default()
            ),
        ),
    };
    let icon = format!(
        r#"<span class="railnav-icon">{}</span>"#,
        render_svg_html(&props.icon.props, &props.icon.paths, context)
    );
    let label = localized_span("railnav-label", &props.label, props.i18n.as_deref());
    let item = format!("<{tag}{attributes}>{icon}{}</{tag}>", if show_label { label.as_str() } else { "" });
    if show_label {
        item
    } else {
        format!(
            r#"<span class="tooltip railnav-tooltip" data-dowe-tooltip>{item}<span class="tooltip-popover is-solid is-muted position-end" role="tooltip"><span class="tooltip-arrow"></span>{label}</span></span>"#
        )
    }
}

fn render_side_nav_arrow_html(base: &str, context: &ReactiveRenderContext) -> String {
    let arrow = side_nav_submenu_arrow_icon();
    format!(
        r#"<span class="{base}-chevron" aria-hidden="true">{}</span>"#,
        render_svg_html(&arrow.props, &arrow.paths, context)
    )
}

fn render_side_nav_entry_html(
    base: &str,
    props: &SideNavItemProps,
    classes: &str,
    context: &ReactiveRenderContext,
) -> String {
    let (tag, attrs, close) = side_nav_entry_tags(base, props, classes, context);
    format!(
        "<{tag}{attrs}>{}{}</{close}>",
        render_side_nav_icon_html(base, props.icon.as_ref(), context),
        render_side_nav_content_html(base, props)
    )
}

fn side_nav_entry_tags(
    base: &str,
    props: &SideNavItemProps,
    classes: &str,
    context: &ReactiveRenderContext,
) -> (&'static str, String, &'static str) {
    let classes = class_attr(
        classes
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>(),
    );
    match props.navigation.as_ref() {
        Some(action) => (
            "a",
            format!("{classes}{}", side_nav_navigation_attrs(base, action)),
            "a",
        ),
        None if props.on_click.is_some() => (
            "button",
            format!(
                r#"{classes} type="button" data-dowe-click="{}""#,
                escape_attr(&context.action_id(props.on_click.as_deref().expect("onClick")))
            ),
            "button",
        ),
        None => ("div", classes, "div"),
    }
}

fn side_nav_navigation_attrs(base: &str, action: &NavigationAction) -> String {
    match action {
        NavigationAction::Internal {
            path,
            fragment,
            operation,
        } => {
            let href = internal_href(path, fragment.as_deref());
            format!(
                r#"{} data-dowe-{base}-href="{}""#,
                navigation_attrs(&href, *operation),
                escape_attr(path)
            )
        }
        NavigationAction::Section {
            fragment,
            operation,
        } => navigation_attrs(&format!("#{fragment}"), *operation),
        NavigationAction::External {
            url,
            web_target,
            native_external_mode,
        } => external_attrs(url, *web_target, *native_external_mode),
        NavigationAction::Back => r#" data-dowe-history="back""#.to_string(),
    }
}

fn render_side_nav_icon_html(
    base: &str,
    icon: Option<&SideNavIcon>,
    context: &ReactiveRenderContext,
) -> String {
    icon.map(|icon| {
        format!(
            r#"<span class="{base}-icon">{}</span>"#,
            render_svg_html(&icon.props, &icon.paths, context)
        )
    })
    .unwrap_or_default()
}

fn render_side_nav_content_html(base: &str, props: &SideNavItemProps) -> String {
    let description = props
        .description
        .as_deref()
        .map(|value| localized_span(&format!("{base}-description"), value, props.description_i18n.as_deref()))
        .unwrap_or_default();
    let status = props
        .status
        .as_deref()
        .map(|value| localized_span(&format!("{base}-status"), value, props.status_i18n.as_deref()))
        .unwrap_or_default();
    format!(
        r#"<span class="{base}-copy">{}{description}</span>{status}"#,
        localized_span(&format!("{base}-label"), &props.label, props.i18n.as_deref())
    )
}

fn localized_span(class: &str, value: &str, i18n: Option<&str>) -> String {
    let i18n = i18n
        .map(|key| format!(r#" data-dowe-i18n="{}""#, escape_attr(key)))
        .unwrap_or_default();
    format!(
        r#"<span class="{}"{i18n}>{}</span>"#,
        escape_attr(class),
        escape_html(value)
    )
}

fn svg_path_fill(fill: SvgPathFill) -> String {
    match fill {
        SvgPathFill::None => "none".to_string(),
        SvgPathFill::CurrentColor => "currentColor".to_string(),
        SvgPathFill::Color(token) => format!("var(--dowe-{})", token.as_str()),
        SvgPathFill::RawFill { color, .. } | SvgPathFill::RawStroke { color, .. } => color.to_string(),
        SvgPathFill::LiteralFill { red, green, blue, .. }
        | SvgPathFill::LiteralStroke { red, green, blue, .. } => {
            format!("#{red:02x}{green:02x}{blue:02x}")
        }
        SvgPathFill::Fill { color, .. } | SvgPathFill::Stroke { color, .. } => color
            .map(|token| format!("var(--dowe-{})", token.as_str()))
            .unwrap_or_else(|| "currentColor".to_string()),
    }
}

fn svg_path_attributes(paint: SvgPathFill) -> String {
    match paint {
        SvgPathFill::RawFill { color, opacity, even_odd } => format!(
            " fill=\"{}\"{}{}",
            escape_attr(color),
            if opacity == 255 { String::new() } else { format!(" opacity=\"{:.3}\"", opacity as f32 / 255.0) },
            if even_odd { " fill-rule=\"evenodd\" clip-rule=\"evenodd\"" } else { "" }
        ),
        SvgPathFill::RawStroke { color, opacity, width, line_cap, line_join } => format!(
            " fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\" stroke-linecap=\"{}\" stroke-linejoin=\"{}\"{}",
            escape_attr(color),
            width as f32 / 100.0,
            match line_cap { SvgLineCap::Butt => "butt", SvgLineCap::Round => "round", SvgLineCap::Square => "square" },
            match line_join { SvgLineJoin::Miter => "miter", SvgLineJoin::Round => "round", SvgLineJoin::Bevel => "bevel" },
            if opacity == 255 { String::new() } else { format!(" opacity=\"{:.3}\"", opacity as f32 / 255.0) }
        ),
        SvgPathFill::LiteralFill { red, green, blue, opacity, even_odd } => format!(
            " fill=\"#{red:02x}{green:02x}{blue:02x}\"{}{}",
            if opacity == 255 { String::new() } else { format!(" opacity=\"{:.3}\"", opacity as f32 / 255.0) },
            if even_odd { " fill-rule=\"evenodd\" clip-rule=\"evenodd\"" } else { "" }
        ),
        SvgPathFill::LiteralStroke { red, green, blue, opacity, width, line_cap, line_join } => format!(
            " fill=\"none\" stroke=\"#{red:02x}{green:02x}{blue:02x}\" stroke-width=\"{:.2}\" stroke-linecap=\"{}\" stroke-linejoin=\"{}\"{}",
            width as f32 / 100.0,
            match line_cap { SvgLineCap::Butt => "butt", SvgLineCap::Round => "round", SvgLineCap::Square => "square" },
            match line_join { SvgLineJoin::Miter => "miter", SvgLineJoin::Round => "round", SvgLineJoin::Bevel => "bevel" },
            if opacity == 255 { String::new() } else { format!(" opacity=\"{:.3}\"", opacity as f32 / 255.0) }
        ),
        SvgPathFill::Fill { color, opacity, even_odd } => format!(
            " fill=\"{}\"{}{}",
            escape_attr(&color.map(|token| format!("var(--dowe-{})", token.as_str())).unwrap_or_else(|| "currentColor".to_string())),
            if opacity == 255 { String::new() } else { format!(" opacity=\"{:.3}\"", opacity as f32 / 255.0) },
            if even_odd { " fill-rule=\"evenodd\" clip-rule=\"evenodd\"" } else { "" }
        ),
        SvgPathFill::Stroke { color, opacity, width, line_cap, line_join } => format!(
            " fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\" stroke-linecap=\"{}\" stroke-linejoin=\"{}\"{}",
            escape_attr(&color.map(|token| format!("var(--dowe-{})", token.as_str())).unwrap_or_else(|| "currentColor".to_string())),
            width as f32 / 100.0,
            match line_cap { SvgLineCap::Butt => "butt", SvgLineCap::Round => "round", SvgLineCap::Square => "square" },
            match line_join { SvgLineJoin::Miter => "miter", SvgLineJoin::Round => "round", SvgLineJoin::Bevel => "bevel" },
            if opacity == 255 { String::new() } else { format!(" opacity=\"{:.3}\"", opacity as f32 / 255.0) }
        ),
        _ => format!(" fill=\"{}\"", escape_attr(&svg_path_fill(paint))),
    }
}
