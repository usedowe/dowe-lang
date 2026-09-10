fn collect_special_node_classes(node: &ViewNode, classes: &mut BTreeSet<String>) {
    match node {
            ViewNode::Code { props } => {
                classes.extend(variant_classes("code-block", &props.style));
            }
            ViewNode::Video { props } => {
                classes.extend(video_classes(props));
            }
            ViewNode::Iframe { props } => {
                classes.extend(iframe_classes(props));
            }
            ViewNode::Device { props, iframe } => {
                let mut device_classes = vec!["device".to_string()];
                append_style_classes(&mut device_classes, &props.style);
                classes.extend(device_classes);
                classes.extend(iframe_classes(iframe));
                classes.extend([
                    "device-toolbar".to_string(),
                    "device-toggle".to_string(),
                    "device-stage".to_string(),
                    "device-viewport".to_string(),
                    "is-active".to_string(),
                ]);
                for option in &props.options {
                    classes.extend(svg_classes(&option.icon.props.style));
                }
            }
            ViewNode::Canvas { props } => {
                classes.extend(canvas_classes(props));
            }
            ViewNode::Candlestick { props } => {
                classes.extend(candlestick_classes(props));
                classes.insert("candlestick-canvas".to_string());
                classes.insert("candlestick-empty".to_string());
            }
            ViewNode::Diagram { props } => {
                classes.extend(diagram_classes(props));
                classes.insert("diagram-canvas".to_string());
                classes.insert("diagram-edges-layer".to_string());
                classes.insert("diagram-nodes-layer".to_string());
                classes.insert("diagram-node".to_string());
                classes.insert("diagram-node-label".to_string());
                classes.insert("diagram-edge".to_string());
                classes.insert("diagram-edge-label".to_string());
                classes.insert("diagram-empty".to_string());
            }
            ViewNode::ArcChart { props } => {
                collect_chart_classes("arc-chart-container", &props.common, classes);
            }
            ViewNode::AreaChart { props } => {
                collect_chart_classes("area-chart-container", &props.common, classes);
            }
            ViewNode::BarChart { props } => {
                collect_chart_classes("bar-chart-container", &props.common, classes);
            }
            ViewNode::LineChart { props } => {
                collect_chart_classes("line-chart-container", &props.common, classes);
            }
            ViewNode::PieChart { props } => {
                collect_chart_classes("pie-chart-container", &props.common, classes);
            }
            ViewNode::Table { props } => {
                classes.extend(table_wrapper_classes(props));
                classes.insert("table-container".to_string());
                classes.extend(table_classes(props));
                classes.insert("table-header".to_string());
                classes.insert("table-head".to_string());
                classes.insert("table-head-content".to_string());
                classes.insert("table-head-label".to_string());
                classes.insert("table-body".to_string());
                classes.insert("table-empty-row".to_string());
                classes.insert("table-empty-cell".to_string());
                classes.insert("empty-state".to_string());
                classes.insert("empty-content".to_string());
                classes.insert("empty-title".to_string());
                classes.insert("empty-description".to_string());
            }
            ViewNode::Divider { props } => {
                classes.extend(divider_classes(props));
            }
            ViewNode::Alert { props } => {
                classes.extend(variant_classes("alert", &props.style));
                classes.insert("alert-close".to_string());
            }
            ViewNode::Svg { props, .. } => {
                classes.extend(svg_classes(&props.style));
            }
            ViewNode::Title { props, .. } => {
                classes.extend(text_classes("title", props));
            }
            ViewNode::Text { props, .. } => {
                classes.extend(text_classes("text", props));
            }
        _ => {}
    }
}

fn collect_chart_classes(base: &str, props: &ChartCommonProps, classes: &mut BTreeSet<String>) {
    classes.extend(chart_classes(base, props));
    classes.insert("dowe-chart-viewport".to_string());
    classes.insert("dowe-chart-svg".to_string());
    classes.insert("dowe-chart-loading".to_string());
    classes.insert("dowe-chart-empty".to_string());
    classes.insert("dowe-chart-legend".to_string());
    classes.insert("dowe-chart-legend-item".to_string());
    classes.insert("dowe-chart-legend-color".to_string());
    classes.insert("dowe-chart-axis-line".to_string());
    classes.insert("dowe-chart-axis-label".to_string());
    classes.insert("dowe-chart-grid-line".to_string());
    classes.insert("dowe-chart-line".to_string());
    classes.insert("dowe-chart-area".to_string());
    classes.insert("dowe-chart-point".to_string());
    classes.insert("dowe-chart-bar".to_string());
    classes.insert("dowe-chart-slice".to_string());
    classes.insert("dowe-chart-arc".to_string());
    classes.insert("dowe-chart-center-value".to_string());
    classes.insert("dowe-chart-center-label".to_string());
}
