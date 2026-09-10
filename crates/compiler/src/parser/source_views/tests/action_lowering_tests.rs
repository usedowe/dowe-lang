#[test]
fn parses_portable_standard_library_view_syntax() {
    let tree = parse_page(
        r#"page standardLibraryPage
  signal text value:"  value  "
  signal values value:[]
  signal result value:""
  fn run
    set result source:str.trim value:text
    set result source:math.sum values:values
    set result source:parse.int value:text fallback:0
    set result source:url.querySet value:text name:"page" param:text
    set result source:csv.parse value:text header:true
    set result source:sort.by values:values field:"score" direction:"desc"
    set result source:list.filterContains values:values field:"name" value:text
    set result source:json.get value:text path:"name" fallback:""
    set result source:date.now"#,
    )
    .expect("tree");
    let ViewNode::Scope { actions, .. } = tree else {
        panic!("scope");
    };
    let ViewActionKind::Sequence(statements) = &actions[0].kind else {
        panic!("sequence");
    };
    let names = statements
        .iter()
        .filter_map(|statement| match statement {
            ViewFunctionStatement::Assign(assign) => assign.call.as_ref().map(|call| call.name()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let names = names.iter().map(String::as_str).collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "str.trim",
            "math.sum",
            "parse.int",
            "url.querySet",
            "csv.parse",
            "sort.by",
            "list.filterContains",
            "json.get",
            "date.now",
        ]
    );
}

#[test]
fn parses_every_fill_emitted_by_svg_conversion() {
    let svg = r##"<svg viewBox="0 0 16 8">
<path d="M0 0L1 1Z" fill="#000001"/>
<path d="M1 0L2 1Z" fill="#000002"/>
<path d="M2 0L3 1Z" fill="#000003"/>
<path d="M3 0L4 1Z" fill="#000004"/>
<path d="M4 0L5 1Z" fill="#000005"/>
<path d="M5 0L6 1Z" fill="#000006"/>
<path d="M6 0L7 1Z" fill="#000007"/>
<path d="M7 0L8 1Z" fill="#000008"/>
</svg>"##;
    let call = dowe_stdlib::StdlibCall {
        namespace: "parse".to_string(),
        function: "svg".to_string(),
        args: vec![dowe_stdlib::StdlibArgument {
            name: "value".to_string(),
            value: dowe_stdlib::StdlibValue::String(svg.to_string()),
        }],
    };
    let converted = dowe_stdlib::evaluate(&call, |_| None)
        .expect("conversion")
        .as_str()
        .expect("source")
        .lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n");

    parse_page(&format!("page converted\n{converted}")).expect("converted svg");
}

#[test]
fn parses_set_reference_negation_and_literals() {
    let tree = parse_page(
        r#"page menuPage
  signal openMenu value:false
  signal drawerVisible value:true
  fn copy
    set openMenu value:drawerVisible
  fn toggle
    set openMenu value:!openMenu
  fn open
    set openMenu value:true
  fn close
    set openMenu value:false
  Box
    Text
      "Menu""#,
    )
    .expect("tree");
    let ViewNode::Scope { actions, .. } = tree else {
        panic!("scope");
    };
    let sources = actions
        .iter()
        .map(|action| match &action.kind {
            ViewActionKind::Assign(action) => action.source.as_str(),
            _ => panic!("set"),
        })
        .collect::<Vec<_>>();

    assert_eq!(
        sources,
        [
            "drawerVisible",
            "!openMenu",
            "$dowe:bool:true",
            "$dowe:bool:false"
        ]
    );
}

#[test]
fn keeps_bare_on_click_as_action_reference() {
    let tree = parse_page(
        r#"page menuPage
  signal openDrawer value:false
  fn openDrawer
    set openDrawer value:true
  IconButton label:"menu" variant:"ghost" onClick:openDrawer icon:"menu-dots""#,
    )
    .expect("tree");
    let ViewNode::Scope { actions, .. } = tree else {
        panic!("scope");
    };

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].name, "openDrawer");
    let ViewActionKind::Assign(action) = &actions[0].kind else {
        panic!("set action");
    };
    assert_eq!(action.target, "openDrawer");
    assert_eq!(action.source, "$dowe:bool:true");
}

#[test]
fn lowers_inline_on_click_state_updates() {
    let tree = parse_page(
        r#"page menuPage
  signal openDrawer value:false
  signal counter value:0
  signal name value:""
  Button onClick:{ set:openDrawer value:!openDrawer }
    "Toggle"
  Button onClick:{ set:counter add:1 }
    "Increment"
  Button onClick:{ set:name value:"Ada" }
    "Name"
  Button onClick:{ set:name append:"!" }
    "Append""#,
    )
    .expect("tree");
    let ViewNode::Scope { actions, .. } = tree else {
        panic!("scope");
    };

    assert_eq!(actions.len(), 4);
    let assignments = actions
        .iter()
        .map(|action| {
            let ViewActionKind::Assign(assign) = &action.kind else {
                panic!("inline set");
            };
            assign
        })
        .collect::<Vec<_>>();
    assert_eq!(assignments[0].target, "openDrawer");
    assert_eq!(assignments[0].source, "!openDrawer");
    assert_eq!(assignments[1].target, "counter");
    assert_eq!(assignments[1].source, "$dowe:onClick:add");
    assert_eq!(assignments[2].target, "name");
    assert_eq!(assignments[2].source, "$dowe:string:Ada");
    assert_eq!(assignments[3].target, "name");
    assert_eq!(assignments[3].source, "$dowe:onClick:append");
    assert_eq!(
        assignments[1].call.as_ref().expect("add call").function,
        "add"
    );
    assert_eq!(
        assignments[3].call.as_ref().expect("append call").function,
        "join"
    );
}

