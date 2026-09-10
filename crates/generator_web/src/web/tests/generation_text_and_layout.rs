#[test]
fn renders_box_and_text_as_div_and_paragraph() {
    assert_eq!(
        render_page_body(&layout_tree(), &page_tree()),
        r#"<div class="box"><p class="dowe-text text-md">Layout</p><div class="box"><p class="dowe-text text-md">Login</p></div></div>"#
    );
}

#[test]
fn renders_title_as_selected_heading_tag_on_web() {
    let tree = ViewNode::Title {
        props: TextProps {
            as_tag: Some("h1".to_string()),
            ..Default::default()
        },
        value: "Main heading".to_string(),
    };
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains("<h1 class=\"dowe-title title-md\">Main heading</h1>"));
}

#[test]
fn preserves_multiline_text_in_one_paragraph() {
    let tree = ViewNode::Title {
        props: TextProps::default(),
        value: "Full-stack development,\nfrom one codebase".to_string(),
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/views/pages/home.dowe"),
        "page home",
        &tree,
    );
    let html = render_page_body(&ViewNode::Children, &tree);
    let css =
        super::design_css_for_trees([&tree], &FontConfig::default(), &DesignConfig::default());

    assert!(html.contains(
        "<h2 class=\"dowe-title title-md\">Full-stack development,\nfrom one codebase</h2>"
    ));
    assert!(css.contains("white-space:pre-line;"));
    assert!(page.content.contains("Full-stack development"));
}

#[test]
fn renders_section_markup_and_background_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Section {
        props: StyleProps {
            boxed: true,
            text: Some(ResponsiveValue::scalar(ColorToken::BackgroundText)),
            background: Some(ResponsiveValue::ordered(vec![
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xs,
                    value: SectionBackground::Slate,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Md,
                    value: SectionBackground::Aurora,
                },
            ])),
            ..Default::default()
        },
        children: vec![text("Hero")],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    let html = render_page_body(&ViewNode::Children, &page_tree);

    assert!(html.contains("<section"));
    assert!(html.contains(
        "<div class=\"section-body is-boxed px-4 md:px-6 py-10 md:py-16\"><p class=\"dowe-text text-md\">Hero</p></div>"
    ));
    assert!(!html.contains("<section class=\"section is-boxed"));
    assert!(html.contains(
        "section color-backgroundText has-background background-slate md:background-aurora"
    ));
    assert!(page.css_content.contains(
        "background-image:linear-gradient(135deg,var(--dowe-muted),var(--dowe-surface),var(--dowe-background));"
    ));
    assert!(page.css_content.contains("background-image:linear-gradient(135deg,var(--dowe-primary),var(--dowe-secondary),var(--dowe-accent));"));
    assert!(page.css_content.contains("@media (min-width:768px)"));
    let base_vertical_padding = page
        .css_content
        .find(".py-10{padding-top:2.5rem;padding-bottom:2.5rem;}")
        .expect("base section vertical padding");
    let responsive_vertical_padding = page
        .css_content
        .rfind(".md\\:py-16{padding-top:4rem;padding-bottom:4rem;}")
        .expect("responsive section vertical padding");
    assert!(base_vertical_padding < responsive_vertical_padding);
    let design_css = super::design_css();
    assert!(design_css.contains(".section-body{width:100%;}"));
    assert!(design_css.contains(".section-body.is-boxed{max-width:96rem;margin-inline:auto;}"));
}

#[test]
fn makes_height_bounded_section_body_available_to_full_height_children() {
    let page_tree = ViewNode::Section {
        props: StyleProps {
            sizing: dowe_components::SizingProps {
                min_h: Some(ResponsiveValue::scalar(SizeValue::ViewportMinus(
                    ScaleValue::from_half_steps(0),
                ))),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![ViewNode::Grid {
            props: GridProps {
                columns: Some(ResponsiveValue::scalar(GridTracks::Count(1))),
                style: StyleProps {
                    sizing: dowe_components::SizingProps {
                        min_h: Some(ResponsiveValue::scalar(SizeValue::Full)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Grid")],
        }],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/login.dowe"),
        "page loginPage",
        &page_tree,
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);

    assert!(html.contains("min-h-vh-0"));
    assert!(html.contains("section-body section-body-has-height px-4 md:px-6 py-10 md:py-16"));
    assert!(html.contains("min-h-full"));
    assert!(page.css_content.contains(".min-h-full"));
    let design_css = super::design_css();
    assert!(design_css.contains(
        ".section{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));flex-direction:column;}"
    ));
    assert!(design_css.contains(".section-body{flex:1 1 auto;min-height:0;}"));
    assert!(design_css.contains(
        ".section-body-has-height>.h-full,.section-body-has-height>.min-h-full{flex:1 1 auto;min-height:0;}"
    ));
}

#[test]
fn emits_responsive_flex_item_classes_without_changing_grid_child_layout() {
    let flex = ResponsiveValue::ordered(vec![
        ResponsiveEntry {
            breakpoint: Breakpoint::Xs,
            value: dowe_components::FlexItem::Fill,
        },
        ResponsiveEntry {
            breakpoint: Breakpoint::Md,
            value: dowe_components::FlexItem::None,
        },
    ]);
    let tree = ViewNode::Section {
        props: StyleProps {
            sizing: dowe_components::SizingProps {
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
                    flex: Some(flex.clone()),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![ViewNode::Grid {
                props: GridProps {
                    style: StyleProps {
                        flex: Some(flex),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("Grid item")],
            }],
        }],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/flex.dowe"),
        "page flexPage",
        &tree,
    );
    let html = render_page_body(&ViewNode::Children, &tree);

    assert_eq!(html.matches("flex-1 md:flex-none").count(), 2);
    assert!(page.css_content.contains(".flex-1{flex:1 1 0%;}"));
    assert!(page.css_content.contains(".md\\:flex-none{flex:0 0 auto;}"));
    assert!(super::design_css().contains(
        ".grid{--dowe-component-display:grid;display:var(--dowe-show,var(--dowe-component-display));"
    ));
}

#[test]
fn emits_responsive_auto_and_full_height_constraints_for_containers() {
    let sizing = dowe_components::SizingProps {
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
    let tree = ViewNode::Children;
    let page_tree = ViewNode::Scope {
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
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/heights.dowe"),
        "page heightsPage",
        &page_tree,
    );
    let html = render_page_body(&tree, &page_tree);

    assert_eq!(html.matches(" h-auto").count(), 3);
    assert_eq!(html.matches(" md:h-full").count(), 3);
    assert_eq!(html.matches(" min-h-auto").count(), 3);
    assert_eq!(html.matches(" md:min-h-full").count(), 3);
    assert_eq!(html.matches(" max-h-auto").count(), 3);
    assert_eq!(html.matches(" md:max-h-full").count(), 3);
    assert!(page.css_content.contains(".h-auto{height:auto;}"));
    assert!(page.css_content.contains(".min-h-auto{min-height:auto;}"));
    assert!(page.css_content.contains(".max-h-auto{max-height:auto;}"));
    assert!(page.css_content.contains(".md\\:h-full{height:100%;}"));
    assert!(
        page.css_content
            .contains(".md\\:min-h-full{min-height:100%;}")
    );
    assert!(
        page.css_content
            .contains(".md\\:max-h-full{max-height:100%;}")
    );
}

#[test]
fn renders_flex_defaults_with_full_width_and_auto_height() {
    let page_tree = ViewNode::Flex {
        props: dowe_components::LayoutProps {
            justify: Some(ResponsiveValue::scalar(dowe_components::Justify::Center)),
            align: Some(ResponsiveValue::scalar(dowe_components::Align::Center)),
            ..Default::default()
        },
        children: vec![text("Flex")],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    let html = render_page_body(&ViewNode::Children, &page_tree);
    let design_css = super::design_css();

    assert!(html.contains("class=\"flex direction-row justify-center align-center\""));
    assert!(design_css.contains(".flex{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));width:100%;height:auto;}"));
    assert!(
        page.css_content
            .contains(".justify-center{justify-content:center;}")
    );
    assert!(
        page.css_content
            .contains(".align-center{align-items:center;}")
    );
}

#[test]
fn renders_flex_end_inside_cover_card() {
    let flex = ViewNode::Flex {
        props: dowe_components::LayoutProps {
            direction: ResponsiveValue::scalar(dowe_components::FlexDirection::Column),
            justify: Some(ResponsiveValue::scalar(dowe_components::Justify::End)),
            gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                ScaleValue::from_half_steps(4),
            )))),
            style: StyleProps {
                sizing: dowe_components::SizingProps {
                    min_h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                        ScaleValue::from_half_steps(120),
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Foreground")],
    };
    let card = ViewNode::Card {
        props: VariantProps {
            style: StyleProps {
                cover: Some(ResponsiveValue::scalar(CoverSource(
                    "https://images.example/card.jpg".to_string(),
                ))),
                overlay: Some(ResponsiveValue::scalar(OverlayPaint::BlackOpacity(
                    "0.62".to_string(),
                ))),
                text: Some(ResponsiveValue::scalar(ColorToken::White)),
                rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
                sizing: dowe_components::SizingProps {
                    min_h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                        ScaleValue::from_half_steps(144),
                    ))),
                    ..Default::default()
                },
                spacing: dowe_components::SpacingProps {
                    p: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![flex],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &card,
    );
    let html = render_page_body(&ViewNode::Children, &card);

    assert!(html.contains("has-cover"));
    assert!(html.contains("has-overlay"));
    assert!(html.contains("min-h-60 direction-column justify-end gap-2"));
    assert!(
        page.css_content
            .contains(".justify-end{justify-content:flex-end;}")
    );
    assert!(page.css_content.contains(".min-h-60{min-height:15rem;}"));
    assert!(
        page.css_content
            .contains("background-image:url(\"https://images.example/card.jpg\")")
    );
    assert!(page.css_content.contains("rgba(0,0,0,0.62)"));
}

