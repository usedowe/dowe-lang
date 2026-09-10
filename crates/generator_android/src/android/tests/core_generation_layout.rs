#[test]
fn generates_compose_text_alignment() {
    let mut aligned = route();
    aligned.page_tree = ViewNode::Text {
        props: TextProps {
            align: Some(ResponsiveValue::scalar(TextAlign::Center)),
            ..Default::default()
        },
        value: "Aligned".to_string(),
    };
    let output = generate_android(
        &[aligned],
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
        "Text(\"Aligned\", modifier = Modifier.fillMaxWidth(), color = Color.Unspecified,"
    ));
    assert!(views.content.contains("textAlign = TextAlign.Center"));
    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("setGravity(doweResponsiveInt(viewportWidth")
    );
    assert!(dev.content.contains("Gravity.CENTER_HORIZONTAL"));
}

#[test]
fn generates_text_and_title_background_for_compose_and_dev_launcher() {
    let mut styled = route();
    styled.layout_tree = ViewNode::Children;
    styled.page_tree = ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::Text {
                props: TextProps {
                    size: Some(ResponsiveValue::scalar(TextSize::Sm)),
                    style: StyleProps {
                        text: Some(ResponsiveValue::scalar(ColorToken::AccentText)),
                        bg: Some(ResponsiveValue::scalar(ColorToken::Accent)),
                        rounded: Some(ResponsiveValue::scalar(RoundedSize::Full)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                value: "Text badge".to_string(),
            },
            ViewNode::Title {
                props: TextProps {
                    size: Some(ResponsiveValue::scalar(TextSize::Sm)),
                    style: StyleProps {
                        text: Some(ResponsiveValue::scalar(ColorToken::AccentText)),
                        bg: Some(ResponsiveValue::scalar(ColorToken::Accent)),
                        rounded: Some(ResponsiveValue::scalar(RoundedSize::Full)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                value: "Title badge".to_string(),
            },
        ],
    };
    let output = generate_android(
        &[styled],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    let text_modifier = ".doweRounded(doweResponsive(viewportWidth, xs = 999.dp)).doweBackground(doweResponsive(viewportWidth, xs = DoweDesign.accent))";
    assert_eq!(views.content.matches(text_modifier).count(), 2);
    assert!(views.content.contains("DoweDesign.accentText"));

    let dev = dev_java_source(&output);
    assert_eq!(
        dev.content
            .matches("Background = doweResponsiveInt(viewportWidth, DOWE_ACCENT")
            .count(),
        2
    );
}

#[test]
fn resets_android_dev_route_after_process_relaunch() {
    let output = generate_android(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let hot_host = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevHostActivity.java"))
        .expect("hot host");
    let dev = dev_java_source(&output);

    assert!(!hot_host.content.contains("HMR_ROUTE"));
    assert!(!hot_host.content.contains("persistCurrentPath()"));
    assert!(hot_host.content.contains(
        "boolean initialMount = activeModule == null;\n            String path = initialMount ? null : activeModulePath();"
    ));
    assert!(
        hot_host
            .content
            .contains("mount.invoke(module, path, initialMount ? getIntent() : null)")
    );
    assert!(hot_host.content.contains(
        "private String activeModulePath() {\n        if (activeModule != null && activePath != null)"
    ));
    assert!(
        hot_host
            .content
            .contains("return null;\n    }\n\n    private void poll()")
    );
    assert!(dev.content.contains(
        "if (doweCanRoute(preferredPath)) {\n            currentPath = preferredPath;\n        }\n        doweApplyIntentRoute();"
    ));
    assert!(dev.content.contains(
        "if (data == null) {\n            return;\n        }\n        String path = data.getPath();"
    ));
}

#[test]
fn preserves_fixed_height_for_empty_grid_items_on_android() {
    let mut fixed_height_route = route();
    fixed_height_route.page_tree = ViewNode::Grid {
        props: GridProps {
            columns: Some(ResponsiveValue::scalar(GridTracks::Count(3))),
            gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                ScaleValue::from_half_steps(4),
            )))),
            style: StyleProps {
                shadow: Some(ResponsiveValue::scalar(ShadowSize::Lg)),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![ViewNode::Box {
            props: StyleProps {
                bg: Some(ResponsiveValue::scalar(ColorToken::Primary)),
                border: Some(ResponsiveValue::scalar(BorderWidth(1))),
                rounded: Some(ResponsiveValue::scalar(RoundedSize::Sm)),
                sizing: dowe_components::SizingProps {
                    h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                        ScaleValue::from_half_steps(16),
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: Vec::new(),
        }],
    };

    let output = generate_android(
        &[fixed_height_route],
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

    assert!(compose.content.contains(
        ".doweBackground(doweResponsive(viewportWidth, xs = DoweDesign.primary)).doweHeight(doweResponsive(viewportWidth, xs = DoweSize.Fixed(32.dp)))"
    ));
    assert!(
        compose
            .content
            .contains("shape = RoundedCornerShape(0.dp), color = Color.Black")
    );
    assert!(
        dev.content
            .contains("Height = doweResponsiveInt(viewportWidth")
    );
    assert!(dev.content.contains("doweShadow(view0"));
    assert!(dev.content.contains(", 0f, null);"));
    assert!(dev.content.contains("int childHeight = childParams == null ? ViewGroup.LayoutParams.WRAP_CONTENT : childParams.height;"));
    assert!(
        dev.content
            .contains("int childHeightSpec = getChildMeasureSpec(")
    );
    assert!(
        dev.content
            .contains("child.measure(childWidthSpec, childHeightSpec);")
    );
}

#[test]
fn preserves_fixed_box_width_inside_android_grids() {
    let mut fixed_width_route = route();
    fixed_width_route.page_tree = ViewNode::Grid {
        props: GridProps {
            columns: Some(ResponsiveValue::scalar(GridTracks::Count(1))),
            ..Default::default()
        },
        children: vec![
            ViewNode::Box {
                props: StyleProps {
                    sizing: dowe_components::SizingProps {
                        w: Some(ResponsiveValue::scalar(SizeValue::Scale(
                            ScaleValue::from_half_steps(24),
                        ))),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("H")],
            },
            ViewNode::Box {
                props: StyleProps::default(),
                children: vec![text("Full width")],
            },
        ],
    };

    let output = generate_android(
        &[fixed_width_route],
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

    assert!(
        compose
            .content
            .contains(".doweWidth(doweResponsive(viewportWidth, xs = DoweSize.Fixed(48.dp)))")
    );
    assert!(
        dev.content
            .contains("Width = doweResponsiveInt(viewportWidth, 48,")
    );
    assert!(dev.content.contains(
        "int childWidth = childParams == null ? ViewGroup.LayoutParams.WRAP_CONTENT : childParams.width;"
    ));
    assert!(dev.content.contains(
        "MeasureSpec.makeMeasureSpec(Math.min(childWidth, cellWidth), MeasureSpec.EXACTLY)"
    ));
    assert!(dev.content.contains(
        "child.layout(childLeft, rowTop, childLeft + child.getMeasuredWidth(), rowTop + child.getMeasuredHeight());"
    ));
}

#[test]
fn generates_non_throwing_dev_json_stringify_runtime() {
    let output = generate_android(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);

    assert!(dev.content.contains(
        "if (\"json.stringify\".equals(name)) return doweJsonString(args.get(\"value\"), Boolean.TRUE.equals(args.get(\"pretty\")));"
    ));
    assert!(
        dev.content
            .contains("private String doweJsonString(Object value, boolean pretty) {")
    );
    assert!(!dev.content.contains(
        "if (\"json.stringify\".equals(name)) return doweJson(args.get(\"value\")).toString();"
    ));
}

#[test]
fn generates_android_box_border_for_compose_and_dev_launcher() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Box {
        props: StyleProps {
            border: Some(ResponsiveValue::scalar(BorderWidth(2))),
            rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
            ..Default::default()
        },
        children: vec![text("Bordered")],
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
        ".border(doweResponsive(viewportWidth, xs = 2.dp) ?: 0.dp, DoweDesign.backgroundText, RoundedCornerShape(doweResponsive(viewportWidth, xs = 12.dp) ?: DoweDesign.radius))"
    ));

    let dev = dev_java_source(&output);

    assert!(dev.content.contains(
        "private GradientDrawable doweStyledBackground(int color, Integer strokeColor, Integer strokeWidth, float radius)"
    ));
    assert!(dev.content.contains(
        "view0.setBackground(doweStyledBackground(Color.TRANSPARENT, DOWE_BACKGROUND_TEXT, doweResponsiveInt(viewportWidth, 2, null, null, null, null), doweFloat(doweResponsiveFloat(viewportWidth, 12f, null, null, null, null), DOWE_RADIUS)))"
    ));
    assert!(
        dev.content
            .contains("background.setStroke(doweDp(strokeWidth), strokeColor)")
    );
}

#[test]
fn preserves_explicit_rounded_values_across_android_renderers() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Box {
        props: StyleProps {
            bg: Some(ResponsiveValue::scalar(ColorToken::Primary)),
            rounded: Some(ResponsiveValue::scalar(RoundedSize::Full)),
            ..Default::default()
        },
        children: vec![ViewNode::Button {
            props: VariantProps {
                style: StyleProps {
                    rounded: Some(ResponsiveValue::scalar(RoundedSize::Full)),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Continue")],
        }],
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
    let dev = dev_java_source(&output);

    assert!(views.content.contains(".doweRounded(doweResponsive(viewportWidth, xs = 999.dp)).doweBackground(doweResponsive(viewportWidth, xs = DoweDesign.primary))"));
    assert!(views.content.contains(
        "RoundedCornerShape(doweResponsive(viewportWidth, xs = 999.dp) ?: DoweDesign.radius)"
    ));
    assert!(dev.content.contains(
        "doweRound(view0, doweResponsiveFloat(viewportWidth, 999f, null, null, null, null))"
    ));
    assert!(dev.content.contains(
        "doweRound(view1, doweResponsiveFloat(viewportWidth, 999f, null, null, null, null))"
    ));
    assert!(dev.content.contains("view.setClipToOutline(true)"));
}

#[test]
fn generates_android_card_cover_and_foreground_layers_for_dev_launcher() {
    let mut cover_route = route();
    cover_route.layout_tree = ViewNode::Children;
    cover_route.page_tree = ViewNode::Card {
        props: VariantProps {
            style: StyleProps {
                cover: Some(ResponsiveValue::scalar(CoverSource("/assets/card.webp".to_string()))),
                overlay: Some(ResponsiveValue::scalar(OverlayPaint::BlackOpacity("0.62".to_string()))),
                text: Some(ResponsiveValue::scalar(ColorToken::White)),
                spacing: SpacingProps {
                    p: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                    ..Default::default()
                },
                sizing: SizingProps {
                    min_h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                        ScaleValue::from_half_steps(144),
                    ))),
                    ..Default::default()
                },
                rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![ViewNode::Flex {
            props: LayoutProps {
                direction: ResponsiveValue::scalar(dowe_components::FlexDirection::Column),
                justify: Some(ResponsiveValue::scalar(Justify::End)),
                gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                    ScaleValue::from_half_steps(4),
                )))),
                style: StyleProps {
                    sizing: SizingProps {
                        min_h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                            ScaleValue::from_half_steps(120),
                        ))),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Readable")],
        }],
    };
    let output = generate_android(
        &[cover_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);
    assert!(generated.contains("FrameLayout view0 = new FrameLayout(this)"));
    assert!(generated.contains("runtime.doweImage"));
    assert!(generated.contains("CoverOverlay.setBackgroundColor"));
    assert!(generated.contains("Content = doweContainer(false)"));
    assert!(generated.contains("ContentLeft = 0"));
    assert!(generated.contains("runtime.DOWE_JUSTIFY_END"));
    assert!(generated.contains("MinHeight = runtime.doweResponsiveInt(viewportWidth, 240"));
    assert!(generated.contains("Math.max(verticalPadding + childrenHeight + gapTotal, getSuggestedMinimumHeight())"));
    assert!(generated.contains("int intrinsicHeightSpec = MeasureSpec.makeMeasureSpec(0, MeasureSpec.UNSPECIFIED)"));
    assert!(generated.contains("val intrinsicConstraints = constraints.copy(minHeight = 0, maxHeight = Constraints.Infinity)"));
    assert!(generated.contains("runtime.doweText(\"Readable\", runtime.doweResponsiveInt(viewportWidth, Color.WHITE"));
}

