#[test]
fn completions_include_functions_signals_and_env() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    fs::write(root.path().join(".env.example"), "BACKEND_URL=\n").expect("env");
    let document = LanguageDocument {
        path: root.path().join("pages/blogs.dowe"),
        source: "page blogsPage\n  signal blog value:{ title:\"\" }\n  fn saveBlog\n    reset blog\n  Button onClick:\n    Save\n  Input bind:\n  Text\n    env.\n  Text\n    blog.\n".to_string(),
    };

    let actions = complete_document(root.path(), &document, 5, 18);
    let signals = complete_document(root.path(), &document, 7, 14);
    let env = complete_document(root.path(), &document, 9, 9);
    let dynamic_fields = complete_document(root.path(), &document, 11, 10);

    assert!(actions.iter().any(|item| item.label == "saveBlog"));
    assert!(signals.iter().any(|item| item.label == "blog.title"));
    assert!(env.iter().any(|item| item.label == "BACKEND_URL"));
    assert!(dynamic_fields.iter().any(|item| item.label == "title"));
    assert!(
        actions
            .iter()
            .any(|item| item.kind == LanguageCompletionKind::Function)
    );
}

#[test]
fn language_support_recognizes_view_constants() {
    let root = tempdir().expect("root");
    let document = LanguageDocument {
        path: root.path().join("pages/catalog.dowe"),
        source: "page catalog\n  Grid\n    const plan value:{ name:\"Starter\" }\n    Text\n      plan.\n"
            .to_string(),
    };
    let base = complete_document(root.path(), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "const"));
    let fields = complete_document(root.path(), &document, 5, "      plan.".len() + 1);
    assert!(fields.iter().any(|item| item.label == "name"));
    assert!(
        document_symbols(root.path(), &document)
            .iter()
            .flat_map(|symbol| symbol.children.iter())
            .any(|symbol| symbol.name == "const plan")
    );
    assert!(
        hover_at(root.path(), &document, 3, 6).is_some_and(|hover| hover.contains("immutable"))
    );
}

#[test]
fn queue_publication_completions_and_documentation_are_available() {
    let root = tempdir().expect("root");
    let document = LanguageDocument {
        path: root.path().join("main.dowe"),
        source: "main\n  server port:0\n    route \"/messages\"\n      handler\n        queue appQueue provider:\"dowe\" host:\"local\" port:4150 account:\"app\" secret:\"secret\" vhost:\"jobs\"\n        msg sent conn:appQueue.publish queue:\"notifications\" payload:{ event:\"created\" }\n        log sent.\n"
            .to_string(),
    };
    let fields = complete_document(root.path(), &document, 7, "        log sent.".len() + 1);
    let queue_document = LanguageDocument {
        path: root.path().join("main.dowe"),
        source: "queue\n".to_string(),
    };
    let message_document = LanguageDocument {
        path: root.path().join("main.dowe"),
        source: "msg\n".to_string(),
    };

    assert_eq!(
        fields
            .iter()
            .map(|completion| completion.label.as_str())
            .collect::<Vec<_>>(),
        ["ok", "id"]
    );
    assert!(
        hover_at(root.path(), &queue_document, 1, 1)
            .is_some_and(|hover| hover.contains("queue service") && hover.contains("vhost"))
    );
    assert!(
        hover_at(root.path(), &message_document, 1, 1).is_some_and(|hover| hover
            .contains("conn:<queue>.publish")
            && hover.contains("{ ok, id }"))
    );
}

#[test]
fn completions_and_hover_include_server_tasks_and_cron() {
    let document = LanguageDocument {
        path: Path::new("/project/main.dowe").to_path_buf(),
        source: "main\n  server port:8080\n    init\n      task fn:cleanup args:{ event:{ source:\"startup\" } } after:\"headers\"\n      task\n        log args.source\n      cron fn:cleanup schedule:\"0 * * * *\"\n"
            .to_string(),
    };
    let completions = complete_document(Path::new("/project"), &document, 4, 7);
    let task_props = complete_document(
        Path::new("/project"),
        &document,
        4,
        document.source.lines().nth(3).expect("task line").len() + 1,
    );
    let cron_props = complete_document(
        Path::new("/project"),
        &document,
        7,
        document.source.lines().nth(6).expect("cron line").len() + 1,
    );

    assert!(completions.iter().any(|item| item.label == "task"));
    assert!(!completions.iter().any(|item| item.label == "go"));
    assert!(completions.iter().any(|item| item.label == "cron"));
    assert!(task_props.iter().any(|item| item.label == "fn"));
    assert!(task_props.iter().any(|item| item.label == "after"));
    assert!(cron_props.iter().any(|item| item.label == "fn"));
    assert!(
        hover_at(Path::new("/project"), &document, 4, 7)
            .expect("task hover")
            .contains("real upstream response headers")
    );
    let after_column = document
        .source
        .lines()
        .nth(3)
        .expect("task line")
        .find("after")
        .expect("after prop")
        + 2;
    assert!(
        hover_at(Path::new("/project"), &document, 4, after_column)
            .expect("after hover")
            .contains("after:\"headers\"")
    );
    assert!(
        hover_at(Path::new("/project"), &document, 5, 7)
            .expect("inline task hover")
            .contains("inline")
    );
    assert!(
        hover_at(Path::new("/project"), &document, 7, 7)
            .expect("cron hover")
            .contains("UTC")
    );
}

#[test]
fn hover_documents_theme_and_fonts_configuration() {
    let document = LanguageDocument {
        path: Path::new("/project/theme.dowe").to_path_buf(),
        source: "theme\n  fonts default:\"manrope\" install:[\"manrope\",\"inter\"]\n  design defaultTheme:\"light\"\n    Card variant:\"outline\" scheme:\"primary\" radius:\"xs\" shadow:\"xs\"\n    Button variant:\"solid\" scheme:\"secondary\" size:\"md\"\n    Avatar radius:\"full\" size:\"md\"\n    Chip variant:\"solid\" scheme:\"secondary\" radius:\"full\" size:\"sm\"\n    Text font:\"manrope\"\n    Title font:\"syne\"\n    theme name:\"light\"\n      colors:\n        primary color:\"#1F3A5F\" text:\"#FFFFFF\" title:\"#FFFFFE\"\n        happy color:\"#176C75\" text:\"#FFFFFE\" title:\"#FFFFFE\"\n"
            .to_string(),
    };

    let root_theme = hover_at(Path::new("/project"), &document, 1, 2).expect("root theme hover");
    assert!(root_theme.contains("canonical project theme configuration"));
    assert!(root_theme.contains("`fonts`"));
    assert!(root_theme.contains("`design`"));

    let design = hover_at(Path::new("/project"), &document, 3, 4).expect("design hover");
    assert!(design.contains("`Card`"));
    assert!(design.contains("`Button`"));
    assert!(design.contains("explicit usage prop"));

    let card = hover_at(Path::new("/project"), &document, 4, 6).expect("Card hover");
    assert!(card.contains("theme defaults"));
    assert!(card.contains("`radius` or `rounded`"));
    assert!(card.contains("component usage always wins"));

    let text = hover_at(Path::new("/project"), &document, 8, 6).expect("Text hover");
    assert!(text.contains("project-wide default font"));
    assert!(text.contains("component instance always wins"));

    let title = hover_at(Path::new("/project"), &document, 9, 6).expect("Title hover");
    assert!(title.contains("generated font assets"));

    let text_fonts = complete_document(
        Path::new("/project"),
        &document,
        8,
        "    Text font:".len() + 1,
    );
    assert!(text_fonts.iter().any(|item| item.label == "\"manrope\""));
    assert!(text_fonts.iter().any(|item| item.label == "\"syne\""));

    let fonts = hover_at(Path::new("/project"), &document, 2, 4).expect("fonts hover");
    assert!(fonts.contains("Dowe's built-in catalog"));
    assert!(fonts.contains("`\"manrope\"`"));
    assert!(fonts.contains("`\"puritan\"`"));

    let default = hover_at(Path::new("/project"), &document, 2, 10).expect("default hover");
    assert!(default.contains("`fonts.default`"));
    assert!(default.contains("quoted font token"));

    let install = hover_at(Path::new("/project"), &document, 2, 28).expect("install hover");
    assert!(install.contains("`fonts.install`"));
    assert!(install.contains("effective generated font set"));

    let named_theme = hover_at(Path::new("/project"), &document, 10, 6).expect("named theme hover");
    assert!(named_theme.contains("named color theme"));
    assert!(named_theme.contains("`extends`"));
    assert!(named_theme.contains("`colors`"));
    assert!(named_theme.contains("`color`, `text`, and `title`"));
    assert!(named_theme.contains("Component defaults belong"));

    let family = hover_at(Path::new("/project"), &document, 12, 10).expect("family hover");
    assert!(family.contains("grouped semantic color family"));
    assert!(family.contains("normalized"));

    let happy = hover_at(Path::new("/project"), &document, 13, 10).expect("happy hover");
    assert!(happy.contains("grouped semantic color family"));

    let role = hover_at(Path::new("/project"), &document, 12, 36).expect("role hover");
    assert!(role.contains("ordinary content"));

    let family_document = LanguageDocument {
        path: Path::new("/project/theme.dowe").to_path_buf(),
        source: "theme\n  design defaultTheme:\"light\"\n    theme name:\"light\"\n      colors:\n        \n"
            .to_string(),
    };
    let families = complete_document(Path::new("/project"), &family_document, 5, 9);
    assert!(families.iter().any(|item| item.label == "primary"));
    assert!(!families.iter().any(|item| item.label == "softPrimary"));
    assert!(!families.iter().any(|item| item.label == "primaryText"));

    let role_document = LanguageDocument {
        path: Path::new("/project/theme.dowe").to_path_buf(),
        source: "theme\n  design defaultTheme:\"light\"\n    theme name:\"light\"\n      colors:\n        primary \n"
            .to_string(),
    };
    let roles = complete_document(Path::new("/project"), &role_document, 5, 17);
    assert!(roles.iter().any(|item| item.label == "color"));
    assert!(roles.iter().any(|item| item.label == "text"));
    assert!(roles.iter().any(|item| item.label == "title"));
}

