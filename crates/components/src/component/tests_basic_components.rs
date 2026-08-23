#[test]
fn validates_box_children() {
    assert!(box_node(Vec::new()).is_ok());
    assert!(box_node(vec![text_node("Hello").expect("text")]).is_ok());
}

#[test]
fn validates_text_content() {
    assert_eq!(
        text_node("   ").expect_err("text error"),
        ComponentError::text_requires_static_text(BuiltinComponent::Text)
    );

    assert_eq!(
        text_node("  Hello  ").expect("text"),
        ViewNode::Text {
            props: Default::default(),
            value: "Hello".to_string()
        }
    );
}

#[test]
fn validates_code_source_and_highlighting() {
    let node = code_node(
        vec![
            string_prop("language", "dowe"),
            string_prop("variant", "ghost"),
            string_prop("scheme", "surface"),
        ],
        "page loginPage\n  meta name:\"title\" content:\"Login\"\n  Card scheme:\"primary\"\n    Text\n      Login".to_string(),
    )
    .expect("code");

    match node {
        ViewNode::Code { props } => {
            assert_eq!(props.language, CodeLanguage::Dowe);
            assert_eq!(
                props
                    .tokens
                    .iter()
                    .map(|token| token.text.as_str())
                    .collect::<String>(),
                props.source
            );
            assert!(
                props
                    .tokens
                    .iter()
                    .any(|token| token.kind == CodeTokenKind::Keyword && token.text == "page")
            );
            assert!(
                props
                    .tokens
                    .iter()
                    .any(|token| token.kind == CodeTokenKind::Keyword && token.text == "meta")
            );
            assert!(
                props
                    .tokens
                    .iter()
                    .any(|token| token.kind == CodeTokenKind::Type && token.text == "Card")
            );
            assert!(
                props
                    .tokens
                    .iter()
                    .any(|token| token.kind == CodeTokenKind::Attribute && token.text == "scheme")
            );
        }
        _ => panic!("code"),
    }

    assert_eq!(
        code_node(Vec::new(), String::new()).expect_err("content"),
        ComponentError::invalid_prop("content", "non-empty multiline string")
    );
    assert_eq!(
        code_node(vec![string_prop("language", "ruby")], "puts()".to_string())
            .expect_err("language"),
        ComponentError::invalid_prop(
            "language",
            "dowe, typescript, javascript, go, rust or python"
        )
    );
}

#[test]
fn highlights_javascript_python_and_reactive_code_segments() {
    let javascript = code_node(
        vec![string_prop("language", "javascript")],
        "const value = new Promise(() => true);".to_string(),
    )
    .expect("javascript");
    let python = code_node(
        vec![string_prop("language", "python")],
        "def greet(name: str):\n  # welcome\n  return f\"Hello {name}\"".to_string(),
    )
    .expect("python");
    let template = code_node(
        vec![
            string_prop("language", "dowe"),
            ComponentProp {
                name: "template".to_string(),
                value: PropValue::Boolean(true),
            },
        ],
        "Button scheme:\"{scheme}\"\n  \"Preview\"".to_string(),
    )
    .expect("template");

    for node in [javascript, python] {
        let ViewNode::Code { props } = node else {
            panic!("code")
        };
        assert_eq!(
            props
                .tokens
                .iter()
                .map(|token| token.text.as_str())
                .collect::<String>(),
            props.source
        );
        assert!(
            props
                .tokens
                .iter()
                .any(|token| token.kind == CodeTokenKind::Keyword)
        );
        assert!(
            props
                .tokens
                .iter()
                .any(|token| token.kind == CodeTokenKind::Type)
        );
    }

    let ViewNode::Code { props } = template else {
        panic!("code")
    };
    assert!(props.template_segments.iter().any(|segment| matches!(
        segment,
        CodeTemplateSegment::Static { tokens, .. }
            if tokens.iter().any(|token| token.kind == CodeTokenKind::Type)
    )));
}

#[test]
fn validates_video_source_and_defaults() {
    let node = video_node(vec![
        string_prop("src", "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8"),
        string_prop("poster", "/images/video.jpg"),
    ])
    .expect("video");

    match node {
        ViewNode::Video { props } => {
            assert_eq!(
                props.src,
                "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8"
            );
            assert_eq!(props.poster.as_deref(), Some("/images/video.jpg"));
            assert!(!props.autoplay);
            assert_eq!(props.aspect, VideoAspect::Horizontal);
            assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
        }
        _ => panic!("video"),
    }

    assert_eq!(
        video_node(Vec::new()).expect_err("src"),
        ComponentError::invalid_prop("src", "https URL")
    );
    assert_eq!(
        video_node(vec![string_prop("src", "http://example.com/video.mp4")]).expect_err("https"),
        ComponentError::invalid_prop("src", "https URL")
    );
    assert_eq!(
        video_node(vec![
            string_prop("src", "https://example.com/video.mp4"),
            string_prop("aspect", "wide"),
        ])
        .expect_err("aspect"),
        ComponentError::invalid_prop("aspect", "horizontal, vertical or square")
    );
}

#[test]
fn validates_iframe_source_policy_and_defaults() {
    let node = iframe_node(vec![
        string_prop("src", "https://example.com/embed"),
        string_prop("title", "Example embed"),
        string_prop("allow", "fullscreen; autoplay"),
        string_prop("sandbox", "scripts same-origin"),
    ])
    .expect("iframe");
    let ViewNode::Iframe { props } = node else {
        panic!("iframe")
    };
    assert_eq!(props.loading, IframeLoading::Lazy);
    assert_eq!(props.allow, vec!["fullscreen", "autoplay"]);
    assert_eq!(
        props.sandbox,
        Some(vec!["scripts".to_string(), "same-origin".to_string()])
    );
    assert!(!props.allow_fullscreen);
    let internal = iframe_node(vec![
        string_prop("src", "/examples/appbar-one"),
        string_prop("title", "Local example"),
    ])
    .expect("internal iframe");
    let ViewNode::Iframe { props } = internal else {
        panic!("iframe")
    };
    assert_eq!(props.src, "/examples/appbar-one");
    assert_eq!(
        iframe_node(vec![
            string_prop("src", "http://example.com"),
            string_prop("title", "Example")
        ])
        .expect_err("https"),
        ComponentError::invalid_prop("src", "https URL or internal route")
    );
    assert_eq!(
        iframe_node(vec![
            string_prop("src", "//example.com"),
            string_prop("title", "Example")
        ])
        .expect_err("scheme relative"),
        ComponentError::invalid_prop("src", "https URL or internal route")
    );
    assert_eq!(
        iframe_node(vec![
            string_prop("src", "/examples/../admin"),
            string_prop("title", "Example")
        ])
        .expect_err("traversal"),
        ComponentError::invalid_prop("src", "https URL or internal route")
    );
    assert_eq!(
        iframe_node(vec![string_prop("src", "https://example.com")]).expect_err("title"),
        ComponentError::invalid_prop("title", "non-empty string")
    );
}

#[test]
fn validates_device_profile_and_iframe_child() {
    let iframe = iframe_node(vec![
        string_prop("src", "/preview"),
        string_prop("title", "Preview"),
    ])
    .expect("iframe");
    let node =
        device_node(vec![string_prop("device", "laptop")], vec![iframe.clone()]).expect("device");
    let ViewNode::Device {
        props,
        iframe: nested,
    } = node
    else {
        panic!("device")
    };
    assert_eq!(props.device, DeviceProfile::Laptop);
    assert_eq!(props.device.dimensions(), (1440, 900));
    assert_eq!(nested.title, "Preview");
    assert_eq!(
        device_node(vec![string_prop("device", "watch")], vec![iframe.clone()])
            .expect_err("device"),
        ComponentError::invalid_prop("device", "mobile, tablet, laptop or monitor")
    );
    assert!(device_node(Vec::new(), Vec::new()).is_err());
    assert!(device_node(Vec::new(), vec![iframe.clone(), iframe]).is_err());
}

#[test]
fn validates_canvas_props_and_defaults() {
    let node = canvas_component_node(vec![
        string_prop("scene", "gameScene"),
        string_prop("label", "Game scene"),
        string_prop("fit", "cover"),
        number_prop("viewWidth", 640),
        number_prop("viewHeight", 360),
        number_prop("fps", 30),
        boolean_prop("autoplay", false),
        string_prop("background", "surface"),
        boolean_prop("pixelated", true),
        string_prop("onPointer", "capturePointer"),
        string_prop("onKey", "captureKey"),
        string_prop("onMotion", "captureMotion"),
        number_prop("motionRate", 45),
    ])
    .expect("canvas");
    match node {
        ViewNode::Canvas { props } => {
            assert_eq!(props.scene, "gameScene");
            assert_eq!(props.view_width, 640);
            assert_eq!(props.view_height, 360);
            assert_eq!(props.fit, CanvasFit::Cover);
            assert_eq!(props.fps, 30);
            assert!(!props.autoplay);
            assert_eq!(
                props.background,
                CanvasBackground::Color(ColorToken::Surface)
            );
            assert!(props.pixelated);
            assert_eq!(props.label, "Game scene");
            assert_eq!(props.on_pointer.as_deref(), Some("capturePointer"));
            assert_eq!(props.on_key.as_deref(), Some("captureKey"));
            assert_eq!(props.on_motion.as_deref(), Some("captureMotion"));
            assert_eq!(props.motion_rate, 45);
        }
        _ => panic!("canvas"),
    }

    assert!(canvas_component_node(vec![string_prop("label", "Missing scene")]).is_err());
    assert!(
        canvas_component_node(vec![
            string_prop("scene", "scene"),
            string_prop("label", "")
        ])
        .is_err()
    );
    assert!(
        canvas_component_node(vec![
            string_prop("scene", "scene"),
            string_prop("label", "Scene"),
            number_prop("fps", 121)
        ])
        .is_err()
    );
    assert!(
        canvas_component_node(vec![
            string_prop("scene", "scene"),
            string_prop("label", "Scene"),
            string_prop("fit", "center")
        ])
        .is_err()
    );
    assert!(
        canvas_component_node(vec![
            string_prop("scene", "scene"),
            string_prop("label", "Scene"),
            number_prop("motionRate", 61)
        ])
        .is_err()
    );
}
