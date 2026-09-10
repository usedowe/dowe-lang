#[test]
fn generates_responsive_section_centering_for_compose_and_dev_android() {
    let mut route = section_route();
    let ViewNode::Box { children, .. } = &mut route.page_tree else {
        panic!("section route root");
    };
    let ViewNode::Section { props, .. } = &mut children[0] else {
        panic!("section route child");
    };
    props.center_x = Some(ResponsiveValue::ordered(vec![
        ResponsiveEntry {
            breakpoint: Breakpoint::Xs,
            value: false,
        },
        ResponsiveEntry {
            breakpoint: Breakpoint::Md,
            value: true,
        },
    ]));

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
        "horizontalAlignment = (doweResponsive(viewportWidth, xs = false, md = true) ?: false) ? Alignment.CenterHorizontally : Alignment.Start"
    ));
    assert!(
        views
            .content
            .contains("Column(modifier = Modifier.fillMaxWidth()")
    );

    let dev = dev_java_source(&output);
    assert!(dev.content.contains(
        "setGravity((Boolean.TRUE.equals(false) ? Gravity.CENTER_VERTICAL : Gravity.TOP) | (Boolean.TRUE.equals(doweResponsiveBool(viewportWidth, false, null, true, null, null)) ? Gravity.CENTER_HORIZONTAL : Gravity.START));"
    ));
}

#[test]
fn fills_height_bounded_section_body_for_compose_and_dev_android() {
    let mut route = section_route();
    let ViewNode::Box { children, .. } = &mut route.page_tree else {
        panic!("section route root");
    };
    let ViewNode::Section {
        props,
        children: section_children,
    } = &mut children[0]
    else {
        panic!("section route child");
    };
    props.sizing.min_h = Some(ResponsiveValue::scalar(SizeValue::ViewportMinus(
        ScaleValue::from_half_steps(0),
    )));
    *section_children = vec![ViewNode::Grid {
        props: GridProps {
            columns: Some(ResponsiveValue::scalar(GridTracks::Count(1))),
            style: StyleProps {
                sizing: SizingProps {
                    min_h: Some(ResponsiveValue::scalar(SizeValue::Full)),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Grid")],
    }];

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
        "Column(modifier = Modifier.widthIn(max = 1536.dp).fillMaxWidth().fillMaxHeight().dowePadding"
    ));
    assert!(
        views
            .content
            .contains("doweMinHeight(doweResponsive(viewportWidth, xs = DoweSize.Full))")
    );

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("MinHeight = doweResponsiveInt"));
    assert!(
        dev.content
            .contains("== ViewGroup.LayoutParams.MATCH_PARENT")
    );
    assert!(
        dev.content
            .contains("Params.height = ViewGroup.LayoutParams.MATCH_PARENT")
    );
}

#[test]
fn generates_responsive_auto_and_full_height_constraints_for_containers() {
    let sizing = SizingProps {
        h: Some(ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: SizeValue::Auto,
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: SizeValue::Full,
            },
        ])),
        min_h: Some(ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: SizeValue::Auto,
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: SizeValue::Full,
            },
        ])),
        max_h: Some(ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: SizeValue::Auto,
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: SizeValue::Full,
            },
        ])),
        ..Default::default()
    };
    let mut height_route = route();
    height_route.layout_tree = ViewNode::Children;
    height_route.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![
            ViewNode::Box {
                props: StyleProps {
                    sizing: sizing.clone(),
                    ..Default::default()
                },
                children: Vec::new(),
            },
            ViewNode::Section {
                props: StyleProps {
                    sizing: sizing.clone(),
                    ..Default::default()
                },
                children: Vec::new(),
            },
            ViewNode::Grid {
                props: GridProps {
                    style: StyleProps {
                        sizing,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: Vec::new(),
            },
        ],
    };
    let output = generate_android(
        &[height_route],
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

    assert!(
        views
            .content
            .contains("doweResponsive(viewportWidth, xs = DoweSize.Auto, md = DoweSize.Full)")
    );
    assert!(
        views
            .content
            .contains("DoweSize.Full -> doweMaxParentHeight()")
    );
    assert!(dev.content.contains(
        "if (view0Width != null) { view0SizeParams.width = doweDimension(view0Width); }"
    ));
    assert!(dev.content.contains(
        "view0SizeParams = new ViewGroup.LayoutParams(\n                view0Width != null ? doweDimension(view0Width) : ViewGroup.LayoutParams.WRAP_CONTENT,"
    ));
    assert!(
        dev.content
            .contains("ViewGroup.LayoutParams view0MinHeightParams")
    );
    assert!(
        dev.content
            .contains("ViewGroup.LayoutParams view1SizeParams")
    );
    assert!(
        dev.content
            .contains("LinearLayout.LayoutParams view1Params")
    );
    assert!(dev.content.contains(
        "if (value == ViewGroup.LayoutParams.MATCH_PARENT || value == ViewGroup.LayoutParams.WRAP_CONTENT)"
    ));
    assert!(
        dev.content
            .contains("value == ViewGroup.LayoutParams.WRAP_CONTENT")
    );
}

#[test]
fn generates_responsive_cover_box_height_for_android_dev_and_compose() {
    let mut cover_route = route();
    cover_route.layout_tree = ViewNode::Children;
    cover_route.page_tree = ViewNode::Box {
        props: StyleProps {
            cover: Some(ResponsiveValue::scalar(CoverSource(
                "/assets/img/guarias-login.webp".to_string(),
            ))),
            extras: Some(Box::new(StyleExtras {
                position: dowe_components::PositionProps {
                    mode: BoxPosition::Relative,
                    ..Default::default()
                },
                ..Default::default()
            })),
            sizing: SizingProps {
                h: Some(ResponsiveValue::ordered(vec![
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Xs,
                        value: SizeValue::Scale(ScaleValue::from_half_steps(112)),
                    },
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: SizeValue::Full,
                    },
                ])),
                min_h: Some(ResponsiveValue::ordered(vec![
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Xs,
                        value: SizeValue::Auto,
                    },
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: SizeValue::Full,
                    },
                ])),
                max_h: Some(ResponsiveValue::ordered(vec![
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Xs,
                        value: SizeValue::Auto,
                    },
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: SizeValue::Full,
                    },
                ])),
                w: Some(ResponsiveValue::scalar(SizeValue::Full)),
                ..Default::default()
            },
            ..Default::default()
        },
        children: Vec::new(),
    };

    let output = generate_android(
        &[cover_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(
        views
            .content
            .contains("DoweSize.Fixed(224.dp), md = DoweSize.Full")
    );
    assert!(views.content.contains("DoweCoverBox("));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains(
        "Integer view0Height = doweResponsiveInt(viewportWidth, 224, null, ViewGroup.LayoutParams.MATCH_PARENT, null, null);"
    ));
    assert!(dev.content.contains(
        "view0SizeParams = new ViewGroup.LayoutParams(\n                view0Width != null ? doweDimension(view0Width) : ViewGroup.LayoutParams.WRAP_CONTENT,\n                view0Height != null ? doweDimension(view0Height) : ViewGroup.LayoutParams.WRAP_CONTENT"
    ));
    assert!(dev.content.contains(
        "view0MinHeightParams = new ViewGroup.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.MATCH_PARENT);"
    ));
    assert!(dev.content.contains("doweConstrain(view0,"));
    assert!(
        dev.content
            .contains("CoverImage.setScaleType(ImageView.ScaleType.CENTER_CROP)")
    );
}

#[test]
fn generates_responsive_section_gap_for_compose_and_dev_android() {
    let mut route = section_route();
    let ViewNode::Box { children, .. } = &mut route.page_tree else {
        panic!("section route root");
    };
    let ViewNode::Section { props, .. } = &mut children[0] else {
        panic!("section route child");
    };
    props.gap = Some(ResponsiveValue::ordered(vec![
        ResponsiveEntry {
            breakpoint: Breakpoint::Xs,
            value: GapValue::Single(GapSize::Scale(ScaleValue(4))),
        },
        ResponsiveEntry {
            breakpoint: Breakpoint::Md,
            value: GapValue::Single(GapSize::Scale(ScaleValue(8))),
        },
    ]));

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
        "verticalArrangement = Arrangement.spacedBy(doweResponsive(viewportWidth, xs = 8.dp, md = 16.dp) ?: 0.dp)"
    ));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains(
        "doweAdd(view2, view3, doweResponsiveInt(viewportWidth, 8, null, 16, null, null), false);"
    ));
}

