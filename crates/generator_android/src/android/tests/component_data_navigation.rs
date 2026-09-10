#[test]
fn generates_fragment_aware_native_history_and_deep_links() {
    let output = generate_android(
        &[index_route_with_signup_link(), signup_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);

    assert!(dev.content.contains("private String currentPath = \"/\";"));
    assert!(dev
        .content
        .contains("private String currentFragment = null;"));
    assert!(dev
        .content
        .contains("private static final class DoweRouteEntry"));
    assert_eq!(dev.content.matches("final class DoweDevRoute").count(), 2);
    assert_eq!(
        dev.content
            .matches("static void render(DoweDevActivity this, ViewGroup root)")
            .count(),
        2
    );
    assert!(dev
        .content
        .contains("setOnClickListener(v -> doweNavigate(\"push\", \"/signup\", \"join\"))"));
    assert!(dev
        .content
        .contains("setOnClickListener(v -> doweNavigate(\"replace\", currentPath, \"hero\"))"));
    assert!(dev.content.contains("setOnClickListener(v -> doweBack())"));
    assert!(dev.content.contains(
            "setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT))"
        ));
    assert!(dev.content.contains("setAllCaps(false)"));
    assert!(dev
        .content
        .contains("backStack.add(new DoweRouteEntry(currentPath, currentFragment));"));
    assert!(dev.content.contains("private boolean doweCanSection"));
    assert!(dev.content.contains("data.getFragment()"));
    assert!(dev
        .content
        .contains("public void handleBack() {\n        doweBack();"));
    assert!(dev
        .content
        .contains("\"/\".equals(path) || \"/signup\".equals(path)"));
    assert!(dev.content.contains("doweApplyIntentRoute();"));
    assert!(dev.content.contains("doweScrollToFragment();"));
    assert!(dev.content.contains("if (path.equals(currentPath)) {"));
    assert!(dev.content.contains("currentFragment = resolvedFragment;"));
    assert!(dev.content.contains(
        "} else {\n                doweScrollToFragment();\n            }\n            return;"
    ));
    assert!(dev.content.contains(
        "if (currentFragment == null) {\n                scrollView.scrollTo(0, 0);\n            } else {\n                doweScrollToFragment();\n            }"
    ));
    assert!(dev
        .content
        .contains("target.getLocationInWindow(targetLocation);"));
    assert!(dev
        .content
        .contains("pinnedAppBar.getLocationInWindow(appBarLocation);"));
    assert!(dev.content.contains(
        "scrollView.smoothScrollTo(0, destination);"
    ));
    assert!(dev
        .content
        .contains("laidOutTarget.post(() -> doweRevealSection(laidOutTarget));"));
    assert!(dev
        .content
        .contains("if (laidOutTarget != null && laidOutTarget.isLaidOut())"));
    assert!(!dev.content.contains("doweTopRelativeToRoot"));
    assert!(dev.content.contains(r#"doweRegisterSection("hero", "#));

    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains("private data class DoweRouteEntry"));
    assert!(views
        .content
        .contains("fun navigate(operation: String, target: String, fragment: String?)"));
    assert!(views
        .content
        .contains(r#"{ navigate("push", "/signup", "join") }"#));
    assert!(views
        .content
        .contains(r#"{ navigate("replace", "", "hero") }"#));
    assert!(views.content.contains("BackHandler(enabled = true)"));
    assert!(views.content.contains("class DoweSectionRegistry"));
    assert!(views
        .content
        .contains("LaunchedEffect(currentEntry.path) {\n        scrollState.scrollTo(0)\n    }"));
    assert!(views.content.contains(
        "if (currentEntry.fragment == null) {\n            scrollState.scrollTo(0)\n        } else if (targetSection != null)"
    ));
    assert!(views
        .content
        .contains("scrollState.animateScrollTo(targetSection)"));
    assert!(views
        .content
        .contains("viewportWidth: Dp, scrollState: ScrollState"));
    assert!(views
        .content
        .contains(r#".doweSection(sectionRegistry, "hero")"#));

    let main_activity = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("MainActivity.kt"))
        .expect("main activity");
    assert!(main_activity
        .content
        .contains("override fun onNewIntent(intent: Intent)"));
    assert!(main_activity
        .content
        .contains("intent?.data?.fragment?.takeIf"));
    assert!(main_activity.content.contains("incomingRequest += 1"));
    assert!(views.content.contains("LaunchedEffect(navigationRequest)"));
}

