fn append_dev_activity_routes(
    output: &mut String,
    routes: &[ViewRoute],
    route_classes: &[String],
    route_layouts: &[Option<usize>],
    layout_state_keys: &[Option<usize>],
) {
    output.push_str(
    r#"    private void renderCurrentRoute() {
        renderCurrentRoute(true);
    }

    private void renderCurrentRoute(boolean scrollToFragment) {
        doweOverlayRender++;
        dowePrepareCurrentRoute();
        dowePageContainerView = null;
        root.removeAllViews();
        View pinnedAppBar = ((ViewGroup) scrollView.getParent()).findViewWithTag("dowe-pinned-appbar");
        if (pinnedAppBar != null) {
            ((ViewGroup) scrollView.getParent()).removeView(pinnedAppBar);
        }
        View pinnedAppBarSafeArea = ((ViewGroup) scrollView.getParent()).findViewWithTag("dowe-pinned-appbar-safe-area");
        if (pinnedAppBarSafeArea != null) {
            ((ViewGroup) scrollView.getParent()).removeView(pinnedAppBarSafeArea);
        }
        View pinnedAppBarBottomSafeArea = ((ViewGroup) scrollView.getParent()).findViewWithTag("dowe-pinned-appbar-bottom-safe-area");
        if (pinnedAppBarBottomSafeArea != null) {
            ((ViewGroup) scrollView.getParent()).removeView(pinnedAppBarBottomSafeArea);
        }
        View pinnedAppBarDivider = ((ViewGroup) scrollView.getParent()).findViewWithTag("dowe-pinned-appbar-divider");
        if (pinnedAppBarDivider != null) {
            ((ViewGroup) scrollView.getParent()).removeView(pinnedAppBarDivider);
        }
        if (dowePinnedAppBarAnimator != null) {
            dowePinnedAppBarAnimator.cancel();
            dowePinnedAppBarAnimator = null;
        }
        dowePinnedAppBarDockOnScroll = false;
        dowePinnedAppBarPlaceholder = null;
        dowePinnedAppBarDivider = null;
        View fixedFab = ((ViewGroup) scrollView.getParent()).findViewWithTag("dowe-fixed-fab");
        if (fixedFab != null) {
            ((ViewGroup) scrollView.getParent()).removeView(fixedFab);
        }
        sectionViews.clear();
        externalOpen = false;
"#,
    );

    for (index, ((route, class_name), layout_index)) in routes
        .iter()
        .zip(route_classes)
        .zip(route_layouts)
        .enumerate()
    {
        let branch = if index == 0 { "if" } else { "else if" };
        let render_root = if layout_index.is_some() {
            "root"
        } else {
            "doweCreatePageContainer(root)"
        };
        output.push_str(&format!(
            "        {branch} (\"{}\".equals(currentPath)) {{\n            {class_name}.render(this, {render_root});\n        }}\n",
            escape_java(&route.route_path)
        ));
    }

    if let Some(((route, class_name), layout_index)) = routes
        .first()
        .zip(route_classes.first())
        .zip(route_layouts.first())
    {
        let render_root = if layout_index.is_some() {
            "root"
        } else {
            "doweCreatePageContainer(root)"
        };
        output.push_str(&format!(
            "        else {{\n            currentPath = \"{}\";\n            {class_name}.render(this, {render_root});\n        }}\n",
            escape_java(&route.route_path)
        ));
    }

    output.push_str(
        "        if (doweActiveOverlay != null && doweActiveOverlay.isShowing() && doweOverlayClaimed != doweOverlayRender) {\n            doweActiveOverlay.dismiss();\n        }\n        doweAutoload();\n        if (scrollToFragment) {\n            if (currentFragment == null) {\n                scrollView.scrollTo(0, 0);\n            } else {\n                doweScrollToFragment();\n            }\n        }\n        doweUpdateSafeAreaColors();\n        doweApplySystemBarAppearance();\n        doweApplySafeAreaColors();\n    }\n\n",
    );

    output.push_str("    private void doweInitializeState() {\n");
    for class_name in route_classes {
        output.push_str(&format!("        {class_name}.initialize(this);\n"));
    }
    output.push_str("    }\n\n    private void dowePrepareCurrentRoute() {\n");
    for (index, route) in routes.iter().enumerate() {
        let layout_key = layout_state_keys
            .get(index)
            .and_then(|key| *key)
            .map(|key| format!("layout:{key}"))
            .unwrap_or_else(|| route.route_path.clone());
        let layout_signals = dev_signal_ids(&route.layout_tree)
            .iter()
            .map(|id| format!("\"{}\"", escape_java(id)))
            .collect::<Vec<_>>()
            .join(", ");
        let page_signals = dev_signal_ids(&route.page_tree)
            .iter()
            .map(|id| format!("\"{}\"", escape_java(id)))
            .collect::<Vec<_>>()
            .join(", ");
        let startup = startup_ids(&dev_reactive_route(&compose_tree(
            &route.layout_tree,
            &route.page_tree,
        )));
        let layout_startup = startup_ids(&dev_reactive_route(&route.layout_tree));
        let layout_startup_set = layout_startup.iter().collect::<BTreeSet<_>>();
        let page_startup = startup
            .iter()
            .filter(|id| !layout_startup_set.contains(id))
            .map(|id| format!("\"{}\"", escape_java(id)))
            .collect::<Vec<_>>()
            .join(", ");
        let layout_startup = layout_startup
            .iter()
            .map(|id| format!("\"{}\"", escape_java(id)))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "        if (\"{}\".equals(currentPath)) {{\n            dowePrepareState(\"{}\", \"{}\", new String[] {{{}}}, new String[] {{{}}}, new String[] {{{}}}, new String[] {{{}}});\n            return;\n        }}\n",
            escape_java(&route.route_path),
            escape_java(&route.route_path),
            escape_java(&layout_key),
            layout_signals,
            page_signals,
            layout_startup,
            page_startup,
        ));
    }
    output.push_str("    }\n\n    private void doweAutoload() {\n");
    for class_name in route_classes {
        output.push_str(&format!("        {class_name}.autoload(this);\n"));
    }
    output.push_str("    }\n\n");

    output.push_str(&dev_activity_navigation(routes_first_path(routes)));

    if routes.is_empty() {
        output.push_str("        return false;\n");
    } else {
        let route_checks = routes
            .iter()
            .map(|route| format!("\"{}\".equals(path)", escape_java(&route.route_path)))
            .collect::<Vec<_>>()
            .join(" || ");
        output.push_str(&format!("        return {route_checks};\n"));
    }

    output.push_str(
        "    }\n\n    private boolean doweCanSection(String path, String fragment) {\n        if (fragment == null) {\n            return true;\n        }\n",
    );
    for route in routes {
        let section_checks = route
            .sections
            .iter()
            .map(|section| format!("\"{}\".equals(fragment)", escape_java(&section.id)))
            .collect::<Vec<_>>()
            .join(" || ");
        output.push_str(&format!(
            "        if (\"{}\".equals(path)) {{\n            return {};\n        }}\n",
            escape_java(&route.route_path),
            if section_checks.is_empty() {
                "false".to_string()
            } else {
                section_checks
            }
        ));
    }
    output.push_str("        return false;\n    }\n\n");
}

fn startup_ids(reactive: &DevReactiveRoute) -> Vec<String> {
    reactive
        .init
        .iter()
        .chain(&reactive.autoload)
        .cloned()
        .collect()
}
