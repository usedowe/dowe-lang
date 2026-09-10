#[test]
fn static_html_preserves_stable_development_asset_paths() {
    let document = r#"<link href="/design.css"><script src="/router.js"></script>"#;
    let static_document = super::static_html_document(document, "../");

    assert!(static_document.contains(r#"href="../design.css""#));
    assert!(static_document.contains(r#"src="../router.js""#));
    assert!(!static_document.contains("design.css.html"));
}

#[test]
fn emits_container_refactor_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Grid {
        props: GridProps {
            columns: Some(ResponsiveValue::scalar(GridTracks::Count(3))),
            rows: Some(ResponsiveValue::scalar(GridTracks::Count(2))),
            justify: Some(ResponsiveValue::scalar(GridAlignment::Center)),
            gap: Some(ResponsiveValue::scalar(GapValue::Pair(
                GapSize::Px(10),
                GapSize::Px(20),
            ))),
            ..Default::default()
        },
        children: vec![
            ViewNode::Box {
                props: StyleProps {
                    cover: Some(ResponsiveValue::ordered(vec![
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Xs,
                            value: CoverSource("/mobile.jpg".to_string()),
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Md,
                            value: CoverSource("/desktop.jpg".to_string()),
                        },
                    ])),
                    overlay: Some(ResponsiveValue::scalar(OverlayPaint::BlackOpacity(
                        "0.6".to_string(),
                    ))),
                    extras: Some(Box::new(dowe_components::StyleExtras {
                        grid_item: dowe_components::GridItemProps {
                            col_span: Some(ResponsiveValue::scalar(GridSpan(2))),
                            ..Default::default()
                        },
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                children: vec![text("Hero")],
            },
            ViewNode::Card {
                props: VariantProps {
                    variant: Some(ComponentVariant::Solid),
                    color: Some(dowe_components::ColorFamily::Surface),
                    ..Default::default()
                },
                children: vec![text("Card")],
            },
        ],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("grid-cols-3"));
    assert!(page.content.contains("grid-justify-center"));
    assert!(page.css_content.contains("justify-content:center;"));
    assert!(page.content.contains("col-span-2"));
    assert!(page.content.contains("has-cover"));
    assert!(page.content.contains("has-overlay"));
    assert!(
        page.css_content
            .contains("grid-template-columns:repeat(3,minmax(0,1fr));")
    );
    assert!(
        page.css_content
            .contains("grid-template-rows:repeat(2,minmax(0,1fr));")
    );
    assert!(page.css_content.contains("row-gap:10px;column-gap:20px;"));
    assert!(
        page.css_content
            .contains("background-image:url(\"/mobile.jpg\")")
    );
    assert!(page.css_content.contains("@media (min-width:768px)"));
    assert!(page.css_content.contains("rgba(0,0,0,0.6)"));
    assert!(page.css_content.contains(".card.is-solid.is-surface"));
}

#[test]
fn emits_responsive_css_in_ascending_breakpoint_blocks() {
    let page_tree = ViewNode::Grid {
        props: GridProps {
            columns: Some(ResponsiveValue::ordered(vec![
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xs,
                    value: GridTracks::Count(1),
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Sm,
                    value: GridTracks::Count(2),
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Md,
                    value: GridTracks::Count(3),
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Lg,
                    value: GridTracks::Count(4),
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xl,
                    value: GridTracks::Count(5),
                },
            ])),
            rows: Some(ResponsiveValue::ordered(vec![
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xs,
                    value: GridTracks::Count(1),
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Sm,
                    value: GridTracks::Count(2),
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Md,
                    value: GridTracks::Count(3),
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Lg,
                    value: GridTracks::Count(4),
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xl,
                    value: GridTracks::Count(5),
                },
            ])),
            ..Default::default()
        },
        children: vec![text("Responsive grid")],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/views/pages/responsive.dowe"),
        "page ResponsivePage",
        &page_tree,
    );
    let css = &page.css_content;
    let base = css
        .find(".grid-cols-1{grid-template-columns:repeat(1,minmax(0,1fr));}")
        .expect("base responsive rule");
    let sm = css
        .find("@media (min-width:640px)")
        .expect("sm responsive block");
    let md = css
        .find("@media (min-width:768px)")
        .expect("md responsive block");
    let lg = css
        .find("@media (min-width:1024px)")
        .expect("lg responsive block");
    let xl = css
        .find("@media (min-width:1280px)")
        .expect("xl responsive block");

    assert!(base < sm && sm < md && md < lg && lg < xl);
    for min_width in [640, 768, 1024, 1280] {
        assert_eq!(
            css.matches(&format!("@media (min-width:{min_width}px)"))
                .count(),
            1
        );
    }
}

#[test]
fn emits_portable_box_positioning_css() {
    let page_tree = ViewNode::Box {
        props: StyleProps {
            extras: Some(Box::new(dowe_components::StyleExtras {
                position: dowe_components::PositionProps {
                    mode: BoxPosition::Relative,
                    ..Default::default()
                },
                ..Default::default()
            })),
            ..Default::default()
        },
        children: vec![
            ViewNode::Box {
                props: StyleProps {
                    extras: Some(Box::new(dowe_components::StyleExtras {
                        position: dowe_components::PositionProps {
                            mode: BoxPosition::Absolute,
                            top: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                            right: Some(ResponsiveValue::ordered(vec![
                                ResponsiveEntry {
                                    breakpoint: Breakpoint::Xs,
                                    value: ScaleValue::from_half_steps(8),
                                },
                                ResponsiveEntry {
                                    breakpoint: Breakpoint::Md,
                                    value: ScaleValue::from_half_steps(12),
                                },
                            ])),
                            ..Default::default()
                        },
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                children: vec![text("Proof")],
            },
            ViewNode::Box {
                props: StyleProps {
                    extras: Some(Box::new(dowe_components::StyleExtras {
                        position: dowe_components::PositionProps {
                            mode: BoxPosition::Fixed,
                            ..Default::default()
                        },
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                children: vec![text("Persistent")],
            },
        ],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/positioning.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("position-relative"));
    assert!(page.content.contains("position-absolute"));
    assert!(page.content.contains("position-fixed top-0 left-0"));
    assert!(page.content.contains("top-4"));
    assert!(page.content.contains("right-4 md:right-6"));
    assert!(
        page.css_content
            .contains(".position-relative{position:relative;}")
    );
    assert!(
        page.css_content
            .contains(".position-absolute{position:absolute;}")
    );
    assert!(
        page.css_content
            .contains(".position-fixed{position:fixed;}")
    );
    assert!(page.css_content.contains(".top-4{top:1rem;}"));
    assert!(page.css_content.contains(".top-0{top:0rem;}"));
    assert!(page.css_content.contains(".left-0{left:0rem;}"));
    assert!(page.css_content.contains(".right-4{right:1rem;}"));
    assert!(
        page.css_content
            .contains("@media (min-width:768px){.md\\:right-6{right:1.5rem;}}")
    );
}

#[test]
fn emits_reset_and_font_css() {
    let css = super::design_css();

    assert!(css.contains("body{--dowe-content-text:var(--dowe-backgroundText);--dowe-content-title:var(--dowe-backgroundTitle);margin:0;"));
    assert!(css.contains(
        ".dowe-text{color:var(--dowe-content-text,var(--dowe-backgroundText));white-space:pre-line;}"
    ));
    assert!(
        css.contains(
            ".dowe-title{color:var(--dowe-content-title,var(--dowe-backgroundTitle));white-space:pre-line;}"
        )
    );
    assert!(css.contains("p,h1,h2,h3,h4,h5,h6{margin:0;"));
    assert!(css.contains("a{color:inherit;text-decoration:inherit;}"));
    assert!(css.contains("button,input,textarea,select{font:inherit;color:inherit;margin:0;}"));
    assert!(css.contains(
        ".input::-webkit-outer-spin-button,.input::-webkit-inner-spin-button{-webkit-appearance:none;margin:0;}"
    ));
    assert!(css.contains(".input[type='number']{appearance:textfield;-moz-appearance:textfield;}"));
    assert!(css.contains(
        ".input:-webkit-autofill,.input:-webkit-autofill:hover,.input:-webkit-autofill:focus,.input:-webkit-autofill:active{-webkit-box-shadow:0 0 0 30px transparent inset!important;-webkit-text-fill-color:inherit!important;background-color:transparent!important;background-image:none!important;color:inherit!important;transition:background-color 5000000s ease-in-out 0s!important;}"
    ));
    assert!(css.contains(
        ".input:-moz-autofill{background-color:transparent!important;color:inherit!important;}"
    ));
    assert!(css.contains("--dowe-font-inter"));
    assert!(css.contains("@font-face{font-family:\"Dowe Inter\""));
    assert!(css.contains("src:url(\"/fonts/inter/inter-regular.ttf\") format(\"truetype\")"));
    assert!(css.contains("font-display:optional"));
}

#[test]
fn rewrites_static_route_hrefs_for_desktop_fallback() {
    let document = r##"<a class="button" href="/signup#join" data-dowe-nav="push" data-dowe-href="/signup#join">Signup</a><a class="button" href="/" data-dowe-nav="push" data-dowe-href="/">Home</a><link rel="stylesheet" href="/design-test.css">"##;
    let index = super::static_html_document(document, "");
    let page = super::static_html_document(document, "../");

    assert!(index.contains(
        r##"href="pages/signup.html#join" data-dowe-nav="push" data-dowe-href="/signup#join""##
    ));
    assert!(index.contains(r##"href="index.html" data-dowe-nav="push" data-dowe-href="/""##));
    assert!(index.contains(r#"href="design-test.css""#));
    assert!(page.contains(
        r##"href="signup.html#join" data-dowe-nav="push" data-dowe-href="/signup#join""##
    ));
    assert!(page.contains(r##"href="../index.html" data-dowe-nav="push" data-dowe-href="/""##));
    assert!(page.contains(r#"href="../design-test.css""#));
}
