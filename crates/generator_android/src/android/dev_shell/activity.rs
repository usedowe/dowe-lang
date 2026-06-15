fn dev_activity(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    app_bundle: &str,
) -> String {
    let mut output = String::from(dev_activity_header());
    if app_bundle != "dev.dowe.generated" {
        output.insert_str(
            "package dev.dowe.generated;\n\n".len(),
            &format!("import {}.R;\n", app_bundle),
        );
    }
    output.push_str(&dev_design_constants(design_config.default_theme()));
    output.push_str(&format!(
        "    private LinearLayout root;\n    private ScrollView scrollView;\n    private int viewportWidth;\n    private String currentPath = \"{}\";\n    private String currentFragment = null;\n    private boolean externalOpen = false;\n    private final ArrayList<DoweRouteEntry> backStack = new ArrayList<>();\n    private final HashMap<String, Object> doweState = new HashMap<>();\n    private final HashMap<String, Object> doweInitial = new HashMap<>();\n    private final HashMap<String, DoweAction> doweActions = new HashMap<>();\n    private final HashMap<String, View> sectionViews = new HashMap<>();\n    private final HashSet<String> doweLoaded = new HashSet<>();\n\n",
        escape_java(routes_first_path(routes))
    ));
    output.push_str(
        r#"    private static final class DoweRouteEntry {
        private final String path;
        private final String fragment;

        private DoweRouteEntry(String path, String fragment) {
            this.path = path;
            this.fragment = fragment;
        }
    }

"#,
    );
    output.push_str("    private static final class DoweEnvironment {\n");
    for (name, value) in environment {
        output.push_str(&format!(
            "        private static final String {} = \"{}\";\n",
            name,
            escape_java(value)
        ));
    }
    if !environment.iter().any(|(name, _)| name == "BACKEND_URL") {
        output.push_str("        private static final String BACKEND_URL = \"\";\n");
    }
    output.push_str("    }\n\n");
    output.push_str(
        r#"    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        doweConfigureWindow();
        FrameLayout background = new FrameLayout(this);
        background.setBackgroundColor(DOWE_BACKGROUND);
        background.setLayoutParams(new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setGravity(Gravity.TOP | Gravity.START);
        root.setBackgroundColor(DOWE_BACKGROUND);
        root.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        scrollView = new ScrollView(this);
        scrollView.setFillViewport(true);
        scrollView.addView(root);
        background.addView(scrollView, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        viewportWidth = getResources().getConfiguration().screenWidthDp;
        doweInitializeState();
        doweApplyIntentRoute();
        doweRegisterBackHandler();
        setContentView(background);
        doweApplySystemInsets(scrollView);
        renderCurrentRoute();
    }

    private void doweConfigureWindow() {
        getWindow().setStatusBarColor(Color.TRANSPARENT);
        getWindow().setNavigationBarColor(Color.TRANSPARENT);
        if (Build.VERSION.SDK_INT >= 29) {
            getWindow().setNavigationBarContrastEnforced(false);
        }
        if (Build.VERSION.SDK_INT >= 30) {
            getWindow().setDecorFitsSystemWindows(false);
        }
        getWindow().getDecorView().setSystemUiVisibility(
            View.SYSTEM_UI_FLAG_LAYOUT_STABLE |
            View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN |
            View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION |
            View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR |
            View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR
        );
    }

    private void doweApplySystemInsets(View view) {
        view.setOnApplyWindowInsetsListener((target, insets) -> {
            if (Build.VERSION.SDK_INT >= 30) {
                Insets safe = insets.getInsets(WindowInsets.Type.systemBars() | WindowInsets.Type.displayCutout());
                target.setPadding(safe.left, safe.top, safe.right, safe.bottom);
            } else {
                target.setPadding(
                    insets.getSystemWindowInsetLeft(),
                    insets.getSystemWindowInsetTop(),
                    insets.getSystemWindowInsetRight(),
                    insets.getSystemWindowInsetBottom()
                );
            }
            return insets;
        });
        view.requestApplyInsets();
    }

    @Override
    protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        setIntent(intent);
        doweApplyIntentRoute();
        renderCurrentRoute();
    }

    @Override
    public void onBackPressed() {
        doweBack();
    }

    private void doweRegisterBackHandler() {
        if (Build.VERSION.SDK_INT >= 33) {
            getOnBackInvokedDispatcher().registerOnBackInvokedCallback(
                android.window.OnBackInvokedDispatcher.PRIORITY_DEFAULT,
                this::doweBack
            );
        }
    }

    private void renderCurrentRoute() {
        renderCurrentRoute(true);
    }

    private void renderCurrentRoute(boolean scrollToFragment) {
        root.removeAllViews();
        sectionViews.clear();
        externalOpen = false;
"#,
    );

    for (index, route) in routes.iter().enumerate() {
        let branch = if index == 0 { "if" } else { "else if" };
        output.push_str(&format!(
            "        {branch} (\"{}\".equals(currentPath)) {{\n            {}(root);\n        }}\n",
            escape_java(&route.route_path),
            dev_route_method_name(&route.route_path)
        ));
    }

    if let Some(route) = routes.first() {
        output.push_str(&format!(
            "        else {{\n            currentPath = \"{}\";\n            {}(root);\n        }}\n",
            escape_java(&route.route_path),
            dev_route_method_name(&route.route_path)
        ));
    }

    output.push_str(
        "        doweAutoload();\n        if (scrollToFragment) {\n            doweScrollToFragment();\n        }\n    }\n\n",
    );

    let (layouts, route_layouts) = reusable_dev_layouts(routes);
    for (route, layout_index) in routes.iter().zip(route_layouts) {
        let context = ComposeReactiveContext::default();
        let mut counter = 0;
        if let Some(layout_index) = layout_index {
            let page_method = dev_route_page_method_name(&route.route_path);
            output.push_str(&format!(
                "    private void {}(LinearLayout root) {{\n        renderLayout{layout_index}(root, this::{page_method});\n    }}\n\n",
                dev_route_method_name(&route.route_path),
            ));
            output.push_str(&format!(
                "    private void {page_method}(ViewGroup root) {{\n        int viewportWidth = this.viewportWidth;\n"
            ));
            render_dev_android_node(
                &route.page_tree,
                "root",
                None,
                false,
                &mut counter,
                &mut output,
                None,
                None,
                &context,
                None,
            );
            output.push_str("    }\n\n");
        } else {
            output.push_str(&format!(
                "    private void {}(LinearLayout root) {{\n        int viewportWidth = this.viewportWidth;\n",
                dev_route_method_name(&route.route_path)
            ));
            let tree = compose_tree(&route.layout_tree, &route.page_tree);
            render_dev_android_node(
                &tree,
                "root",
                None,
                false,
                &mut counter,
                &mut output,
                None,
                None,
                &context,
                None,
            );
            output.push_str("    }\n\n");
        }
    }
    for (index, layout) in layouts.into_iter().enumerate() {
        output.push_str(&format!(
            "    private void renderLayout{index}(ViewGroup root, Consumer<ViewGroup> page) {{\n        int viewportWidth = this.viewportWidth;\n"
        ));
        let context = ComposeReactiveContext::default();
        let mut counter = 0;
        render_dev_android_node(
            layout,
            "root",
            None,
            false,
            &mut counter,
            &mut output,
            None,
            None,
            &context,
            Some("page.accept"),
        );
        output.push_str("    }\n\n");
    }

    output.push_str("    private void doweInitializeState() {\n");
    for route in routes {
        let tree = compose_tree(&route.layout_tree, &route.page_tree);
        let reactive = dev_reactive_route(&tree);
        for initial in reactive.initial {
            output.push_str(&format!("        {initial}\n"));
        }
        for action in reactive.actions {
            output.push_str(&format!("        {action}\n"));
        }
    }
    output.push_str("    }\n\n    private void doweAutoload() {\n");
    for route in routes {
        let tree = compose_tree(&route.layout_tree, &route.page_tree);
        let reactive = dev_reactive_route(&tree);
        for id in reactive.autoload {
            output.push_str(&format!(
                "        if (\"{}\".equals(currentPath) && doweLoaded.add(\"{}\")) {{\n            doweRunAction(\"{}\", null);\n        }}\n",
                escape_java(&route.route_path),
                escape_java(&id),
                escape_java(&id)
            ));
        }
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

    output.push_str(dev_activity_layout_widgets());
    output.push_str(dev_activity_flex_layout());
    output.push_str(dev_activity_grid_layout());
    output.push_str(dev_activity_svg_parser());
    output.push_str(dev_activity_svg_view());
    output.push_str(dev_activity_drawables_media());
    output.push_str(dev_activity_candlestick_runtime());
    output.push_str(dev_activity_chart_runtime());
    output.push_str(dev_activity_code_and_forms());
    output.push_str(dev_activity_responsive_helpers());

    output = output.replace(
        "__DOWE_DEFAULT_FONT__",
        font_config
            .default_family
            .catalog_entry()
            .android_family_name,
    );
    output = output.replace(
        "__DOWE_JAVA_REACTIVE_RUNTIME__",
        dev_java_reactive_runtime(),
    );
    output
}
