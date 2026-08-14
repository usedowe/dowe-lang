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
    output = output.replace(
        "__DOWE_PHONE_COUNTRIES__",
        &compose_phone_country_catalog(),
    );
    output = output.replace(
        "__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__",
        SIDE_NAV_SUBMENU_ARROW_PATH,
    );
    replace_android_font_support(&mut output, font_config, font_families);

    if routes.first().is_some() {
        output.push_str(
            r#"    val context = LocalContext.current
    val initialPath = if (DoweRoutes.paths.contains(startPath)) startPath else DoweRoutes.initialPath
    val initialFragment = startFragment?.takeIf { DoweRoutes.sections[initialPath]?.contains(it) == true }
    var currentEntry by remember { mutableStateOf(DoweRouteEntry(initialPath, initialFragment)) }
    var routeRevision by remember { mutableIntStateOf(0) }
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
            if (operation == "replace") routeRevision += 1
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
        CompositionLocalProvider(LocalContentColor provides DoweDesign.backgroundText, LocalDoweTitleColor provides DoweDesign.backgroundTitle) {
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
                val viewportWidth = maxWidth
                key(currentEntry.path, routeRevision) {
                    DoweRouteDispatcher(currentEntry.path, viewportWidth, scrollState, sectionRegistry, ::navigate, ::goBack, ::openExternal)
                }
"#,
        );
    output.push_str("            }\n        }\n        }\n    }\n");
    } else {
        output.push_str("    Column {\n    }\n");
    }

    output.push_str("}\n");
    output.push_str(compose_reactive_runtime());
    output.push_str(&compose_route_dispatcher(routes));

    for (route_index, route) in routes.iter().enumerate() {
        output.push_str(&format!(
            "\nprivate object DowePageShard{route_index} {{\n"
        ));
        output.push('\n');
        output.push_str("@Composable\n");
        output.push_str(&format!(
            "fun {}(viewportWidth: Dp, scrollState: ScrollState, sectionRegistry: DoweSectionRegistry, navigate: (String, String, String?) -> Unit, goBack: () -> Unit, openExternal: (String, String) -> Unit) {{\n",
            compose_screen_name(&route.route_path)
        ));
        let tree = compose_tree(&route.layout_tree, &route.page_tree);
        let fixed_boxes = fixed_box_nodes(&tree);
        let fixed_fabs = fixed_fab_nodes(&tree);
        let reactive = compose_reactive_route(&tree);
        output.push_str(&format!(
            "    val activePath = \"{}\"\n    val doweContext = LocalContext.current\n    val state = remember {{ DoweReactiveState(context = doweContext, constants = {}, initial = {}, signals = {}, actions = {}, forms = {}) }}\n    val actionScope = rememberCoroutineScope()\n",
            escape_kotlin(&route.route_path),
            reactive.constants,
            reactive.initial,
            reactive.signals,
            reactive.actions,
            reactive.forms
        ));
        output.push_str("    LaunchedEffect(state.redirectPath) { state.redirectPath?.let { path -> state.consumeRedirect(); navigate(\"replace\", path, null) } }\n");
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
        for node in &fixed_boxes {
            let ViewNode::Box { props, children } = node else {
                unreachable!();
            };
            let box_context = compose_reactive_context_for_node(&tree, node).unwrap_or_default();
            render_compose_fixed_box(
                props,
                children,
                8,
                &mut output,
                None,
                font_config.default_family,
                &box_context,
            );
        }
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
        output.push_str("        DoweGlobalToast(toast = state.toast, close = state::closeToast, viewportWidth = viewportWidth)\n");
        output.push_str("    }\n");
        output.push_str("}\n");
        output.push_str("}\n");
    }

    extract_compose_svg_path_helpers(output)
}

fn compose_route_dispatcher(routes: &[ViewRoute]) -> String {
    const ROUTES_PER_GROUP: usize = 24;
    if routes.is_empty() {
        return String::new();
    }
    let parameters = "path: String, viewportWidth: Dp, scrollState: ScrollState, sectionRegistry: DoweSectionRegistry, navigate: (String, String, String?) -> Unit, goBack: () -> Unit, openExternal: (String, String) -> Unit";
    let arguments = "path, viewportWidth, scrollState, sectionRegistry, navigate, goBack, openExternal";
    let mut output = format!(
        "\n@Composable\nprivate fun DoweRouteDispatcher({parameters}) {{\n    when ((DoweRoutes.paths.indexOf(path).coerceAtLeast(0)) / {ROUTES_PER_GROUP}) {{\n"
    );
    for index in 0..routes.len().div_ceil(ROUTES_PER_GROUP) {
        output.push_str(&format!(
            "        {index} -> DoweRouteGroup{index}({arguments})\n"
        ));
    }
    output.push_str(&format!(
        "        else -> DoweRouteGroup0({arguments})\n    }}\n}}\n"
    ));

    for (group_index, group) in routes.chunks(ROUTES_PER_GROUP).enumerate() {
        output.push_str(&format!(
            "\n@Composable\nprivate fun DoweRouteGroup{group_index}({parameters}) {{\n    when (path) {{\n"
        ));
        for route in group {
            let route_index = routes
                .iter()
                .position(|candidate| candidate.route_path == route.route_path)
                .expect("generated Android route");
            let scroll_modifier = if compose_tree_has_persistent_scaffold_app_bar(&compose_tree(
                &route.layout_tree,
                &route.page_tree,
            )) {
                ""
            } else {
                ".verticalScroll(scrollState)"
            };
            output.push_str(&format!(
                "        \"{}\" -> Box(modifier = Modifier.fillMaxSize(){scroll_modifier}) {{ {}(viewportWidth, scrollState, sectionRegistry, navigate, goBack, openExternal) }}\n",
                escape_kotlin(&route.route_path),
                format!(
                    "DowePageShard{}.{}",
                    route_index,
                    compose_screen_name(&route.route_path)
                )
            ));
        }
        let fallback = &group[0];
        let scroll_modifier = if compose_tree_has_persistent_scaffold_app_bar(&compose_tree(
            &fallback.layout_tree,
            &fallback.page_tree,
        )) {
            ""
        } else {
            ".verticalScroll(scrollState)"
        };
        output.push_str(&format!(
            "        else -> Box(modifier = Modifier.fillMaxSize(){scroll_modifier}) {{ {}(viewportWidth, scrollState, sectionRegistry, navigate, goBack, openExternal) }}\n    }}\n}}\n",
            format!(
                "DowePageShard{}.{}",
                routes
                    .iter()
                    .position(|candidate| candidate.route_path == fallback.route_path)
                    .expect("generated Android fallback route"),
                compose_screen_name(&fallback.route_path)
            )
        ));
    }
    output
}

fn extract_compose_svg_path_helpers(output: String) -> String {
    const PREFIX: &str = "listOf(DoweSvgPath(";
    let mut result = String::with_capacity(output.len());
    let mut helpers = Vec::new();
    let mut helper_indexes = BTreeMap::new();
    let mut cursor = 0;

    while let Some(relative_start) = output[cursor..].find(PREFIX) {
        let start = cursor + relative_start;
        result.push_str(&output[cursor..start]);
        let Some(end) = kotlin_call_end(&output, start) else {
            result.push_str(&output[start..]);
            return result;
        };
        let expression = &output[start..end];
        let index = if let Some(index) = helper_indexes.get(expression) {
            *index
        } else {
            let index = helpers.len();
            helper_indexes.insert(expression.to_string(), index);
            helpers.push(expression.to_string());
            index
        };
        result.push_str(&format!(
            "DoweSvgPathShard{}.doweSvgPaths{index}()",
            index / 32
        ));
        cursor = end;
    }
    result.push_str(&output[cursor..]);
    for (index, expression) in helpers.iter().enumerate() {
        if index % 32 == 0 {
            result.push_str(&format!(
                "\nprivate object DoweSvgPathShard{} {{\n",
                index / 32
            ));
        }
        result.push_str(&format!(
            "\nfun doweSvgPaths{index}(): List<DoweSvgPath> = {expression}\n"
        ));
        if index % 32 == 31 || index + 1 == helpers.len() {
            result.push_str("}\n");
        }
    }
    result
}

fn kotlin_call_end(source: &str, start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;

    for (relative, character) in source[start..].char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
            continue;
        }
        match character {
            '"' => quoted = true,
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(start + relative + character.len_utf8());
                }
            }
            _ => {}
        }
    }
    None
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
