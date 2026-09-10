fn render_html_with_inspector(
    node: &ViewNode,
    children_html: Option<&str>,
    inspector: Option<&ViewInspectorMap>,
) -> String {
    if inspector.is_none() {
        return render_html_with_context(node, children_html, &ReactiveRenderContext::default());
    }
    with_view_inspector(inspector, || {
        render_html_with_context(node, children_html, &ReactiveRenderContext::default())
    })
}

fn render_html_with_context(
    node: &ViewNode,
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    register_rendered_node_props(node, context);
    with_view_inspector_node(|| render_html_node_with_context(node, children_html, context))
}

pub fn render_report_for_desktop_routes(
    routes: &[dowe_components::ViewRoute],
) -> dowe_components::RenderReport {
    render_report_for_target(routes, dowe_components::RenderTarget::Desktop)
}

pub fn render_report_for_routes(
    routes: &[dowe_components::ViewRoute],
) -> dowe_components::RenderReport {
    render_report_for_target(routes, dowe_components::RenderTarget::Web)
}

fn render_report_for_target(
    routes: &[dowe_components::ViewRoute],
    target: dowe_components::RenderTarget,
) -> dowe_components::RenderReport {
    let report = dowe_components::RenderReport::from_routes(
        target,
        routes
            .iter()
            .map(|route| dowe_components::RouteRenderReport {
                route_path: route.route_path.clone(),
                accepted: Vec::new(),
                lowered: Vec::new(),
                present: Vec::new(),
                consumed: {
                    let mut entries = consumed_props_for_tree(&route.layout_tree);
                    entries.extend(consumed_props_for_tree(&route.page_tree));
                    entries
                },
                emitted: Vec::new(),
            })
            .collect(),
    );
    debug_assert!(report.validate().is_ok());
    report
}

pub fn render_report_for_tree(tree: &ViewNode) -> dowe_components::RenderReport {
    let report = dowe_components::RenderReport::new(
        dowe_components::RenderTarget::Web,
        consumed_props_for_tree(tree),
    );
    debug_assert!(report.validate().is_ok());
    report
}

pub fn consumed_props_for_tree(tree: &ViewNode) -> Vec<dowe_components::ConsumedProp> {
    let registry = ReactiveRenderContext::default();
    fn collect(node: &ViewNode, context: &ReactiveRenderContext) {
        register_rendered_node_props(node, context);
        for children in dowe_components::node_child_groups(node) {
            for child in children {
                collect(child, context);
            }
        }
    }
    collect(tree, &registry);
    registry.consumed_props.borrow().entries().to_vec()
}

pub fn consumed_props_for_node(node: &ViewNode) -> Vec<dowe_components::ConsumedProp> {
    let context = ReactiveRenderContext::default();
    register_rendered_node_props(node, &context);
    context.consumed_props.borrow().entries().to_vec()
}

