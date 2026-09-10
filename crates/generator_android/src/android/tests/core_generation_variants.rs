#[test]
fn generates_tinted_card_shadow_for_compose() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Card {
        props: VariantProps {
            style: StyleProps {
                shadow: Some(ResponsiveValue::scalar(ShadowSize::Lg)),
                shadow_color: Some(ColorFamily::Primary),
                rounded: Some(ResponsiveValue::scalar(RoundedSize::Md)),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Raised")],
    };

    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains(
        ".doweShadow(radius = doweResponsive(viewportWidth, xs = 44.dp) ?: 0.dp, shape = RoundedCornerShape(doweResponsive(viewportWidth, xs = 8.dp) ?: DoweDesign.radius), color = DoweDesign.primary, alpha = 0.28f)"
    ));
    assert!(views.content.contains(
        ".doweShadow(radius = doweResponsive(viewportWidth, xs = 44.dp) ?: 0.dp, shape = RoundedCornerShape(doweResponsive(viewportWidth, xs = 8.dp) ?: DoweDesign.radius), color = DoweDesign.primary, alpha = 0.28f).doweRounded(doweResponsive(viewportWidth, xs = 8.dp))"
    ));
    assert!(views.content.contains("private fun Modifier.doweShadow("));
    assert!(views.content.contains("dropShadow("));
    assert!(views.content.contains("DoweDropShadow("));
    assert!(
        views
            .content
            .contains("elevation = CardDefaults.cardElevation(defaultElevation = 0.dp)")
    );
    assert!(
        !views
            .content
            .contains("CardDefaults.cardElevation(defaultElevation = doweResponsive")
    );
}

#[test]
fn generates_centered_icon_button_without_empty_android_label() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::Button {
                props: VariantProps {
                    style: StyleProps {
                        sizing: SizingProps {
                            w: Some(ResponsiveValue::scalar(SizeValue::Scale(
                                ScaleValue::from_half_steps(20),
                            ))),
                            h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                                ScaleValue::from_half_steps(20),
                            ))),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    size: Some(ButtonSize::Md),
                    icon_start: Some(solar_control_icon("settings").expect("settings icon")),
                    icon_only: true,
                    label: Some("Open settings".to_string()),
                    navigation: Some(NavigationAction::Internal {
                        path: "/settings".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                    ..Default::default()
                },
                children: Vec::new(),
            },
            ViewNode::Button {
                props: VariantProps {
                    style: StyleProps {
                        spacing: SpacingProps {
                            px: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                            py: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(5))),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    navigation: Some(NavigationAction::Internal {
                        path: "/save".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                    ..Default::default()
                },
                children: vec![text("Save")],
            },
        ],
    };
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let compose = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("Compose pages");
    let dev = dev_java_source(&output);
    let route_shards = output
        .files
        .iter()
        .filter(|file| {
            file.relative_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("DoweDevRoute"))
        })
        .map(|file| file.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(compose.content.contains(".semantics { contentDescription = \"Open settings\" }"));
    assert!(compose
        .content
        .contains(".doweWidth(doweResponsive(viewportWidth, xs = DoweSize.Fixed(40.dp)))"));
    assert!(compose
        .content
        .contains(".doweHeight(doweResponsive(viewportWidth, xs = DoweSize.Fixed(40.dp)))"));
    assert!(
        compose
            .content
            .contains("onClick = { navigate(\"push\", \"/settings\", null) }")
    );
    assert!(
        compose
            .content
            .contains("contentPadding = PaddingValues(start = doweResponsive(viewportWidth")
    );
    assert!(
        compose
            .content
            .contains("onClick = { navigate(\"push\", \"/save\", null) }")
    );
    assert!(dev.content.contains("setGravity(Gravity.CENTER)"));
    assert!(
        dev.content
            .contains("setContentDescription(\"Open settings\")")
    );
    assert!(
        dev.content
            .contains("setOnClickListener(v -> doweNavigate(\"push\", \"/settings\", null))")
    );
    assert!(
        dev.content
            .contains("setOnClickListener(v -> doweNavigate(\"push\", \"/save\", null))")
    );
    assert!(!route_shards.contains(r#"doweText("")"#));
}

#[test]
fn emits_generic_variant_bindings_on_android() {
    let mut route = route();
    route.page_tree = ViewNode::Button {
        props: VariantProps {
            variant_binding: Some(dowe_components::PropBinding::string("item.variant")),
            ..Default::default()
        },
        children: vec![text("Action")],
    };
    let generated = all_android_source(&generate_android(&[route], &FontConfig::default(), &DesignConfig::default(), &[]));
    assert!(!generated.is_empty());
}

#[test]
fn generates_java_runtime_catalogs_from_component_contract() {
    let output = generate_android(&[route()], &FontConfig::default(), &DesignConfig::default(), &[]);
    let source = dev_java_source(&output).content;

    assert!(source.contains("DOWE_PROP_SCHEMES = {\"primary\", \"secondary\", \"accent\", \"muted\", \"success\", \"info\", \"warning\", \"danger\"}"));
}

#[test]
fn generates_java_runtime_variant_metadata_and_refresh() {
    let mut route = route();
    route.page_tree = ViewNode::Button {
        props: VariantProps {
            reactive: ReactiveVariantProps {
                variant: Some("item.variant".to_string()),
                scheme: Some("theme.scheme".to_string()),
                size: Some("item.size".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Action")],
    };
    let generated = all_android_source(&generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));
    assert!(generated.contains("DOWE_VARIANT_TAG") || generated.contains("doweApplyReactiveVariant"));
    assert!(generated.contains("doweApplyReactiveVariant(view)"));
    assert!(generated.contains("doweButtonContent(variant, scheme)"));
}

#[test]
fn preserves_static_button_variants_with_reactive_scheme_on_android() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "scheme01".to_string(),
            name: "schemeChoice".to_string(),
            storage_key: "schemeChoice".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::String("primary".to_string()),
            schema: None,
        }],
        actions: Vec::new(),
        children: [
            ComponentVariant::Solid,
            ComponentVariant::Solid,
            ComponentVariant::Outlined,
            ComponentVariant::Ghost,
        ]
        .into_iter()
        .map(|variant| ViewNode::Button {
            props: VariantProps {
                variant: Some(variant),
                reactive: ReactiveVariantProps {
                    scheme: Some("schemeChoice".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Action")],
        })
        .collect(),
    };
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);

    assert!(generated.contains("doweButtonContainer(\"solid\", state.text("));

    assert!(generated.contains("doweButtonContainer(\"outlined\", state.text("));
    assert!(generated.contains("doweButtonContainer(\"ghost\", state.text("));

    assert!(generated.contains("doweButtonContainer(\"outlined\", doweTextValue("));
    assert!(generated.contains("if (\"outlined\" == \"outlined\") BorderStroke"));
    assert!(generated.contains("\"outlined\".equals(\"outlined\")"));
}

#[test]
fn generates_diffuse_semantic_shadows_for_portable_components() {
    let shadow_style = |size, color| StyleProps {
        shadow: Some(ResponsiveValue::scalar(size)),
        shadow_color: Some(color),
        ..Default::default()
    };
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::Card {
                props: VariantProps {
                    style: StyleProps {
                        rounded: Some(ResponsiveValue::scalar(RoundedSize::Md)),
                        ..shadow_style(ShadowSize::Md, ColorFamily::Primary)
                    },
                    ..Default::default()
                },
                children: vec![text("Card")],
            },
            ViewNode::Button {
                props: VariantProps {
                    style: shadow_style(ShadowSize::Sm, ColorFamily::Secondary),
                    ..Default::default()
                },
                children: vec![text("Button")],
            },
            ViewNode::Avatar {
                props: AvatarProps {
                    style: VariantProps {
                        style: shadow_style(ShadowSize::Lg, ColorFamily::Accent),
                        ..Default::default()
                    },
                    src: None,
                    name: Some("Dowe".to_string()),
                    name_binding: None,
                    alt: "Dowe".to_string(),
                    alt_binding: None,
                    size: AvatarSize::Md,
                    size_binding: None,
                    status: None,
                    bordered: false,
                },
                icon: None,
            },
            ViewNode::Chip {
                props: ChipProps {
                    style: VariantProps {
                        style: shadow_style(ShadowSize::Xs, ColorFamily::Success),
                        ..Default::default()
                    },
                    on_close: None,
                },
                value: "Chip".to_string(),
                start: None,
                end: None,
            },
            ViewNode::Input {
                props: VariantProps {
                    style: StyleProps {
                        sizing: SizingProps {
                            w: Some(ResponsiveValue::scalar(SizeValue::Full)),
                            ..Default::default()
                        },
                        rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
                        ..shadow_style(ShadowSize::Md, ColorFamily::Info)
                    },
                    variant: Some(ComponentVariant::Outlined),
                    color: Some(ColorFamily::Info),
                    label: Some("Workspace".to_string()),
                    placeholder: Some("dowe-app".to_string()),
                    ..Default::default()
                },
            },
        ],
    };

    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    for expected in [
        "radius = doweResponsive(viewportWidth, xs = 24.dp) ?: 0.dp, shape = RoundedCornerShape(doweResponsive(viewportWidth, xs = 8.dp) ?: DoweDesign.radius), color = DoweDesign.primary, alpha = 0.28f",
        "radius = doweResponsive(viewportWidth, xs = 12.dp) ?: 0.dp, shape = RoundedCornerShape(DoweDesign.radius), color = DoweDesign.secondary, alpha = 0.28f",
        "radius = doweResponsive(viewportWidth, xs = 44.dp) ?: 0.dp, shape = RoundedCornerShape(999.dp), color = DoweDesign.accent, alpha = 0.28f",
        "radius = doweResponsive(viewportWidth, xs = 2.dp) ?: 0.dp, shape = RoundedCornerShape(null ?: DoweDesign.radius), color = DoweDesign.success, alpha = 0.28f",
        "radius = doweResponsive(viewportWidth, xs = 24.dp) ?: 0.dp, shape = RoundedCornerShape(doweResponsive(viewportWidth, xs = 12.dp) ?: DoweDesign.radius), color = DoweDesign.info, alpha = 0.28f",
    ] {
        assert!(views.content.contains(expected), "missing {expected}");
    }
    assert!(views.content.contains("radius <= 2.dp -> 0.12f"));
    assert!(views.content.contains("radius <= 12.dp -> 0.14f"));
    assert!(views.content.contains("radius <= 24.dp -> 0.16f"));
    assert!(views.content.contains("radius <= 44.dp -> 0.18f"));
    assert!(views.content.contains("else -> 0.22f"));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("BlurMaskFilter.Blur.NORMAL"));
    assert!(dev.content.contains("canvas.clipOutPath(surface)"));
    assert!(dev.content.contains("doweDrawChildShadows(this, canvas)"));
    assert!(dev.content.contains("view.setStateListAnimator(null);"));
    assert!(!dev.content.contains("setOutlineAmbientShadowColor"));
    assert!(!dev.content.contains("setOutlineSpotShadowColor"));
    for expected in [
        "doweResponsiveInt(viewportWidth, 24, null, null, null, null), DOWE_PRIMARY, doweFloat(doweResponsiveFloat(viewportWidth, 8f, null, null, null, null), DOWE_RADIUS), 0.28f",
        "doweResponsiveInt(viewportWidth, 12, null, null, null, null), DOWE_SECONDARY, DOWE_RADIUS, 0.28f",
        "doweResponsiveInt(viewportWidth, 44, null, null, null, null), DOWE_ACCENT, 999f, 0.28f",
        "doweResponsiveInt(viewportWidth, 2, null, null, null, null), DOWE_SUCCESS, DOWE_RADIUS, 0.28f",
    ] {
        assert!(dev.content.contains(expected), "missing {expected}");
    }
    let field_line = dev
        .content
        .lines()
        .find(|line| line.contains(".setHint(\"dowe-app\")"))
        .expect("input field");
    let field = field_line
        .trim_start()
        .split('.')
        .next()
        .expect("field variable");
    assert!(dev.content.contains(&format!(
        "doweShadow({field}, doweResponsiveInt(viewportWidth, 24, null, null, null, null), DOWE_INFO, doweFloat(doweResponsiveFloat(viewportWidth, 12f, null, null, null, null), DOWE_RADIUS), 0.28f);"
    )));
    assert!(dev.content.contains(&format!(
        "doweRound({field}, doweResponsiveFloat(viewportWidth, 12f, null, null, null, null));"
    )));
}

