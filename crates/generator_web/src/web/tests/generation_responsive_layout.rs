#[test]
fn renders_responsive_section_centering_on_the_body() {
    let page_tree = ViewNode::Section {
        props: StyleProps {
            center_x: Some(ResponsiveValue::ordered(vec![
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xs,
                    value: false,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Md,
                    value: true,
                },
            ])),
            ..Default::default()
        },
        children: vec![text("Centered")],
    };
    let _page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    let html = render_page_body(&ViewNode::Children, &page_tree);
    assert!(html.contains(
        "<div class=\"section-body section-center-x-false md:section-center-x-true px-4 md:px-6 py-10 md:py-16\">"
    ));
    let design_css = super::design_css();
    assert!(
        design_css
            .contains(".section-body{display:flex;flex-direction:column;align-items:flex-start;}")
    );
    assert!(design_css.contains(".section-body.section-center-x-true{align-items:center;}"));
    assert!(design_css.contains(
        "@media (min-width:768px){.md\\:section-center-x-true{align-items:center;}.md\\:section-center-x-false{align-items:flex-start;}.md\\:section-center-y-true{justify-content:center;}.md\\:section-center-y-false{justify-content:flex-start;}}"
    ));
}

#[test]
fn renders_responsive_section_gap_on_the_body() {
    let page_tree = ViewNode::Section {
        props: StyleProps {
            gap: Some(ResponsiveValue::ordered(vec![
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xs,
                    value: GapValue::Single(GapSize::Scale(ScaleValue(4))),
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Md,
                    value: GapValue::Single(GapSize::Scale(ScaleValue(8))),
                },
            ])),
            ..Default::default()
        },
        children: vec![text("First"), text("Second")],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    let html = render_page_body(&ViewNode::Children, &page_tree);
    assert!(html.contains("section-body gap-2 md:gap-4 px-4 md:px-6 py-10 md:py-16"));
    assert!(page.css_content.contains(".gap-2{gap:0.5rem;}"));
    assert!(page.css_content.contains(".md\\:gap-4{gap:1rem;}"));
}

#[test]
fn emits_explicit_section_body_padding_css() {
    let page_tree = ViewNode::Section {
        props: StyleProps {
            spacing: dowe_components::SpacingProps {
                p: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(24))),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Content")],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);

    assert!(html.contains("<section class=\"section\"><div class=\"section-body p-12\">"));
    assert!(page.css_content.contains(".p-12{padding:3rem;}"));
}

#[test]
fn preserves_default_section_horizontal_padding_with_vertical_override() {
    let page_tree = ViewNode::Section {
        props: StyleProps {
            spacing: dowe_components::SpacingProps {
                py: Some(ResponsiveValue::ordered(vec![
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Xs,
                        value: ScaleValue::from_half_steps(12),
                    },
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: ScaleValue::from_half_steps(20),
                    },
                ])),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Content")],
    };
    let html = render_page_body(&ViewNode::Children, &page_tree);

    assert!(html.contains(
        "<section class=\"section\"><div class=\"section-body px-4 md:px-6 py-6 md:py-10\">"
    ));
}

#[test]
fn scopes_layout_and_page_reactivity_by_generated_id() {
    let root = Path::new("/project");
    let layout_tree = reactive_tree("layout01", "action01", true);
    let page_tree = reactive_tree("page0001", "action02", false);
    let layout = build_layout_chunk(
        root,
        Path::new("/project/src/layouts/auth.dowe"),
        "layout reactive",
        &layout_tree,
    );
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/login.dowe"),
        "page reactive",
        &page_tree,
    );
    let html = render_page_body(&layout_tree, &page_tree);

    assert!(layout.content.contains("doweLayout"));
    assert!(layout.content.contains(r#""id":"layout01""#));
    assert!(layout.content.contains(r#""target":"layout01""#));
    assert!(page.content.contains(r#""id":"page0001""#));
    assert!(page.content.contains(r#""target":"page0001""#));
    assert!(html.contains(r#"data-dowe-bind="layout01.message""#));
    assert!(html.contains(r#"data-dowe-bind="page0001.message""#));
    assert!(html.contains(r#"data-dowe-click="action01""#));
    assert!(html.contains(r#"data-dowe-click="action02""#));
}

#[test]
fn emits_interactive_motion_classes_rules_and_chip_event() {
    let mut style = VariantProps::default();
    style.style.set_animation(Some(ViewAnimation::ScaleIn));
    {
        let motion = style.style.motion_mut();
        motion.rotate = Some(ResponsiveValue::scalar(ViewRotation(-7)));
        motion.scale = Some(ResponsiveValue::scalar(ViewScale(105)));
        motion.translate_x = Some(ResponsiveValue::scalar(ViewTranslation(-3)));
        motion.translate_y = Some(ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: ViewTranslation(0),
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: ViewTranslation(4),
            },
        ]));
        motion.transition = Some(ViewTransition::Spring);
        motion.gesture = Some(ViewGesture::Lift);
    }
    style.element.on_click = Some("selectMobile".to_string());
    let tree = ViewNode::Chip {
        props: ChipProps {
            style,
            on_close: None,
        },
        value: "Mobile Apps".to_string(),
        start: None,
        end: None,
    };

    let html = render_page_body(&ViewNode::Children, &tree);
    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/views/pages/motion.dowe"),
        "motion",
        &tree,
    );

    assert!(html.contains("has-transform"));
    assert!(html.contains("rotate-neg-7"));
    assert!(html.contains("scale-1_05"));
    assert!(html.contains("translate-x-neg-1.5"));
    assert!(html.contains("md:translate-y-2"));
    assert!(html.contains("transition-spring"));
    assert!(html.contains("gesture-lift"));
    assert!(html.contains("data-dowe-click=\"selectMobile\""));
    assert!(chunk.css_content.contains("--dowe-rotate:-7deg"));
    assert!(chunk.css_content.contains("--dowe-scale:1.05"));
    assert!(chunk.css_content.contains("--dowe-translate-x:-0.75rem"));
}

