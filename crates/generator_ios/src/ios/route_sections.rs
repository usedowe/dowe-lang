const IOS_ROUTE_EXPRESSION_WEIGHT_LIMIT: usize = 14;

struct IosRouteBranch<'a> {
    node: &'a ViewNode,
    flow: NativeFlow,
}

fn ios_route_branches(nodes: &[ViewNode]) -> Vec<IosRouteBranch<'_>> {
    let mut branches = Vec::new();
    for node in nodes {
        collect_ios_route_branches(node, NativeFlow::Block, true, &mut branches);
    }
    branches
}

fn collect_ios_route_branches<'a>(
    node: &'a ViewNode,
    flow: NativeFlow,
    neutral_context: bool,
    branches: &mut Vec<IosRouteBranch<'a>>,
) {
    let heavy_expression = ios_route_expression_weight(node) >= IOS_ROUTE_EXPRESSION_WEIGHT_LIMIT;
    for (children, child_flow, child_context) in ios_route_branch_children(node, flow, neutral_context)
    {
        for child in children {
            if heavy_expression && child_context {
                branches.push(IosRouteBranch {
                    node: child,
                    flow: child_flow,
                });
            }
            collect_ios_route_branches(child, child_flow, child_context, branches);
        }
    }
}

fn ios_route_expression_weight(node: &ViewNode) -> usize {
    let own = match node {
        ViewNode::SelectTheme { .. } => 16,
        ViewNode::Code { .. }
        | ViewNode::Table { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Map { .. }
        | ViewNode::Canvas { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. } => 12,
        ViewNode::Text { .. } | ViewNode::Title { .. } => 4,
        _ => 1,
    };
    ios_route_branch_children(node, NativeFlow::Block, true)
        .into_iter()
        .flat_map(|(children, _, _)| children.iter())
        .fold(own, |weight, child| {
            weight.saturating_add(ios_route_expression_weight(child))
        })
}

fn ios_route_branch_children(
    node: &ViewNode,
    flow: NativeFlow,
    neutral_context: bool,
) -> Vec<(&[ViewNode], NativeFlow, bool)> {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => vec![
            (content.as_slice(), flow, neutral_context),
            (children.as_slice(), flow, neutral_context),
        ],
        ViewNode::Scope { children, .. } => {
            vec![(children.as_slice(), flow, neutral_context)]
        }
        ViewNode::Box { props, children } | ViewNode::Section { props, children } => vec![(
            children.as_slice(),
            NativeFlow::Block,
            neutral_context && props.font.is_none(),
        )],
        ViewNode::Flex { props, children } => vec![(
            children.as_slice(),
            NativeFlow::Inline,
            neutral_context && props.style.font.is_none(),
        )],
        ViewNode::Grid { props, children } => vec![(
            children.as_slice(),
            NativeFlow::Block,
            neutral_context && props.style.font.is_none(),
        )],
        ViewNode::Card { props, children } => vec![(
            children.as_slice(),
            NativeFlow::Block,
            neutral_context && props.style.font.is_none(),
        )],
        ViewNode::Brand { props, children } => vec![(
            children.as_slice(),
            NativeFlow::Inline,
            neutral_context && props.style.font.is_none(),
        )],
        ViewNode::Banner { props, children } => vec![(
            children.as_slice(),
            NativeFlow::Block,
            neutral_context && props.style.font.is_none(),
        )],
        _ => Vec::new(),
    }
}
