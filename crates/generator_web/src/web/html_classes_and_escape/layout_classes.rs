fn grid_classes(props: &GridProps) -> Vec<String> {
    let mut classes = vec!["grid".to_string()];
    append_style_classes(&mut classes, &props.style);
    append_responsive_classes(&mut classes, "grid-cols", props.columns.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(&mut classes, "grid-rows", props.rows.as_ref(), |value| {
        value.class_suffix()
    });
    append_responsive_classes(
        &mut classes,
        "grid-justify",
        props.justify.as_ref(),
        |value| value.as_str().to_string(),
    );
    append_responsive_classes(&mut classes, "grid-align", props.align.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(&mut classes, "gap", props.gap.as_ref(), |value| {
        value.class_suffix()
    });
    classes
}

fn variant_classes(base: &str, props: &VariantProps) -> Vec<String> {
    let mut classes = vec![base.to_string()];
    if base == "button" {
        classes.push(format!(
            "button-{}",
            props.size.unwrap_or(ButtonSize::Md).as_str()
        ));
    }
    append_style_classes(&mut classes, &props.style);
    if base == "card" {
        append_container_visual_classes(&mut classes, &props.style);
        append_responsive_classes(&mut classes, "card-color", props.style.text.as_ref(), |value| {
            value.as_str().to_string()
        });
    }
    let default_variant = if base == "accordion" {
        ComponentVariant::Ghost
    } else {
        ComponentVariant::Solid
    };
    classes.push(format!(
        "is-{}",
        props.variant.unwrap_or(default_variant).as_str()
    ));
    classes.push(format!(
        "is-{}",
        props.color.unwrap_or(ColorFamily::Primary).as_str()
    ));
    classes
}

fn bar_classes(base: &str, props: &BarProps) -> Vec<String> {
    let mut classes = variant_classes(base, &props.style);
    if base == "appbar" {
        classes.push(format!("position-{}", props.position.as_str()));
    }
    if props.bordered {
        classes.push("is-bordered".to_string());
    }
    if props.blurred {
        classes.push("is-blurred".to_string());
    }
    if props.floating {
        classes.push("is-floating".to_string());
    }
    if props.hide_on_scroll {
        classes.push("is-hide-on-scroll".to_string());
    }
    if props.dock_on_scroll {
        classes.push("is-dock-on-scroll".to_string());
    }
    classes
}

fn bar_content_classes(base: &str, props: &BarProps) -> Vec<String> {
    let mut classes = vec![format!("{base}-content")];
    if props.boxed && base != "footer" {
        classes.push("is-boxed".to_string());
    }
    classes
}

fn footer_inner_classes(props: &BarProps) -> Vec<String> {
    let mut classes = vec!["footer-inner".to_string()];
    if props.boxed {
        classes.push("is-boxed".to_string());
    }
    classes
}

fn side_nav_classes(base: &str, props: &SideNavProps) -> Vec<String> {
    let mut classes = variant_classes(base, &props.style);
    classes.push(format!("{base}-{}", props.size.as_str()));
    if props.wide {
        classes.push("is-wide".to_string());
    }
    classes
}

fn rail_nav_classes(props: &RailNavProps) -> Vec<String> {
    let mut classes = variant_classes("railnav", &props.style);
    classes.push(format!("railnav-{}", props.size.as_str()));
    if props.show_labels {
        classes.push("has-labels".to_string());
    }
    classes
}

fn sidebar_classes(props: &SidebarProps) -> Vec<String> {
    variant_classes("sidebar", &props.style)
}

fn nav_menu_classes(props: &NavMenuProps) -> Vec<String> {
    let mut classes = variant_classes("navmenu", &props.style);
    classes.push(format!("navmenu-{}", props.size.as_str()));
    classes
}

fn scaffold_classes(props: &ScaffoldProps) -> Vec<String> {
    let mut classes = vec!["scaffold".to_string()];
    append_style_classes(&mut classes, &props.style);
    if props.boxed {
        classes.push("is-boxed".to_string());
    }
    classes
}

fn tabs_classes(props: &TabsProps) -> Vec<String> {
    let mut classes = vec![
        "tabs".to_string(),
        format!("is-{}", props.position.as_str()),
    ];
    if props.variant == TabsVariant::Stepper {
        classes.push("stepper".to_string());
    }
    append_style_classes(&mut classes, &props.style);
    classes
}

fn tabs_list_classes(props: &TabsProps) -> Vec<String> {
    vec![
        "tabs-list".to_string(),
        format!("is-{}", props.variant.as_str()),
        format!("is-{}", props.color.as_str()),
    ]
}

fn drawer_panel_classes(props: &DrawerProps) -> Vec<String> {
    let mut classes = vec!["drawer-panel".to_string()];
    append_show_classes(&mut classes, props.style.element.show.as_ref());
    classes
}

fn drawer_classes(props: &DrawerProps) -> Vec<String> {
    let mut classes = variant_classes("drawer", &props.style);
    classes.push(format!("is-{}", props.position.as_str()));
    classes
}

fn avatar_classes(props: &AvatarProps) -> Vec<String> {
    let mut classes = variant_classes("avatar", &props.style);
    classes.push(format!("avatar-{}", props.size.as_str()));
    if props.bordered {
        classes.push("is-bordered".to_string());
    }
    if props.style.element.on_click.is_some() || props.style.navigation.is_some() {
        classes.push("is-clickable".to_string());
    }
    classes
}

fn avatar_group_classes(props: &AvatarGroupProps) -> Vec<String> {
    let mut classes = variant_classes("avatar-group", &props.style);
    classes.push(format!("avatar-group-{}", props.size.as_str()));
    if props.inline {
        classes.push("is-inline".to_string());
    }
    if props.auto_fit {
        classes.push("is-auto-fit".to_string());
    }
    if props.bordered {
        classes.push("is-bordered".to_string());
    }
    classes
}

fn chat_box_classes(props: &ChatBoxProps) -> Vec<String> {
    let mut classes = variant_classes("chat-box", &props.style);
    classes.push(format!("is-{}", props.mode.as_str()));
    classes
}

fn empty_classes(props: &EmptyProps) -> Vec<String> {
    let mut classes = variant_classes("empty", &props.style);
    classes.push(format!("is-{}", props.kind.as_str()));
    classes
}

fn marquee_classes(props: &MarqueeProps) -> Vec<String> {
    let mut classes = vec![
        "marquee".to_string(),
        format!("is-{}", props.orientation.as_str()),
        format!("is-{}", props.speed.as_str()),
    ];
    append_style_classes(&mut classes, &props.style);
    if props.pause_on_hover {
        classes.push("pause-on-hover".to_string());
    }
    if props.reverse {
        classes.push("is-reverse".to_string());
    }
    if props.fade {
        classes.push("has-fade".to_string());
    }
    classes
}

fn type_writer_classes(props: &TypeWriterProps) -> Vec<String> {
    let mut classes = vec!["typewriter".to_string()];
    append_style_classes(&mut classes, &props.style);
    classes
}

fn rich_text_classes(props: &TextProps) -> Vec<String> {
    let mut classes = vec!["rich-text".to_string()];
    let typography = if props.title { "title" } else { "text" };
    if let Some(size) = &props.size {
        append_responsive_classes(&mut classes, typography, Some(size), |value| {
            value.as_str().to_string()
        });
    } else {
        classes.push(format!("{typography}-md"));
    }
    append_style_classes(&mut classes, &props.style);
    append_responsive_classes(&mut classes, "weight", props.weight.as_ref(), |value| {
        value.as_str().to_string()
    });
    append_responsive_classes(
        &mut classes,
        "tracking",
        props.letter_spacing.as_ref(),
        |value| value.as_str().to_string(),
    );
    classes
}

