fn generated_views(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    font_families: &BTreeSet<FontFamily>,
    design_config: &DesignConfig,
) -> String {
    let tree_runtime = swift_runtime_tree();
    let content_controls = swift_runtime_content_controls();
    let mut output = [
        swift_runtime_foundation(),
        swift_runtime_media(),
        swift_runtime_capture(),
        content_controls.as_str(),
        swift_runtime_data_display(),
        tree_runtime.as_str(),
        swift_runtime_diagram(),
        swift_runtime_canvas(),
        swift_runtime_avatar_chat(),
        swift_runtime_empty_motion_text(),
        swift_runtime_rich_controls_map(),
        swift_runtime_badge_chip_skeleton(),
        swift_runtime_anchored_popover(),
        swift_runtime_overlays(),
        swift_runtime_navigation_controls(),
        swift_runtime_drawer_runtime(),
        swift_runtime_input_controls(),
        swift_runtime_select_controls(),
        swift_runtime_combo_controls(),
        swift_runtime_drag_drop_controls(),
        swift_runtime_editor_image_controls(),
        swift_runtime_password_control(),
        swift_runtime_phone_control(),
        swift_runtime_pin_control(),
        swift_runtime_textarea_control(),
        swift_runtime_dropzone(),
        swift_runtime_svg_runtime(),
        swift_runtime_layout_helpers(),
        swift_runtime_route_helpers(),
        swift_runtime_root_view_start(),
    ]
    .concat();
    output = output.replace("__DOWE_DESIGN__", &swift_design_block(design_config));
    output = output.replace(
        "__DOWE_DEFAULT_FONT__",
        &swift_font_return(font_config.default_family),
    );
    output = output.replace("__DOWE_FONT_CASES__", &swift_font_cases(font_families));
    output = output.replace("__DOWE_FONT_SWITCH__", &swift_font_switch(font_families));
    output = output.replace(
        "__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__",
        SIDE_NAV_SUBMENU_ARROW_PATH,
    );
    if routes.first().is_some() {
        let page_transition_duration =
            dowe_components::VIEW_PAGE_TRANSITION_DURATION_SECONDS.to_string();
        let easing = dowe_components::VIEW_PAGE_TRANSITION_EASING;
        let page_transition_runtime = r#"        GeometryReader { geometry in
            ZStack {
                routeContent(currentEntry, viewportWidth: geometry.size.width, viewportHeight: geometry.size.height)
                    .id(routeRevision)
                    .transition(.asymmetric(insertion: .opacity, removal: .identity))
                    .environment(\.dowePageEntranceSuppressed, pageEntranceSuppressed)
            }
            .animation(reduceMotion || pageTransitionSequence == 0 ? nil : .timingCurve(__DOWE_PAGE_TRANSITION_X1__, __DOWE_PAGE_TRANSITION_Y1__, __DOWE_PAGE_TRANSITION_X2__, __DOWE_PAGE_TRANSITION_Y2__, duration: __DOWE_PAGE_TRANSITION_DURATION__), value: pageTransitionSequence)
                .frame(width: geometry.size.width, height: geometry.size.height, alignment: .topLeading)
                .clipped()
            DoweSafeAreaReporter { insets in
                if !doweInsetsEqual(safeAreaInsets, insets) {
                    safeAreaInsets = insets
                }
            }
            .frame(width: CGFloat(0), height: CGFloat(0))
            .allowsHitTesting(false)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(
            DoweSafeAreaBackground(
                topColor: doweSafeAreaTopColor(currentEntry.path),
                bottomColor: doweSafeAreaBottomColor(currentEntry.path),
                topInset: safeAreaInsets.top,
                bottomInset: safeAreaInsets.bottom
            )
            .ignoresSafeArea()
        )
        .foregroundStyle(DoweDesign.backgroundText)
        .simultaneousGesture(backSwipeGesture)
        .sheet(item: $externalUrl) { item in
            DoweExternalWebView(url: item.url)
        }
        .onOpenURL { url in
            applyDeepLink(url)
        }
        .environment(\.doweTitleColor, DoweDesign.backgroundTitle)
        .onChange(of: currentEntry.path) { _, path in
            routeChanged(path)
        }
"#
        .replace("__DOWE_PAGE_TRANSITION_DURATION__", &page_transition_duration)
        .replace("__DOWE_PAGE_TRANSITION_X1__", &easing.0.to_string())
        .replace("__DOWE_PAGE_TRANSITION_Y1__", &easing.1.to_string())
        .replace("__DOWE_PAGE_TRANSITION_X2__", &easing.2.to_string())
        .replace("__DOWE_PAGE_TRANSITION_Y2__", &easing.3.to_string());
        output.push_str(&page_transition_runtime);
    } else {
        output.push_str("        EmptyView()\n");
    }

    output.push_str(
        r#"    }

"#,
    );
    output.push_str(&swift_safe_area_color_methods(routes));
    output.push_str(
        r#"    private var currentEntry: DoweRouteEntry {
        navigationPath.last ?? rootEntry
    }

    private var backSwipeGesture: some Gesture {
        DragGesture(minimumDistance: CGFloat(16), coordinateSpace: .local)
            .onEnded { value in
                let horizontal = value.translation.width
                if value.startLocation.x <= CGFloat(28) && horizontal >= CGFloat(72) && abs(value.translation.height) < horizontal {
                    goBack()
                }
            }
    }

"#,
    );

    if let Some(route) = routes.first() {
        output.push_str(
            "    @ViewBuilder\n    private func routeContent(_ entry: DoweRouteEntry, viewportWidth: CGFloat, viewportHeight: CGFloat) -> some View {\n        switch entry.path {\n",
        );
        for route in routes {
            output.push_str(&format!(
                "        case \"{}\":\n            {}(viewportWidth: viewportWidth, viewportHeight: viewportHeight, activeFragment: entry.fragment, navigate: navigate, goBack: goBack, openExternal: openExternal)\n",
                route.route_path,
                swift_view_name(&route.route_path)
            ));
        }
        output.push_str(&format!(
            "        default:\n            {}(viewportWidth: viewportWidth, viewportHeight: viewportHeight, activeFragment: entry.fragment, navigate: navigate, goBack: goBack, openExternal: openExternal)\n",
            swift_view_name(&route.route_path)
        ));
        output.push_str("        }\n    }\n\n");
    }

    output.push_str(
        r#"    private func beginPageTransition() {
        pageEntranceSuppressed = true
        if !reduceMotion {
            pageTransitionSequence += 1
        }
    }

    private func navigate(_ operation: String, _ target: String, _ fragment: String?) {
        let path = target.isEmpty ? currentEntry.path : target
        guard DoweRoutes.paths.contains(path) else {
            return
        }
        let resolvedFragment = fragment.flatMap { value in
            DoweRoutes.sections[path]?.contains(value) == true ? value : nil
        }
        let destination = DoweRouteEntry(path: path, fragment: resolvedFragment)
        if destination == currentEntry {
            if operation == "replace" {
                pageEntranceSuppressed = false
                routeRevision += 1
            }
            return
        }
        if destination.path != currentEntry.path {
            beginPageTransition()
        }
        routeRevision += 1
        if operation == "replace" {
            if navigationPath.isEmpty {
                rootEntry = destination
            } else {
                navigationPath[navigationPath.count - 1] = destination
            }
        } else {
            navigationPath.append(destination)
        }
    }

    private func goBack() {
        if externalUrl != nil {
            externalUrl = nil
        } else if !navigationPath.isEmpty {
            let previous = navigationPath.last!
            if previous.path != currentEntry.path {
                beginPageTransition()
            }
            navigationPath.removeLast()
        } else if currentEntry.path != DoweRoutes.initialPath || currentEntry.fragment != nil {
            if currentEntry.path != DoweRoutes.initialPath {
                beginPageTransition()
            }
            rootEntry = DoweRouteEntry(path: DoweRoutes.initialPath, fragment: nil)
        }
    }

    private func openExternal(_ mode: String, _ target: String) {
        guard let url = URL(string: target) else {
            return
        }
        if mode == "webview" {
            externalUrl = DoweExternalUrl(url: url)
        } else {
            UIApplication.shared.open(url)
        }
    }

    private func applyDeepLink(_ url: URL) {
        let path = url.path.isEmpty ? DoweRoutes.initialPath : url.path
        if DoweRoutes.paths.contains(path) {
            navigate("replace", path, url.fragment)
        }
    }
}

func doweScroll(_ proxy: ScrollViewProxy, _ fragment: String?) {
    guard let fragment else {
        proxy.scrollTo("__dowe_page_top", anchor: .top)
        return
    }
    withAnimation(.easeInOut(duration: 0.28)) {
        proxy.scrollTo(fragment, anchor: .top)
    }
}

func doweInsetsEqual(_ lhs: EdgeInsets, _ rhs: EdgeInsets) -> Bool {
    lhs.top == rhs.top && lhs.leading == rhs.leading && lhs.bottom == rhs.bottom && lhs.trailing == rhs.trailing
}

"#,
    );
    output.push_str(&swift_reactive_runtime());

    output
}

