#[test]
fn parses_draw_erase_defaults_and_mode_binding() {
    for (component, source_prop, mode, expected, binding, draw) in [
        ("Draw", "bind", "drawMode:\"erase\"", "erase", false, true),
        ("Draw", "bind", "", "pen", false, true),
        ("Draw", "scene", "", "pen", false, false),
        ("Canvas", "scene", "", "pen", false, false),
        ("Draw", "bind", "drawMode:mode", "mode", true, true),
    ] {
        let source = format!(
            "page drawPage\n  signal layers value:[]\n  signal mode value:\"erase\"\n  {component} {source_prop}:layers {mode} label:\"Editor\""
        );
        let tree = parse_page(&source).expect("valid draw mode");
        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope")
        };
        let ViewNode::Canvas { props } = &children[0] else {
            panic!("canvas")
        };
        assert_eq!(props.draw_mode, expected);
        assert_eq!(props.draw_mode_binding, binding);
        assert_eq!(props.draw, draw);
        assert_eq!(
            props.layer_bind.as_deref(),
            (source_prop == "bind").then_some("layers")
        );
    }
    for mode in ["select", "erase"] {
        let source = format!(
            "page canvasPage\n  signal layers value:[]\n  Canvas scene:layers drawMode:\"{mode}\" label:\"Canvas\""
        );
        let error = parse_page(&source).expect_err("Draw-only mode");
        assert!(error.to_string().contains("expected pen, rect or circle"));
    }
}

#[test]
fn parses_draw_with_bound_layers_and_layer_events() {
    let tree = parse_page(
        r#"page drawPage
  signal layers value:[{ id:"circle-1" type:"circle" x:80 y:60 radius:20 fill:"primary" }]
  signal selected value:""
  fn addLayer
    set selected value:""
  fn changeLayer
    set selected value:""
  fn removeLayer
    set selected value:""
  fn selectLayer
    set selected value:""
  Draw bind:layers selected:selected draw:true drawMode:"select" label:"Layer editor" onLayerAdd:addLayer onLayerChange:changeLayer onLayerRemove:removeLayer onLayerSelect:selectLayer"#,
    )
    .expect("draw tree");
    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope");
    };
    let ViewNode::Canvas { props } = &children[0] else {
        panic!("draw canvas");
    };
    assert_eq!(props.scene, "layers");
    assert_eq!(props.layer_bind.as_deref(), Some("layers"));
    assert_eq!(props.selected_layer.as_deref(), Some("selected"));
    assert_eq!(props.draw_mode, "select");
    assert!(props.draw);
    assert_eq!(props.on_layer_add.as_deref(), Some("addLayer"));
    assert_eq!(props.on_layer_change.as_deref(), Some("changeLayer"));
    assert_eq!(props.on_layer_remove.as_deref(), Some("removeLayer"));
    assert_eq!(props.on_layer_select.as_deref(), Some("selectLayer"));

    let error = parse_page(
        r#"page drawPage
  signal layers value:[]
  Draw bind:"layers" label:"Invalid layer binding""#,
    )
    .expect_err("quoted bind");
    assert!(error.to_string().contains("signal array path"));
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
  Candlestick data:candles stream:"/api/market/candles" variant:"solid" scheme:"surface" upColor:"success" downColor:"danger" emptyLabel:"Waiting" maxPoints:120"#,
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
    assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
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

