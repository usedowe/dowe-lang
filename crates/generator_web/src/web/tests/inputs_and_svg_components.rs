#[test]
fn renders_labeled_input_and_select_markup() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::Input {
                props: VariantProps {
                    label: Some("Name".to_string()),
                    placeholder: Some("Full name".to_string()),
                    label_floating: true,
                    size: Some(ButtonSize::Sm),
                    icon_start: Some(solar_control_icon("magnifier").expect("start icon")),
                    icon_end: Some(solar_control_icon("close-circle").expect("end icon")),
                    ..Default::default()
                },
            },
            ViewNode::Input {
                props: VariantProps {
                    label: Some("Email".to_string()),
                    element: ElementProps {
                        show: Some(VisibilityCondition::Signal("showEmail".to_string())),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ViewNode::Select {
                props: VariantProps {
                    label: Some("Role".to_string()),
                    placeholder: Some("Choose role".to_string()),
                    label_floating: true,
                    size: Some(ButtonSize::Lg),
                    ..Default::default()
                },
                options: vec![
                    SelectOption {
                        value: "admin".to_string(),
                        label: "Admin".to_string(),
                        description: None,
                    },
                    SelectOption {
                        value: "viewer".to_string(),
                        label: "Viewer".to_string(),
                        description: Some("Read only".to_string()),
                    },
                ],
                option_each: None,
            },
        ],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("is-floating"));
    assert!(
        page.content
            .contains(r#"class=\"field\" data-dowe-show=\"showEmail\""#)
    );
    assert!(page.content.contains(r#"class=\"control is-sm"#));
    assert!(page.content.contains(r#"class=\"control is-lg"#));
    assert!(page.content.contains("has-start-adornment"));
    assert!(page.content.contains(r#"placeholder=\"Full name\""#));
    assert!(
        page.content
            .contains(r#"class=\"control-icon icon-start\""#)
    );
    assert!(page.content.contains(r#"class=\"control-icon icon-end\""#));
    assert!(page.content.contains("data-dowe-select"));
    assert!(page.content.contains(r#"<svg class=\"select-arrow\""#));
    assert!(
        page.content
            .contains("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4")
    );
    assert!(page.content.contains(r#"data-dowe-option-value=\"admin\""#));
    assert!(page.content.contains("select-option-description"));
    assert!(page.content.contains("Read only"));
}

#[test]
fn renders_phone_floating_label_inside_number_input_shell() {
    let tree = ViewNode::Phone {
        props: PhoneProps {
            style: bound_style("profile.phone", "Phone number", "Enter phone number"),
            value: None,
            country: Some("US".to_string()),
            dial_code_name: "dialCode".to_string(),
            search_placeholder: "Search countries".to_string(),
            empty_text: "No countries".to_string(),
            loading_text: "Loading".to_string(),
            priority_countries: vec!["US".to_string()],
            disabled: false,
            name: None,
            help_text: None,
            error_text: None,
        },
    };
    let html = render_page_body(&ViewNode::Children, &tree);
    let trigger = html
        .find(r#"class="phone-country-trigger""#)
        .expect("country trigger");
    let shell = html
        .find(r#"class="phone-input-shell""#)
        .expect("phone input shell");
    let label = html
        .find(r#"class="control-label">Phone number</span>"#)
        .expect("floating phone label");
    let input = html
        .find(r#"class="phone-input input""#)
        .expect("phone input");

    assert!(trigger < shell);
    assert!(shell < label);
    assert!(label < input);
    assert!(
        super::design_css().contains(
            ".phone-input-shell>.control-label{left:.75rem;max-width:calc(100% - 1.5rem);}"
        )
    );
}

#[test]
fn renders_svg_markup_and_color_classes() {
    let root = Path::new("/project");
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &svg_tree(),
    );

    assert!(page.content.contains(r#"<svg"#));
    assert!(page.content.contains(r#"class=\"svg"#));
    assert!(page.content.contains("color-accent"));
    assert!(page.content.contains("w-8"));
    assert!(page.content.contains("h-8"));
    assert!(
        page.content
            .contains(r#"xmlns=\"http://www.w3.org/2000/svg\""#)
    );
    assert!(page.content.contains(r#"viewBox=\"0 0 24 24\""#));
    assert!(page.content.contains(r#"aria-hidden=\"true\""#));
    assert!(
        page.content
            .contains(r#"<path d=\"M0 0h24v24H0z\" fill=\"none\"></path>"#)
    );
    assert!(
        page.content
            .contains(r#"<path d=\"M22 12c0-5.523-4.477-10-10-10\" fill=\"currentColor\" fill-rule=\"evenodd\" clip-rule=\"evenodd\" transform=\"matrix(2 0 0 2 4 6)\"></path>"#)
    );
    assert!(page.css_content.contains(".svg"));
    assert!(
        page.css_content
            .contains(".color-accent{--dowe-content-text:var(--dowe-accent);--dowe-content-title:var(--dowe-accent);color:var(--dowe-accent);}")
    );
    assert!(page.css_content.contains(".w-8{width:2rem;}"));
    assert!(page.css_content.contains(".h-8{height:2rem;}"));
}

#[test]
fn preserves_svg_intrinsic_ratio_when_web_dimension_is_omitted() {
    let mut tree = svg_tree();
    let ViewNode::Svg { props, .. } = &mut tree else {
        panic!("svg tree");
    };
    props.style.sizing.w = None;

    let html = render_page_body(&ViewNode::Children, &tree);

    assert!(html.contains(r#"class="svg color-accent h-8""#));
    assert!(!html.contains("w-8"));
}

#[test]
fn renders_runtime_svg_as_safe_data_surface() {
    let root = Path::new("/project");
    let tree = dowe_components::svg_component_node(
        vec![ComponentProp {
            name: "data".to_string(),
            value: PropValue::String("icon.svg".to_string()),
        }],
        Vec::new(),
    )
    .expect("runtime Svg");
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &tree,
    );

    assert!(page.content.contains("data-dowe-svg-data=\\\"icon.svg\\\""));
    assert!(!page.content.contains("<path"));
}

#[test]
fn renders_dynamic_icon_binding_surface() {
    let root = Path::new("/project");
    let tree = dowe_components::icon_component_node(vec![
        ComponentProp {
            name: "name".to_string(),
            value: PropValue::String("@icon-binding:iconName".to_string()),
        },
        ComponentProp {
            name: "fill".to_string(),
            value: PropValue::String("muted".to_string()),
        },
    ])
    .expect("dynamic Icon");
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &tree,
    );

    assert!(
        page.content
            .contains("data-dowe-icon-name=\\\"iconName\\\"")
    );
    assert!(page.content.contains("color-muted"));
    assert!(page.content.contains("viewBox=\\\"0 0 24 24\\\""));
    assert!(!page.content.contains("<path"));
}

#[test]
fn renders_viewport_minus_height_classes() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Box {
        props: StyleProps {
            sizing: dowe_components::SizingProps {
                h: Some(ResponsiveValue::scalar(
                    dowe_components::SizeValue::ViewportMinus(ScaleValue::from_half_steps(32)),
                )),
                min_h: Some(ResponsiveValue::scalar(
                    dowe_components::SizeValue::ViewportMinus(ScaleValue::from_half_steps(40)),
                )),
                max_w: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                    ScaleValue::from_half_steps(128),
                ))),
                max_h: Some(ResponsiveValue::scalar(
                    dowe_components::SizeValue::ViewportMinus(ScaleValue::from_half_steps(48)),
                )),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("vh-16"));
    assert!(page.content.contains("min-h-vh-20"));
    assert!(page.content.contains("max-w-64"));
    assert!(page.content.contains("max-h-vh-24"));
    assert!(
        page.css_content
            .contains(".vh-16{height:calc(100vh - 4rem);}")
    );
    assert!(
        page.css_content
            .contains(".min-h-vh-20{min-height:calc(100vh - 5rem);}")
    );
    assert!(page.css_content.contains(".max-w-64{max-width:16rem;}"));
    assert!(
        page.css_content
            .contains(".max-h-vh-24{max-height:calc(100vh - 6rem);}")
    );
}

#[test]
fn renders_percentage_width_classes() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Box {
        props: StyleProps {
            sizing: dowe_components::SizingProps {
                w: Some(ResponsiveValue::scalar(SizeValue::Percent(30))),
                min_w: Some(ResponsiveValue {
                    entries: vec![
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Xs,
                            value: SizeValue::Percent(40),
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Md,
                            value: SizeValue::Percent(100),
                        },
                    ],
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(
        page.content
            .contains("w-pct-30 min-w-pct-40 md:min-w-pct-100")
    );
    assert!(page.css_content.contains(".w-pct-30{width:30%;}"));
    assert!(page.css_content.contains(".min-w-pct-40{min-width:40%;}"));
    assert!(
        page.css_content
            .contains(".md\\:min-w-pct-100{min-width:100%;}")
    );
}

#[test]
fn renders_camera_and_microphone_capture_contract() {
    let page_tree = ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::Camera {
                props: CameraProps {
                    style: VariantProps::default(),
                    facing: CameraFacing::User,
                    label: "Take photo".to_string(),
                    disabled: false,
                    on_start: Some("cameraStart".to_string()),
                    on_capture: Some("cameraCapture".to_string()),
                    on_error: Some("cameraError".to_string()),
                },
            },
            ViewNode::Microphone {
                props: MicrophoneProps {
                    style: VariantProps::default(),
                    label: "Record audio".to_string(),
                    max_duration: Some(30),
                    disabled: false,
                    on_start: Some("microphoneStart".to_string()),
                    on_stop: Some("microphoneStop".to_string()),
                    on_error: Some("microphoneError".to_string()),
                },
            },
        ],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/capture.dowe"),
        "capture",
        &page_tree,
    );
    let runtime_chunks = super::runtime_chunks_for_trees(&ViewNode::Children, &page_tree);
    assert_eq!(
        runtime_chunks
            .iter()
            .map(|chunk| chunk.name)
            .collect::<Vec<_>>(),
        vec!["media"]
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);
    let router = full_runtime_for_test();

    assert!(html.contains("data-dowe-camera"));
    assert!(html.contains("data-dowe-camera-facing=\"user\""));
    assert!(html.contains("data-dowe-camera-on-capture=\"cameraCapture\""));
    assert!(html.contains("data-dowe-microphone"));
    assert!(html.contains("data-dowe-microphone-max-duration=\"30\""));
    assert!(page.css_content.contains(".camera"));
    assert!(page.css_content.contains(".microphone"));
    assert!(router.contains("function hydrateCameras(root)"));
    assert!(router.contains("navigator.mediaDevices.getUserMedia"));
    assert!(router.contains("function hydrateMicrophones(root)"));
    assert!(router.contains("new MediaRecorder(microphone.__doweMicrophoneStream)"));
    assert!(router.contains("function closeCameraFrames(view)"));
    assert!(router.contains("function closeMicrophoneFrames(view)"));
}
