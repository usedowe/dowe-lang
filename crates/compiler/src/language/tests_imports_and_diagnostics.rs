#[test]
fn diagnostics_report_invalid_import() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    fs::create_dir_all(root.path().join("routes")).expect("routes");
    let path = root.path().join("routes/view.dowe");
    let document = LanguageDocument {
        path,
        source: "import Missing from \"../pages/missing\"\nviews viewRoutes\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == LanguageDiagnosticSeverity::Error
            && diagnostic.message.contains("does not exist")
    }));
}

#[test]
fn diagnostics_accept_imported_reusable_view_components() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("components")).expect("components");
    fs::create_dir_all(root.path().join("layouts")).expect("layouts");
    fs::write(
        root.path().join("components/views-navigation.dowe"),
        concat!(
            "component ViewsNavigation\n",
            "  SideNav variant:\"ghost\" scheme:\"muted\" size:\"sm\" wide:true\n",
            "    item label:\"Overview\" href:\"/docs/views\"\n"
        ),
    )
    .expect("component");
    let document = LanguageDocument {
        path: root.path().join("layouts/views-layout.dowe"),
        source: concat!(
            "import ViewsNavigation from \"@/components/views-navigation\"\n",
            "layout ViewsLayout\n",
            "  signal openMenu value:false\n",
            "  Scaffold\n",
            "    start\n",
            "      Sidebar\n",
            "        body\n",
            "          ViewsNavigation\n",
            "    main\n",
            "      children\n",
            "    overlays\n",
            "      Drawer bind:openMenu\n",
            "        body\n",
            "          ViewsNavigation\n"
        )
        .to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn diagnostics_resolve_imports_from_nearest_nested_project_root() {
    let root = tempdir().expect("tempdir");
    fs::write(root.path().join("main.dowe"), "main\n").expect("root main");
    let nested_project = root.path().join("projects/commerce-ops");
    fs::create_dir_all(nested_project.join("views/layouts")).expect("layouts");
    fs::create_dir_all(nested_project.join("views/pages")).expect("pages");
    fs::create_dir_all(nested_project.join("views/routes")).expect("routes");
    fs::write(nested_project.join("main.dowe"), "main\n").expect("nested main");
    fs::write(
        nested_project.join("views/layouts/ops.dowe"),
        "layout OpsLayout\n  Box\n    children\n",
    )
    .expect("layout");
    fs::write(
        nested_project.join("views/pages/dashboard.dowe"),
        "page dashboardPage\n  Text\n    \"Dashboard\"\n",
    )
    .expect("dashboard");
    fs::write(
        nested_project.join("views/pages/inventory.dowe"),
        "page inventoryPage\n  Text\n    \"Inventory\"\n",
    )
    .expect("inventory");
    let document = LanguageDocument {
        path: nested_project.join("views/routes/view.dowe"),
        source: concat!(
            "import OpsLayout from \"@/views/layouts/ops\"\n",
            "import dashboardPage from \"@/views/pages/dashboard\"\n",
            "import inventoryPage from \"@/views/pages/inventory\"\n\n",
            "views viewRoutes\n",
            "  group path:\"/\" layout:OpsLayout\n",
            "    route path:\"\" page:dashboardPage\n",
            "    route path:\"inventory\" page:inventoryPage\n"
        )
        .to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
    let import_location = definition_at(root.path(), &document, 1, 8).expect("definition");
    assert_eq!(
        import_location.path,
        nested_project.join("views/layouts/ops.dowe")
    );
}

#[test]
fn diagnostics_and_definition_support_server_config_module_imports() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("domains/accounts")).expect("domain");
    fs::write(
        root.path().join("domains/accounts/storage.dowe"),
        "database db provider:\"dowe\" host:\"127.0.0.1\" port:4147 account:\"api\" secret:\"secret\" name:\"iptv\"\n",
    )
    .expect("config");
    let document = LanguageDocument {
        path: root.path().join("domains/accounts/list.dowe"),
        source: concat!(
            "import db from \"./storage\"\n\n",
            "fn listAccountsRepository\n",
            "  query rows conn:db.list table:\"directvAccounts\"\n",
            "  return value:{ rows:rows }\n"
        )
        .to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
    let import_location = definition_at(root.path(), &document, 1, 8).expect("definition");
    assert_eq!(
        import_location.path,
        root.path().join("domains/accounts/storage.dowe")
    );
    assert_eq!(import_location.range, LanguageRange::single_line(1, 1, 8));
}
#[test]
fn diagnostics_report_entry_files_under_src() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src")).expect("src");

    for (file_name, source) in [("main.dowe", "main\n"), ("theme.dowe", "theme\n")] {
        let document = LanguageDocument {
            path: root.path().join("src").join(file_name),
            source: source.to_string(),
        };
        let diagnostics = analyze_document(root.path(), &document);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == LanguageDiagnosticSeverity::Error
                && diagnostic.message.contains(&format!("src/{file_name}"))
                && diagnostic.message.contains("project-root")
        }));
    }
}

#[test]
fn diagnostics_report_removed_environment_dowe() {
    let root = tempdir().expect("tempdir");
    let document = LanguageDocument {
        path: root.path().join("env.dowe"),
        source: "env\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == LanguageDiagnosticSeverity::Error
            && diagnostic.message.contains("no longer supported")
            && diagnostic.message.contains(".env")
    }));
}

#[test]
fn diagnostics_validate_view_functions_and_bindings() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    let path = root.path().join("pages/blogs.dowe");
    let document = LanguageDocument {
        path,
        source: "page blogsPage\n  signal blog value:{ title:\"\" }\n  Button onClick:missing\n    \"Save\"\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("unknown fn `missing`"))
    );
}

#[test]
fn completions_keep_bare_on_click_for_functions() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("pages");
    let document = LanguageDocument {
        path: root.path().join("pages/menu.dowe"),
        source: "page menuPage\n  signal openDrawer value:false\n  fn openMenu\n    set openDrawer value:true\n  IconButton label:\"menu\" icon:\"menu-dots\" onClick:\n"
            .to_string(),
    };

    let completions = complete_document(root.path(), &document, 5, 65);

    assert!(
        completions
            .iter()
            .any(|completion| completion.label == "openMenu")
    );
    assert!(
        !completions
            .iter()
            .any(|completion| completion.label == "openDrawer")
    );
}

#[test]
fn diagnostics_reject_unknown_view_signal_fields() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("pages/blogs.dowe"),
        source: "page blogsPage\n  signal blog value:{ title:\"\" }\n  Input bind:blog.content\n"
            .to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("unknown signal path `blog.content`")
    }));
}

#[test]
fn diagnostics_reject_unknown_inferred_handler_fields() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("handlers")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("handlers/blogs.dowe"),
        source: "handler createBlog\n  database db provider:\"dowe\" host:\"127.0.0.1\" port:4147 account:\"api\" secret:\"secret\" name:\"app\"\n  query created conn:db.insert table:\"blogs\" value:{ title:\"\" }\n  log created.content\n  return json:created\n"
            .to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("unknown field `created.content`")
    }));

    let response_document = LanguageDocument {
        path: root.path().join("handlers/blogs.dowe"),
        source: "handler createBlog\n  database db provider:\"dowe\" host:\"127.0.0.1\" port:4147 account:\"api\" secret:\"secret\" name:\"app\"\n  query created conn:db.insert table:\"blogs\" value:{ title:\"\" }\n  return json:{ data:created.content }\n"
            .to_string(),
    };
    let response_diagnostics = analyze_document(root.path(), &response_document);
    assert!(response_diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("unknown field `created.content`")
    }));
}

#[test]
fn diagnostics_accept_remote_store_env_in_handler_files() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("handlers")).expect("src");
    fs::write(root.path().join(".env.example"), "DB_HOST=\nDB_TOKEN=\n").expect("env");
    let document = LanguageDocument {
        path: root.path().join("handlers/appointments.dowe"),
        source: concat!(
            "handler listAppointments req\n",
            "  database db provider:\"dowe\" host:env.DB_HOST port:4147 account:\"clinic-api\" secret:env.DB_TOKEN name:\"clinic\"\n",
            "  query appointments conn:db.list table:\"appointments\"\n",
            "  return json:{ ok:true data:appointments }\n"
        )
        .to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn diagnostics_and_completions_support_d1_database_provider() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("config")).expect("config");
    fs::write(
        root.path().join(".env.example"),
        "ACCOUNT_ID=\nCLOUDFLARE_API_TOKEN=\n",
    )
    .expect("env");
    let document = LanguageDocument {
        path: root.path().join("config/database.dowe"),
        source:
            "database db provider:\"d1\" account:env.ACCOUNT_ID secret:env.CLOUDFLARE_API_TOKEN name:\"database-id\"\n"
                .to_string(),
    };

    assert!(analyze_document(root.path(), &document).is_empty());

    let completion_document = LanguageDocument {
        path: root.path().join("config/database.dowe"),
        source: "database db provider:\n".to_string(),
    };
    let completions = complete_document(
        root.path(),
        &completion_document,
        1,
        completion_document.source.trim_end().len() + 1,
    );

    assert!(completions.iter().any(|item| item.label == "\"postgres\""));
    assert!(completions.iter().any(|item| item.label == "\"d1\""));
    assert!(completions.iter().any(|item| item.label == "\"dowe\""));
}

#[test]
fn diagnostics_accept_text_typography_props() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("pages/login.dowe"),
        source: "page loginPage\n  Text size:\"md\" align:\"center\" color:\"primaryText\" i18n:\"auth.login.title\"\n    \"Login\"\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );

    let alignment_document = LanguageDocument {
        path: root.path().join("pages/alignment.dowe"),
        source: "page alignmentPage\n  Text align:\n    \"Color\"\n".to_string(),
    };
    let alignment_completions = complete_document(root.path(), &alignment_document, 2, 14);
    assert!(
        alignment_completions
            .iter()
            .any(|item| item.label == "\"start\"")
    );
    assert!(
        alignment_completions
            .iter()
            .any(|item| item.label == "\"center\"")
    );
    assert!(
        alignment_completions
            .iter()
            .any(|item| item.label == "\"end\"")
    );
    assert!(
        alignment_completions
            .iter()
            .any(|item| item.label == "\"justify\"")
    );

    let completion_document = LanguageDocument {
        path: root.path().join("pages/colors.dowe"),
        source: "page colorsPage\n  Text color:\n    \"Color\"\n".to_string(),
    };
    let completions = complete_document(root.path(), &completion_document, 2, 14);
    assert!(
        completions
            .iter()
            .any(|item| item.label == "\"primaryTitle\"")
    );
    assert!(
        !completions
            .iter()
            .any(|item| item.label == "\"softPrimaryTitle\"")
    );
    assert!(!completions.iter().any(|item| item.label == "\"onPrimary\""));
}

