#[test]
fn emits_view_motion_markup_and_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Box {
        props: StyleProps {
            extras: Some(Box::new(StyleExtras {
                motion: ViewMotionStyle {
                    animation: Some(ViewAnimation::FadeIn),
                    ..Default::default()
                },
                ..Default::default()
            })),
            ..Default::default()
        },
        children: vec![ViewNode::Card {
            props: VariantProps {
                style: StyleProps {
                    extras: Some(Box::new(StyleExtras {
                        motion: ViewMotionStyle {
                            animation: Some(ViewAnimation::SlideUp),
                            ..Default::default()
                        },
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Motion")],
        }],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/motion.dowe"),
        "page",
        &page_tree,
    );
    let css = super::design_css();

    assert!(page.content.contains("animate-fade-in"));
    assert!(page.content.contains("animate-slide-up"));
    assert!(
        page.css_content
            .contains(".animate-fade-in{animation:dowe-fade-in 220ms ease-out both;}")
    );
    assert!(
        page.css_content
            .contains(".animate-slide-up{animation:dowe-slide-up 220ms ease-out both;}")
    );
    assert!(css.contains("@keyframes dowe-fade-in"));
    assert!(css.contains("scale(var(--dowe-scale)) scale(var(--dowe-gesture-scale))"));
    assert!(!css.contains("calc(var(--dowe-scale) *"));
    assert!(css.contains(
        ".has-transform{--dowe-rotate:0deg;--dowe-scale:1;--dowe-translate-x:0rem;--dowe-translate-y:0rem;--dowe-gesture-rotate:0deg;--dowe-gesture-scale:1;--dowe-gesture-x:0rem;--dowe-gesture-y:0rem;"
    ));
    assert!(css.contains("dowe-entrance-pending"));
    assert!(css.contains("@media (prefers-reduced-motion:reduce)"));
}

#[test]
fn emits_press_feedback_scale_for_web_controls() {
    let mut style = VariantProps::default();
    style.style.motion_mut().gesture = Some(ViewGesture::Press);
    let tree = ViewNode::Button {
        props: style,
        children: vec![text("Save")],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/press.dowe"),
        "press",
        &tree,
    );

    assert!(page.content.contains("has-transform"));
    assert!(page.content.contains("gesture-press"));
    assert!(super::design_css().contains(".gesture-press:active{--dowe-gesture-scale:.94;}"));
}

#[test]
fn emits_button_size_and_variant_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Button {
        props: VariantProps {
            variant: Some(ComponentVariant::Solid),
            color: Some(ColorFamily::Warning),
            size: Some(ButtonSize::Lg),
            style: StyleProps {
                rounded: Some(ResponsiveValue::scalar(RoundedSize::Full)),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Warn")],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("button-lg"));
    assert!(page.content.contains("rounded-full"));
    assert!(page.content.contains("is-solid"));
    assert!(page.content.contains("is-warning"));
    assert!(
        page.css_content
            .contains(".button-lg{padding:0.75rem 1.25rem;height:2.75rem;}")
    );
    assert!(
        page.css_content
            .contains(".rounded-full{border-radius:9999px;}")
    );
    assert!(page.css_content.contains(".button.is-solid.is-warning"));
}

#[test]
fn keeps_button_content_on_one_line_without_flex_shrink() {
    let css = super::design_css();

    assert!(css.contains(
        ".button{--dowe-component-display:inline-flex;display:var(--dowe-show,var(--dowe-component-display));flex:0 0 auto;"
    ));
    assert!(css.contains("flex:0 0 auto;width:fit-content;"));
    assert!(css.contains(
        "font:inherit;text-decoration:none;white-space:nowrap;user-select:none;-webkit-user-select:none;}"
    ));
}

#[test]
fn emits_centered_proportional_icon_button_css() {
    let css = super::design_css();

    assert!(css.contains(
        ".button>[data-dowe-button-icon-start],.button>[data-dowe-button-icon-end]{display:inline-flex;flex:0 0 auto;align-items:center;justify-content:center;}"
    ));
    assert!(css.contains(
        ".icon-button.button-xs>[data-dowe-button-icon-start]>.svg{width:1rem;height:1rem;}"
    ));
    assert!(css.contains(
        ".icon-button.button-sm>[data-dowe-button-icon-start]>.svg{width:1.25rem;height:1.25rem;}"
    ));
    assert!(css.contains(
        ".icon-button.button-md>[data-dowe-button-icon-start]>.svg{width:1.5rem;height:1.5rem;}"
    ));
    assert!(css.contains(
        ".icon-button.button-lg>[data-dowe-button-icon-start]>.svg{width:2rem;height:2rem;}"
    ));
    assert!(css.contains(
        ".icon-button.button-xl>[data-dowe-button-icon-start]>.svg{width:2.5rem;height:2.5rem;}"
    ));
    assert!(css.contains(
        ".device-toggle{border:1px solid var(--dowe-backgroundText);color:var(--dowe-backgroundText);"
    ));
    assert!(css.contains(
        ".device[data-dowe-studio-inspector] .device-viewport>.iframe{pointer-events:auto}"
    ));
}

#[test]
fn binds_icon_and_text_button_actions_to_the_full_web_control() {
    let tree = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::Button {
                props: VariantProps {
                    element: ElementProps {
                        on_click: Some("openSettings".to_string()),
                        ..Default::default()
                    },
                    icon_start: Some(solar_control_icon("settings").expect("settings icon")),
                    icon_only: true,
                    label: Some("Open settings".to_string()),
                    ..Default::default()
                },
                children: Vec::new(),
            },
            ViewNode::Button {
                props: VariantProps {
                    element: ElementProps {
                        on_click: Some("save".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("Save")],
            },
        ],
    };
    let html = render_page_body(&ViewNode::Children, &tree);
    let icon_start = html
        .find("aria-label=\"Open settings\"")
        .expect("icon button");
    let icon_open = html[..icon_start]
        .rfind("<button")
        .expect("icon opening tag");
    let icon_close = html[icon_start..]
        .find("</button>")
        .map(|offset| icon_start + offset)
        .expect("icon closing tag");
    let icon_output = &html[icon_open..icon_close];
    assert!(icon_output.contains("icon-button"));
    assert!(icon_output.contains("data-dowe-click=\"openSettings\""));
    assert!(icon_output.contains("data-dowe-button-icon-start"));
    let text_start = html.find("data-dowe-click=\"save\"").expect("text button");
    let text_open = html[..text_start]
        .rfind("<button")
        .expect("text opening tag");
    let text_close = html[text_start..]
        .find("</button>")
        .map(|offset| text_start + offset)
        .expect("text closing tag");
    let text_output = &html[text_open..text_close];
    assert!(text_output.contains("class=\"button"));
    assert!(text_output.contains("data-dowe-click=\"save\""));
    assert!(text_output.contains("Save"));
}

#[test]
fn emits_text_weight_override_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Text {
        props: TextProps {
            weight: Some(ResponsiveValue::ordered(vec![
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xs,
                    value: TextWeight::Thin,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Md,
                    value: TextWeight::Extralight,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Lg,
                    value: TextWeight::Black,
                },
            ])),
            ..Default::default()
        },
        value: "Weight".to_string(),
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("weight-thin"));
    assert!(page.content.contains("md:weight-extralight"));
    assert!(page.content.contains("lg:weight-black"));
    assert!(page.css_content.contains(".weight-thin{font-weight:100;}"));
    assert!(
        page.css_content
            .contains(".md\\:weight-extralight{font-weight:200;}")
    );
    assert!(
        page.css_content
            .contains(".lg\\:weight-black{font-weight:900;}")
    );
}

#[test]
fn emits_text_alignment_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Text {
        props: TextProps {
            align: Some(ResponsiveValue::ordered(vec![
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xs,
                    value: TextAlign::Center,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Lg,
                    value: TextAlign::End,
                },
            ])),
            ..Default::default()
        },
        value: "Aligned".to_string(),
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );
    assert!(page.content.contains("text-align-center"));
    assert!(page.content.contains("lg:text-align-end"));
    assert!(
        page.css_content
            .contains(".text-align-center{text-align:center;}")
    );
    assert!(
        page.css_content
            .contains(".lg\\:text-align-end{text-align:end;}")
    );
}

