#[test]
fn every_builtin_view_component_and_prop_has_editor_documentation() {
    let components = [
        "Box",
        "Section",
        "Flex",
        "Grid",
        "Input",
        "Select",
        "Option",
        "Code",
        "Video",
        "Canvas",
        "Candlestick",
        "Diagram",
        "ArcChart",
        "AreaChart",
        "BarChart",
        "LineChart",
        "PieChart",
        "Table",
        "Divider",
        "Button",
        "Brand",
        "Banner",
        "ToggleTheme",
        "SelectTheme",
        "Fab",
        "fabAction",
        "Slider",
        "Dropzone",
        "ComboBox",
        "comboOption",
        "CsvField",
        "csvColumn",
        "DragDrop",
        "dragGroup",
        "dragItem",
        "Editor",
        "ImageCropper",
        "Password",
        "Phone",
        "Pin",
        "Textarea",
        "Alert",
        "Svg",
        "Path",
        "AppBar",
        "Footer",
        "BottomBar",
        "NavMenu",
        "SideNav",
        "RailNav",
        "Sidebar",
        "Scaffold",
        "Splash",
        "Drawer",
        "Avatar",
        "Badge",
        "Chip",
        "Skeleton",
        "Modal",
        "AlertDialog",
        "Tooltip",
        "Toast",
        "Dropdown",
        "Command",
        "AvatarGroup",
        "ChatBox",
        "Empty",
        "Marquee",
        "TypeWriter",
        "RichText",
        "Record",
        "ToggleGroup",
        "Collapsible",
        "Countdown",
        "Map",
        "Audio",
        "Image",
        "Accordion",
        "Carousel",
        "Checkbox",
        "Color",
        "Date",
        "DateRange",
        "RadioGroup",
        "RadioCard",
        "Toggle",
        "Card",
        "Tabs",
        "tab",
        "Stepper",
        "step",
        "Title",
        "Text",
    ];
    let root = Path::new("/project");
    let base_document = LanguageDocument {
        path: Path::new("/project/pages/docs.dowe").to_path_buf(),
        source: String::new(),
    };
    let base_completions = complete_document(root, &base_document, 1, 1);

    for component in components {
        assert!(
            base_completions.iter().any(|completion| {
                completion.label == component && completion.documentation.is_some()
            }),
            "missing component completion documentation for {component}"
        );
        let source = format!("page docsPage\n  {component} \n");
        let document = LanguageDocument {
            path: Path::new("/project/pages/docs.dowe").to_path_buf(),
            source,
        };
        let hover = hover_at(root, &document, 2, 3).expect("component hover");
        assert!(
            hover.contains(&format!("`{component}`")),
            "{component}: {hover}"
        );
        assert!(hover.contains("Accepted props"), "{component}: {hover}");

        let completions =
            complete_document(root, &document, 2, format!("  {component} ").len() + 1);
        assert!(!completions.is_empty(), "missing props for {component}");
        for completion in completions {
            let documentation = completion
                .documentation
                .as_deref()
                .expect("prop completion documentation");
            assert!(
                documentation.contains(component),
                "{component}.{}",
                completion.label
            );

            let source = format!("page docsPage\n  {component} {}:true\n", completion.label);
            let document = LanguageDocument {
                path: Path::new("/project/pages/docs.dowe").to_path_buf(),
                source,
            };
            let hover =
                hover_at(root, &document, 2, component.len() + 4).expect("component prop hover");
            assert!(
                hover.contains(&format!("{component}.{}", completion.label)),
                "{component}.{}: {hover}",
                completion.label
            );
        }
    }
}

#[test]
fn scaffold_editor_documentation_lists_accepted_regions() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/docs.dowe").to_path_buf(),
        source: "page docsPage\n  Scaffold\n    main\n      Text\n        \"Content\"\n"
            .to_string(),
    };

    let hover = hover_at(Path::new("/project"), &document, 2, 3).expect("Scaffold hover");
    assert!(hover.contains("Accepted children"));
    assert!(hover.contains("`appBar`"));
    assert!(hover.contains("`start`"));
    assert!(hover.contains("`main`"));
    assert!(hover.contains("`end`"));
    assert!(hover.contains("`bottomBar`"));
    assert!(hover.contains("`overlays`"));

    let completion = complete_document(Path::new("/project"), &document, 1, 1)
        .into_iter()
        .find(|item| item.label == "Scaffold")
        .expect("Scaffold completion");
    let documentation = completion.documentation.expect("Scaffold documentation");
    assert!(documentation.contains("Accepted children"));
    assert!(documentation.contains("`main` (required region)"));
}

#[test]
fn layout_bar_editor_documentation_lists_full_width_regions() {
    for component in ["AppBar", "Footer"] {
        let document = LanguageDocument {
            path: Path::new("/project/pages/docs.dowe").to_path_buf(),
            source: format!(
                "page docsPage\n  {component}\n    center\n      Text\n        \"Content\"\n"
            ),
        };

        let hover = hover_at(Path::new("/project"), &document, 2, 3).expect("layout bar hover");
        assert!(hover.contains("`top` (optional full-width region)"));
        assert!(hover.contains("`start` (optional region)"));
        assert!(hover.contains("`end` (optional region)"));
        assert!(hover.contains("`bottom` (optional full-width region)"));
    }
}

#[test]
fn bottom_bar_editor_support_lists_tabs_and_navigation_props() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/docs.dowe").to_path_buf(),
        source: "page docsPage\n  BottomBar\n    tab \n      Icon name:\"home\"\n".to_string(),
    };

    let hover = hover_at(Path::new("/project"), &document, 2, 3).expect("BottomBar hover");
    assert!(hover.contains("Accepted children"));
    assert!(hover.contains("`tab`"));
    assert!(!hover.contains("`center`"));

    let props = complete_document(Path::new("/project"), &document, 3, 9);
    assert!(props.iter().any(|item| item.label == "href"));
    assert!(props.iter().any(|item| item.label == "label"));
    assert!(props.iter().any(|item| item.label == "featured"));
}

#[test]
fn main_editor_documentation_lists_accepted_children() {
    let document = LanguageDocument {
        path: Path::new("/project/main.dowe").to_path_buf(),
        source: "main\n  views:siteRoutes\n".to_string(),
    };

    let hover = hover_at(Path::new("/project"), &document, 1, 1).expect("main hover");
    assert!(hover.contains("Accepted props"));
    assert!(hover.contains("None"));
    assert!(hover.contains("Accepted children"));
    assert!(hover.contains("`app`"));
    assert!(hover.contains("`views:<symbol|array>`"));
    assert!(hover.contains("`server`"));
    assert!(hover.contains("`desktop`"));
}

#[test]
fn server_constructs_and_portable_utilities_have_editor_documentation() {
    let constructs = [
        "main",
        "server",
        "databases",
        "tls",
        "endpoints",
        "route",
        "method",
        "handler",
        "middleware",
        "fn",
        "database",
        "entity",
        "seeder",
        "insert",
        "cache",
        "kv",
        "vector",
        "emb",
        "queue",
        "msg",
        "websocket",
        "udp",
        "tcp",
        "rtp",
        "model",
        "cors",
        "init",
        "redirect",
        "response",
        "return",
        "str",
        "math",
        "parse",
        "url",
        "csv",
        "sort",
        "list",
        "json",
        "date",
        "id",
        "request",
        "file",
        "if",
        "next",
        "log",
        "info",
        "warn",
        "error",
        "task",
        "cron",
        "send",
        "bridge",
        "bearer",
        "http",
        "agent",
        "ws",
        "jwt",
        "spawn",
        "crypto",
        "commit",
        "rollback",
    ];
    let root = Path::new("/project");
    let base_document = LanguageDocument {
        path: Path::new("/project/main.dowe").to_path_buf(),
        source: String::new(),
    };
    let base_completions = complete_document(root, &base_document, 1, 1);

    for construct in constructs {
        assert!(
            base_completions.iter().any(|completion| {
                completion.label == construct && completion.documentation.is_some()
            }),
            "missing server completion documentation for {construct}"
        );
        let document = LanguageDocument {
            path: Path::new("/project/main.dowe").to_path_buf(),
            source: format!("{construct}\n"),
        };
        let hover = hover_at(root, &document, 1, 1).expect("server hover");
        assert!(
            hover.contains(&format!("`{construct}")),
            "{construct}: {hover}"
        );
    }

    for signature in dowe_stdlib::signatures() {
        let name = format!("{}.{}", signature.namespace, signature.function);
        assert!(
            base_completions.iter().any(|completion| {
                completion.label == name && completion.documentation.is_some()
            }),
            "missing stdlib completion documentation for {name}"
        );
        let document = LanguageDocument {
            path: Path::new("/project/main.dowe").to_path_buf(),
            source: format!("{name}\n"),
        };
        let hover = hover_at(root, &document, 1, 1).expect("stdlib hover");
        assert!(hover.contains(&name), "{name}: {hover}");
        assert!(hover.contains(signature.description), "{name}: {hover}");
    }

    let server_document = LanguageDocument {
        path: Path::new("/project/main.dowe").to_path_buf(),
        source: "server ".to_string(),
    };
    let server_props = complete_document(root, &server_document, 1, "server ".len() + 1);
    let port = server_props
        .iter()
        .find(|completion| completion.label == "port")
        .expect("server port completion");
    assert!(
        port.documentation
            .as_deref()
            .is_some_and(|documentation| documentation.contains("server.port"))
    );

    let http_document = LanguageDocument {
        path: Path::new("/project/main.dowe").to_path_buf(),
        source: "http upstream ".to_string(),
    };

    let tls_document = LanguageDocument {
        path: Path::new("/project/main.dowe").to_path_buf(),
        source: "tls ".to_string(),
    };
    let tls_props = complete_document(root, &tls_document, 1, "tls ".len() + 1);
    for prop in [
        "mode",
        "domains",
        "email",
        "staging",
        "cache",
        "domainsFrom",
        "refreshSeconds",
        "httpPort",
    ] {
        assert!(
            tls_props.iter().any(|completion| {
                completion.label == prop && completion.documentation.is_some()
            }),
            "missing tls {prop}"
        );
    }
    let http_props = complete_document(root, &http_document, 1, "http upstream ".len() + 1);
    for prop in ["base", "path", "json"] {
        assert!(
            http_props.iter().any(|completion| {
                completion.label == prop && completion.documentation.is_some()
            }),
            "missing http {prop}"
        );
    }
}

