#[test]
fn accepts_view_group_roots_in_dev_route_shards() {
    for layout_index in [None, Some(0)] {
        let source = super::dev_route_shard(&route(), layout_index, "DoweDevRouteTest", "dev.test");
        assert!(source.contains("static void render(DoweDevActivity runtime, ViewGroup root)"));
    }
}

#[test]
fn generates_persistent_view_store_for_compose_and_dev_shell() {
    let mut persistent = route();
    persistent.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "session01".to_string(),
            name: "session".to_string(),
            storage_key: "views/store/session:session".to_string(),
            scope: dowe_components::ViewSignalScope::Global,
            storage: dowe_components::ViewSignalStorage::Local,
            initial: ViewSignalValue::Object(vec![(
                "token".to_string(),
                ViewSignalValue::String(String::new()),
            )]),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![text("Session")],
    };
    let output = generate_android(
        &[persistent],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        generated
            .contains("DoweSignalMetadata(\"views/store/session:session\", \"global\", \"local\")")
    );
    assert!(generated.contains(
        "dowePutSignalMetadata(\"session01\", \"views/store/session:session\", \"global\", \"local\")"
    ));
    assert!(generated.contains("getSharedPreferences(\"dowe_view_state\""));
    assert!(generated.contains("compatibleSignalValue(stored, initial[id])"));
    assert!(generated.contains("candidate.scope == \"global\" && candidate.name == metadata.name"));
}

#[test]
fn generates_flex_item_behavior_for_flex_parents_but_not_grid_children() {
    let mut flex_route = route();
    flex_route.layout_tree = ViewNode::Children;
    flex_route.page_tree = ViewNode::Section {
        props: StyleProps {
            sizing: SizingProps {
                h: Some(ResponsiveValue::scalar(SizeValue::ViewportMinus(
                    ScaleValue::from_half_steps(0),
                ))),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![ViewNode::Grid {
            props: GridProps {
                style: StyleProps {
                    flex: Some(ResponsiveValue::ordered(vec![
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Xs,
                            value: FlexItem::Fill,
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Md,
                            value: FlexItem::None,
                        },
                    ])),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![ViewNode::Grid {
                props: GridProps {
                    style: StyleProps {
                        flex: Some(ResponsiveValue::scalar(FlexItem::Fill)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("Grid item")],
            }],
        }],
    };
    let output = generate_android(
        &[flex_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let compose = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("compose pages");
    let dev = dev_java_source(&output);

    assert_eq!(
        compose
            .content
            .matches("Modifier.weight(1f, fill = true)")
            .count(),
        1
    );
    assert!(
        compose
            .content
            .contains("xs = Modifier.weight(1f, fill = true), md = Modifier")
    );
    assert!(compose.content.contains(".fillMaxHeight()"));
    assert!(dev.content.contains("DOWE_FLEX_FILL"));
    assert!(dev.content.contains("DOWE_FLEX_NONE"));
    assert!(dev.content.contains("doweApplyFlexItem("));
    assert!(dev.content.contains("doweMeasureColumn"));
    assert!(all_android_source(&output).contains("int[] rowHeights = new int[0];"));
}

#[test]
fn preserves_multiline_text_in_compose_and_dev_shell() {
    let mut multiline = route();
    multiline.page_tree = ViewNode::Title {
        props: TextProps::default(),
        value: "Full-stack development,\nfrom one codebase".to_string(),
    };
    let output = generate_android(
        &[multiline],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let compose = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("Compose pages");
    assert!(
        compose
            .content
            .contains("Full-stack development,\\nfrom one codebase")
    );

    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("Full-stack development,\\nfrom one codebase")
    );
}

#[test]
fn inherits_container_foreground_and_preserves_text_overrides() {
    let mut color_route = route();
    color_route.layout_tree = ViewNode::Children;
    color_route.page_tree = container_foreground_tree();
    let output = generate_android(
        &[color_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let compose = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("Compose pages");

    assert!(compose.content.contains(
        "CompositionLocalProvider(LocalContentColor provides (doweResponsive(viewportWidth, xs = DoweDesign.primaryText) ?: LocalContentColor.current))"
    ));
    assert!(
        compose
            .content
            .contains("Text(\"Box inherited\", modifier = Modifier, color = Color.Unspecified")
    );
    assert!(compose.content.contains(
        "Text(\"Box override\", modifier = Modifier, color = doweResponsive(viewportWidth, xs = DoweDesign.danger) ?: LocalContentColor.current"
    ));
    assert!(compose.content.contains(
        "CardDefaults.cardColors(containerColor = DoweDesign.muted, contentColor = DoweDesign.mutedText)"
    ));
    assert!(
        compose
            .content
            .contains("Text(\"Card inherited\", modifier = Modifier, color = Color.Unspecified")
    );
    assert!(compose.content.contains(
        "CompositionLocalProvider(LocalDoweTitleColor provides DoweDesign.mutedTitle)"
    ));
    assert!(compose.content.contains(
        "Text(\"Card title inherited\", modifier = Modifier, color = LocalDoweTitleColor.current"
    ));
    assert!(compose.content.contains(
        "Text(\"Card override\", modifier = Modifier, color = doweResponsive(viewportWidth, xs = DoweDesign.warning) ?: LocalDoweTitleColor.current"
    ));
    for (label, token) in [
        ("Section inherited", "secondaryText"),
        ("Flex inherited", "accentText"),
        ("Grid inherited", "mutedText"),
        ("Brand inherited", "surfaceText"),
        ("Banner inherited", "infoText"),
        ("Marquee inherited", "warningText"),
        ("Scaffold inherited", "dangerText"),
    ] {
        assert!(compose.content.contains(&format!(
            "CompositionLocalProvider(LocalContentColor provides (doweResponsive(viewportWidth, xs = DoweDesign.{token}) ?: LocalContentColor.current))"
        )));
        assert!(compose.content.contains(&format!(
            "Text(\"{label}\", modifier = Modifier, color = Color.Unspecified"
        )));
    }
    assert!(
        compose
            .content
            .contains("CompositionLocalProvider(LocalContentColor provides contentColor)")
    );
    assert!(compose.content.contains(
        "Text(\"Collapsible inherited\", modifier = Modifier, color = Color.Unspecified"
    ));
    assert!(compose.content.contains(
        "DoweTypeWriter(texts = listOf(\"TypeWriter inherited\"), typeSpeed = 10, deleteSpeed = 5, afterTyped = 20, afterDeleted = 10, repeat = false, contentColor = LocalContentColor.current"
    ));

    let dev = dev_java_source(&output);
    let box_inherited = dev
        .content
        .find("doweText(\"Box inherited\"")
        .expect("Box inherited text");
    assert!(dev.content[box_inherited..box_inherited + 320].contains("DOWE_PRIMARY_TEXT"));
    let box_override = dev
        .content
        .find("doweText(\"Box override\"")
        .expect("Box override text");
    assert!(dev.content[box_override..box_override + 320].contains("DOWE_DANGER"));
    assert!(
        dev.content
            .contains("doweText(\"Card inherited\", DOWE_MUTED_TEXT")
    );
    assert!(
        dev.content
            .contains("doweText(\"Card title inherited\", DOWE_MUTED_TITLE")
    );
    let card_override = dev
        .content
        .find("doweText(\"Card override\"")
        .expect("Card override text");
    assert!(dev.content[card_override..card_override + 320].contains("DOWE_WARNING"));
    for (label, token) in [
        ("Section inherited", "DOWE_SECONDARY_TEXT"),
        ("Flex inherited", "DOWE_ACCENT_TEXT"),
        ("Grid inherited", "DOWE_MUTED_TEXT"),
        ("Brand inherited", "DOWE_SURFACE_TEXT"),
        ("Banner inherited", "DOWE_INFO_TEXT"),
        ("Marquee inherited", "DOWE_WARNING_TEXT"),
        ("Scaffold inherited", "DOWE_DANGER_TEXT"),
    ] {
        let start = dev
            .content
            .find(&format!("doweText(\"{label}\""))
            .unwrap_or_else(|| panic!("{label} text"));
        assert!(
            dev.content[start..start + 320].contains(token),
            "{label} should inherit {token}"
        );
    }
}

#[test]
fn generates_fixed_fab_as_native_overlay_with_dowe_icons() {
    let mut fab_route = route();
    fab_route.page_tree = fixed_fab_page();
    let output = generate_android(
        &[fab_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);

    assert!(generated.contains("var doweFixedFabOpen0 by remember"));
    assert!(generated.contains("Modifier.fillMaxSize().padding(horizontal"));
    assert!(generated.contains("if (doweFixedFabOpen0)"));
    assert!(generated.contains(
        "verticalArrangement = Arrangement.spacedBy(12.dp, alignment = Alignment.Bottom)"
    ));
    assert!(generated.contains(".rotate(if (doweFixedFabOpen0) 45f else 0f)"));
    assert!(
        generated.contains(".doweGesture(DoweGesturePreset.Press, DoweTransitionPreset.Smooth)")
    );
    assert!(generated.contains("DoweSvg(viewBox ="));
    assert!(generated.contains("setTag(\"dowe-fixed-fab\")"));
    assert!(generated.contains("Gravity.BOTTOM | Gravity.END"));
    assert!(generated.contains(".setRotation(open ? 45f : 0f)"));
    assert!(generated.contains("doweWrapContentWidth("));
    assert!(!generated.contains("setMinimumWidth(doweDp(180))"));
    assert!(!generated.contains("Text(\"+\")"));

    let mut top_fab_route = route();
    top_fab_route.page_tree = fixed_fab_page_at(OverlayCornerPosition::TopRight);
    let top_generated = all_android_source(&generate_android(
        &[top_fab_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));
    let mut top_lines = top_generated.lines();
    let trigger_line = top_lines
        .find(|line| line.contains(".setContentDescription(\"Open actions\")"))
        .expect("top Fab trigger");
    let trigger = trigger_line
        .trim()
        .split('.')
        .next()
        .expect("top Fab trigger variable");
    assert!(top_generated.contains(&format!("doweGesture({trigger}, \"press\", \"smooth\");")));
    let first_child_addition = top_lines
        .find(|line| line.contains("doweAdd("))
        .expect("top Fab first child addition");
    assert!(first_child_addition.contains(&format!(", {trigger},")));
}

#[test]
fn generates_relative_absolute_and_fixed_boxes_as_native_overlays() {
    let mut positioned_route = route();
    positioned_route.page_tree = positioned_box_page();
    let output = generate_android(
        &[positioned_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);

    assert!(generated.contains("Alignment.TopEnd"));
    assert!(generated.contains(".padding(top = doweResponsive(viewportWidth, xs = 16.dp) ?: 0.dp, end = doweResponsive(viewportWidth, xs = 24.dp) ?: 0.dp)"));
    assert!(generated.contains("Alignment.BottomEnd"));
    assert!(generated.contains(".padding(end = doweResponsive(viewportWidth, xs = 16.dp) ?: 0.dp, bottom = doweResponsive(viewportWidth, xs = 16.dp) ?: 0.dp)"));
    assert!(generated.contains("FrameLayout"));
    assert!(generated.contains("Gravity.TOP | Gravity.END"));
    assert!(generated.contains("dowe-fixed-box"));
    let dev = dev_java_source(&output).content;
    let proof = dev
        .find("doweText(\"Proof\"")
        .expect("positioned box content");
    let visibility = dev[..proof]
        .rfind("if (doweShow(doweResponsiveBool(viewportWidth, false, null, null, true, null))) {")
        .expect("positioned box visibility");
    assert!(proof - visibility < 2_000);
    let relative_width = dev[..proof]
        .rfind("setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));")
        .expect("relative box width");
    assert!(proof - relative_width < 3_000);
}

#[test]
fn generates_relative_box_cover_from_project_assets_for_compose_and_dev_launcher() {
    let mut cover_route = route();
    cover_route.page_tree = relative_box_cover_page();
    let output = generate_android(
        &[cover_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);
    assert!(generated.contains("DoweCoverBox("));
    assert!(generated.contains("/assets/img/guarias-login.webp"));
    let cover_runtime = generated
        .split("private fun DoweCoverBox")
        .nth(1)
        .expect("cover runtime")
        .split("private fun DoweGrid")
        .next()
        .expect("cover runtime boundary");
    assert!(cover_runtime.contains("doweLoadImageBitmap(context, source)"));
    assert!(generated.contains("connection.responseCode !in 200..299"));
    assert!(generated.contains("BitmapFactory.decodeByteArray(bytes, 0, bytes.size)"));
    assert!(cover_runtime.contains("modifier = Modifier.fillMaxSize()"));
    assert!(!cover_runtime.contains("setImageURI(Uri.parse(source))"));

    let dev = dev_java_source(&output).content;
    assert!(dev.contains("/assets/img/guarias-login.webp"));
    assert!(dev.contains("CoverImage.setScaleType(ImageView.ScaleType.CENTER_CROP)"));
    assert!(dev.contains("connection.getResponseCode() < 200"));
    assert!(dev.contains("BitmapFactory.decodeByteArray(imageBytes, 0, imageBytes.length)"));
    assert!(dev.contains("doweLoadImageBitmap(view"));
}

