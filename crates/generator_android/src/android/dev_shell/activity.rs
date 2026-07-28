struct DevActivitySources {
    core: String,
    shards: Vec<DevActivityShard>,
}

struct DevActivityShard {
    file_name: String,
    content: String,
}

fn dev_activity_sources(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    font_families: &BTreeSet<FontFamily>,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    app_bundle: &str,
) -> DevActivitySources {
    let has_phone_fields = routes.iter().any(|route| {
        dev_tree_has_phone_field(&route.layout_tree) || dev_tree_has_phone_field(&route.page_tree)
    });
    let (layouts, route_layouts) = reusable_dev_layouts(routes);
    let route_classes = routes
        .iter()
        .map(|route| dev_route_class_name(&route.route_path))
        .collect::<Vec<_>>();
    let mut output = String::from(dev_activity_header());
    insert_dev_app_r_import(&mut output, app_bundle);
    output.push_str(&dev_design_constants(design_config));
    output.push_str("    private DoweVideoLayout dowePictureInPictureVideo;\n    private boolean dowePictureInPictureRestoreFullscreen;\n    private static final int DOWE_DROPZONE_REQUEST = 5107;\n    private String doweDropzoneKey;\n    private long doweDropzoneMaxSize = -1L;\n    private boolean doweDropzoneMultiple;\n");
    output.push_str(&format!(
        "    private final Activity doweActivity;\n    private Intent doweIntent;\n    private LinearLayout root;\n    private ScrollView scrollView;\n    private int viewportWidth;\n    private String currentPath = \"{}\";\n    private String currentFragment = null;\n    private String doweMountedPath = null;\n    private String doweMountedLayout = null;\n    private boolean externalOpen = false;\n    private Runnable doweDrawerNavigationClose = null;\n    private final ArrayList<DoweRouteEntry> backStack = new ArrayList<>();\n    private final HashMap<String, Object> doweState = new HashMap<>();\n    private final HashMap<String, Object> doweInitial = new HashMap<>();\n    private final HashMap<String, String[]> doweSignalMetadata = new HashMap<>();\n    private final HashMap<String, Object> doweGlobalState = new HashMap<>();\n    private final HashMap<String, String> doweGlobalStorage = new HashMap<>();\n    private final HashMap<String, DoweAction> doweActions = new HashMap<>();\n    private final HashMap<String, View> sectionViews = new HashMap<>();\n    private final HashSet<String> doweLoaded = new HashSet<>();\n\n",
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
        r#"    public DoweDevActivity(Activity activity) {
        super(activity, android.R.style.Theme_Material_Light_NoActionBar);
        doweActivity = activity;
    }

    public void mount(String preferredPath, Intent launchIntent) {
        doweIntent = launchIntent;
        String storedTheme = getSharedPreferences("dowe", 0).getString("theme-preference", DOWE_DEFAULT_THEME);
        doweApplyTheme(storedTheme == null ? DOWE_DEFAULT_THEME : storedTheme);
        doweConfigureWindow();
        FrameLayout background = new FrameLayout(this);
        background.setBackgroundColor(DOWE_BACKGROUND);
        background.setLayoutParams(new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        background.setClipChildren(false);
        background.setClipToPadding(false);
        root = new DoweLinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setGravity(Gravity.TOP | Gravity.START);
        root.setBackgroundColor(DOWE_BACKGROUND);
        root.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        scrollView = new ScrollView(this);
        scrollView.setFillViewport(true);
        scrollView.setClipChildren(false);
        scrollView.setClipToPadding(false);
        scrollView.addView(root, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        background.addView(scrollView, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        viewportWidth = getResources().getConfiguration().screenWidthDp;
        doweInitializeState();
        doweApplyIntentRoute();
        if (doweCanRoute(preferredPath)) {
            currentPath = preferredPath;
        }
        doweActivity.setContentView(background);
        doweApplySystemInsets(scrollView);
        renderCurrentRoute();
    }

    private void doweSetTheme(String name) {
        getSharedPreferences("dowe", 0).edit().putString("theme-preference", name).apply();
        doweApplyTheme(name);
        doweApplySystemBarAppearance();
        root.setBackgroundColor(DOWE_BACKGROUND);
        ((View) scrollView.getParent()).setBackgroundColor(DOWE_BACKGROUND);
        renderCurrentRoute(false);
    }

    public String currentPath() {
        return currentPath;
    }

    public void handleBack() {
        doweBack();
    }

    public void handleIntent(Intent intent) {
        doweIntent = intent;
        doweApplyIntentRoute();
        renderCurrentRoute();
    }

    public void handleActivityResult(int requestCode, int resultCode, Intent data) {
        if (requestCode != DOWE_DROPZONE_REQUEST || resultCode != Activity.RESULT_OK || data == null || doweDropzoneKey == null) {
            return;
        }
        ArrayList<Uri> uris = new ArrayList<>();
        if (data.getClipData() != null) {
            for (int index = 0; index < data.getClipData().getItemCount(); index++) {
                uris.add(data.getClipData().getItemAt(index).getUri());
            }
        } else if (data.getData() != null) {
            uris.add(data.getData());
        }
        ArrayList<String> files = new ArrayList<>();
        if (doweDropzoneMultiple) {
            Object previous = doweState.get(doweDropzoneKey);
            if (previous instanceof ArrayList<?>) {
                for (Object file : (ArrayList<?>) previous) {
                    files.add(String.valueOf(file));
                }
            }
        }
        for (Uri uri : uris) {
            long size = doweDropzoneFileSize(uri);
            if (doweDropzoneMaxSize >= 0L && size >= 0L && size > doweDropzoneMaxSize) {
                continue;
            }
            String label = doweDropzoneFileLabel(uri, size);
            if (!files.contains(label)) {
                files.add(label);
            }
            if (!doweDropzoneMultiple) {
                break;
            }
        }
        doweState.put(doweDropzoneKey, files);
        runOnUiThread(() -> renderCurrentRoute(false));
    }

    private void doweOpenDropzonePicker(String key, String accept, boolean multiple, long maxSize) {
        doweDropzoneKey = key;
        doweDropzoneMaxSize = maxSize;
        doweDropzoneMultiple = multiple;
        Intent picker = new Intent(Intent.ACTION_OPEN_DOCUMENT);
        picker.addCategory(Intent.CATEGORY_OPENABLE);
        String[] mimeTypes = doweDropzoneMimeTypes(accept);
        picker.setType(mimeTypes.length == 1 ? mimeTypes[0] : "*/*");
        picker.putExtra(Intent.EXTRA_MIME_TYPES, mimeTypes);
        picker.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, multiple);
        doweActivity.startActivityForResult(picker, DOWE_DROPZONE_REQUEST);
    }

    private String[] doweDropzoneMimeTypes(String accept) {
        if (accept == null || accept.trim().isEmpty()) {
            return new String[]{"*/*"};
        }
        String[] values = accept.split(",");
        ArrayList<String> types = new ArrayList<>();
        for (String value : values) {
            String trimmed = value.trim();
            if (!trimmed.isEmpty()) {
                types.add(trimmed);
            }
        }
        return types.isEmpty() ? new String[]{"*/*"} : types.toArray(new String[0]);
    }

    private long doweDropzoneFileSize(Uri uri) {
        android.database.Cursor cursor = getContentResolver().query(uri, new String[]{OpenableColumns.SIZE}, null, null, null);
        if (cursor == null) {
            return -1L;
        }
        try {
            if (cursor.moveToFirst() && !cursor.isNull(0)) {
                return cursor.getLong(0);
            }
        } finally {
            cursor.close();
        }
        return -1L;
    }

    private String doweDropzoneFileLabel(Uri uri, long size) {
        String name = uri.getLastPathSegment();
        android.database.Cursor cursor = getContentResolver().query(uri, new String[]{OpenableColumns.DISPLAY_NAME}, null, null, null);
        if (cursor != null) {
            try {
                if (cursor.moveToFirst() && !cursor.isNull(0)) {
                    name = cursor.getString(0);
                }
            } finally {
                cursor.close();
            }
        }
        String label = name == null || name.isEmpty() ? "Selected file" : name;
        return size < 0L ? label : label + " (" + doweDropzoneSizeLabel(size) + ")";
    }

    private String doweDropzoneSizeLabel(long size) {
        if (size >= 1024L * 1024L * 1024L) return (size / (1024L * 1024L * 1024L)) + " GB";
        if (size >= 1024L * 1024L) return (size / (1024L * 1024L)) + " MB";
        if (size >= 1024L) return (size / 1024L) + " KB";
        return size + " Bytes";
    }

    private String doweDropzoneText(String key, String placeholder) {
        Object value = doweState.get(key);
        if (!(value instanceof ArrayList<?>) || ((ArrayList<?>) value).isEmpty()) {
            return "Upload\\n" + placeholder;
        }
        StringBuilder text = new StringBuilder("Selected files");
        for (Object file : (ArrayList<?>) value) {
            text.append("\\n").append(String.valueOf(file));
        }
        return text.toString();
    }

    private Window getWindow() {
        return doweActivity.getWindow();
    }

    private Intent getIntent() {
        return doweIntent;
    }

    private void runOnUiThread(Runnable action) {
        doweActivity.runOnUiThread(action);
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
        doweApplySystemBarAppearance();
    }

    private void doweApplySystemBarAppearance() {
        boolean useDarkIcons = Color.luminance(DOWE_BACKGROUND) > 0.179f;
        if (Build.VERSION.SDK_INT >= 30) {
            int mask = android.view.WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS |
                android.view.WindowInsetsController.APPEARANCE_LIGHT_NAVIGATION_BARS;
            getWindow().getInsetsController().setSystemBarsAppearance(useDarkIcons ? mask : 0, mask);
        } else {
            int visibility = View.SYSTEM_UI_FLAG_LAYOUT_STABLE |
                View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN |
                View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION;
            if (useDarkIcons) {
                visibility |= View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR |
                    View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR;
            }
            getWindow().getDecorView().setSystemUiVisibility(visibility);
        }
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
            doweRelayoutPinnedAppBar();
            return insets;
        });
        view.requestApplyInsets();
    }

    private void renderCurrentRoute() {
        renderCurrentRoute(true);
    }

    private void renderCurrentRoute(boolean scrollToFragment) {
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
        View fixedFab = ((ViewGroup) scrollView.getParent()).findViewWithTag("dowe-fixed-fab");
        if (fixedFab != null) {
            ((ViewGroup) scrollView.getParent()).removeView(fixedFab);
        }
        sectionViews.clear();
        externalOpen = false;
"#,
    );

    for (index, (route, class_name)) in routes.iter().zip(&route_classes).enumerate() {
        let branch = if index == 0 { "if" } else { "else if" };
        output.push_str(&format!(
            "        {branch} (\"{}\".equals(currentPath)) {{\n            {class_name}.render(this, root);\n        }}\n",
            escape_java(&route.route_path)
        ));
    }

    if let Some((route, class_name)) = routes.first().zip(route_classes.first()) {
        output.push_str(&format!(
            "        else {{\n            currentPath = \"{}\";\n            {class_name}.render(this, root);\n        }}\n",
            escape_java(&route.route_path)
        ));
    }

    output.push_str(
        "        doweAutoload();\n        if (scrollToFragment) {\n            if (currentFragment == null) {\n                scrollView.scrollTo(0, 0);\n            } else {\n                doweScrollToFragment();\n            }\n        }\n    }\n\n",
    );

    output.push_str("    private void doweInitializeState() {\n");
    for class_name in &route_classes {
        output.push_str(&format!("        {class_name}.initialize(this);\n"));
    }
    output.push_str("    }\n\n    private void doweAutoload() {\n");
    for class_name in &route_classes {
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

    output.push_str(dev_activity_layout_widgets());
    output.push_str(dev_activity_flex_layout());
    output.push_str(dev_activity_grid_layout());
    output.push_str(dev_activity_svg_parser());
    output.push_str(dev_activity_svg_view());
    output.push_str(dev_activity_drawables_media());
    output.push_str(dev_activity_candlestick_runtime());
    output.push_str(dev_activity_chart_runtime());
    output.push_str(dev_activity_canvas_runtime());
    output.push_str(dev_activity_code_and_forms());
    if has_phone_fields {
        output.push_str(dev_activity_phone_flag_runtime());
    } else {
        output.push_str(dev_activity_empty_phone_flag_runtime());
    }
    output.push_str(dev_activity_responsive_helpers());
    output = output.replace(
        "__DOWE_ANDROID_DEV_FONT_SUPPORT__",
        &android_dev_font_support(font_families),
    );
    output = output.replace(
        "__DOWE_DEFAULT_FONT__",
        font_config
            .default_family
            .catalog_entry()
            .android_family_name,
    );
    output = output.replace("__DOWE_JAVA_REACTIVE_RUNTIME__", dev_java_reactive_runtime());

    let mut shards = routes
        .iter()
        .zip(&route_layouts)
        .zip(&route_classes)
        .map(|((route, layout_index), class_name)| DevActivityShard {
            file_name: format!("{class_name}.java"),
            content: dev_route_shard(route, *layout_index, class_name, app_bundle),
        })
        .collect::<Vec<_>>();
    shards.extend(
        layouts
            .iter()
            .enumerate()
            .map(|(index, layout)| DevActivityShard {
                file_name: format!("{}.java", dev_layout_class_name(index)),
                content: dev_layout_shard(layout, index, app_bundle),
            }),
    );
    if has_phone_fields {
        shards.extend(dev_phone_flag_shards(app_bundle));
    }

    DevActivitySources {
        core: expose_dev_activity_members(output),
        shards,
    }
}
