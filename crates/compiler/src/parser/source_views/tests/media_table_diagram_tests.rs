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
  Table data:users variant:"solid" scheme:"surface" size:"lg" striped:true bordered:true dividers:true emptyTitle:"No users" emptyDescription:"Invite users"
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
    assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
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

#[test]
fn parses_diagram_with_signal_paths_and_interaction_props() {
    let tree = parse_page(
        r#"page diagramPage
  signal nodes value:[{ id:"input" x:20 y:20 width:120 height:48 label:"Input" }]
  signal edges value:[{ id:"edge" source:"input" target:"input" }]
  Diagram nodes:nodes edges:edges fitView:true minimap:true onNodeClick:selectNode onNodeDrag:moveNode onConnect:connectNodes"#,
    )
    .expect("diagram");
    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope");
    };
    let ViewNode::Diagram { props } = &children[0] else {
        panic!("diagram");
    };
    assert_eq!(props.nodes, "nodes");
    assert_eq!(props.edges, "edges");
    assert!(props.fit_view);
    assert!(props.minimap);
    assert_eq!(props.on_node_click.as_deref(), Some("selectNode"));
    assert_eq!(props.on_node_drag.as_deref(), Some("moveNode"));
    assert_eq!(props.on_connect.as_deref(), Some("connectNodes"));
}
