#[test]
fn parses_svg_component_with_path_children() {
    let tree = parse_page(
            r#"page iconPage
  Svg viewBox:"0 0 24 24" color:"accent" w:8 h:8
    Path d:"M0 0h24v24H0z" fill:"none"
    Path fill:"currentColor" fillRule:"evenodd" d:"M22 12c0-5.523-4.477-10-10-10S2 6.477 2 12s4.477 10 10 10s10-4.477 10-10"
    Path fill:"accent" d:"M1 1h2v2H1z""#,
        )
        .expect("tree");

    let ViewNode::Svg { props, paths } = tree else {
        panic!("svg");
    };
    assert_eq!(props.view_box.as_str(), "0 0 24 24");
    assert_eq!(paths.len(), 3);
    assert_eq!(paths[0].fill, SvgPathFill::None);
    assert_eq!(
        paths[1].fill,
        SvgPathFill::Fill {
            color: None,
            opacity: 255,
            even_odd: true,
        }
    );
    assert_eq!(paths[2].fill, SvgPathFill::Color(ColorToken::Accent));
}

#[test]
fn parses_namespaced_svg_logo_icon() {
    let tree = parse_page(
        r#"page logoPage
  Icon name:"svg-logos:github-icon" w:10 h:10"#,
    )
    .expect("SVG logo Icon");

    let ViewNode::Svg { props, paths } = tree else {
        panic!("SVG logo");
    };
    assert!(!props.is_animated());
    assert!(props.motion.is_some());
    assert!(!paths.is_empty());
    assert!(paths.iter().any(|path| matches!(
        path.fill,
        SvgPathFill::LiteralFill { .. } | SvgPathFill::LiteralStroke { .. }
    )));
}

#[test]
fn parses_brand_with_svg_navigation_and_box_sizing() {
    let tree = parse_page(
        r#"page brandPage
  Brand href:"/" label:"Dowe home" w:{ xs:24 md:32 } h:8
    Svg viewBox:"0 0 24 24" w:"full" h:"full"
      Path d:"M2 2H22V22H2Z" fill:"primary""#,
    )
    .expect("Brand");

    let ViewNode::Brand { props, children } = tree else {
        panic!("brand");
    };
    assert_eq!(props.label.as_deref(), Some("Dowe home"));
    assert!(matches!(
        props.navigation,
        Some(NavigationAction::Internal { ref path, .. }) if path == "/"
    ));
    assert_eq!(props.style.sizing.w.expect("width").entries.len(), 2);
    assert!(props.style.sizing.h.is_some());
    assert!(matches!(children.as_slice(), [ViewNode::Svg { .. }]));
}

#[test]
fn parses_banner_with_required_external_navigation() {
    let tree = parse_page(
        r#"page bannerPage
  Banner href:"https://dowe.dev/cloud" label:"Explore Dowe Cloud" p:{ xs:5 md:8 }
    Grid columns:{ xs:1 md:2 } gap:4
      Title
        "Build beyond code"
      Text
        "Explore Dowe Cloud""#,
    )
    .expect("Banner");

    let ViewNode::Banner { props, children } = tree else {
        panic!("banner");
    };
    assert_eq!(props.label.as_deref(), Some("Explore Dowe Cloud"));
    assert!(matches!(
        props.navigation,
        NavigationAction::External {
            ref url,
            web_target: WebTarget::Blank,
            native_external_mode: NativeExternalMode::System,
        } if url == "https://dowe.dev/cloud"
    ));
    assert!(props.style.spacing.p.is_some());
    assert!(matches!(children.as_slice(), [ViewNode::Grid { .. }]));

    for source in [
        "page invalid\n  Banner\n    Text\n      \"Missing href\"",
        "page invalid\n  Banner href:\"/pricing\"\n    Text\n      \"Internal\"",
        "page invalid\n  Banner href:\"https://dowe.dev\"",
    ] {
        assert!(parse_page(source).is_err(), "{source}");
    }
}

#[test]
fn parses_runtime_svg_data_and_rejects_mixed_geometry() {
    let tree = parse_page(
        r#"page iconPage
  signal icons value:[]
  each in:icons as:icon key:icon.id
    Svg data:icon.svg color:"primary" w:12 h:12"#,
    )
    .expect("runtime Svg");
    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope");
    };
    let ViewNode::Each { children, .. } = &children[0] else {
        panic!("each");
    };
    let ViewNode::Svg { props, paths } = &children[0] else {
        panic!("svg");
    };
    assert_eq!(props.data.as_deref(), Some("icon.svg"));
    assert!(paths.is_empty());

    let mixed = parse_page(
        r#"page iconPage
  signal icon value:{ svg:{} }
  Svg data:icon.svg viewBox:"0 0 24 24"
    Path d:"M0 0h24v24H0z""#,
    )
    .expect_err("mixed Svg");
    assert!(mixed.to_string().contains("cannot combine"));
}

#[test]
fn parses_video_component_with_hls_source() {
    let tree = parse_page(
            r#"page videoPage
  Video src:"https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8" poster:"/images/video.jpg" autoplay:true aspect:"vertical" variant:"solid" scheme:"accent""#,
        )
        .expect("tree");

    let ViewNode::Video { props } = tree else {
        panic!("video");
    };
    assert_eq!(
        props.src,
        "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8"
    );
    assert_eq!(props.poster.as_deref(), Some("/images/video.jpg"));
    assert!(props.autoplay);
    assert_eq!(props.aspect, VideoAspect::Vertical);
    assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
    assert_eq!(props.style.color, Some(ColorFamily::Accent));
}

#[test]
fn parses_code_multiline_content_with_relative_indentation() {
    let tree = parse_page(
        r#"page codePage
  Code:
    language:"dowe"
    content:"""
      page example
        Text
          "Hello"

        Button
          "Continue"
    """"#,
    )
    .expect("code");
    let ViewNode::Code { props } = tree else {
        panic!("code")
    };
    assert_eq!(
        props.source,
        "page example\n  Text\n    \"Hello\"\n\n  Button\n    \"Continue\""
    );
}
#[test]
fn parses_canvas_component_and_validates_scene_signal() {
    let tree = parse_page(
            r#"page canvasPage
  signal input value:{ x:80 y:60 }
  signal scene value:[{ type:"circle" x:80 y:60 radius:20 fill:"primary" bind:{ x:"input.x" y:"input.y" } }]
  fn capture
    set input value:item
  Canvas scene:scene viewWidth:640 viewHeight:360 fit:"cover" fps:30 autoplay:false background:"surface" pixelated:true label:"Game scene" onPointer:capture onKey:capture onMotion:capture motionRate:24 w:"full" h:48"#,
        )
        .expect("tree");
    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope")
    };
    let ViewNode::Canvas { props } = &children[0] else {
        panic!("canvas")
    };
    assert_eq!(props.scene, "scene");
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
    assert_eq!(props.on_pointer.as_deref(), Some("capture"));
    assert_eq!(props.on_key.as_deref(), Some("capture"));
    assert_eq!(props.on_motion.as_deref(), Some("capture"));
    assert_eq!(props.motion_rate, 24);

    let multiline = parse_page(
        r#"page canvasPage
  signal input:
    value:{ x:80 y:60 }
  signal scene:
    value:[
      {
        type:"circle"
        x:80
        y:60
        radius:20
        fill:"primary"
        bind:{ x:"input.x" y:"input.y" }
      },
    ]
  fn capture
    set input value:item
  Canvas:
    scene:scene
    viewWidth:640
    viewHeight:360
    fit:"cover"
    fps:30
    autoplay:false
    background:"surface"
    pixelated:true
    label:"Game scene"
    onPointer:capture
    onKey:capture
    onMotion:capture
    motionRate:24
    w:"full"
    h:48"#,
    )
    .expect("multiline tree");
    let ViewNode::Scope { children, .. } = multiline else {
        panic!("multiline scope")
    };
    let ViewNode::Canvas {
        props: multiline_props,
    } = &children[0]
    else {
        panic!("multiline canvas")
    };
    assert_eq!(multiline_props, props);

    let error = parse_page(
        r#"page canvasPage
  signal scene value:{ type:"circle" }
  Canvas scene:scene label:"Invalid scene""#,
    )
    .expect_err("scene type");
    assert!(
        error
            .to_string()
            .contains("signal `scene` in `scene` must be an array")
    );

    let error = parse_page(
        r#"page canvasPage
  signal scene value:[]
  Canvas scene:scene label:"Invalid action" onPointer:missing"#,
    )
    .expect_err("action");
    assert!(error.to_string().contains("unknown fn `missing`"));
}

#[test]
fn parses_game_component_with_socket_bindings_and_lifecycle_actions() {
    let tree = parse_page(
        r#"page gamePage
  signal scene value:[{ type:"circle" x:80 y:60 radius:20 fill:"primary" }]
  signal endpoint value:"/game"
  signal outbound value:{ type:"move" x:80 }
  signal connection value:"closed"
  fn connected
    set connection value:"open"
  fn received
    set connection value:item.kind
  fn disconnected
    set connection value:"closed"
  fn failed
    set connection value:"error"
  Game scene:scene label:"Network game" socket:endpoint send:outbound status:connection onOpen:connected onMessage:received onClose:disconnected onError:failed reconnect:false reconnectDelay:500 w:"full" h:80"#,
    )
    .expect("game tree");
    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope")
    };
    let ViewNode::Game { props } = &children[0] else {
        panic!("game")
    };
    assert_eq!(props.scene.as_deref(), Some("scene"));
    assert_eq!(props.socket.as_deref(), Some("endpoint"));
    assert!(props.socket_binding);
    assert_eq!(props.send.as_deref(), Some("outbound"));
    assert_eq!(props.status.as_deref(), Some("connection"));
    assert_eq!(props.on_open.as_deref(), Some("connected"));
    assert_eq!(props.on_message.as_deref(), Some("received"));
    assert_eq!(props.on_close.as_deref(), Some("disconnected"));
    assert_eq!(props.on_error.as_deref(), Some("failed"));
    assert!(!props.reconnect);
    assert_eq!(props.reconnect_delay, 500);

    let error = parse_page(
        r#"page gamePage
  signal scene value:[]
  Game scene:scene label:"Invalid socket" socket:"http://localhost/game""#,
    )
    .expect_err("socket scheme");
    assert!(error.to_string().contains("ws/wss URL or absolute backend path"));

    let error = parse_page(
        r#"page gamePage
  signal scene value:[]
  Game scene:scene label:"Invalid delay" reconnectDelay:99"#,
    )
    .expect_err("reconnect delay");
    assert!(error.to_string().contains("integer from 100 through 60000"));
}

#[test]
fn parses_game_raycast3d_world_and_camera_contract() {
    let tree = parse_page(
        r##"page fortressPage
  signal world value:{ map:["11111" "10001" "10001" "10001" "11111"] wall:"#5f6fff" floor:"#151b2e" sprites:[{ id:"imp" x:3.5 y:2.5 color:"#ff4d6d" }] }
  signal camera value:{ x:1.5 y:2.5 angle:0 fov:1.047 pitch:0 }
  fn fireWeapon
    set camera.angle value:item.data.angle
  Game renderer:"raycast3d" world:world camera:camera controls:"doom" moveSpeed:3 turnSpeed:150 onFire:fireWeapon label:"Dowe Fortress" w:"full" h:96"##,
    )
    .expect("raycast game tree");
    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope")
    };
    let ViewNode::Game { props } = &children[0] else {
        panic!("game")
    };
    assert_eq!(props.renderer, GameRenderer::Raycast3d);
    assert_eq!(props.controls, GameControls::Doom);
    assert_eq!(props.scene, None);
    assert_eq!(props.world.as_deref(), Some("world"));
    assert_eq!(props.camera.as_deref(), Some("camera"));
    assert_eq!(props.move_speed, 3);
    assert_eq!(props.turn_speed, 150);
    assert_eq!(props.on_fire.as_deref(), Some("fireWeapon"));
}
