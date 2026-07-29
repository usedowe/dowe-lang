    #[test]
    fn parses_svg_component_with_path_children() {
        let tree = parse_page(
            r#"page iconPage
  Svg viewBox:"0 0 24 24" color:"tertiary" w:8 h:8
    Path d:"M0 0h24v24H0z" fill:"none"
    Path fill:"currentColor" d:"M22 12c0-5.523-4.477-10-10-10S2 6.477 2 12s4.477 10 10 10s10-4.477 10-10"
    Path fill:"tertiary" d:"M1 1h2v2H1z""#,
        )
        .expect("tree");

        let ViewNode::Svg { props, paths } = tree else {
            panic!("svg");
        };
        assert_eq!(props.view_box.as_str(), "0 0 24 24");
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].fill, SvgPathFill::None);
        assert_eq!(paths[1].fill, SvgPathFill::CurrentColor);
        assert_eq!(paths[2].fill, SvgPathFill::Color(ColorToken::Tertiary));
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
  Video src:"https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8" poster:"/images/video.jpg" autoplay:true aspect:"vertical" variant:"soft" scheme:"tertiary""#,
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
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.style.color, Some(ColorFamily::Tertiary));
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
    fn rejects_legacy_code_lines() {
        let error = parse_page(
            r#"page codePage
  Code lines:["page example"]"#,
        )
        .expect_err("legacy lines");
        assert!(
            error
                .to_string()
                .contains("was replaced by multiline `content`")
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
    fn parses_candlestick_component_with_typed_data_and_stream() {
        let tree = parse_page(
            r#"type Candle
  time:string
  open:number
  high:number
  low:number
  close:number

page marketPage
  signal candles type:Candle[] value:[{ time:"2026-06-01T09:30:00Z" open:102 high:108 low:99 close:106 }]
  Candlestick data:candles stream:"/api/market/candles" variant:"soft" scheme:"surface" upColor:"success" downColor:"danger" emptyLabel:"Waiting" maxPoints:120"#,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Candlestick { props } = &children[0] else {
            panic!("candlestick");
        };
        assert_eq!(props.data, "candles");
        assert_eq!(props.stream.as_deref(), Some("/api/market/candles"));
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.style.color, Some(ColorFamily::Surface));
        assert_eq!(props.up_color, ColorToken::Success);
        assert_eq!(props.down_color, ColorToken::Danger);
        assert_eq!(props.empty_label, "Waiting");
        assert_eq!(props.max_points, 120);
    }

    #[test]
    fn rejects_invalid_candlestick_usage() {
        let missing = parse_page(
            r#"page marketPage
  Candlestick"#,
        )
        .expect_err("missing");
        assert!(
            missing
                .to_string()
                .contains("invalid value for prop `data`: expected signal array path")
        );

        let wrong_type = parse_page(
            r#"page marketPage
  signal candles value:{ time:"" }
  Candlestick data:candles"#,
        )
        .expect_err("wrong type");
        assert!(
            wrong_type
                .to_string()
                .contains("signal `candles` in `data` must be an array")
        );

        let missing_field = parse_page(
            r#"type Candle
  time:string
  open:number
  high:number
  low:number

page marketPage
  signal candles type:Candle[] value:[]
  Candlestick data:candles"#,
        )
        .expect_err("missing field");
        assert!(
            missing_field
                .to_string()
                .contains("Candlestick data item must include `close`")
        );

        let invalid_candle = parse_page(
            r#"page marketPage
  signal candles value:[{ time:"1" open:10 high:9 low:8 close:10 }]
  Candlestick data:candles"#,
        )
        .expect_err("invalid candle");
        assert!(
            invalid_candle
                .to_string()
                .contains("Candlestick data item violates OHLC bounds")
        );

        let stream = parse_page(
            r#"page marketPage
  signal candles value:[]
  Candlestick data:candles stream:"http://example.com/events""#,
        )
        .expect_err("stream");
        assert!(
            stream
                .to_string()
                .contains("invalid value for prop `stream`: expected absolute path or https URL")
        );

        let child = parse_page(
            r#"page marketPage
  signal candles value:[]
  Candlestick data:candles
    Text
      "Invalid""#,
        )
        .expect_err("child");
        assert!(
            child
                .to_string()
                .contains("children are not valid for this component")
        );
    }

    #[test]
    fn parses_chart_components_with_typed_signals() {
        let tree = parse_page(
            r#"type ChartPoint
  x:number
  y:number

type ChartSlice
  label:string
  value:number

page chartPage
  signal points type:ChartPoint[] value:[{ x:1 y:12 }, { x:2 y:18 }]
  signal slices type:ChartSlice[] value:[{ label:"Docs" value:40 }, { label:"CLI" value:60 }]
  Box
    LineChart data:points curve:"smooth" palette:"ocean" size:"lg" showGradientFill:true
    AreaChart data:points legendPosition:"bottom" fillOpacity:0.42 showPoints:true
    BarChart data:slices grouped:true showValues:true
    ArcChart data:slices legendPosition:"right" thickness:18 showInlineLabels:true
    PieChart data:slices donut:true donutWidth:72"#,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Box {
            children: chart_children,
            ..
        } = &children[0]
        else {
            panic!("box");
        };
        let ViewNode::LineChart { props } = &chart_children[0] else {
            panic!("line chart");
        };
        assert_eq!(props.common.data.as_deref(), Some("points"));
        assert_eq!(props.common.palette, ChartPalette::Ocean);
        assert_eq!(props.common.size, ChartSize::Lg);
        assert_eq!(props.curve, ChartCurve::Smooth);
        assert!(props.show_gradient_fill);

        let ViewNode::AreaChart { props } = &chart_children[1] else {
            panic!("area chart");
        };
        assert_eq!(props.common.legend_position, ChartLegendPosition::Bottom);
        assert_eq!(props.fill_opacity, 42);
        assert!(props.show_points);

        let ViewNode::BarChart { props } = &chart_children[2] else {
            panic!("bar chart");
        };
        assert!(props.grouped);
        assert!(props.show_values);

        let ViewNode::ArcChart { props } = &chart_children[3] else {
            panic!("arc chart");
        };
        assert_eq!(props.common.legend_position, ChartLegendPosition::Right);
        assert_eq!(props.thickness, 18);
        assert!(props.show_inline_labels);

        let ViewNode::PieChart { props } = &chart_children[4] else {
            panic!("pie chart");
        };
        assert!(props.donut);
        assert_eq!(props.donut_width, 72);
    }

    #[test]
    fn rejects_invalid_chart_data_shape() {
        let error = parse_page(
            r#"page chartPage
  signal points value:[{ x:1 }]
  LineChart data:points"#,
        )
        .expect_err("line chart data");
        assert!(
            error
                .to_string()
                .contains("LineChart data item must include `y`")
        );
    }

    #[test]
    fn parses_table_component_with_typed_data_and_columns() {
        let tree = parse_page(
            r#"type UserRow
  name:string
  status:string

page usersPage
  signal users type:UserRow[] value:[{ name:"Ana" status:"active" }]
  Table data:users variant:"soft" scheme:"surface" size:"lg" striped:true bordered:true dividers:true emptyTitle:"No users" emptyDescription:"Invite users"
    column field:"name" label:"Name"
    column field:"status" label:"Status" align:"end" width:"8rem""#,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Table { props } = &children[0] else {
            panic!("table");
        };
        assert_eq!(props.data, "users");
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.style.color, Some(ColorFamily::Surface));
        assert_eq!(props.size, TableSize::Lg);
        assert!(props.striped);
        assert!(props.bordered);
        assert!(props.dividers);
        assert_eq!(props.empty_title, "No users");
        assert_eq!(props.empty_description, "Invite users");
        assert_eq!(props.columns.len(), 2);
        assert_eq!(props.columns[1].field, "status");
        assert_eq!(props.columns[1].align, TableColumnAlign::End);
        assert_eq!(props.columns[1].width.as_deref(), Some("8rem"));
    }
