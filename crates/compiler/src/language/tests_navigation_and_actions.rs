#[test]
fn server_source_completions_offer_capability_selectors() {
    let root = Path::new("/project");
    for (source, expected) in [
        ("handler normalize\n  str result source:\n", "\"trim\""),
        ("handler readQuery\n  request query source:\n", "\"query\""),
        (
            "handler readBytes\n  request payload source:\n",
            "\"bytes\"",
        ),
        (
            "websocket \"/events\"\n  onMessage\n    ws event source:\n",
            "\"json\"",
        ),
        ("handler transform\n  agent chat source:\n", "\"chat\""),
    ] {
        let line = source.lines().count();
        let prefix = source.lines().last().expect("completion line");
        let document = LanguageDocument {
            path: Path::new("/project/server/handlers/example.dowe").to_path_buf(),
            source: source.to_string(),
        };
        let completions = complete_document(root, &document, line, prefix.len() + 1);
        assert!(
            completions
                .iter()
                .any(|completion| completion.label == expected),
            "missing {expected} completion for {prefix}"
        );
    }
}

#[test]
fn definition_resolves_imports_and_env() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    fs::write(
        root.path().join("pages/blogs.dowe"),
        "page blogsPage\n  Box\n",
    )
    .expect("page");
    fs::write(root.path().join(".env.example"), "BACKEND_URL=\n").expect("env");
    let document = LanguageDocument {
        path: root.path().join("routes/view.dowe"),
        source: "import blogsPage from \"../pages/blogs\"\nviews viewRoutes\n  route path:\"blogs\" page:blogsPage\n".to_string(),
    };

    let import_location = definition_at(root.path(), &document, 1, 9).expect("definition");
    assert_eq!(import_location.path, root.path().join("pages/blogs.dowe"));

    let page = LanguageDocument {
        path: root.path().join("pages/blogs.dowe"),
        source: "page blogsPage\n  Text\n    env.BACKEND_URL\n".to_string(),
    };
    let env_location = definition_at(root.path(), &page, 3, 18).expect("env definition");
    assert_eq!(env_location.path, root.path().join(".env.example"));
}

#[test]
fn document_symbols_include_routes_and_handlers() {
    let document = LanguageDocument {
        path: Path::new("/project/main.dowe").to_path_buf(),
        source: "main\n  server port:8080\n    route \"/api/status\"\n      response text:\"OK\"\n"
            .to_string(),
    };

    let symbols = document_symbols(Path::new("/project"), &document);

    assert_eq!(symbols[0].name, "main");
    assert!(
        symbols[0]
            .children
            .iter()
            .any(|symbol| symbol.name == "server")
    );
}

#[test]
fn import_completions_use_project_root_aliases() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("views/layouts")).expect("layouts");
    fs::create_dir_all(root.path().join("views/routes")).expect("routes");
    fs::create_dir_all(root.path().join("server/handlers")).expect("handlers");
    fs::write(
        root.path().join("views/layouts/lab.dowe"),
        "layout LabLayout\n  children\n",
    )
    .expect("layout");
    fs::write(
        root.path().join("server/handlers/status.dowe"),
        "handler getStatus req\n  return text:\"OK\"\n",
    )
    .expect("handler");
    fs::write(root.path().join("main.dowe"), "main\n").expect("main");
    let document = LanguageDocument {
        path: root.path().join("views/routes/view.dowe"),
        source: "import LabLayout from \"\"\nviews viewRoutes\n".to_string(),
    };

    let completions = complete_document(
        root.path(),
        &document,
        1,
        "import LabLayout from \"".len() + 1,
    );

    assert!(
        completions
            .iter()
            .any(|completion| completion.label == "@/views/layouts/lab")
    );
    assert!(
        completions
            .iter()
            .any(|completion| completion.label == "@/server/handlers/status")
    );
    assert!(
        completions
            .iter()
            .all(|completion| completion.label != "@/main")
    );
}

#[test]
fn code_actions_import_exact_exports_with_project_root_aliases() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("views/pages")).expect("pages");
    fs::create_dir_all(root.path().join("views/routes")).expect("routes");
    fs::write(root.path().join("views/pages/home.dowe"), "page HomePage\n").expect("page");
    let source = "import LandingLayout from \"@/layouts/landing\"\n\nviews viewRoutes\n  group path:\"/\" layout:LandingLayout\n    route path:\"\" page:HomePage\n";
    let document = LanguageDocument {
        path: root.path().join("views/routes/views.dowe"),
        source: source.to_string(),
    };

    let actions = code_actions_at(root.path(), &document, 5, 27);

    assert_eq!(actions.len(), 1);
    assert_eq!(
        actions[0].title,
        "Import HomePage from \"@/views/pages/home\""
    );
    assert_eq!(actions[0].edit.range.start.line, 1);
    assert_eq!(
        actions[0].edit.new_text,
        "\nimport HomePage from \"@/views/pages/home\""
    );
}

#[test]
fn code_actions_offer_multiple_candidates_and_skip_existing_imports() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages/admin")).expect("admin");
    fs::create_dir_all(root.path().join("pages/public")).expect("public");
    fs::create_dir_all(root.path().join("routes")).expect("routes");
    fs::write(root.path().join("pages/admin/home.dowe"), "page HomePage\n").expect("admin page");
    fs::write(
        root.path().join("pages/public/home.dowe"),
        "page HomePage\n",
    )
    .expect("public page");
    let document = LanguageDocument {
        path: root.path().join("routes/views.dowe"),
        source: "views viewRoutes\n  route path:\"\" page:HomePage\n".to_string(),
    };

    let actions = code_actions_at(root.path(), &document, 2, 23);

    assert_eq!(actions.len(), 2);
    assert_eq!(
        actions[0].edit.new_text,
        "import HomePage from \"@/pages/admin/home\"\n\n"
    );
    let imported = LanguageDocument {
        path: document.path,
        source: "import HomePage from \"@/pages/admin/home\"\n\nviews viewRoutes\n  route path:\"\" page:HomePage\n".to_string(),
    };
    assert!(code_actions_at(root.path(), &imported, 4, 23).is_empty());
}

#[test]
fn code_actions_add_imports_to_the_existing_module_import() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("handlers")).expect("handlers");
    fs::create_dir_all(root.path().join("server")).expect("server");
    fs::write(
        root.path().join("handlers/blogs.dowe"),
        "handler listBlogs\nhandler createBlog\n",
    )
    .expect("handlers");
    let document = LanguageDocument {
        path: root.path().join("server/api.dowe"),
        source: "import listBlogs from \"../handlers/blogs\"\n\nendpoints apiRoutes\n  route \"/api/blogs\"\n    method POST handler:createBlog\n".to_string(),
    };

    let actions = code_actions_at(root.path(), &document, 5, 25);

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].edit.range.start.line, 1);
    assert_eq!(actions[0].edit.range.start.column, 1);
    assert_eq!(
        actions[0].edit.new_text,
        "import listBlogs, createBlog from \"../handlers/blogs\""
    );
}

#[test]
fn code_actions_preserve_braced_multiple_imports() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("handlers")).expect("handlers");
    fs::create_dir_all(root.path().join("server")).expect("server");
    fs::write(
        root.path().join("handlers/blogs.dowe"),
        "handler listBlogs\nhandler createBlog\n",
    )
    .expect("handlers");
    let document = LanguageDocument {
        path: root.path().join("server/api.dowe"),
        source: "import { listBlogs } from \"../handlers/blogs\"\n\nendpoints apiRoutes\n  route \"/api/blogs\"\n    method POST handler:createBlog\n".to_string(),
    };

    let actions = code_actions_at(root.path(), &document, 5, 25);

    assert_eq!(actions.len(), 1);
    assert_eq!(
        actions[0].edit.new_text,
        "import { listBlogs, createBlog } from \"../handlers/blogs\""
    );
}

#[test]
fn code_actions_skip_builtins_and_local_symbols() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("components")).expect("components");
    fs::create_dir_all(root.path().join("pages")).expect("pages");
    fs::write(
        root.path().join("components/button.dowe"),
        "component Button\n  Text\n    \"Custom\"\n",
    )
    .expect("component");
    fs::write(root.path().join("components/save.dowe"), "fn save\n").expect("action");
    let document = LanguageDocument {
        path: root.path().join("pages/home.dowe"),
        source: "page HomePage\n  fn save\n  Button onClick:save\n    \"Save\"\n".to_string(),
    };

    assert!(code_actions_at(root.path(), &document, 3, 4).is_empty());
    assert!(code_actions_at(root.path(), &document, 3, 18).is_empty());
}

