fn dev_activity_navigation(first_path: &str) -> String {
    format!(
        r#"    private void doweNavigate(String operation, String target, String fragment) {{
        String path = target.isEmpty() ? currentPath : target;
        if (!doweCanRoute(path)) {{
            return;
        }}
        String resolvedFragment = doweCanSection(path, fragment) ? fragment : null;
        doweCloseDrawerForNavigation();
        if (path.equals(currentPath) && Objects.equals(resolvedFragment, currentFragment)) {{
            return;
        }}
        if ("replace".equals(operation)) {{
            currentPath = path;
            currentFragment = resolvedFragment;
        }} else {{
            backStack.add(new DoweRouteEntry(currentPath, currentFragment));
            currentPath = path;
            currentFragment = resolvedFragment;
        }}
        renderCurrentRoute();
    }}

    private void doweBack() {{
        doweCloseDrawerForNavigation();
        if (externalOpen) {{
            renderCurrentRoute();
        }} else if (!backStack.isEmpty()) {{
            DoweRouteEntry previous = backStack.remove(backStack.size() - 1);
            currentPath = previous.path;
            currentFragment = previous.fragment;
            renderCurrentRoute();
        }} else if (!currentPath.equals("{}") || currentFragment != null) {{
            currentPath = "{}";
            currentFragment = null;
            renderCurrentRoute();
        }}
    }}

    private void doweOpenExternal(String mode, String target) {{
        doweCloseDrawerForNavigation();
        if ("webview".equals(mode)) {{
            root.removeAllViews();
            externalOpen = true;
            WebView webView = new WebView(this);
            webView.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
            webView.loadUrl(target);
            root.addView(webView);
        }} else {{
            startActivity(new Intent(Intent.ACTION_VIEW, Uri.parse(target)));
        }}
    }}

    private void doweCloseDrawerForNavigation() {{
        if (doweDrawerNavigationClose != null) {{
            Runnable close = doweDrawerNavigationClose;
            doweDrawerNavigationClose = null;
            close.run();
        }}
    }}

    private void doweApplyIntentRoute() {{
        Uri data = getIntent() == null ? null : getIntent().getData();
        String path = data == null ? null : data.getPath();
        if (path == null || path.isEmpty()) {{
            path = "/";
        }}
        if (doweCanRoute(path)) {{
            currentPath = path;
            String fragment = data == null ? null : data.getFragment();
            currentFragment = doweCanSection(path, fragment) ? fragment : null;
            backStack.clear();
        }}
    }}

    private boolean doweCanRoute(String path) {{
"#,
        escape_java(first_path),
        escape_java(first_path)
    )
}
