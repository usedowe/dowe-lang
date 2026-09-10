struct DevActivitySources {
    core: String,
    shards: Vec<DevActivityShard>,
}

struct DevActivityShard {
    file_name: String,
    content: String,
}

include!("activity_initial_output.rs");
include!("activity_lifecycle.rs");
include!("activity_media.rs");
include!("activity_window.rs");
include!("activity_routes.rs");
include!("activity_runtime.rs");

fn dev_activity_sources(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    font_families: &BTreeSet<FontFamily>,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    app_bundle: &str,
) -> DevActivitySources {
    let has_phones = routes.iter().any(|route| {
        dev_tree_has_phone(&route.layout_tree) || dev_tree_has_phone(&route.page_tree)
    });
    let has_dynamic_icons = routes.iter().any(|route| {
        dowe_components::tree_has_dynamic_icon(&route.layout_tree)
            || dowe_components::tree_has_dynamic_icon(&route.page_tree)
    });
    let (layouts, route_layouts) = reusable_dev_layouts(routes);
    let route_classes = routes
        .iter()
        .map(|route| dev_route_class_name(&route.route_path))
        .collect::<Vec<_>>();
    let mut output = dev_activity_initial_output(routes, design_config, environment, app_bundle);
    append_dev_activity_lifecycle(&mut output);
    append_dev_activity_media(&mut output);
    append_dev_activity_window(&mut output);
    append_dev_activity_routes(&mut output, routes, &route_classes, &route_layouts);
    append_dev_activity_runtime(&mut output, has_dynamic_icons, has_phones);
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
    output = output.replace(
        "__DOWE_JAVA_REACTIVE_RUNTIME__",
        &dev_java_reactive_runtime(),
    );
    output = output.replace(
        "__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__",
        SIDE_NAV_SUBMENU_ARROW_PATH,
    );
    let easing = dowe_components::VIEW_PAGE_TRANSITION_EASING;
    output = output
        .replace(
            "__DOWE_PAGE_TRANSITION_DURATION_MS__",
            &dowe_components::VIEW_PAGE_TRANSITION_DURATION_MS.to_string(),
        )
        .replace("__DOWE_PAGE_TRANSITION_X1__", &easing.0.to_string())
        .replace("__DOWE_PAGE_TRANSITION_Y1__", &easing.1.to_string())
        .replace("__DOWE_PAGE_TRANSITION_X2__", &easing.2.to_string())
        .replace("__DOWE_PAGE_TRANSITION_Y2__", &easing.3.to_string());

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
    if has_phones {
        shards.extend(dev_phone_flag_shards(app_bundle));
    }
    if has_dynamic_icons {
        shards.extend(dev_dynamic_icon_shards(app_bundle));
    }

    DevActivitySources {
        core: expose_dev_activity_members(output),
        shards,
    }
}

fn dev_safe_area_color_methods(routes: &[ViewRoute]) -> String {
    let mut output = String::from("    private void doweUpdateSafeAreaColors() {\n");
    for route in routes {
        let (top, bottom) = dowe_components::route_scaffold_safe_area_colors(route);
        output.push_str(&format!(
            "        if (\"{}\".equals(currentPath)) {{\n            doweSafeAreaTopColor = {};\n            doweSafeAreaBottomColor = {};\n            return;\n        }}\n",
            escape_java(&route.route_path),
            java_color(top),
            java_color(bottom),
        ));
    }
    output.push_str(
        "        doweSafeAreaTopColor = DOWE_BACKGROUND;\n        doweSafeAreaBottomColor = DOWE_BACKGROUND;\n    }\n\n",
    );
    output
}
