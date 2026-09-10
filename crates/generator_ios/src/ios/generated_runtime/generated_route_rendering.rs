fn swift_safe_area_color_methods(routes: &[ViewRoute]) -> String {
    let mut output = String::from(
        "    private func doweSafeAreaTopColor(_ path: String) -> Color {\n        switch path {\n",
    );
    for route in routes {
        let (top, _) = dowe_components::route_scaffold_safe_area_colors(route);
        output.push_str(&format!(
            "        case \"{}\": return {}\n",
            escape_swift(&route.route_path),
            color_ref(top)
        ));
    }
    output.push_str("        default: return DoweDesign.background\n        }\n    }\n\n");
    output.push_str(
        "    private func doweSafeAreaBottomColor(_ path: String) -> Color {\n        switch path {\n",
    );
    for route in routes {
        let (_, bottom) = dowe_components::route_scaffold_safe_area_colors(route);
        output.push_str(&format!(
            "        case \"{}\": return {}\n",
            escape_swift(&route.route_path),
            color_ref(bottom)
        ));
    }
    output.push_str("        default: return DoweDesign.background\n        }\n    }\n\n");
    output
}

fn generated_route_view(
    route: &ViewRoute,
    font_config: &FontConfig,
    layout_index: Option<usize>,
) -> String {
    let mut output = String::from("import SwiftUI\n\n");
    output.push_str(&format!(
        "struct {}: View {{\n",
        swift_view_name(&route.route_path)
    ));
    output.push_str("    let viewportWidth: CGFloat\n");
    output.push_str("    let viewportHeight: CGFloat\n");
    output.push_str("    let activeFragment: String?\n");
    output.push_str("    let navigate: (String, String, String?) -> Void\n");
    output.push_str("    let goBack: () -> Void\n");
    output.push_str("    let openExternal: (String, String) -> Void\n");
    output.push_str(&format!(
        "    private let activePath = \"{}\"\n",
        escape_swift(&route.route_path)
    ));
    let tree = if matches!(route.layout_tree, ViewNode::Children) {
        std::borrow::Cow::Borrowed(&route.page_tree)
    } else {
        std::borrow::Cow::Owned(compose_tree(&route.layout_tree, &route.page_tree))
    };
    let fixed_boxes = fixed_box_nodes(&tree);
    let fixed_fabs = fixed_fab_nodes(&tree);
    let reactive = swift_reactive_route(&tree);
    for index in 0..fixed_fabs.len() {
        output.push_str(&format!(
            "    @State private var doweFixedFabOpen{index} = false\n"
        ));
    }
    output.push_str(&format!(
        "    @StateObject private var state = DoweReactiveState(constants: {}, initial: {}, signals: {}, actions: {}, forms: {})\n",
        reactive.constants, reactive.initial, reactive.signals, reactive.actions, reactive.forms
    ));
    let route_tree = if layout_index.is_some() {
        &route.page_tree
    } else {
        &tree
    };
    let (route_nodes, route_context) = swift_route_body_nodes(route_tree);
    let route_branches = ios_route_branches(route_nodes);
    let route_expressions = route_branches
        .iter()
        .enumerate()
        .map(|(index, branch)| (swift_node_key(branch.node), format!("routeBranch{index}()")))
        .collect::<BTreeMap<_, _>>();
    let route_context = route_context.with_node_expressions(route_expressions.clone());
    let persistent_app_bar = swift_tree_has_persistent_scaffold_app_bar(&tree);
    output.push_str("    var body: some View {\n        ZStack(alignment: .topLeading) {\n        ScrollViewReader { proxy in\n");
    if !persistent_app_bar {
        output.push_str("            ScrollView {\n");
    }
    if let Some(layout_index) = layout_index {
        output.push_str(&format!(
            "                DoweLayout{layout_index}(\n                    viewportWidth: viewportWidth,\n                    viewportHeight: viewportHeight,\n                    activePath: activePath,\n                    state: state,\n                    navigate: navigate,\n                    goBack: goBack,\n                    openExternal: openExternal\n                ) {{\n"
        ));
        for index in 0..route_nodes.len() {
            output.push_str(&format!("                    routeSection{index}()\n"));
            if index == 0 {
                output.push_str("                    .id(\"__dowe_page_top\")\n");
            }
        }
        output.push_str("                }\n");
    } else {
        for index in 0..route_nodes.len() {
            output.push_str(&format!("                routeSection{index}()\n"));
            if index == 0 {
                output.push_str("                .id(\"__dowe_page_top\")\n");
            }
        }
    }
    if !persistent_app_bar {
        output.push_str("            }\n");
    }
    output.push_str("            .onAppear { doweScroll(proxy, activeFragment) }\n            .onChange(of: activeFragment) { _, value in doweScroll(proxy, value) }\n        }\n");
    for index in 0..fixed_boxes.len() {
        output.push_str(&format!("            fixedBox{index}()\n"));
    }
    for (index, node) in fixed_fabs.iter().enumerate() {
        if let Some(condition) = swift_fixed_fab_splash_condition(&tree, node) {
            output.push_str(&format!(
                "            if {condition} {{\n                fixedFab{index}()\n            }}\n"
            ));
        } else {
            output.push_str(&format!("            fixedFab{index}()\n"));
        }
    }
    output.push_str("            DoweGlobalToast(toast: state.toast, close: state.closeToast)\n");
    output.push_str("        }\n");
    let startup = reactive
        .init
        .iter()
        .chain(&reactive.autoload)
        .collect::<Vec<_>>();
    if !startup.is_empty() {
        output.push_str(&format!(
            "        .task {{ state.load([{}]) }}\n",
            startup
                .iter()
                .map(|value| format!("\"{}\"", escape_swift(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    output.push_str("        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n        .background(DoweDesign.background)\n        .foregroundStyle(DoweDesign.backgroundText)\n");
    output.push_str("        .environment(\\.doweTitleColor, DoweDesign.backgroundTitle)\n");
    output.push_str("        .onChange(of: state.redirectPath) { _, path in if let path { state.consumeRedirect(); navigate(\"replace\", path, nil) } }\n");
    output.push_str("    }\n\n");
    for (index, node) in route_nodes.iter().enumerate() {
        output.push_str(&format!(
            "    @ViewBuilder\n    private func routeSection{index}() -> some View {{\n"
        ));
        render_swift_node_in_flow(
            node,
            8,
            &mut output,
            NativeFlow::Block,
            None,
            font_config.default_family,
            &route_context,
        );
        output.push_str("    }\n\n");
    }
    for (index, branch) in route_branches.iter().enumerate() {
        output.push_str(&format!(
            "    @ViewBuilder\n    private func routeBranch{index}() -> some View {{\n"
        ));
        let branch_context = swift_reactive_context_for_node(route_tree, branch.node)
            .unwrap_or_else(|| route_context.clone())
            .with_node_expressions(route_expressions.clone())
            .without_node_expression(branch.node);
        render_swift_node_in_flow(
            branch.node,
            8,
            &mut output,
            branch.flow,
            None,
            font_config.default_family,
            &branch_context,
        );
        output.push_str("    }\n\n");
    }
    for (index, node) in fixed_boxes.iter().enumerate() {
        let ViewNode::Box { props, children } = node else {
            unreachable!();
        };
        output.push_str(&format!(
            "    @ViewBuilder\n    private func fixedBox{index}() -> some View {{\n"
        ));
        let box_context =
            swift_reactive_context_for_node(&tree, node).unwrap_or_else(|| route_context.clone());
        render_swift_fixed_box(
            props,
            children,
            8,
            &mut output,
            None,
            font_config.default_family,
            &box_context,
        );
        output.push_str("    }\n\n");
    }
    for (index, node) in fixed_fabs.iter().enumerate() {
        let ViewNode::Fab { props, actions } = node else {
            unreachable!();
        };
        output.push_str(&format!(
            "    @ViewBuilder\n    private func fixedFab{index}() -> some View {{\n"
        ));
        let fab_context =
            swift_reactive_context_for_node(&tree, node).unwrap_or_else(|| route_context.clone());
        render_swift_fab(
            props,
            actions,
            8,
            &mut output,
            &fab_context,
            Some(&format!("doweFixedFabOpen{index}")),
        );
        output.push_str("    }\n\n");
    }
    output.push_str("}\n");
    output
}

