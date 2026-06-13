include!("generated_views/foundation.rs");
include!("generated_views/media_forms.rs");
include!("generated_views/data_code_svg.rs");
include!("generated_views/avatar_chat.rs");
include!("generated_views/empty_motion_text.rs");
include!("generated_views/rich_controls_map.rs");
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
        android_runtime_avatar_chat(),
        android_runtime_empty_motion_text(),
        android_runtime_rich_controls_map(),
        android_runtime_overlays(),
        android_runtime_navigation_drawer_layout(),
        android_runtime_input_helpers(),
        android_runtime_app_start(),
    ]
    .concat();
    output = output.replace(
        "__DOWE_DESIGN__",
        &android_design_block(design_config.default_theme()),
    );
    replace_android_font_support(&mut output, font_config, font_families);

    if routes.first().is_some() {
        output.push_str(
            r#"    val initialPath = if (DoweRoutes.paths.contains(startPath)) startPath else DoweRoutes.initialPath
    val initialFragment = startFragment?.takeIf { DoweRoutes.sections[initialPath]?.contains(it) == true }
    var currentEntry by remember { mutableStateOf(DoweRouteEntry(initialPath, initialFragment)) }
    var externalUrl by remember { mutableStateOf<String?>(null) }
    val backStack = remember { mutableStateListOf<DoweRouteEntry>() }
    val context = LocalContext.current
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
    LaunchedEffect(currentEntry.path, currentEntry.fragment, targetSection) {
        if (targetSection != null) {
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
            BoxWithConstraints(modifier = Modifier.fillMaxSize().safeDrawingPadding().verticalScroll(scrollState), contentAlignment = Alignment.TopStart) {
                when (currentEntry.path) {
"#,
        );
        for route in routes {
            output.push_str(&format!(
                "                    \"{}\" -> {}(maxWidth, sectionRegistry, ::navigate, ::goBack, ::openExternal)\n",
                route.route_path,
                compose_screen_name(&route.route_path)
            ));
        }
        if let Some(route) = routes.first() {
            output.push_str(&format!(
                "                    else -> {}(maxWidth, sectionRegistry, ::navigate, ::goBack, ::openExternal)\n",
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
            "fun {}(viewportWidth: Dp, sectionRegistry: DoweSectionRegistry, navigate: (String, String, String?) -> Unit, goBack: () -> Unit, openExternal: (String, String) -> Unit) {{\n",
            compose_screen_name(&route.route_path)
        ));
        let tree = compose_tree(&route.layout_tree, &route.page_tree);
        let reactive = compose_reactive_route(&tree);
        output.push_str(&format!(
            "    val activePath = \"{}\"\n    val state = remember {{ DoweReactiveState(initial = {}, actions = {}) }}\n    val actionScope = rememberCoroutineScope()\n",
            escape_kotlin(&route.route_path),
            reactive.initial,
            reactive.actions
        ));
        for id in &reactive.autoload {
            output.push_str(&format!(
                "    LaunchedEffect(\"{}\") {{ state.run(\"{}\") }}\n",
                escape_kotlin(id),
                escape_kotlin(id)
            ));
        }
        render_compose_node(&tree, 4, &mut output, font_config.default_family);
        output.push_str("}\n");
    }

    output
}
