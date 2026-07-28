const DEV_ROUTE_METHOD_SOURCE_LIMIT: usize = 48 * 1024;

fn dev_route_shard(
    route: &ViewRoute,
    layout_index: Option<usize>,
    class_name: &str,
    app_bundle: &str,
) -> String {
    let shared_layout = layout_index.map(dev_layout_class_name);
    let mut page_helpers = String::new();
    let mut output = dev_shard_header(app_bundle);
    output.push_str(&format!(
        "@SuppressWarnings({{\"unchecked\", \"deprecation\"}})\nfinal class {class_name} {{\n    private {class_name}() {{}}\n\n    static void render(DoweDevActivity runtime, LinearLayout root) {{\n"
    ));
    if let Some(class_name) = &shared_layout {
        output.push_str(&format!(
            "        {class_name}.render(runtime, root, pageRoot -> renderPage(runtime, pageRoot));\n"
        ));
        output.push_str("    }\n\n    private static void renderPage(DoweDevActivity runtime, ViewGroup root) {\n");
        let rendering = dev_route_page_rendering(&route.page_tree);
        output.push_str(&rendering.body);
        page_helpers = rendering.helpers;
    } else {
        output.push_str("        int viewportWidth = runtime.viewportWidth;\n");
        let mut body = String::new();
        let mut counter = 0;
        let tree = compose_tree(&route.layout_tree, &route.page_tree);
        render_dev_android_node(
            &tree,
            "root",
            None,
            false,
            &mut counter,
            &mut body,
            None,
            None,
            &ComposeReactiveContext::default(),
            None,
        );
        output.push_str(&qualify_dev_shard_fragment(&body));
    }
    output.push_str("    }\n");
    output.push_str(&page_helpers);
    output.push_str("\n    static void initialize(DoweDevActivity runtime) {\n");
    if let Some(class_name) = &shared_layout {
        output.push_str(&format!("        {class_name}.initialize(runtime);\n"));
    }
    let composed_tree = compose_tree(&route.layout_tree, &route.page_tree);
    let startup_reactive = dev_reactive_route(&composed_tree);
    let startup = startup_reactive
        .init
        .iter()
        .chain(&startup_reactive.autoload)
        .map(|id| format!("\"{}\"", escape_java(id)))
        .collect::<Vec<_>>();
    let (layout_key, layout_ids, page_ids) = if let Some(class_name) = &shared_layout {
        let layout_reactive = dev_reactive_route(&route.layout_tree);
        let page_reactive = dev_reactive_route(&route.page_tree);
        (
            class_name.clone(),
            layout_reactive
                .init
                .iter()
                .chain(&layout_reactive.autoload)
                .map(|id| format!("\"{}\"", escape_java(id)))
                .collect::<Vec<_>>(),
            page_reactive
                .init
                .iter()
                .chain(&page_reactive.autoload)
                .map(|id| format!("\"{}\"", escape_java(id)))
                .collect::<Vec<_>>(),
        )
    } else {
        (
            route.route_path.clone(),
            Vec::new(),
            startup.clone(),
        )
    };
    let reactive = if shared_layout.is_some() {
        dev_reactive_route(&route.page_tree)
    } else {
        dev_reactive_route(&compose_tree(&route.layout_tree, &route.page_tree))
    };
    for metadata in reactive.metadata {
        output.push_str(&qualify_dev_shard_fragment(&format!("        {metadata}\n")));
    }
    for initial in reactive.initial {
        output.push_str(&qualify_dev_shard_fragment(&format!("        {initial}\n")));
    }
    for action in reactive.actions {
        output.push_str(&qualify_dev_shard_fragment(&format!("        {action}\n")));
    }
    output.push_str("    }\n\n    static void autoload(DoweDevActivity runtime) {\n");
    output.push_str(&format!(
        "        if (\"{}\".equals(runtime.currentPath)) {{\n            runtime.dowePrepareStartup(\"{}\", \"{}\", new String[] {{{}}}, new String[] {{{}}});\n",
        escape_java(&route.route_path),
        escape_java(&route.route_path),
        escape_java(&layout_key),
        layout_ids.join(", "),
        page_ids.join(", ")
    ));
    if !startup.is_empty() {
        output.push_str(&format!(
            "            runtime.doweRunStartup(new String[] {{{}}});\n",
            startup.join(", ")
        ));
    }
    output.push_str("        }\n");
    output.push_str("    }\n}\n");
    output
}

struct DevRoutePageRendering {
    body: String,
    helpers: String,
}

fn dev_route_page_rendering(page: &ViewNode) -> DevRoutePageRendering {
    let mut body = String::from("        int viewportWidth = runtime.viewportWidth;\n");
    let mut rendered = String::new();
    let mut counter = 0;
    render_dev_android_node(
        page,
        "root",
        None,
        false,
        &mut counter,
        &mut rendered,
        None,
        None,
        &ComposeReactiveContext::default(),
        None,
    );
    if rendered.len() < DEV_ROUTE_METHOD_SOURCE_LIMIT {
        body.push_str(&qualify_dev_shard_fragment(&rendered));
        return DevRoutePageRendering {
            body,
            helpers: String::new(),
        };
    }
    let ViewNode::Scope {
        constants,
        signals,
        actions,
        children,
    } = page
    else {
        body.push_str(&qualify_dev_shard_fragment(&rendered));
        return DevRoutePageRendering {
            body,
            helpers: String::new(),
        };
    };
    if children.len() < 2 {
        body.push_str(&qualify_dev_shard_fragment(&rendered));
        return DevRoutePageRendering {
            body,
            helpers: String::new(),
        };
    }

    body.clear();
    let context = ComposeReactiveContext::default().with_scope(constants, signals, actions);
    let mut helpers = String::new();
    let mut counter = 0;
    for (index, child) in children.iter().enumerate() {
        body.push_str(&format!("        renderPagePart{index}(runtime, root);\n"));
        helpers.push_str(&format!(
            "\n    private static void renderPagePart{index}(DoweDevActivity runtime, ViewGroup root) {{\n        int viewportWidth = runtime.viewportWidth;\n"
        ));
        let mut part = String::new();
        render_dev_android_node(
            child,
            "root",
            None,
            false,
            &mut counter,
            &mut part,
            None,
            None,
            &context,
            None,
        );
        helpers.push_str(&qualify_dev_shard_fragment(&part));
        helpers.push_str("    }\n");
    }
    DevRoutePageRendering { body, helpers }
}

fn dev_layout_shard(layout: &ViewNode, index: usize, app_bundle: &str) -> String {
    let class_name = dev_layout_class_name(index);
    let mut output = dev_shard_header(app_bundle);
    output.push_str(&format!(
        "@SuppressWarnings({{\"unchecked\", \"deprecation\"}})\nfinal class {class_name} {{\n    private {class_name}() {{}}\n\n    static void render(DoweDevActivity runtime, ViewGroup root, Consumer<ViewGroup> page) {{\n        int viewportWidth = runtime.viewportWidth;\n"
    ));
    let mut body = String::new();
    let mut counter = 0;
    render_dev_android_node(
        layout,
        "root",
        None,
        false,
        &mut counter,
        &mut body,
        None,
        None,
        &ComposeReactiveContext::default(),
        Some("page.accept"),
    );
    output.push_str(&qualify_dev_shard_fragment(&body));
    output.push_str("    }\n\n    static void initialize(DoweDevActivity runtime) {\n");
    let reactive = dev_reactive_route(layout);
    for metadata in reactive.metadata {
        output.push_str(&qualify_dev_shard_fragment(&format!(
            "        {metadata}\n"
        )));
    }
    for initial in reactive.initial {
        output.push_str(&qualify_dev_shard_fragment(&format!("        {initial}\n")));
    }
    for action in reactive.actions {
        output.push_str(&qualify_dev_shard_fragment(&format!("        {action}\n")));
    }
    output.push_str(
        "    }\n\n    static void autoload(DoweDevActivity runtime, String routePath) {\n",
    );
    for id in reactive.autoload {
        output.push_str(&format!(
            "        if (routePath.equals(runtime.currentPath) && runtime.doweLoaded.add(\"{}\")) {{\n            runtime.doweRunAction(\"{}\", null);\n        }}\n",
            escape_java(&id),
            escape_java(&id)
        ));
    }
    output.push_str("    }\n}\n");
    output
}

fn dev_shard_header(app_bundle: &str) -> String {
    let header = dev_activity_header();
    let declaration = header
        .find("@SuppressWarnings")
        .expect("Android dev activity declaration");
    let mut output = header[..declaration].to_string();
    insert_dev_app_r_import(&mut output, app_bundle);
    output
}

fn insert_dev_app_r_import(output: &mut String, app_bundle: &str) {
    if app_bundle != "dev.dowe.generated" {
        output.insert_str(
            "package dev.dowe.generated;\n\n".len(),
            &format!("import {}.R;\n", app_bundle),
        );
    }
}

fn expose_dev_activity_members(output: String) -> String {
    output
        .replace("\n    private ", "\n    ")
        .replace("\n        private ", "\n        ")
}

fn dev_route_class_name(route: &str) -> String {
    format!(
        "DoweDevRoute{}{:08x}",
        pascal_route(route),
        stable_dev_name_hash(route)
    )
}

fn dev_layout_class_name(index: usize) -> String {
    format!("DoweDevLayout{index}")
}

fn stable_dev_name_hash(value: &str) -> u32 {
    value.as_bytes().iter().fold(0x811c9dc5, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(0x01000193)
    })
}

fn qualify_dev_shard_fragment(fragment: &str) -> String {
    let chars = fragment.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(fragment.len() + fragment.len() / 8);
    let mut index = 0;
    let mut quote = None;
    let mut escaped = false;
    while index < chars.len() {
        let value = chars[index];
        if let Some(active_quote) = quote {
            output.push(value);
            if escaped {
                escaped = false;
            } else if value == '\\' {
                escaped = true;
            } else if value == active_quote {
                quote = None;
            }
            index += 1;
            continue;
        }
        if value == '"' || value == '\'' {
            quote = Some(value);
            output.push(value);
            index += 1;
            continue;
        }
        if value.is_ascii_alphabetic() || value == '_' || value == '$' {
            let start = index;
            index += 1;
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric()
                    || chars[index] == '_'
                    || chars[index] == '$')
            {
                index += 1;
            }
            let identifier = chars[start..index].iter().collect::<String>();
            let qualified = start > 0 && chars[start - 1] == '.';
            if qualified {
                output.push_str(&identifier);
            } else if identifier == "this" {
                output.push_str("runtime");
            } else if identifier.starts_with("dowe") || identifier.starts_with("DOWE_") {
                output.push_str("runtime.");
                output.push_str(&identifier);
            } else if identifier.starts_with("Dowe") && identifier != "DoweDevActivity" {
                output.push_str("DoweDevActivity.");
                output.push_str(&identifier);
            } else if matches!(
                identifier.as_str(),
                "currentPath"
                    | "currentFragment"
                    | "scrollView"
                    | "renderCurrentRoute"
                    | "getResources"
                    | "getString"
                    | "getSharedPreferences"
                    | "runOnUiThread"
            ) {
                output.push_str("runtime.");
                output.push_str(&identifier);
            } else {
                output.push_str(&identifier);
            }
            continue;
        }
        output.push(value);
        index += 1;
    }
    output
}
