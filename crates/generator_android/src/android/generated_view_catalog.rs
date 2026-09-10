fn generated_views(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    font_families: &BTreeSet<FontFamily>,
    design_config: &DesignConfig,
) -> String {
    let tree_runtime = android_runtime_tree();
    let mut output = [
        android_runtime_foundation(),
        android_runtime_media_video(),
        android_runtime_media_device_iframe(),
        android_runtime_media_audio(),
        android_runtime_media_image(),
        android_runtime_media_accordion_carousel(),
        android_runtime_media_checkbox(),
        android_runtime_media_color(),
        android_runtime_media_date(),
        android_runtime_media_choice(),
        android_runtime_media_upload(),
        android_runtime_capture(),
        android_runtime_data_code_svg(),
        tree_runtime.as_str(),
        android_runtime_diagram(),
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
    output = output.replace("__DOWE_PHONE_COUNTRIES__", &compose_phone_country_catalog());
    output = output.replace(
        "__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__",
        SIDE_NAV_SUBMENU_ARROW_PATH,
    );
    if routes.iter().any(|route| {
        dowe_components::tree_has_dynamic_icon(&route.layout_tree)
            || dowe_components::tree_has_dynamic_icon(&route.page_tree)
    }) {
        output.insert_str(0, &android_dynamic_icon_runtime());
    }
    replace_android_font_support(&mut output, font_config, font_families);
    output.push_str(&compose_safe_area_color_methods(routes));

    if routes.first().is_some() {
        output.push_str(
            r#"    val context = LocalContext.current
    val pageMotionEnabled = dowePageMotionEnabled(context)
    val initialPath = if (DoweRoutes.paths.contains(startPath)) startPath else DoweRoutes.initialPath
    val initialFragment = startFragment?.takeIf { DoweRoutes.sections[initialPath]?.contains(it) == true }
    var currentEntry by remember { mutableStateOf(DoweRouteEntry(initialPath, initialFragment)) }
    var routeRevision by remember { mutableIntStateOf(0) }
    var pageEntranceSuppressed by remember { mutableStateOf(false) }
    var pageTransitionSequence by remember { mutableIntStateOf(0) }
    var externalUrl by remember { mutableStateOf<String?>(null) }
    val backStack = remember { mutableStateListOf<DoweRouteEntry>() }
    val scrollState = rememberScrollState()
    val sectionRegistry = remember(currentEntry.path) { DoweSectionRegistry() }
    val targetSection = currentEntry.fragment?.let { sectionRegistry.positions[it] }
    fun beginPageTransition() {
        pageEntranceSuppressed = true
        if (pageMotionEnabled) pageTransitionSequence += 1
    }
    fun navigate(operation: String, target: String, fragment: String?) {
        val path = target.ifEmpty { currentEntry.path }
        if (!DoweRoutes.paths.contains(path)) {
            return
        }
        val destination = DoweRouteEntry(path, fragment?.takeIf { DoweRoutes.sections[path]?.contains(it) == true })
        if (destination == currentEntry) {
            if (operation == "replace") {
                pageEntranceSuppressed = false
                routeRevision += 1
            }
            return
        }
        if (destination.path != currentEntry.path) beginPageTransition()
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
            val previous = backStack.last()
            if (previous.path != currentEntry.path) beginPageTransition()
            currentEntry = backStack.removeAt(backStack.lastIndex)
        } else if (currentEntry.path != DoweRoutes.initialPath || currentEntry.fragment != null) {
            if (currentEntry.path != DoweRoutes.initialPath) beginPageTransition()
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
    doweApplySystemBarIconAppearance(currentEntry.path)
    Box(modifier = Modifier.fillMaxSize().background(DoweDesign.background)) {
        Column(modifier = Modifier.fillMaxSize()) {
            Spacer(modifier = Modifier.fillMaxWidth().windowInsetsTopHeight(WindowInsets.safeDrawing).background(doweSafeAreaTopColor(currentEntry.path)))
            Spacer(modifier = Modifier.weight(1f))
            Spacer(modifier = Modifier.fillMaxWidth().windowInsetsBottomHeight(WindowInsets.safeDrawing).background(doweSafeAreaBottomColor(currentEntry.path)))
        }
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
                CompositionLocalProvider(LocalDowePageEntranceSuppressed provides pageEntranceSuppressed) {
                    AnimatedContent(
                        targetState = currentEntry.path,
                        transitionSpec = {
                            if (pageMotionEnabled && pageTransitionSequence > 0) {
                                fadeIn(animationSpec = tween(durationMillis = __DOWE_PAGE_TRANSITION_DURATION_MS, easing = CubicBezierEasing(__DOWE_PAGE_TRANSITION_X1_F, __DOWE_PAGE_TRANSITION_Y1_F, __DOWE_PAGE_TRANSITION_X2_F, __DOWE_PAGE_TRANSITION_Y2_F))) togetherWith ExitTransition.None
                            } else {
                                EnterTransition.None togetherWith ExitTransition.None
                            }
                        },
                        label = "dowe-page-transition"
                    ) { path ->
                        key(path, routeRevision) {
                            DoweRouteDispatcher(path, viewportWidth, scrollState, sectionRegistry, ::navigate, ::goBack, ::openExternal)
                        }
                    }
                }
"#,
        );
        output.push_str("            }\n        }\n        }\n    }\n");
    } else {
        output.push_str("    Column {\n    }\n");
    }

    output.push_str("}\n");
    output.push_str(&compose_reactive_runtime());
    output.push_str(&compose_route_dispatcher(routes));

    for (route_index, route) in routes.iter().enumerate() {
        output.push_str(&format!("\nprivate object DowePageShard{route_index} {{\n"));
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

    let easing = dowe_components::VIEW_PAGE_TRANSITION_EASING;
    let output = output
        .replace(
            "__DOWE_PAGE_TRANSITION_DURATION_MS",
            &dowe_components::VIEW_PAGE_TRANSITION_DURATION_MS.to_string(),
        )
        .replace("__DOWE_PAGE_TRANSITION_X1_F", &format!("{}f", easing.0))
        .replace("__DOWE_PAGE_TRANSITION_Y1_F", &format!("{}f", easing.1))
        .replace("__DOWE_PAGE_TRANSITION_X2_F", &format!("{}f", easing.2))
        .replace("__DOWE_PAGE_TRANSITION_Y2_F", &format!("{}f", easing.3));
    extract_compose_svg_path_helpers(output)
}

