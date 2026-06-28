include!("generated_runtime/foundation.rs");
include!("generated_runtime/media.rs");
include!("generated_runtime/content_controls.rs");
include!("generated_runtime/data_display.rs");
include!("generated_runtime/avatar_chat.rs");
include!("generated_runtime/empty_motion_text.rs");
include!("generated_runtime/rich_controls_map.rs");
include!("generated_runtime/badge_chip_skeleton.rs");
include!("generated_runtime/overlays.rs");
include!("generated_runtime/navigation_controls.rs");
include!("generated_runtime/drawer_runtime.rs");
include!("generated_runtime/input_select_runtime.rs");
include!("generated_runtime/svg_runtime.rs");
include!("generated_runtime/layout_helpers.rs");
include!("generated_runtime/route_helpers.rs");
include!("generated_runtime/root_view_start.rs");

fn generated_views(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    font_families: &BTreeSet<FontFamily>,
    design_config: &DesignConfig,
) -> String {
    let mut output = [
        swift_runtime_foundation(),
        swift_runtime_media(),
        swift_runtime_content_controls(),
        swift_runtime_data_display(),
        swift_runtime_avatar_chat(),
        swift_runtime_empty_motion_text(),
        swift_runtime_rich_controls_map(),
        swift_runtime_badge_chip_skeleton(),
        swift_runtime_overlays(),
        swift_runtime_navigation_controls(),
        swift_runtime_drawer_runtime(),
        swift_runtime_input_select_runtime(),
        swift_runtime_svg_runtime(),
        swift_runtime_layout_helpers(),
        swift_runtime_route_helpers(),
        swift_runtime_root_view_start(),
    ]
    .concat();
    output = output.replace(
        "__DOWE_DESIGN__",
        &swift_design_block(design_config.default_theme()),
    );
    output = output.replace(
        "__DOWE_DEFAULT_FONT__",
        &swift_font_return(font_config.default_family),
    );
    output = output.replace("__DOWE_FONT_CASES__", &swift_font_cases(font_families));
    output = output.replace("__DOWE_FONT_SWITCH__", &swift_font_switch(font_families));

    if routes.first().is_some() {
        output.push_str("        GeometryReader { geometry in\n            routeContent(currentEntry, viewportWidth: doweSafeAreaWidth(geometry, safeAreaInsets))\n                .frame(width: doweSafeAreaWidth(geometry, safeAreaInsets), height: doweSafeAreaHeight(geometry, safeAreaInsets), alignment: .topLeading)\n                .clipped()\n                .offset(x: safeAreaInsets.leading, y: safeAreaInsets.top)\n            DoweSafeAreaReporter { insets in\n                if !doweInsetsEqual(safeAreaInsets, insets) {\n                    safeAreaInsets = insets\n                }\n            }\n            .frame(width: CGFloat(0), height: CGFloat(0))\n            .allowsHitTesting(false)\n        }\n        .ignoresSafeArea()\n        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n        .background(DoweDesign.background.ignoresSafeArea())\n        .foregroundStyle(DoweDesign.onBackground)\n        .simultaneousGesture(backSwipeGesture)\n        .sheet(item: $externalUrl) { item in\n            DoweExternalWebView(url: item.url)\n        }\n        .onOpenURL { url in\n            applyDeepLink(url)\n        }\n");
    } else {
        output.push_str("        EmptyView()\n");
    }

    output.push_str(
        r#"    }

    private var currentEntry: DoweRouteEntry {
        navigationPath.last ?? rootEntry
    }

    private var backSwipeGesture: some Gesture {
        DragGesture(minimumDistance: CGFloat(16), coordinateSpace: .local)
            .onEnded { value in
                let horizontal = value.translation.width
                if value.startLocation.x <= CGFloat(28) && horizontal >= CGFloat(72) && abs(value.translation.height) < horizontal {
                    goBack()
                }
            }
    }

"#,
    );

    if let Some(route) = routes.first() {
        output.push_str(
            "    @ViewBuilder\n    private func routeContent(_ entry: DoweRouteEntry, viewportWidth: CGFloat) -> some View {\n        switch entry.path {\n",
        );
        for route in routes {
            output.push_str(&format!(
                "        case \"{}\":\n            {}(viewportWidth: viewportWidth, activeFragment: entry.fragment, navigate: navigate, goBack: goBack, openExternal: openExternal)\n",
                route.route_path,
                swift_view_name(&route.route_path)
            ));
        }
        output.push_str(&format!(
            "        default:\n            {}(viewportWidth: viewportWidth, activeFragment: entry.fragment, navigate: navigate, goBack: goBack, openExternal: openExternal)\n",
            swift_view_name(&route.route_path)
        ));
        output.push_str("        }\n    }\n\n");
    }

    output.push_str(
        r#"    private func navigate(_ operation: String, _ target: String, _ fragment: String?) {
        let path = target.isEmpty ? currentEntry.path : target
        guard DoweRoutes.paths.contains(path) else {
            return
        }
        let resolvedFragment = fragment.flatMap { value in
            DoweRoutes.sections[path]?.contains(value) == true ? value : nil
        }
        let destination = DoweRouteEntry(path: path, fragment: resolvedFragment)
        guard destination != currentEntry else {
            return
        }
        if operation == "replace" {
            if navigationPath.isEmpty {
                rootEntry = destination
            } else {
                navigationPath[navigationPath.count - 1] = destination
            }
        } else {
            navigationPath.append(destination)
        }
    }

    private func goBack() {
        if externalUrl != nil {
            externalUrl = nil
        } else if !navigationPath.isEmpty {
            navigationPath.removeLast()
        } else if currentEntry.path != DoweRoutes.initialPath || currentEntry.fragment != nil {
            rootEntry = DoweRouteEntry(path: DoweRoutes.initialPath, fragment: nil)
        }
    }

    private func openExternal(_ mode: String, _ target: String) {
        guard let url = URL(string: target) else {
            return
        }
        if mode == "webview" {
            externalUrl = DoweExternalUrl(url: url)
        } else {
            UIApplication.shared.open(url)
        }
    }

    private func applyDeepLink(_ url: URL) {
        let path = url.path.isEmpty ? DoweRoutes.initialPath : url.path
        if DoweRoutes.paths.contains(path) {
            navigate("replace", path, url.fragment)
        }
    }
}

func doweScroll(_ proxy: ScrollViewProxy, _ fragment: String?) {
    guard let fragment else {
        return
    }
    withAnimation(.easeInOut(duration: 0.28)) {
        proxy.scrollTo(fragment, anchor: .top)
    }
}

func doweSafeAreaWidth(_ geometry: GeometryProxy, _ insets: EdgeInsets) -> CGFloat {
    max(CGFloat(0), geometry.size.width - insets.leading - insets.trailing)
}

func doweSafeAreaHeight(_ geometry: GeometryProxy, _ insets: EdgeInsets) -> CGFloat {
    max(CGFloat(0), geometry.size.height - insets.top - insets.bottom)
}

func doweInsetsEqual(_ lhs: EdgeInsets, _ rhs: EdgeInsets) -> Bool {
    lhs.top == rhs.top && lhs.leading == rhs.leading && lhs.bottom == rhs.bottom && lhs.trailing == rhs.trailing
}

"#,
    );
    output.push_str(swift_reactive_runtime());

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
    output.push_str("    let activeFragment: String?\n");
    output.push_str("    let navigate: (String, String, String?) -> Void\n");
    output.push_str("    let goBack: () -> Void\n");
    output.push_str("    let openExternal: (String, String) -> Void\n");
    output.push_str(&format!(
        "    private let activePath = \"{}\"\n",
        escape_swift(&route.route_path)
    ));
    let tree = compose_tree(&route.layout_tree, &route.page_tree);
    let reactive = swift_reactive_route(&tree);
    output.push_str(&format!(
        "    @StateObject private var state = DoweReactiveState(initial: {}, actions: {})\n",
        reactive.initial, reactive.actions
    ));
    let route_tree = if layout_index.is_some() {
        &route.page_tree
    } else {
        &tree
    };
    let (route_nodes, route_context) = swift_route_body_nodes(route_tree);
    output.push_str(
        "    var body: some View {\n        ScrollViewReader { proxy in\n            ScrollView {\n",
    );
    if let Some(layout_index) = layout_index {
        output.push_str(&format!(
            "                DoweLayout{layout_index}(\n                    viewportWidth: viewportWidth,\n                    activePath: activePath,\n                    state: state,\n                    navigate: navigate,\n                    goBack: goBack,\n                    openExternal: openExternal\n                ) {{\n"
        ));
        for index in 0..route_nodes.len() {
            output.push_str(&format!("                    routeSection{index}()\n"));
        }
        output.push_str("                }\n");
    } else {
        for index in 0..route_nodes.len() {
            output.push_str(&format!("                routeSection{index}()\n"));
        }
    }
    output.push_str("            }\n            .onAppear { doweScroll(proxy, activeFragment) }\n            .onChange(of: activeFragment) { _, value in doweScroll(proxy, value) }\n        }\n");
    if !reactive.autoload.is_empty() {
        output.push_str(&format!(
            "        .task {{ state.load([{}]) }}\n",
            reactive
                .autoload
                .iter()
                .map(|value| format!("\"{}\"", escape_swift(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    output.push_str("        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n        .background(DoweDesign.background)\n        .foregroundStyle(DoweDesign.onBackground)\n");
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
    output.push_str("}\n");
    output
}

fn swift_route_body_nodes(tree: &ViewNode) -> (&[ViewNode], SwiftReactiveContext) {
    let context = SwiftReactiveContext::default();
    match tree {
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => (children.as_slice(), context.with_scope(signals, actions)),
        _ => (std::slice::from_ref(tree), context),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NativeFlow {
    Block,
    GridItem,
    Inline,
}

impl NativeFlow {
    fn is_block(self) -> bool {
        matches!(self, Self::Block | Self::GridItem)
    }

    fn is_grid_item(self) -> bool {
        matches!(self, Self::GridItem)
    }
}
