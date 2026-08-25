#[test]
fn validates_candlestick_props_and_defaults() {
    let node = candlestick_node(vec![
        string_prop("data", "candles"),
        string_prop("stream", "/api/market/candles"),
        string_prop("variant", "ghost"),
        string_prop("scheme", "surface"),
        string_prop("upColor", "success"),
        string_prop("downColor", "danger"),
        string_prop("emptyLabel", "Waiting for candles"),
        number_prop("maxPoints", 120),
    ])
    .expect("candlestick");

    match node {
        ViewNode::Candlestick { props } => {
            assert_eq!(props.data, "candles");
            assert_eq!(props.stream.as_deref(), Some("/api/market/candles"));
            assert_eq!(props.style.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
            assert_eq!(props.up_color, ColorToken::Success);
            assert_eq!(props.down_color, ColorToken::Danger);
            assert_eq!(props.empty_label, "Waiting for candles");
            assert_eq!(props.max_points, 120);
            assert!(props.style.style.sizing.h.is_some());
        }
        _ => panic!("candlestick"),
    }

    let default_node =
        candlestick_node(vec![string_prop("data", "candles")]).expect("default candlestick");
    match default_node {
        ViewNode::Candlestick { props } => {
            assert_eq!(props.stream, None);
            assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
            assert_eq!(props.up_color, ColorToken::Success);
            assert_eq!(props.down_color, ColorToken::Danger);
            assert_eq!(props.empty_label, "No candle data");
            assert_eq!(props.max_points, 240);
        }
        _ => panic!("candlestick"),
    }
}

#[test]
fn validates_chart_component_props_and_defaults() {
    let arc = arc_chart_component_node(vec![
        string_prop("data", "segments"),
        string_prop("palette", "ocean"),
        string_prop("legendPosition", "left"),
        number_prop("thickness", 18),
        boolean_prop("showInlineLabels", true),
    ])
    .expect("arc chart");
    match arc {
        ViewNode::ArcChart { props } => {
            assert_eq!(props.common.data.as_deref(), Some("segments"));
            assert_eq!(props.common.palette, ChartPalette::Ocean);
            assert_eq!(props.common.legend_position, ChartLegendPosition::Left);
            assert_eq!(props.common.size, ChartSize::Md);
            assert_eq!(props.thickness, 18);
            assert!(props.show_inline_labels);
        }
        _ => panic!("arc chart"),
    }

    let area = area_chart_component_node(vec![
        string_prop("series", "traffic"),
        string_prop("curve", "smooth"),
        number_string_prop("fillOpacity", "0.42"),
        boolean_prop("showPoints", true),
    ])
    .expect("area chart");
    match area {
        ViewNode::AreaChart { props } => {
            assert_eq!(props.common.series.as_deref(), Some("traffic"));
            assert_eq!(props.common.legend_position, ChartLegendPosition::Bottom);
            assert_eq!(props.curve, ChartCurve::Smooth);
            assert_eq!(props.fill_opacity, 42);
            assert!(props.show_points);
        }
        _ => panic!("area chart"),
    }

    let bar = bar_chart_component_node(vec![
        string_prop("data", "sales"),
        string_prop("size", "lg"),
        string_prop("scheme", "surface"),
        boolean_prop("grouped", true),
    ])
    .expect("bar chart");
    match bar {
        ViewNode::BarChart { props } => {
            assert_eq!(props.common.data.as_deref(), Some("sales"));
            assert_eq!(props.common.size, ChartSize::Lg);
            assert_eq!(props.common.style.color, Some(ColorFamily::Surface));
            assert!(props.grouped);
        }
        _ => panic!("bar chart"),
    }

    let line = line_chart_component_node(vec![
        string_prop("data", "trend"),
        string_prop("palette", "forest"),
        string_prop("curve", "smooth"),
        boolean_prop("showGradientFill", true),
    ])
    .expect("line chart");
    match line {
        ViewNode::LineChart { props } => {
            assert_eq!(props.common.data.as_deref(), Some("trend"));
            assert_eq!(props.common.palette, ChartPalette::Forest);
            assert_eq!(props.curve, ChartCurve::Smooth);
            assert!(props.show_gradient_fill);
        }
        _ => panic!("line chart"),
    }

    let pie = pie_chart_component_node(vec![
        string_prop("data", "segments"),
        boolean_prop("donut", true),
        number_prop("donutWidth", 72),
        string_prop("centerLabel", "Total"),
    ])
    .expect("pie chart");
    match pie {
        ViewNode::PieChart { props } => {
            assert_eq!(props.common.data.as_deref(), Some("segments"));
            assert_eq!(props.common.legend_position, ChartLegendPosition::Right);
            assert!(props.donut);
            assert_eq!(props.donut_width, 72);
            assert_eq!(props.center_label.as_deref(), Some("Total"));
        }
        _ => panic!("pie chart"),
    }
}

#[test]
fn rejects_invalid_candlestick_props() {
    assert_eq!(
        candlestick_node(Vec::new()).expect_err("data"),
        ComponentError::invalid_prop("data", "signal array path")
    );
    assert_eq!(
        candlestick_node(vec![
            string_prop("data", "candles"),
            string_prop("stream", "http://example.com/events")
        ])
        .expect_err("stream"),
        ComponentError::invalid_prop("stream", "absolute path or https URL")
    );
    assert_eq!(
        candlestick_node(vec![
            string_prop("data", "candles"),
            string_prop("upColor", "brand-color")
        ])
        .expect_err("upColor"),
        ComponentError::invalid_prop("upColor", "color token")
    );
    assert_eq!(
        candlestick_node(vec![
            string_prop("data", "candles"),
            number_prop("maxPoints", 0)
        ])
        .expect_err("maxPoints"),
        ComponentError::invalid_prop("maxPoints", "positive integer")
    );
}

#[test]
fn validates_table_props_columns_and_defaults() {
    let name = table_column_component(vec![
        string_prop("field", "name"),
        string_prop("label", "Name"),
        string_prop("width", "12rem"),
    ])
    .expect("name column");
    let role = table_column_component(vec![
        string_prop("field", "profile.role"),
        string_prop("label", "Role"),
        string_prop("align", "end"),
    ])
    .expect("role column");
    let node = table_node(
        vec![
            string_prop("data", "users"),
            string_prop("variant", "ghost"),
            string_prop("scheme", "surface"),
            string_prop("size", "lg"),
            boolean_prop("striped", true),
            boolean_prop("bordered", true),
            boolean_prop("dividers", false),
            string_prop("emptyTitle", "No users"),
            string_prop("emptyDescription", "Invite a user first"),
        ],
        vec![name, role],
    )
    .expect("table");

    match node {
        ViewNode::Table { props } => {
            assert_eq!(props.data, "users");
            assert_eq!(props.style.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
            assert_eq!(props.size, TableSize::Lg);
            assert!(props.striped);
            assert!(props.bordered);
            assert!(!props.dividers);
            assert_eq!(props.empty_title, "No users");
            assert_eq!(props.empty_description, "Invite a user first");
            assert_eq!(props.columns.len(), 2);
            assert_eq!(props.columns[0].width.as_deref(), Some("12rem"));
            assert_eq!(props.columns[1].align, TableColumnAlign::End);
        }
        _ => panic!("table"),
    }

    let default_node = table_node(
        vec![string_prop("data", "users")],
        vec![
            table_column_component(vec![
                string_prop("field", "name"),
                string_prop("label", "Name"),
            ])
            .expect("column"),
        ],
    )
    .expect("default table");
    match default_node {
        ViewNode::Table { props } => {
            assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
            assert_eq!(props.style.color, Some(ColorFamily::Surface));
            assert_eq!(props.size, TableSize::Md);
            assert!(!props.striped);
            assert!(!props.bordered);
            assert!(props.dividers);
        }
        _ => panic!("table"),
    }
}

#[test]
fn rejects_invalid_table_props_and_columns() {
    assert_eq!(
        table_node(Vec::new(), Vec::new()).expect_err("columns"),
        ComponentError::invalid_prop_combination("Table requires at least one column")
    );
    assert_eq!(
        table_node(
            vec![
                string_prop("data", "users"),
                string_prop("color", "primary")
            ],
            vec![
                table_column_component(vec![
                    string_prop("field", "name"),
                    string_prop("label", "Name"),
                ])
                .expect("column"),
            ],
        )
        .expect_err("color"),
        ComponentError::new("unknown prop `color` on `Table`; use `scheme` for visual family")
    );
    assert_eq!(
        table_node(
            Vec::new(),
            vec![
                table_column_component(vec![
                    string_prop("field", "name"),
                    string_prop("label", "Name"),
                ])
                .expect("column"),
            ],
        )
        .expect_err("data"),
        ComponentError::invalid_prop("data", "signal array path")
    );
    assert_eq!(
        table_column_component(vec![string_prop("label", "Name")]).expect_err("field"),
        ComponentError::invalid_prop("field", "relative field path")
    );
    assert_eq!(
        table_column_component(vec![
            string_prop("field", ".name"),
            string_prop("label", "Name"),
        ])
        .expect_err("field"),
        ComponentError::invalid_prop("field", "relative field path")
    );
    assert_eq!(
        table_column_component(vec![
            string_prop("field", "name"),
            string_prop("label", "Name"),
            string_prop("align", "right"),
        ])
        .expect_err("align"),
        ComponentError::invalid_prop("align", "start, center or end")
    );
    assert_eq!(
        table_column_component(vec![
            string_prop("field", "name"),
            string_prop("label", "Name"),
            string_prop("width", "calc(100%)"),
        ])
        .expect_err("width"),
        ComponentError::invalid_prop("width", "portable table width")
    );
}

#[test]
fn validates_diagram_component_props_and_defaults() {
    let diagram = diagram_component_node(vec![
        string_prop("nodes", "flow.nodes"),
        string_prop("edges", "flow.edges"),
        string_prop("onNodeClick", "selectNode"),
        boolean_prop("minimap", true),
        string_prop("emptyLabel", "Empty flow"),
    ])
    .expect("diagram");
    match diagram {
        ViewNode::Diagram { props } => {
            assert_eq!(props.nodes, "flow.nodes");
            assert_eq!(props.edges, "flow.edges");
            assert_eq!(props.on_node_click.as_deref(), Some("selectNode"));
            assert!(props.minimap);
            assert!(props.fit_view);
            assert!(props.pan_on_drag);
            assert!(props.zoom_on_scroll);
            assert!(props.controls);
            assert!(props.show_grid);
            assert_eq!(props.empty_label, "Empty flow");
            assert_eq!(
                props.style.style.sizing.h,
                Some(ResponsiveValue::scalar(SizeValue::Scale(
                    ScaleValue::from_half_steps(150)
                )))
            );
        }
        _ => panic!("diagram"),
    }

    assert!(diagram_component_node(vec![string_prop("nodes", "nodes")]).is_err());
    assert!(diagram_component_node(vec![string_prop("edges", "edges")]).is_err());
    assert_eq!(
        diagram_component_node(vec![
            string_prop("nodes", "nodes"),
            string_prop("edges", "edges"),
            string_prop("unknownProp", "value"),
        ])
        .expect_err("unknown prop"),
        ComponentError::unknown_prop(BuiltinComponent::Diagram, "unknownProp")
    );
}
