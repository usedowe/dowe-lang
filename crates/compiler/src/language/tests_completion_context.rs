#[test]
fn views_only_main_has_no_missing_server_diagnostic() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("routes")).expect("routes");
    fs::write(
        root.path().join("routes/view.dowe"),
        "views viewRoutes\n  route path:\"/\" page:homePage\n",
    )
    .expect("views");
    let document = LanguageDocument {
        path: root.path().join("main.dowe"),
        source: "import viewRoutes from \"@/routes/view\"\n\nmain\n  app name:\"Dowe Ui\" bundle:\"dev.dowe.examples.ui\"\n  views:viewRoutes\n"
            .to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn completions_include_show_booleans_and_signals() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("pages/ready.dowe"),
        source: "page readyPage\n  signal isReady value:false\n  Text show:\n    Ready\n  Drawer bind:\n    Text\n      Menu\n"
            .to_string(),
    };

    let completions = complete_document(root.path(), &document, 3, 13);

    assert!(completions.iter().any(|item| item.label == "true"));
    assert!(completions.iter().any(|item| item.label == "false"));
    assert!(completions.iter().any(|item| item.label == "isReady"));

    let drawer = complete_document(root.path(), &document, 5, 15);
    assert!(drawer.iter().any(|item| item.label == "isReady"));
}

#[test]
fn completions_include_i18n_for_nav_menu_entries() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/navigation.dowe").to_path_buf(),
        source: "page navigationPage\n  NavMenu\n    item \n    submenu \n    megamenu \n"
            .to_string(),
    };

    for (line, column) in [(3, 10), (4, 13), (5, 14)] {
        let completions = complete_document(Path::new("/project"), &document, line, column);
        assert!(completions.iter().any(|item| item.label == "i18n"));
    }
}

#[test]
fn hover_documents_primary_and_secondary_i18n_props() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/localized.dowe").to_path_buf(),
        source: "page localizedPage\n  Button i18n:\"actions.save\"\n    \"Save\"\n  SideNav\n    item label:\"Views\" i18n:\"navigation.views\" description:\"Catalog\" descriptionI18n:\"navigation.catalog\" status:\"Ready\" statusI18n:\"navigation.ready\"\n  Tabs\n    tab id:\"overview\" label:\"Overview\" i18n:\"tabs.overview\"\n      Text\n        \"Panel\"\n"
            .to_string(),
    };

    let button = hover_at(Path::new("/project"), &document, 2, 12).expect("button i18n hover");
    assert!(button.contains("translation key"));
    let description =
        hover_at(Path::new("/project"), &document, 5, 75).expect("description i18n hover");
    assert!(description.contains("secondary description"));
    let tab = hover_at(Path::new("/project"), &document, 7, 41).expect("tab i18n hover");
    assert!(tab.contains("translation key"));
}

#[test]
fn completions_and_diagnostics_include_server_middlewares() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("shared/authentication")).expect("middlewares");
    fs::write(
        root.path().join("theme.dowe"),
        "config\n  env\n    variable name:\"JWT_SECRET\" visibility:\"server\" required:false\n",
    )
    .expect("config");
    fs::write(
        root.path().join("shared/authentication/auth.dowe"),
        "middleware requireBearer params:{}\n  bearer token value:req.header.Authorization\n  jwt verified secret:env.JWT_SECRET algorithm:\"HS256\" token:token\n  if verified.valid\n    next context:{ auth:{ subject:verified.claims.sub } }\n  return status:401 json:{ ok:false }\n",
    )
    .expect("middleware");
    let document = LanguageDocument {
        path: root.path().join("main.dowe"),
        source: "import requireBearer from \"@/shared/authentication/auth\"\nmain\n  server port:8080\n    route \"/users/:id\" middleware:[requireBearer]\n      handler req\n        return text:\"Hello\"\n".to_string(),
    };

    let completions = complete_document(root.path(), &document, 4, 35);
    assert!(completions.iter().any(|item| item.label == "requireBearer"));

    let bad_middleware = LanguageDocument {
        path: root.path().join("experimental/authentication.dowe"),
        source: "middleware bad\n  next\n".to_string(),
    };
    let diagnostics = analyze_document(root.path(), &bad_middleware);
    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn completions_and_hover_include_inferred_handler_fields() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("handlers")).expect("src");
    let completion_document = LanguageDocument {
        path: root.path().join("handlers/blogs.dowe"),
        source: "handler createBlog\n  database db provider:\"dowe\" host:\"127.0.0.1\" port:4147 account:\"api\" secret:\"secret\" name:\"app\"\n  query created conn:db.insert table:\"blogs\" value:{ title:\"\" content:\"\" }\n  log created.\n  return json:created\n"
            .to_string(),
    };
    let completions = complete_document(
        root.path(),
        &completion_document,
        4,
        "  log created.".len() + 1,
    );

    assert!(completions.iter().any(|item| item.label == "title"));
    assert!(completions.iter().any(|item| item.label == "content"));
    assert!(completions.iter().any(|item| item.label == "id"));

    let hover_document = LanguageDocument {
        path: root.path().join("handlers/blogs.dowe"),
        source: "handler createBlog\n  database db provider:\"dowe\" host:\"127.0.0.1\" port:4147 account:\"api\" secret:\"secret\" name:\"app\"\n  query created conn:db.insert table:\"blogs\" value:{ title:\"\" }\n  log created.title\n  return json:created\n"
            .to_string(),
    };
    assert_eq!(
        hover_at(root.path(), &hover_document, 4, 10).as_deref(),
        Some("Dowe inferred field `created.title`")
    );
}

#[test]
fn completions_include_inferred_kv_handler_fields() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("handlers")).expect("src");
    let completion_document = LanguageDocument {
        path: root.path().join("handlers/cache.dowe"),
        source: "handler cacheAppointment\n  cache appCache provider:\"dowe\" host:\"127.0.0.1\" port:4148 account:\"app\" secret:\"secret\" name:\"clinic\"\n  kv saved conn:appCache.set key:\"appointment:1\" value:{ patientName:\"Ana\" }\n  log saved.\n  return json:saved\n"
            .to_string(),
    };
    let completions = complete_document(
        root.path(),
        &completion_document,
        4,
        "  log saved.".len() + 1,
    );

    assert!(completions.iter().any(|item| item.label == "ok"));
    assert!(completions.iter().any(|item| item.label == "key"));
}

#[test]
fn completions_and_diagnostics_include_declared_types() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("handlers")).expect("handlers");
    fs::create_dir_all(root.path().join("pages")).expect("pages");
    let handler_document = LanguageDocument {
        path: root.path().join("handlers/users.dowe"),
        source: "type User\n  name:string\n  age:number\n\nhandler createUser\n  const body:User value:req.json\n  log body.\n  database db provider:\"dowe\" host:\"127.0.0.1\" port:4147 account:\"api\" secret:\"secret\" name:\"app\"\n  query created conn:db.insert table:\"users\" value:{ name:body.email }\n  return json:created\n".to_string(),
    };

    let completions = complete_document(root.path(), &handler_document, 7, "  log body.".len() + 1);
    assert!(completions.iter().any(|item| item.label == "name"));
    assert!(completions.iter().any(|item| item.label == "age"));

    let diagnostic_document = LanguageDocument {
        path: root.path().join("handlers/users.dowe"),
        source: "type User\n  name:string\n  age:number\n\nhandler createUser\n  const body:User value:req.json\n  database db provider:\"dowe\" host:\"127.0.0.1\" port:4147 account:\"api\" secret:\"secret\" name:\"app\"\n  query created conn:db.insert table:\"users\" value:{ name:body.email }\n  return json:created\n".to_string(),
    };
    let diagnostics = analyze_document(root.path(), &diagnostic_document);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("unknown field `body.email`"))
    );

    let view_document = LanguageDocument {
        path: root.path().join("pages/blogs.dowe"),
        source: "type BlogItem\n  id:string\n  title:string\n\npage blogsPage\n  signal blogs type:BlogItem[] value:[]\n  Grid\n    each in:blogs as:item key:item.id\n      Text\n        item.\n".to_string(),
    };
    let item_completions =
        complete_document(root.path(), &view_document, 10, "        item.".len() + 1);
    assert!(item_completions.iter().any(|item| item.label == "id"));
    assert!(item_completions.iter().any(|item| item.label == "title"));
}

#[test]
fn completions_include_imported_shared_types() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("types")).expect("types");
    fs::create_dir_all(root.path().join("handlers")).expect("handlers");
    fs::create_dir_all(root.path().join("pages")).expect("pages");
    fs::write(
        root.path().join("types/tickets.dowe"),
        "type TicketInput\n  title:string\n  priority:string\n\n\
type TicketSummary\n  id:string\n  status:string\n",
    )
    .expect("types");
    let handler_document = LanguageDocument {
        path: root.path().join("handlers/tickets.dowe"),
        source: "import TicketInput from \"../types/tickets\"\n\nhandler createTicket\n  const body:TicketInput value:req.json\n  log body.\n  return json:{ ok:true }\n".to_string(),
    };
    let handler_completions =
        complete_document(root.path(), &handler_document, 5, "  log body.".len() + 1);
    assert!(handler_completions.iter().any(|item| item.label == "title"));
    assert!(
        handler_completions
            .iter()
            .any(|item| item.label == "priority")
    );

    let view_document = LanguageDocument {
        path: root.path().join("pages/tickets.dowe"),
        source: "import TicketSummary from \"../types/tickets\"\n\npage ticketsPage\n  signal tickets type:TicketSummary[] value:[]\n  Grid\n    each in:tickets as:item key:item.id\n      Text\n        item.\n".to_string(),
    };
    let view_completions =
        complete_document(root.path(), &view_document, 8, "        item.".len() + 1);
    assert!(view_completions.iter().any(|item| item.label == "id"));
    assert!(view_completions.iter().any(|item| item.label == "status"));
}

#[test]
fn language_supports_imported_persistent_view_store() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("domains/auth")).expect("store");
    fs::create_dir_all(root.path().join("views/pages")).expect("pages");
    let store_path = root.path().join("domains/auth/session.dowe");
    fs::write(
        &store_path,
        "store session persistent:true value:{ authorization:\"\" token:\"\" }\n",
    )
    .expect("store");
    let document = LanguageDocument {
        path: root.path().join("views/pages/auth.dowe"),
        source:
            "import session from \"@/domains/auth/session\"\n\npage authPage\n  Input bind:session.\n"
                .to_string(),
    };

    let completions =
        complete_document(root.path(), &document, 4, "  Input bind:session.".len() + 1);
    assert!(
        completions
            .iter()
            .any(|item| item.label == "session.authorization")
    );
    assert!(completions.iter().any(|item| item.label == "session.token"));

    let definition = definition_at(root.path(), &document, 1, 8).expect("definition");
    assert_eq!(definition.path, store_path);

    let store_document = LanguageDocument {
        path: definition.path,
        source: "store session persistent:true value:{ authorization:\"\" token:\"\" }\n"
            .to_string(),
    };
    assert!(analyze_document(root.path(), &store_document).is_empty());
    assert!(
        document_symbols(root.path(), &store_document)
            .iter()
            .any(|symbol| symbol.name == "store session")
    );
}

