fn android_dynamic_icon_catalog(routes: &[ViewRoute]) -> Vec<(String, String)> {
    dowe_components::dynamic_icon_catalog_for_trees(
        routes
            .iter()
            .flat_map(|route| [&route.layout_tree, &route.page_tree]),
    )
    .expect("computed icon names must be validated before generation")
}

fn android_dynamic_icon_runtime(routes: &[ViewRoute]) -> String {
    let entries = android_dynamic_icon_catalog(routes)
        .iter()
        .map(|(name, payload)| {
            format!(
                "    {} to {},",
                compose_string_literal(&name),
                compose_string_literal(&payload)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "\nprivate val DoweDynamicIconCatalog = mapOf(\n{entries}\n)\n\n@Composable\nprivate fun DoweDynamicIcon(name: String, fallback: String, modifier: Modifier, color: Color, animated: Boolean = false) {{\n    DoweRuntimeSvg(payload = DoweDynamicIconCatalog[name] ?: DoweDynamicIconCatalog[fallback] ?: \"\", modifier = modifier, color = color, animated = animated)\n}}\n"
    )
}

fn compose_safe_area_color_methods(routes: &[ViewRoute]) -> String {
    let mut output =
        String::from("\nfun doweSafeAreaTopColor(path: String): Color = when (path) {\n");
    for route in routes {
        let (top, _) = dowe_components::route_scaffold_safe_area_colors(route);
        output.push_str(&format!(
            "    \"{}\" -> {}\n",
            escape_kotlin(&route.route_path),
            color_ref(top)
        ));
    }
    output.push_str("    else -> DoweDesign.background\n}\n\n");
    output.push_str("fun doweSafeAreaBottomColor(path: String): Color = when (path) {\n");
    for route in routes {
        let (_, bottom) = dowe_components::route_scaffold_safe_area_colors(route);
        output.push_str(&format!(
            "    \"{}\" -> {}\n",
            escape_kotlin(&route.route_path),
            color_ref(bottom)
        ));
    }
    output.push_str("    else -> DoweDesign.background\n}\n\n");
    output.push_str(
        r#"@Composable
fun doweApplySystemBarIconAppearance(path: String) {
    val activity = doweActivity(LocalContext.current)
    val useDarkStatusBarIcons = doweSafeAreaTopColor(path).luminance() > 0.179f
    val useDarkNavigationBarIcons = doweSafeAreaBottomColor(path).luminance() > 0.179f
    SideEffect {
        activity?.let { currentActivity ->
            WindowCompat.getInsetsController(currentActivity.window, currentActivity.window.decorView).apply {
                isAppearanceLightStatusBars = useDarkStatusBarIcons
                isAppearanceLightNavigationBars = useDarkNavigationBarIcons
            }
        }
    }
}

private fun doweActivity(context: Context): Activity? {
    var current = context
    while (current is ContextWrapper && current !is Activity) current = current.baseContext
    return current as? Activity
}
"#,
    );
    output
}

fn compose_route_dispatcher(routes: &[ViewRoute]) -> String {
    const ROUTES_PER_GROUP: usize = 24;
    if routes.is_empty() {
        return String::new();
    }
    let parameters = "path: String, viewportWidth: Dp, scrollState: ScrollState, sectionRegistry: DoweSectionRegistry, navigate: (String, String, String?) -> Unit, goBack: () -> Unit, openExternal: (String, String) -> Unit";
    let arguments =
        "path, viewportWidth, scrollState, sectionRegistry, navigate, goBack, openExternal";
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
