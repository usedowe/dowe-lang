include!("generated_views/foundation.rs");
include!("generated_views/media_forms.rs");
include!("generated_views/data_code_svg.rs");
include!("generated_views/canvas.rs");
include!("generated_views/avatar_chat.rs");
include!("generated_views/empty_motion_text.rs");
include!("generated_views/rich_controls_map.rs");
include!("generated_views/anchored_popover.rs");
include!("generated_views/overlays.rs");
include!("generated_views/navigation_drawer_layout.rs");
include!("generated_views/input_helpers.rs");
include!("generated_views/app_start.rs");

fn generated_views(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    font_families: &BTreeSet<FontFamily>,
    design_config: &DesignConfig,
) -> String {
    let mut output = [
        android_runtime_foundation(),
        android_runtime_media_forms(),
        android_runtime_data_code_svg(),
        android_runtime_canvas(),
        android_runtime_avatar_chat(),
        android_runtime_empty_motion_text(),
        android_runtime_rich_controls_map(),
        android_runtime_anchored_popover(),
        android_runtime_overlays(),
        android_runtime_navigation_drawer_layout(),
        android_runtime_input_helpers(),
        android_runtime_app_start(),
    ]
    .concat();
    output = output.replace("__DOWE_DESIGN__", &android_design_block(design_config));
    replace_android_font_support(&mut output, font_config, font_families);

    if routes.first().is_some() {
        output.push_str(
            r#"    val context = LocalContext.current
    val initialPath = if (DoweRoutes.paths.contains(startPath)) startPath else DoweRoutes.initialPath
    val initialFragment = startFragment?.takeIf { DoweRoutes.sections[initialPath]?.contains(it) == true }
    var currentEntry by remember { mutableStateOf(DoweRouteEntry(initialPath, initialFragment)) }
    var externalUrl by remember { mutableStateOf<String?>(null) }
    val backStack = remember { mutableStateListOf<DoweRouteEntry>() }
    val scrollState = rememberScrollState()
    val sectionRegistry = remember(currentEntry.path) { DoweSectionRegistry() }
    val targetSection = currentEntry.fragment?.let { sectionRegistry.positions[it] }
    fun navigate(operation: String, target: String, fragment: String?) {
        val path = target.ifEmpty { currentEntry.path }
        if (!DoweRoutes.paths.contains(path)) {
            return
        }
        val destination = DoweRouteEntry(path, fragment?.takeIf { DoweRoutes.sections[path]?.contains(it) == true })
        if (destination == currentEntry) {
            return
        }
        if (operation == "replace") {
            currentEntry = destination
        } else {
            backStack.add(currentEntry)
            currentEntry = destination
        }
    }
    fun goBack() {
        if (externalUrl != null) {
            externalUrl = null
        } else if (backStack.isNotEmpty()) {
            currentEntry = backStack.removeAt(backStack.lastIndex)
        } else if (currentEntry.path != DoweRoutes.initialPath || currentEntry.fragment != null) {
            currentEntry = DoweRouteEntry(DoweRoutes.initialPath, null)
        }
    }
    fun openExternal(mode: String, target: String) {
        if (mode == "webview") {
            externalUrl = target
        } else {
            context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(target)))
        }
    }
    LaunchedEffect(navigationRequest) {
        navigate("replace", initialPath, initialFragment)
    }
    LaunchedEffect(currentEntry.path) {
        scrollState.scrollTo(0)
    }
    LaunchedEffect(currentEntry.fragment, targetSection) {
        if (currentEntry.fragment == null) {
            scrollState.scrollTo(0)
        } else if (targetSection != null) {
            scrollState.animateScrollTo(targetSection)
        }
    }
    BackHandler(enabled = true) {
        goBack()
    }
    Box(modifier = Modifier.fillMaxSize().background(DoweDesign.background)) {
        if (externalUrl != null) {
            AndroidView(
                modifier = Modifier.fillMaxSize().safeDrawingPadding(),
                factory = { WebView(it).apply { loadUrl(externalUrl ?: "") } },
                update = {
                    if (it.url != externalUrl) {
                        it.loadUrl(externalUrl ?: "")
                    }
                }
"#,
        );
        output.push_str(
            r#"            )
        } else {
            BoxWithConstraints(modifier = Modifier.fillMaxSize().safeDrawingPadding(), contentAlignment = Alignment.TopStart) {
                when (currentEntry.path) {
"#,
        );
        for route in routes {
            let scroll_modifier = if compose_tree_has_persistent_scaffold_app_bar(&compose_tree(
                &route.layout_tree,
                &route.page_tree,
            )) {
                ""
            } else {
                ".verticalScroll(scrollState)"
            };
            output.push_str(&format!(
                "                    \"{}\" -> Box(modifier = Modifier.fillMaxSize(){scroll_modifier}) {{ {}(maxWidth, scrollState, sectionRegistry, ::navigate, ::goBack, ::openExternal) }}\n",
                route.route_path,
                compose_screen_name(&route.route_path)
            ));
        }
        if let Some(route) = routes.first() {
            let scroll_modifier = if compose_tree_has_persistent_scaffold_app_bar(&compose_tree(
                &route.layout_tree,
                &route.page_tree,
            )) {
                ""
            } else {
                ".verticalScroll(scrollState)"
            };
            output.push_str(&format!(
                "                    else -> Box(modifier = Modifier.fillMaxSize(){scroll_modifier}) {{ {}(maxWidth, scrollState, sectionRegistry, ::navigate, ::goBack, ::openExternal) }}\n",
                compose_screen_name(&route.route_path)
            ));
        }
        output.push_str("                }\n            }\n        }\n    }\n");
    } else {
        output.push_str("    Column {\n    }\n");
    }

    output.push_str("}\n");
    output.push_str(compose_reactive_runtime());

    for route in routes {
        output.push('\n');
        output.push_str("@Composable\n");
        output.push_str(&format!(
            "fun {}(viewportWidth: Dp, scrollState: ScrollState, sectionRegistry: DoweSectionRegistry, navigate: (String, String, String?) -> Unit, goBack: () -> Unit, openExternal: (String, String) -> Unit) {{\n",
            compose_screen_name(&route.route_path)
        ));
        let tree = compose_tree(&route.layout_tree, &route.page_tree);
        let fixed_fabs = fixed_fab_nodes(&tree);
        let reactive = compose_reactive_route(&tree);
        output.push_str(&format!(
            "    val activePath = \"{}\"\n    val doweContext = LocalContext.current\n    val state = remember {{ DoweReactiveState(context = doweContext, constants = {}, initial = {}, signals = {}, actions = {}) }}\n    val actionScope = rememberCoroutineScope()\n",
            escape_kotlin(&route.route_path),
            reactive.constants,
            reactive.initial,
            reactive.signals,
            reactive.actions
        ));
        let startup = reactive
            .init
            .iter()
            .chain(&reactive.autoload)
            .map(|id| format!("\"{}\"", escape_kotlin(id)))
            .collect::<Vec<_>>();
        if !startup.is_empty() {
            output.push_str(&format!(
                "    LaunchedEffect(Unit) {{ state.load(listOf({})) }}\n",
                startup.join(", ")
            ));
        }
        for (index, node) in fixed_fabs.iter().enumerate() {
            let ViewNode::Fab { .. } = node else {
                unreachable!();
            };
            output.push_str(&format!(
                "    var doweFixedFabOpen{index} by remember {{ mutableStateOf(false) }}\n"
            ));
        }
        output.push_str("    Box(modifier = Modifier.fillMaxSize()) {\n");
        render_compose_node(&tree, 8, &mut output, font_config.default_family);
        for (index, node) in fixed_fabs.iter().enumerate() {
            let ViewNode::Fab { props, actions } = node else {
                unreachable!();
            };
            let fab_context = compose_reactive_context_for_node(&tree, node).unwrap_or_default();
            let splash_condition = compose_fixed_fab_splash_condition(&tree, node);
            if let Some(condition) = &splash_condition {
                output.push_str(&format!("        if ({condition}) {{\n"));
            }
            render_compose_fab(
                props,
                actions,
                if splash_condition.is_some() { 12 } else { 8 },
                &mut output,
                &fab_context,
                Some(&format!("doweFixedFabOpen{index}")),
            );
            if splash_condition.is_some() {
                output.push_str("        }\n");
            }
        }
        output.push_str("    }\n");
        output.push_str("}\n");
    }

    output
}

fn compose_fixed_fab_splash_condition(root: &ViewNode, target: &ViewNode) -> Option<String> {
    fn find(
        root: &ViewNode,
        node: &ViewNode,
        target: &ViewNode,
        conditions: &mut Vec<(String, bool)>,
    ) -> bool {
        if std::ptr::eq(node, target) {
            return true;
        }
        if let ViewNode::Splash {
            binding,
            content,
            children,
            ..
        } = node
        {
            let context = compose_reactive_context_for_node(root, node).unwrap_or_default();
            let binding = context.signal_path(binding);
            conditions.push((binding.clone(), false));
            if content
                .iter()
                .any(|child| find(root, child, target, conditions))
            {
                return true;
            }
            conditions.pop();
            conditions.push((binding, true));
            if children
                .iter()
                .any(|child| find(root, child, target, conditions))
            {
                return true;
            }
            conditions.pop();
            return false;
        }
        for group in node_child_groups(node) {
            for child in group {
                if find(root, child, target, conditions) {
                    return true;
                }
            }
        }
        false
    }

    let mut conditions = Vec::new();
    find(root, root, target, &mut conditions);
    (!conditions.is_empty()).then(|| {
        conditions
            .iter()
            .map(|(binding, active)| {
                if *active {
                    format!("state.bool(\"{}\")", escape_kotlin(binding))
                } else {
                    format!("!state.bool(\"{}\")", escape_kotlin(binding))
                }
            })
            .collect::<Vec<_>>()
            .join(" && ")
    })
}

fn compose_tree_has_persistent_scaffold_app_bar(node: &ViewNode) -> bool {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => content
            .iter()
            .chain(children)
            .any(compose_tree_has_persistent_scaffold_app_bar),
        ViewNode::Scope { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. } => children
            .iter()
            .any(compose_tree_has_persistent_scaffold_app_bar),
        ViewNode::Scaffold { app_bar, .. } => app_bar.iter().any(|node| {
            matches!(node, ViewNode::AppBar { props, .. } if props.position != BarPosition::Static)
        }),
        _ => false,
    }
}
