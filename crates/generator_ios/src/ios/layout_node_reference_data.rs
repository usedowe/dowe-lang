fn ios_data_node_references_layout_bindings(node: &ViewNode, bindings: &IosLayoutBindings) -> bool {
    match node {
        ViewNode::Fab { props, actions } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || props
                    .style
                    .element
                    .on_click
                    .as_deref()
                    .is_some_and(|value| bindings.references_action(value))
                || actions.iter().any(|action| {
                    action
                        .on_click
                        .as_deref()
                        .is_some_and(|value| bindings.references_action(value))
                })
        }
        ViewNode::Candlestick { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.data)
        }
        ViewNode::ArcChart { props } => ios_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::AreaChart { props } => ios_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::BarChart { props } => ios_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::LineChart { props } => ios_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::PieChart { props } => ios_chart_references_layout_bindings(&props.common, bindings),
        ViewNode::Table { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.data)
        }
        ViewNode::Tree { props } => {
            ios_variant_references_layout_bindings(&props.style, bindings)
                || bindings.references_signal(&props.data)
                || props.bind.as_deref().is_some_and(|value| bindings.references_signal(value))
                || props.on_select.as_deref().is_some_and(|value| bindings.references_action(value))
        }
        _ => false
    }
}
