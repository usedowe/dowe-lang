use super::{
    LanguageCompletionKind, LanguageDiagnosticSeverity, LanguageDocument, LanguageRange,
    code_actions_at, complete_document, definition_at, document_symbols, format_document, hover_at,
};
use crate::language::analyze_document;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn formatter_normalizes_spacing_and_newline() {
    let source = "page loginPage   \n  Box   p:4\n    Text   size:\"md\"\n      Login";
    let formatted = format_document(
        Path::new("/project"),
        Path::new("/project/pages/login.dowe"),
        source,
    )
    .expect("formatted");

    assert_eq!(
        formatted,
        "page loginPage\n  Box p:4\n    Text size:\"md\"\n      Login\n"
    );
    assert_eq!(
        format_document(
            Path::new("/project"),
            Path::new("/project/pages/login.dowe"),
            &formatted,
        )
        .expect("formatted again"),
        formatted
    );
}

#[test]
fn diagnostics_validate_native_test_documents() {
    let root = tempdir().expect("root");
    let valid = LanguageDocument {
        path: root.path().join("verification/release.dowe"),
        source: "test \"metadata\"\n  assert true value:true\n  assert false value:false\n  assert equal actual:[1 2] expected:[1 2]\n".to_string(),
    };
    let invalid = LanguageDocument {
        path: root.path().join("verification/invalid.dowe"),
        source: "test \"invalid\"\n  assert equal actual:1\n".to_string(),
    };

    let valid_diagnostics = analyze_document(root.path(), &valid);
    let invalid_diagnostics = analyze_document(root.path(), &invalid);
    let completions = complete_document(root.path(), &valid, 1, 1);
    let symbols = document_symbols(root.path(), &valid);

    assert!(valid_diagnostics.is_empty(), "{valid_diagnostics:?}");
    assert!(
        invalid_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("requires `expected`"))
    );
    assert!(
        completions
            .iter()
            .any(|completion| completion.label == "test")
    );
    assert_eq!(symbols[0].name, "test metadata");
}

#[test]
fn formatter_keeps_multiple_imports_in_one_declaration() {
    let path = Path::new("/project/server/api.dowe");
    let source = "import { listBlogs, createBlog } from \"../handlers/blogs\"\n";

    let formatted = format_document(Path::new("/project"), path, source).expect("format");

    assert_eq!(
        formatted,
        "import listBlogs, createBlog from \"../handlers/blogs\"\n"
    );
    assert_eq!(
        format_document(Path::new("/project"), path, &formatted).expect("format twice"),
        formatted
    );
}

#[test]
fn formatter_preserves_main_views_reference_syntax() {
    let source =
        "main\n  app name:\"Dowe\" bundle:\"dev.dowe.web\"\n  views:[siteRoutes docsRoutes]\n";
    let path = Path::new("/project/main.dowe");

    let formatted = format_document(Path::new("/project"), path, source).expect("format");
    assert_eq!(formatted, source);
    assert_eq!(
        format_document(Path::new("/project"), path, &formatted).expect("format twice"),
        source
    );
    assert!(!formatted.contains("views views:"));
}

#[test]
fn formatter_canonicalizes_comma_separated_arrays() {
    let path = Path::new("/project/main.dowe");
    let source = "main\n  views:[siteRoutes, docsRoutes]\n";

    let formatted = format_document(Path::new("/project"), path, source).expect("format");

    assert_eq!(formatted, "main\n  views:[siteRoutes docsRoutes]\n");
    assert_eq!(
        format_document(Path::new("/project"), path, &formatted).expect("format twice"),
        formatted
    );
}

#[test]
fn formatter_preserves_each_property_syntax() {
    let path = Path::new("/project/pages/catalog.dowe");
    let source = "page catalog\n  signal items value:[]\n  each in:items as:item key:item.id\n    Text\n      \"item.id\"\n";
    let formatted = format_document(Path::new("/project"), path, source).expect("format");
    assert_eq!(formatted, source);
    assert_eq!(
        format_document(Path::new("/project"), path, &formatted).expect("format twice"),
        formatted
    );
}

#[test]
fn language_supports_web_metadata_declarations() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("views/pages")).expect("pages");
    let path = root.path().join("views/pages/home.dowe");
    let valid = LanguageDocument {
        path: path.clone(),
        source: "page HomePage\n  meta name:\"title\" content:\"Dowe\"\n  Text\n    \"Home\"\n"
            .to_string(),
    };
    assert!(analyze_document(root.path(), &valid).is_empty());

    let props = complete_document(
        root.path(),
        &LanguageDocument {
            path: path.clone(),
            source: "page HomePage\n  meta \n".to_string(),
        },
        2,
        8,
    );
    assert!(props.iter().any(|item| item.label == "name"));
    assert!(props.iter().any(|item| item.label == "content"));

    let names = complete_document(
        root.path(),
        &LanguageDocument {
            path: path.clone(),
            source: "page HomePage\n  meta name:\n".to_string(),
        },
        2,
        13,
    );
    assert!(names.iter().any(|item| item.label == "\"title\""));
    assert!(names.iter().any(|item| item.label == "\"og:image\""));
    assert!(names.iter().any(|item| item.label == "\"twitter:card\""));

    let hover = hover_at(root.path(), &valid, 2, 4).expect("meta hover");
    assert!(hover.contains("web metadata declaration"));
    let name_hover = hover_at(root.path(), &valid, 2, 9).expect("name hover");
    assert!(name_hover.contains("metadata identifier"));
    let content_hover = hover_at(root.path(), &valid, 2, 22).expect("content hover");
    assert!(content_hover.contains("metadata value"));
}

#[test]
fn formatter_indents_multiline_child_delimiters() {
    let path = Path::new("/project/pages/title.dowe");
    let source = "page titlePage\n  Title\n    \"\"\"\n    Hello,\n    world\n    \"\"\"\n";
    let formatted = format_document(Path::new("/project"), path, source).expect("format");
    assert_eq!(
        formatted,
        "page titlePage\n  Title\n    \"\"\"\n    Hello,\n    world\n    \"\"\"\n"
    );
    assert_eq!(
        format_document(Path::new("/project"), path, &formatted).expect("format twice"),
        formatted
    );
}

#[test]
fn formatter_preserves_multiline_string_content() {
    let path = Path::new("/project/pages/code.dowe");
    let source = "page codePage\n  Code:\n    language:\"dowe\"\n    content:\"\"\"\n      page example\n        Text\n          \"Hello\"\n\n        Button\n          \"Continue\"\n    \"\"\"\n";
    let formatted = format_document(Path::new("/project"), path, source).expect("format");
    assert_eq!(formatted, source);
    assert_eq!(
        format_document(Path::new("/project"), path, &formatted).expect("format twice"),
        formatted
    );
}

#[test]
fn formatter_rejects_misaligned_multiline_string_closing_delimiter() {
    let error = format_document(
        Path::new("/project"),
        Path::new("/project/pages/code.dowe"),
        "page codePage\n  Code:\n    content:\"\"\"\n      page example\n      \"\"\"\n",
    )
    .expect_err("misaligned delimiter");
    assert!(error.to_string().contains("must align with its prop"));
}

#[test]
fn completions_include_each_props() {
    let document = LanguageDocument {
        path: PathBuf::from("/project/pages/catalog.dowe"),
        source: "page catalog\n  signal items value:[]\n  each \n".to_string(),
    };
    let completions = complete_document(Path::new("/project"), &document, 3, 8);
    for prop in ["in", "as", "key"] {
        assert!(completions.iter().any(|item| item.label == prop));
    }
}

#[test]
fn language_support_documents_brand_navigation_and_size() {
    let root = tempdir().expect("root");
    let document = LanguageDocument {
        path: root.path().join("pages/brand.dowe"),
        source: "page brandPage\n  Brand href:\"/\" label:\"Dowe home\" w:32 h:8\n    Text\n      \"Dowe\"\n"
            .to_string(),
    };
    let prop_document = LanguageDocument {
        path: root.path().join("pages/brand-props.dowe"),
        source: "page brandProps\n  Brand \n".to_string(),
    };
    let component_document = LanguageDocument {
        path: root.path().join("pages/brand-component.dowe"),
        source: "page brandComponent\n  Bra\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);
    let props = complete_document(root.path(), &prop_document, 2, 9);
    let components = complete_document(root.path(), &component_document, 2, 6);
    let hover = hover_at(root.path(), &document, 2, 4).expect("Brand hover");

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    for prop in ["href", "label", "w", "h"] {
        assert!(props.iter().any(|item| item.label == prop), "{prop}");
    }
    assert!(components.iter().any(|item| item.label == "Brand"));
    assert!(hover.contains("cross-platform identity component"));
    assert!(hover.contains("one or more identity children"));
}

#[test]
fn language_support_documents_banner_external_navigation() {
    let root = tempdir().expect("root");
    let document = LanguageDocument {
        path: root.path().join("pages/banner.dowe"),
        source: "page bannerPage\n  Banner href:\"https://dowe.dev/cloud\" label:\"Explore Dowe Cloud\" p:6\n    Text\n      \"Build beyond code\"\n"
            .to_string(),
    };
    let prop_document = LanguageDocument {
        path: root.path().join("pages/banner-props.dowe"),
        source: "page bannerProps\n  Banner \n".to_string(),
    };
    let component_document = LanguageDocument {
        path: root.path().join("pages/banner-component.dowe"),
        source: "page bannerComponent\n  Bann\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);
    let props = complete_document(root.path(), &prop_document, 2, 10);
    let components = complete_document(root.path(), &component_document, 2, 7);
    let hover = hover_at(root.path(), &document, 2, 4).expect("Banner hover");

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    for prop in ["href", "label", "bg", "cover", "p", "w", "h"] {
        assert!(props.iter().any(|item| item.label == prop), "{prop}");
    }
    assert!(components.iter().any(|item| item.label == "Banner"));
    assert!(hover.contains("cross-platform external banner component"));
    assert!(hover.contains("one or more banner children"));
}

#[test]
fn formatter_rejects_unsafe_parse() {
    let error = format_document(
        Path::new("/project"),
        Path::new("/project/pages/login.dowe"),
        "page loginPage\n   Box\n",
    )
    .expect_err("error");

    assert!(
        error
            .to_string()
            .contains("indentation must use two spaces")
    );
}

#[test]
fn formatter_wraps_long_property_suites() {
    let source = "page canvasPage\n  Canvas scene:gameScene viewWidth:640 viewHeight:360 fit:\"contain\" fps:60 autoplay:true background:\"background\" label:\"Animated space game scene with a ship and moving asteroids\" w:\"full\" h:96 rounded:\"lg\" border:1 shadow:\"md\"\n";
    let formatted = format_document(
        Path::new("/project"),
        Path::new("/project/pages/canvas.dowe"),
        source,
    )
    .expect("formatted");

    assert_eq!(
        formatted,
        "page canvasPage\n  Canvas:\n    scene:gameScene\n    viewWidth:640\n    viewHeight:360\n    fit:\"contain\"\n    fps:60\n    autoplay:true\n    background:\"background\"\n    label:\"Animated space game scene with a ship and moving asteroids\"\n    w:\"full\"\n    h:96\n    rounded:\"lg\"\n    border:1\n    shadow:\"md\"\n"
    );
    assert_eq!(
        format_document(
            Path::new("/project"),
            Path::new("/project/pages/canvas.dowe"),
            &formatted,
        )
        .expect("formatted again"),
        formatted
    );
}

#[test]
fn formatter_accepts_nested_property_suite_children() {
    let source = "page heroPage\n  Grid:\n    columns:2\n    gap:4\n    Box show:false\n    Box:\n      cover:\"/hero.jpg\"\n      minH:96\n";
    let formatted = format_document(
        Path::new("/project"),
        Path::new("/project/pages/hero.dowe"),
        source,
    )
    .expect("formatted");

    assert_eq!(
        formatted,
        "page heroPage\n  Grid columns:2 gap:4\n    Box show:false\n    Box cover:\"/hero.jpg\" minH:96\n"
    );
    assert_eq!(
        format_document(
            Path::new("/project"),
            Path::new("/project/pages/hero.dowe"),
            &formatted,
        )
        .expect("formatted again"),
        formatted
    );
}

#[test]
fn formatter_preserves_grouped_theme_color_families() {
    let source = "theme\n  design defaultTheme:\"light\"\n    theme name:\"light\"\n      colors:\n        primary color:\"#1F3A5F\" text:\"#FFFFFF\" title:\"#FFFFFE\"\n        softPrimary:\n          color:\"#CCFBF3\"\n          text:\"#073B35\"\n          title:\"#073B35\"\n";
    let formatted = format_document(
        Path::new("/project"),
        Path::new("/project/theme.dowe"),
        source,
    )
    .expect("formatted theme");

    assert_eq!(
        formatted,
        "theme\n  design defaultTheme:\"light\"\n    theme name:\"light\"\n      colors:\n        primary color:\"#1F3A5F\" text:\"#FFFFFF\" title:\"#FFFFFE\"\n        softPrimary color:\"#CCFBF3\" text:\"#073B35\" title:\"#073B35\"\n"
    );
    assert_eq!(
        format_document(
            Path::new("/project"),
            Path::new("/project/theme.dowe"),
            &formatted,
        )
        .expect("formatted theme again"),
        formatted
    );
}

#[test]
fn completes_project_defined_color_families_as_component_schemes() {
    let root = tempdir().expect("root");
    fs::write(
        root.path().join("theme.dowe"),
        r##"theme
  design defaultTheme:"light"
    theme name:"light"
      colors:
        happy color:"#176c75" text:"#fffffe" title:"#fffffe"
        softHappy color:"#d9f3f1" text:"#124d53" title:"#124d53""##,
    )
    .expect("theme");
    let document = LanguageDocument {
        path: root.path().join("views/pages/status.dowe"),
        source: "page statusPage\n  Card scheme:\n".to_string(),
    };

    let completions = complete_document(root.path(), &document, 2, 15);

    assert!(completions.iter().any(|item| item.label == "\"happy\""));
    assert!(!completions.iter().any(|item| item.label == "\"softHappy\""));
}

#[test]
fn formatter_expands_nested_multiline_values() {
    let source = "page canvasPage\n  signal gameScene value:[{ type:\"rect\" x:0 y:0 width:640 height:360 fill:\"background\" },{ type:\"circle\" x:40 y:50 radius:2 fill:\"backgroundText\" opacity:0.5 motion:{ vx:-18 wrap:true } }]\n";
    let formatted = format_document(
        Path::new("/project"),
        Path::new("/project/pages/canvas.dowe"),
        source,
    )
    .expect("formatted");

    assert_eq!(
        formatted,
        "page canvasPage\n  signal gameScene:\n    value:[\n      {\n        type:\"rect\"\n        x:0\n        y:0\n        width:640\n        height:360\n        fill:\"background\"\n      }\n      {\n        type:\"circle\"\n        x:40\n        y:50\n        radius:2\n        fill:\"backgroundText\"\n        opacity:0.5\n        motion:{ vx:-18 wrap:true }\n      }\n    ]\n"
    );
    assert_eq!(
        format_document(
            Path::new("/project"),
            Path::new("/project/pages/canvas.dowe"),
            &formatted,
        )
        .expect("formatted again"),
        formatted
    );
}

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

#[test]
fn diagnostics_and_completions_support_reactive_button_props() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("pages/button.dowe"),
        source: "page buttonPage\n  signal variantChoice value:\"solid\"\n  signal loading value:false\n  signal iconVisible value:true\n  Button variant:variantChoice loading:loading iconStart:{ when:iconVisible value:\"add-circle\" }\n    \"Create\"\n".to_string(),
    };
    let diagnostics = analyze_document(root.path(), &document);
    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn diagnostics_reject_non_boolean_button_loading_signal() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("pages/button.dowe"),
        source: "page buttonPage\n  signal loading value:\"pending\"\n  Button loading:loading\n    \"Create\"\n".to_string(),
    };
    let diagnostics = analyze_document(root.path(), &document);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("expected bool")),
        "expected bool loading diagnostic: {diagnostics:?}"
    );
}

#[test]
fn completes_namespaced_svg_spinner_icon_names() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("views/pages")).expect("pages");
    let document = LanguageDocument {
        path: root.path().join("views/pages/loading.dowe"),
        source: "page LoadingPage\n  Icon name:\n".to_string(),
    };
    let completions = complete_document(root.path(), &document, 2, "  Icon name:".len() + 1);

    assert!(
        completions
            .iter()
            .any(|completion| completion.label == "\"svg-spinners:3-dots-bounce\"")
    );
}

#[test]
fn completes_solar_variant_names_without_icon_style_prop() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("views/pages")).expect("pages");
    let names_document = LanguageDocument {
        path: root.path().join("views/pages/icons.dowe"),
        source: "page IconPage\n  Icon name:\n".to_string(),
    };
    let names = complete_document(root.path(), &names_document, 2, "  Icon name:".len() + 1);
    assert!(
        names
            .iter()
            .any(|completion| completion.label == "\"alt-arrow-right-bold-duotone\"")
    );

    let props_document = LanguageDocument {
        path: root.path().join("views/pages/icons.dowe"),
        source: "page IconPage\n  Icon \n".to_string(),
    };
    let props = complete_document(root.path(), &props_document, 2, "  Icon ".len() + 1);
    assert!(props.iter().any(|completion| completion.label == "name"));
    assert!(props.iter().any(|completion| completion.label == "fill"));
    assert!(!props.iter().any(|completion| completion.label == "style"));

    let removed = LanguageDocument {
        path: root.path().join("views/pages/icons.dowe"),
        source: "page IconPage\n  Icon name:\"alt-arrow-right\" style:\"bold\"\n".to_string(),
    };
    let diagnostics = analyze_document(root.path(), &removed);
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("include the Solar variant in name")),
        "expected removed Icon style diagnostic: {diagnostics:?}"
    );
}

#[test]
fn completes_namespaced_svg_logo_icon_names() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("views/pages")).expect("pages");
    let document = LanguageDocument {
        path: root.path().join("views/pages/brands.dowe"),
        source: "page BrandPage\n  Icon name:\n".to_string(),
    };
    let completions = complete_document(root.path(), &document, 2, "  Icon name:".len() + 1);

    assert!(
        completions
            .iter()
            .any(|completion| completion.label == "\"svg-logos:github-icon\"")
    );
}

#[test]
fn diagnostics_accept_namespaced_svg_logo_icon_names() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("views/pages")).expect("pages");
    let document = LanguageDocument {
        path: root.path().join("views/pages/home-page.dowe"),
        source: "page homePage\n  Flex align:\"center\" gap:3\n    Icon name:\"svg-logos:android-icon\" w:10 h:10\n"
            .to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);
    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn diagnostics_and_completions_support_runtime_svg_data() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("views/pages")).expect("pages");
    let document = LanguageDocument {
        path: root.path().join("views/pages/icons.dowe"),
        source: "page IconPage\n  signal iconData value:\"runtime-svg-json\"\n  Svg data:iconData w:8 h:8\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);
    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );

    let completion_document = LanguageDocument {
        path: root.path().join("views/pages/icons.dowe"),
        source: "page IconPage\n  Svg \n".to_string(),
    };
    let completions = complete_document(root.path(), &completion_document, 2, "  Svg ".len() + 1);
    assert!(completions.iter().any(|item| item.label == "data"));
}

#[test]
fn diagnostics_support_reactive_side_nav_props() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("pages/navigation.dowe"),
        source: "page navigationPage\n  signal variantChoice value:\"ghost\"\n  signal schemeChoice value:\"muted\"\n  signal sizeChoice value:\"md\"\n  signal wideEnabled value:true\n  SideNav variant:variantChoice scheme:schemeChoice size:sizeChoice wide:wideEnabled\n    item label:\"Overview\" href:\"/overview\"\n".to_string(),
    };
    let diagnostics = analyze_document(root.path(), &document);
    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn diagnostics_report_unquoted_static_text_children() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("pages/login.dowe"),
        source: "page loginPage\n  Title\n    header\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("quoted static string literal"))
        .expect("quoted text child diagnostic");

    assert_eq!(
        diagnostic.message,
        "3:5: text child `header` must be a quoted static string literal"
    );
    assert_eq!(diagnostic.range, LanguageRange::single_line(3, 5, 6));
}

#[test]
fn diagnostics_validate_braced_view_text_bindings() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("pages");
    let path = root.path().join("pages/blogs.dowe");
    let valid = LanguageDocument {
        path,
        source: "page blogsPage\n  signal blog value:{ title:\"Title\" }\n  Text\n    \"{blog.title}\"\n  Text\n    \"blog.title\"\n  Text\n    \"Title: {blog.title}\"\n"
            .to_string(),
    };

    assert!(analyze_document(root.path(), &valid).is_empty());
}

#[test]
fn diagnostics_validate_translation_catalogs() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("i18n")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("i18n/en.dowe"),
        source: "translations default:true\n  translation key:\"home.hero.title\" value:\"Dowe\"\n  translation key:\"home.hero.title\" value:\"Dowe\"\n"
            .to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("duplicate translation key"))
    );
}

#[test]
fn diagnostics_accept_svg_paths() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("pages/login.dowe"),
        source: "page loginPage\n  Svg viewBox:\"0 0 24 24\" color:\"accent\" w:8 h:8\n    Path d:\"M0 0h24v24H0z\" fill:\"none\"\n    Path d:\"M3.5 12a8.5 8.5 0 1 1 17 0\" fill:\"currentColor\"\n"
            .to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn completions_include_svg_path_fill_rule() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/icon.dowe").to_path_buf(),
        source: "page iconPage\n  Path \n  Path fillRule:\n".to_string(),
    };

    let props = complete_document(Path::new("/project"), &document, 2, 8);
    assert!(props.iter().any(|item| item.label == "fillRule"));

    let values = complete_document(Path::new("/project"), &document, 3, 17);
    assert!(values.iter().any(|item| item.label == "\"nonzero\""));
    assert!(values.iter().any(|item| item.label == "\"evenodd\""));
}

#[test]
fn diagnostics_place_component_prop_errors_on_prop_token() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("pages/login.dowe"),
        source: "page loginPage\n  Input variant:\"solid\" unknownLabel:test\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "DOWE_PROP")
        .expect("prop diagnostic");

    assert_eq!(
        diagnostic.message,
        "2:25: unknown prop `unknownLabel` on `Input`"
    );
    assert_eq!(diagnostic.range, LanguageRange::single_line(2, 25, 12));
}

#[test]
fn diagnostics_accept_each_item_icon_references() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("pages");
    let document = LanguageDocument {
        path: root.path().join("pages/catalog.dowe"),
        source: "page catalogPage\n  const catalogSources:\n    value:[{ id:\"solar\" name:\"layers-line-duotone\" fill:\"primary\" title:\"Solar\" description:\"Description\" }]\n  Grid\n    each in:catalogSources as:catalog key:catalog.id\n      Icon name:catalog.name fill:catalog.fill w:9 h:9\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);
    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn diagnostics_report_unquoted_static_component_strings() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("pages/login.dowe"),
        source:
            "page loginPage\n  Svg viewBox:\"0 0 24 24\"\n    Path d:\"M0 0h24v24H0z\" fill:none\n"
                .to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "DOWE_PROP")
        .expect("quoted string diagnostic");

    assert!(
        diagnostic
            .message
            .contains("invalid value for prop `fill`: expected quoted static string literal")
    );
    assert_eq!(diagnostic.range.start.line, 3);

    let enum_document = LanguageDocument {
        path: root.path().join("pages/login.dowe"),
        source: "page loginPage\n  Input variant:outlined scheme:primary\n".to_string(),
    };
    let enum_diagnostics = analyze_document(root.path(), &enum_document);
    assert!(
        enum_diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "DOWE_PROP"),
        "unexpected diagnostics for accepted enum values: {enum_diagnostics:?}"
    );
}

#[test]
fn diagnostics_report_unquoted_static_config_strings() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("theme.dowe"),
        source: "theme\n  fonts default:inter install:[\"inter\"]\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "DOWE_PROP")
        .expect("config quoted string diagnostic");

    assert!(
        diagnostic
            .message
            .contains("invalid value for prop `default`: expected quoted static string literal")
    );
    assert_eq!(diagnostic.range.start.line, 2);
}

#[test]
fn diagnostics_accept_input_and_select_form_props() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("pages/login.dowe"),
        source: "page loginPage\n  signal profile value:{ name:\"\" role:\"admin\" }\n  Box\n    Input bind:profile.name label:\"Name\" placeholder:\"Full name\" labelFloating:true size:\"sm\"\n    Select bind:profile.role label:\"Role\" placeholder:\"Choose role\" labelFloating:true size:\"lg\"\n      Option value:\"admin\" label:\"Admin\"\n      Option value:\"viewer\" label:\"Viewer\"\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

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

#[test]
fn completions_include_current_view_component_props() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/login.dowe").to_path_buf(),
        source: "page loginPage\n  Box \n  Section \n  Text \n  Card \n  Button \n  Input \n  Alert \n  Svg \n  Path \n  Select \n    Option \n  Video \n  Divider \n  Tabs \n  Drawer \n"
            .to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "Text"));
    assert!(base.iter().any(|item| item.label == "Section"));
    assert!(base.iter().any(|item| item.label == "Svg"));
    assert!(base.iter().any(|item| item.label == "Select"));
    assert!(base.iter().any(|item| item.label == "AppBar"));
    assert!(base.iter().any(|item| item.label == "Footer"));
    assert!(base.iter().any(|item| item.label == "BottomBar"));
    assert!(base.iter().any(|item| item.label == "SideNav"));
    assert!(base.iter().any(|item| item.label == "RailNav"));
    assert!(base.iter().any(|item| item.label == "Tabs"));
    assert!(base.iter().any(|item| item.label == "Video"));
    assert!(base.iter().any(|item| item.label == "Divider"));
    assert!(base.iter().any(|item| item.label == "Drawer"));
    assert!(base.iter().any(|item| item.label == "ToggleTheme"));
    assert!(base.iter().any(|item| item.label == "SelectTheme"));
    assert!(base.iter().any(|item| item.label == "Fab"));
    assert!(base.iter().any(|item| item.label == "fabAction"));
    assert!(base.iter().any(|item| item.label == "Slider"));
    assert!(base.iter().any(|item| item.label == "Dropzone"));
    assert!(!base.iter().any(|item| item.label == "Body"));

    let box_props = complete_document(Path::new("/project"), &document, 2, 7);
    assert!(box_props.iter().any(|item| item.label == "color"));
    assert!(box_props.iter().any(|item| item.label == "animation"));
    assert!(box_props.iter().any(|item| item.label == "position"));
    assert!(box_props.iter().any(|item| item.label == "top"));
    assert!(box_props.iter().any(|item| item.label == "right"));
    assert!(box_props.iter().any(|item| item.label == "bottom"));
    assert!(box_props.iter().any(|item| item.label == "left"));
    assert!(box_props.iter().any(|item| item.label == "maxW"));
    assert!(box_props.iter().any(|item| item.label == "maxH"));
    assert!(box_props.iter().any(|item| item.label == "flex"));
    assert!(!box_props.iter().any(|item| item.label == "text"));

    let section_props = complete_document(Path::new("/project"), &document, 3, 11);
    assert!(section_props.iter().any(|item| item.label == "background"));
    assert!(section_props.iter().any(|item| item.label == "centerX"));
    assert!(section_props.iter().any(|item| item.label == "gap"));
    assert!(section_props.iter().any(|item| item.label == "flex"));
    assert!(section_props.iter().any(|item| item.label == "boxed"));
    assert!(section_props.iter().any(|item| item.label == "cover"));
    assert!(section_props.iter().any(|item| item.label == "color"));

    let text_props = complete_document(Path::new("/project"), &document, 4, 8);
    assert!(text_props.iter().any(|item| item.label == "color"));
    assert!(text_props.iter().any(|item| item.label == "i18n"));
    assert!(!text_props.iter().any(|item| item.label == "text"));

    for (line, column) in [(5, 8), (6, 10), (7, 9), (8, 9)] {
        let props = complete_document(Path::new("/project"), &document, line, column);
        assert!(props.iter().any(|item| item.label == "scheme"));
        assert!(!props.iter().any(|item| item.label == "color"));
        assert!(!props.iter().any(|item| item.label == "text"));
    }
    let button_props = complete_document(Path::new("/project"), &document, 6, 10);
    assert!(button_props.iter().any(|item| item.label == "iconStart"));
    assert!(button_props.iter().any(|item| item.label == "iconEnd"));
    assert!(button_props.iter().any(|item| item.label == "loading"));
    assert!(
        !button_props
            .iter()
            .any(|item| item.label == "showIconStart")
    );
    for (line, column) in [(5, 8), (8, 9)] {
        let props = complete_document(Path::new("/project"), &document, line, column);
        assert!(!props.iter().any(|item| item.label == "size"));
    }
    let input_props = complete_document(Path::new("/project"), &document, 7, 9);
    assert!(input_props.iter().any(|item| item.label == "size"));
    let card_props = complete_document(Path::new("/project"), &document, 5, 8);
    assert!(card_props.iter().any(|item| item.label == "animation"));

    let svg_props = complete_document(Path::new("/project"), &document, 9, 7);
    assert!(svg_props.iter().any(|item| item.label == "viewBox"));
    assert!(svg_props.iter().any(|item| item.label == "color"));

    let path_props = complete_document(Path::new("/project"), &document, 10, 8);
    assert!(path_props.iter().any(|item| item.label == "d"));
    assert!(path_props.iter().any(|item| item.label == "fill"));

    let select_props = complete_document(Path::new("/project"), &document, 11, 10);
    assert!(select_props.iter().any(|item| item.label == "label"));
    assert!(select_props.iter().any(|item| item.label == "placeholder"));
    assert!(select_props.iter().any(|item| item.label == "size"));

    let option_props = complete_document(Path::new("/project"), &document, 12, 12);
    assert!(option_props.iter().any(|item| item.label == "value"));
    assert!(option_props.iter().any(|item| item.label == "description"));

    let video_props = complete_document(Path::new("/project"), &document, 13, 10);
    assert!(video_props.iter().any(|item| item.label == "src"));
    assert!(video_props.iter().any(|item| item.label == "poster"));
    assert!(video_props.iter().any(|item| item.label == "aspect"));
    assert!(video_props.iter().any(|item| item.label == "scheme"));

    let divider_props = complete_document(Path::new("/project"), &document, 14, 11);
    assert!(divider_props.iter().any(|item| item.label == "orientation"));
    assert!(divider_props.iter().any(|item| item.label == "scheme"));
    assert!(!divider_props.iter().any(|item| item.label == "variant"));

    let tabs_props = complete_document(Path::new("/project"), &document, 15, 8);
    assert!(tabs_props.iter().any(|item| item.label == "variant"));
    assert!(tabs_props.iter().any(|item| item.label == "scheme"));
    assert!(tabs_props.iter().any(|item| item.label == "position"));
    assert!(!tabs_props.iter().any(|item| item.label == "color"));

    let drawer_props = complete_document(Path::new("/project"), &document, 16, 10);
    assert!(drawer_props.iter().any(|item| item.label == "bind"));
    assert!(drawer_props.iter().any(|item| item.label == "position"));
    assert!(drawer_props.iter().any(|item| item.label == "scheme"));
    assert!(drawer_props.iter().any(|item| item.label == "show"));

    let bar_document = LanguageDocument {
        path: Path::new("/project/pages/bars.dowe").to_path_buf(),
        source: "page barsPage\n  AppBar \n  Footer \n  BottomBar \n  SideNav \n  Sidebar \n  RailNav \n"
            .to_string(),
    };
    let appbar_props = complete_document(Path::new("/project"), &bar_document, 2, 11);
    assert!(appbar_props.iter().any(|item| item.label == "floating"));
    assert!(appbar_props.iter().any(|item| item.label == "dockOnScroll"));
    assert!(appbar_props.iter().any(|item| item.label == "bordered"));

    let footer_props = complete_document(Path::new("/project"), &bar_document, 3, 10);
    assert!(footer_props.iter().any(|item| item.label == "boxed"));
    assert!(!footer_props.iter().any(|item| item.label == "floating"));

    let bottombar_props = complete_document(Path::new("/project"), &bar_document, 4, 14);
    assert!(bottombar_props.iter().any(|item| item.label == "floating"));

    let side_nav_props = complete_document(Path::new("/project"), &bar_document, 5, 12);
    assert!(side_nav_props.iter().any(|item| item.label == "scheme"));
    assert!(side_nav_props.iter().any(|item| item.label == "size"));
    assert!(side_nav_props.iter().any(|item| item.label == "wide"));

    let sidebar_props = complete_document(Path::new("/project"), &bar_document, 6, 12);
    assert!(sidebar_props.iter().any(|item| item.label == "scheme"));
    assert!(sidebar_props.iter().any(|item| item.label == "variant"));
    assert!(!sidebar_props.iter().any(|item| item.label == "size"));
    assert!(!sidebar_props.iter().any(|item| item.label == "wide"));

    let rail_nav_props = complete_document(Path::new("/project"), &bar_document, 7, 12);
    assert!(rail_nav_props.iter().any(|item| item.label == "scheme"));
    assert!(rail_nav_props.iter().any(|item| item.label == "size"));
    assert!(rail_nav_props.iter().any(|item| item.label == "showLabels"));

    let control_document = LanguageDocument {
        path: Path::new("/project/pages/controls.dowe").to_path_buf(),
        source:
            "page controlsPage\n  ToggleTheme \n  Fab \n    fabAction \n  Slider \n  Dropzone \n"
                .to_string(),
    };
    let theme_props = complete_document(Path::new("/project"), &control_document, 2, 15);
    assert!(theme_props.iter().any(|item| item.label == "lightLabel"));
    assert!(theme_props.iter().any(|item| item.label == "darkLabel"));
    assert!(theme_props.iter().any(|item| item.label == "scheme"));

    let fab_props = complete_document(Path::new("/project"), &control_document, 3, 7);
    assert!(fab_props.iter().any(|item| item.label == "position"));
    assert!(fab_props.iter().any(|item| item.label == "offsetX"));
    assert!(fab_props.iter().any(|item| item.label == "icon"));

    let fab_action_props = complete_document(Path::new("/project"), &control_document, 4, 15);
    assert!(fab_action_props.iter().any(|item| item.label == "href"));
    assert!(fab_action_props.iter().any(|item| item.label == "onClick"));
    assert!(
        fab_action_props
            .iter()
            .any(|item| item.label == "externalMode")
    );

    let slider_props = complete_document(Path::new("/project"), &control_document, 5, 10);
    assert!(slider_props.iter().any(|item| item.label == "bind"));
    assert!(slider_props.iter().any(|item| item.label == "hideLabel"));
    assert!(slider_props.iter().any(|item| item.label == "step"));

    let dropzone_props = complete_document(Path::new("/project"), &control_document, 6, 12);
    assert!(dropzone_props.iter().any(|item| item.label == "accept"));
    assert!(dropzone_props.iter().any(|item| item.label == "maxSize"));
    assert!(dropzone_props.iter().any(|item| item.label == "errorText"));
}

#[test]
fn completions_include_container_width_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/widths.dowe").to_path_buf(),
        source: "page widthsPage\n  Box w:\n".to_string(),
    };
    let completions = complete_document(Path::new("/project"), &document, 2, 9);
    for value in [
        "full", "sm", "md", "lg", "xl", "2xl", "3xl", "4xl", "5xl", "6xl", "7xl", "10%", "20%",
        "30%", "40%", "50%", "60%", "70%", "80%", "90%", "100%",
    ] {
        assert!(
            completions
                .iter()
                .any(|item| item.label == format!("\"{value}\"")),
            "missing width value {value}"
        );
    }

    let max_width_document = LanguageDocument {
        path: Path::new("/project/pages/widths.dowe").to_path_buf(),
        source: "page widthsPage\n  Box maxW:\n".to_string(),
    };
    let max_width_completions =
        complete_document(Path::new("/project"), &max_width_document, 2, 12);
    assert!(
        max_width_completions
            .iter()
            .all(|item| item.label != "\"50%\"")
    );
}

#[test]
fn completions_and_diagnostics_support_box_positioning() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("pages");
    let document = LanguageDocument {
        path: root.path().join("pages/positioning.dowe"),
        source: "page positioningPage\n  Box position:\"relative\"\n    Box position:\n"
            .to_string(),
    };
    let values = complete_document(root.path(), &document, 3, "    Box position:".len() + 1);
    for value in ["\"static\"", "\"relative\"", "\"absolute\"", "\"fixed\""] {
        assert!(values.iter().any(|item| item.label == value));
    }

    let valid = LanguageDocument {
        path: root.path().join("pages/positioning-valid.dowe"),
        source: "page positioningPage\n  Box position:\"relative\" minH:64 maxW:{ xs:\"full\" md:64 } maxH:\"vh-16\"\n    Box position:\"absolute\" top:4 right:{ xs:4 md:6 }\n      Text\n        \"Proof\"\n"
            .to_string(),
    };
    assert!(
        analyze_document(root.path(), &valid).is_empty(),
        "valid positioned Box should have no diagnostics"
    );
}

#[test]
fn completions_include_section_boxed_boolean_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/landing.dowe").to_path_buf(),
        source: "page landingPage\n  Section boxed:\n".to_string(),
    };

    let values = complete_document(
        Path::new("/project"),
        &document,
        2,
        "  Section boxed:".len() + 1,
    );
    assert!(values.iter().any(|item| item.label == "true"));
    assert!(values.iter().any(|item| item.label == "false"));
}

#[test]
fn completions_include_section_center_boolean_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/landing.dowe").to_path_buf(),
        source: "page landingPage\n  Section centerX:\n".to_string(),
    };

    let values = complete_document(
        Path::new("/project"),
        &document,
        2,
        "  Section centerX:".len() + 1,
    );
    assert!(values.iter().any(|item| item.label == "true"));
    assert!(values.iter().any(|item| item.label == "false"));
}

#[test]
fn completions_include_section_gap_prop() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/landing.dowe").to_path_buf(),
        source: "page landingPage\n  Section \n".to_string(),
    };

    let values = complete_document(Path::new("/project"), &document, 2, "  Section ".len() + 1);
    assert!(values.iter().any(|item| item.label == "gap"));
}

#[test]
fn completions_and_hover_use_page_for_view_routes() {
    let document = LanguageDocument {
        path: Path::new("/project/routes/view.dowe").to_path_buf(),
        source: "views viewRoutes\n  group path:\"/\" layout:RootLayout\n    route \n".to_string(),
    };

    let route_props = complete_document(Path::new("/project"), &document, 3, 11);

    assert!(route_props.iter().any(|item| item.label == "page"));
    assert!(!route_props.iter().any(|item| item.label == "component"));
    assert!(
        hover_at(
            Path::new("/project"),
            &LanguageDocument {
                path: document.path,
                source: "views viewRoutes\n  route path:\"/\" page:homePage\n".to_string(),
            },
            2,
            19,
        )
        .expect("page hover")
        .contains("route.page")
    );
}

#[test]
fn completions_follow_multiline_property_suite_owner() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/canvas.dowe").to_path_buf(),
        source: "page canvasPage\n  Canvas:\n    scene:gameScene\n    \n    fit:\n".to_string(),
    };

    let props = complete_document(Path::new("/project"), &document, 4, 5);
    assert!(props.iter().any(|item| item.label == "viewWidth"));
    assert!(props.iter().any(|item| item.label == "label"));

    let values = complete_document(Path::new("/project"), &document, 5, 9);
    assert!(values.iter().any(|item| item.label == "\"contain\""));
    assert!(values.iter().any(|item| item.label == "\"cover\""));
}

#[test]
fn completions_include_translation_keys() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("i18n")).expect("i18n");
    fs::write(
        root.path().join("i18n/en.dowe"),
        "translations default:true\n  home\n    hero\n      title \"Dowe builds systems.\"\n",
    )
    .expect("english");
    fs::write(
        root.path().join("i18n/es.dowe"),
        "translations\n  home\n    hero\n      title \"Dowe construye sistemas.\"\n",
    )
    .expect("spanish");
    let document = LanguageDocument {
        path: root.path().join("pages/home.dowe"),
        source: "page homePage\n  Title i18n:\n    Dowe builds systems.\n".to_string(),
    };

    let completions = complete_document(root.path(), &document, 2, 14);

    assert!(
        completions
            .iter()
            .any(|item| item.label == "\"home.hero.title\"")
    );
}

#[test]
fn completions_include_quoted_static_component_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/login.dowe").to_path_buf(),
        source: "page loginPage\n  Input scheme:\n  Input variant:\n  Path fill:\n  Button navigate:\n  Alert type:\n  AppBar scheme:\n  Text weight:\n  SideNav size:\n  SideNav scheme:\n  Divider orientation:\n  Divider scheme:\n  Drawer position:\n  Drawer scheme:\n  Tabs variant:\n  Tabs position:\n  Tabs scheme:\n  Carousel variant:\n"
            .to_string(),
    };

    let scheme = complete_document(Path::new("/project"), &document, 2, 16);
    assert!(scheme.iter().any(|item| item.label == "\"primary\""));
    assert!(scheme.iter().any(|item| item.label == "\"secondary\""));
    assert!(!scheme.iter().any(|item| item.label == "primary"));
    assert!(!scheme.iter().any(|item| item.label == "\"surface\""));
    assert!(
        scheme
            .iter()
            .all(|item| item.kind == LanguageCompletionKind::Value)
    );

    let variant = complete_document(Path::new("/project"), &document, 3, 17);
    assert!(variant.iter().any(|item| item.label == "\"outlined\""));
    assert!(variant.iter().any(|item| item.label == "\"ghost\""));

    let fill = complete_document(Path::new("/project"), &document, 4, 13);
    assert!(fill.iter().any(|item| item.label == "\"none\""));
    assert!(fill.iter().any(|item| item.label == "\"currentColor\""));
    assert!(fill.iter().any(|item| item.label == "\"accent\""));

    let navigate = complete_document(Path::new("/project"), &document, 5, 19);
    assert!(navigate.iter().any(|item| item.label == "\"push\""));
    assert!(navigate.iter().any(|item| item.label == "\"replace\""));

    let alert_type = complete_document(Path::new("/project"), &document, 6, 14);
    assert!(alert_type.iter().any(|item| item.label == "\"warning\""));

    let appbar_scheme = complete_document(Path::new("/project"), &document, 7, 18);
    assert!(appbar_scheme.iter().any(|item| item.label == "\"surface\""));
    assert!(
        appbar_scheme
            .iter()
            .any(|item| item.label == "\"background\"")
    );

    let text_weight = complete_document(Path::new("/project"), &document, 8, 15);
    assert!(text_weight.iter().any(|item| item.label == "\"thin\""));
    assert!(
        text_weight
            .iter()
            .any(|item| item.label == "\"extralight\"")
    );
    assert!(text_weight.iter().any(|item| item.label == "\"black\""));

    let side_nav_size = complete_document(Path::new("/project"), &document, 9, 16);
    assert!(side_nav_size.iter().any(|item| item.label == "\"sm\""));
    assert!(side_nav_size.iter().any(|item| item.label == "\"md\""));
    assert!(side_nav_size.iter().any(|item| item.label == "\"lg\""));

    let side_nav_scheme = complete_document(Path::new("/project"), &document, 10, 18);
    assert!(side_nav_scheme.iter().any(|item| item.label == "\"muted\""));
    assert!(
        !side_nav_scheme
            .iter()
            .any(|item| item.label == "\"surface\"" || item.label == "\"background\"")
    );

    let divider_orientation = complete_document(Path::new("/project"), &document, 11, 23);
    assert!(
        divider_orientation
            .iter()
            .any(|item| item.label == "\"horizontal\"")
    );
    assert!(
        divider_orientation
            .iter()
            .any(|item| item.label == "\"vertical\"")
    );

    let divider_scheme = complete_document(Path::new("/project"), &document, 12, 18);
    assert!(
        divider_scheme
            .iter()
            .any(|item| item.label == "\"surface\"")
    );

    let drawer_position = complete_document(Path::new("/project"), &document, 13, 19);
    assert!(drawer_position.iter().any(|item| item.label == "\"start\""));
    assert!(
        drawer_position
            .iter()
            .any(|item| item.label == "\"bottom\"")
    );

    let drawer_scheme = complete_document(Path::new("/project"), &document, 14, 17);
    assert!(drawer_scheme.iter().any(|item| item.label == "\"surface\""));

    let tabs_variant = complete_document(Path::new("/project"), &document, 15, 17);
    assert!(tabs_variant.iter().any(|item| item.label == "\"line\""));
    assert!(tabs_variant.iter().any(|item| item.label == "\"pills\""));

    let tabs_position = complete_document(Path::new("/project"), &document, 16, 18);
    assert!(tabs_position.iter().any(|item| item.label == "\"start\""));
    assert!(tabs_position.iter().any(|item| item.label == "\"bottom\""));

    let tabs_scheme = complete_document(Path::new("/project"), &document, 17, 17);
    assert!(tabs_scheme.iter().any(|item| item.label == "\"surface\""));

    let carousel_variant = complete_document(
        Path::new("/project"),
        &document,
        18,
        "  Carousel variant:".len() + 1,
    );
    for value in [
        "simple",
        "snapping",
        "masonry",
        "rtl",
        "sticky",
        "controls",
        "dots",
        "thumbnails",
        "coverFlow",
        "slideshow",
        "stories",
        "smartStack",
        "cardStack",
        "flipbook",
    ] {
        assert!(
            carousel_variant
                .iter()
                .any(|item| item.label == format!("\"{value}\""))
        );
    }

    let control_document = LanguageDocument {
        path: Path::new("/project/pages/controls.dowe").to_path_buf(),
        source: "page controlsPage\n  Fab position:\n  Fab icon:\n  fabAction icon:\n  Slider size:\n  Dropzone scheme:\n  Dropzone variant:\n"
            .to_string(),
    };
    let fab_position = complete_document(Path::new("/project"), &control_document, 2, 16);
    assert!(fab_position.iter().any(|item| item.label == "\"top-left\""));
    assert!(
        fab_position
            .iter()
            .any(|item| item.label == "\"bottom-right\"")
    );

    let fab_icon = complete_document(Path::new("/project"), &control_document, 3, 12);
    assert!(fab_icon.iter().any(|item| item.label == "\"settings\""));
    assert!(fab_icon.iter().any(|item| item.label == "\"moon\""));

    let action_icon = complete_document(Path::new("/project"), &control_document, 4, 18);
    assert!(action_icon.iter().any(|item| item.label == "\"link\""));
    assert!(action_icon.iter().any(|item| item.label == "\"upload\""));

    let slider_size = complete_document(Path::new("/project"), &control_document, 5, 15);
    assert!(slider_size.iter().any(|item| item.label == "\"sm\""));
    assert!(slider_size.iter().any(|item| item.label == "\"lg\""));
    assert!(!slider_size.iter().any(|item| item.label == "\"xl\""));

    let dropzone_scheme = complete_document(Path::new("/project"), &control_document, 6, 19);
    assert!(
        dropzone_scheme
            .iter()
            .any(|item| item.label == "\"surface\"")
    );
    assert!(
        dropzone_scheme
            .iter()
            .any(|item| item.label == "\"background\"")
    );

    let dropzone_variant = complete_document(Path::new("/project"), &control_document, 7, 20);
    assert!(
        dropzone_variant
            .iter()
            .any(|item| item.label == "\"ghost\"")
    );
    assert!(
        dropzone_variant
            .iter()
            .any(|item| item.label == "\"outlined\"")
    );

    let radio_document = LanguageDocument {
        path: Path::new("/project/pages/radio.dowe").to_path_buf(),
        source: "page radioPage\n  RadioGroup \n  RadioGroup orientation:\n".to_string(),
    };
    let radio_props = complete_document(Path::new("/project"), &radio_document, 2, 14);
    assert!(radio_props.iter().any(|item| item.label == "orientation"));
    assert!(radio_props.iter().any(|item| item.label == "scheme"));
    let radio_orientation = complete_document(Path::new("/project"), &radio_document, 3, 26);
    assert!(
        radio_orientation
            .iter()
            .any(|item| item.label == "\"vertical\"")
    );
    assert!(
        radio_orientation
            .iter()
            .any(|item| item.label == "\"horizontal\"")
    );
}

#[test]
fn completions_include_stepper_props_and_orientation_values() {
    let root = Path::new("/project");
    let component_document = LanguageDocument {
        path: Path::new("/project/pages/onboarding.dowe").to_path_buf(),
        source: "page onboardingPage\n  Stepper \n    step \n".to_string(),
    };
    let stepper = complete_document(root, &component_document, 2, 11);
    assert!(stepper.iter().any(|item| item.label == "scheme"));
    assert!(stepper.iter().any(|item| item.label == "orientation"));
    let step = complete_document(root, &component_document, 3, 10);
    assert!(step.iter().any(|item| item.label == "id"));
    assert!(step.iter().any(|item| item.label == "label"));

    let value_document = LanguageDocument {
        path: Path::new("/project/pages/onboarding.dowe").to_path_buf(),
        source: "page onboardingPage\n  Stepper orientation:\n".to_string(),
    };
    let values = complete_document(root, &value_document, 2, 23);
    assert!(values.iter().any(|item| item.label == "\"horizontal\""));
    assert!(values.iter().any(|item| item.label == "\"vertical\""));
}

#[test]
fn completions_include_appbar_position_prop_and_values() {
    let props_document = LanguageDocument {
        path: Path::new("/project/layouts/main.dowe").to_path_buf(),
        source: "layout MainLayout\n  AppBar \n".to_string(),
    };
    let props = complete_document(Path::new("/project"), &props_document, 2, 10);
    assert!(props.iter().any(|item| item.label == "position"));

    let values_document = LanguageDocument {
        path: Path::new("/project/layouts/main.dowe").to_path_buf(),
        source: "layout MainLayout\n  AppBar position:\n".to_string(),
    };
    let values = complete_document(Path::new("/project"), &values_document, 2, 20);
    for value in ["\"static\"", "\"sticky\"", "\"fixed\""] {
        assert!(values.iter().any(|item| item.label == value));
    }
}

#[test]
fn completions_include_display_overlay_component_props_and_values() {
    let source = [
        "page overlayPage",
        "  Avatar ",
        "  Avatar status:",
        "  Avatar scheme:",
        "  Badge position:",
        "  Chip variant:",
        "  Chip startIcon:",
        "  Chip endIcon:",
        "  Skeleton variant:",
        "  Skeleton animation:",
        "  Modal scheme:",
        "  AlertDialog variant:",
        "  Tooltip position:",
        "  Toast type:",
        "  Toast variant:",
        "  Dropdown ",
        "  Command ",
        "  Command variant:",
        "  item ",
        "  group ",
    ]
    .join("\n");
    let document = LanguageDocument {
        path: Path::new("/project/pages/overlay.dowe").to_path_buf(),
        source,
    };
    let root = Path::new("/project");

    let base = complete_document(root, &document, 1, 1);
    for label in [
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
    ] {
        assert!(base.iter().any(|item| item.label == label));
    }

    let avatar_props = complete_document(root, &document, 2, "  Avatar ".len() + 1);
    assert!(avatar_props.iter().any(|item| item.label == "scheme"));
    assert!(avatar_props.iter().any(|item| item.label == "status"));
    assert!(avatar_props.iter().any(|item| item.label == "onClick"));
    assert!(!avatar_props.iter().any(|item| item.label == "color"));

    let avatar_status = complete_document(root, &document, 3, "  Avatar status:".len() + 1);
    assert!(avatar_status.iter().any(|item| item.label == "\"online\""));
    assert!(avatar_status.iter().any(|item| item.label == "\"away\""));

    let avatar_scheme = complete_document(root, &document, 4, "  Avatar scheme:".len() + 1);
    assert!(avatar_scheme.iter().any(|item| item.label == "\"success\""));
    assert!(avatar_scheme.iter().any(|item| item.label == "\"surface\""));

    let badge_position = complete_document(root, &document, 5, "  Badge position:".len() + 1);
    assert!(
        badge_position
            .iter()
            .any(|item| item.label == "\"bottom-right\"")
    );

    let chip_variant = complete_document(root, &document, 6, "  Chip variant:".len() + 1);
    assert!(chip_variant.iter().any(|item| item.label == "\"outlined\""));
    assert!(chip_variant.iter().any(|item| item.label == "\"ghost\""));

    let chip_start_icon = complete_document(root, &document, 7, "  Chip startIcon:".len() + 1);
    assert!(
        chip_start_icon
            .iter()
            .any(|item| item.label == "\"settings\"")
    );

    let chip_end_icon = complete_document(root, &document, 8, "  Chip endIcon:".len() + 1);
    assert!(
        chip_end_icon
            .iter()
            .any(|item| item.label == "\"magnifier\"")
    );

    let skeleton_variant = complete_document(root, &document, 9, "  Skeleton variant:".len() + 1);
    assert!(
        skeleton_variant
            .iter()
            .any(|item| item.label == "\"circular\"")
    );

    let skeleton_animation =
        complete_document(root, &document, 10, "  Skeleton animation:".len() + 1);
    assert!(
        skeleton_animation
            .iter()
            .any(|item| item.label == "\"none\"")
    );

    let modal_scheme = complete_document(root, &document, 11, "  Modal scheme:".len() + 1);
    assert!(modal_scheme.iter().any(|item| item.label == "\"surface\""));

    let dialog_variant = complete_document(root, &document, 12, "  AlertDialog variant:".len() + 1);
    assert!(dialog_variant.iter().any(|item| item.label == "\"ghost\""));

    let tooltip_position = complete_document(root, &document, 13, "  Tooltip position:".len() + 1);
    assert!(tooltip_position.iter().any(|item| item.label == "\"end\""));

    let toast_type = complete_document(root, &document, 14, "  Toast type:".len() + 1);
    assert!(toast_type.iter().any(|item| item.label == "\"success\""));
    assert!(toast_type.iter().any(|item| item.label == "\"error\""));
    let toast_variant = complete_document(root, &document, 15, "  Toast variant:".len() + 1);
    assert!(
        toast_variant
            .iter()
            .any(|item| item.label == "\"outlined\"")
    );
    assert!(toast_variant.iter().any(|item| item.label == "\"ghost\""));

    let dropdown_props = complete_document(root, &document, 16, "  Dropdown ".len() + 1);
    assert!(dropdown_props.iter().any(|item| item.label == "scheme"));
    assert!(!dropdown_props.iter().any(|item| item.label == "variant"));

    let command_props = complete_document(root, &document, 17, "  Command ".len() + 1);
    assert!(command_props.iter().any(|item| item.label == "shortcut"));
    assert!(command_props.iter().any(|item| item.label == "scheme"));

    let command_variant = complete_document(root, &document, 18, "  Command variant:".len() + 1);
    assert!(command_variant.iter().any(|item| item.label == "\"ghost\""));

    let item_props = complete_document(root, &document, 19, "  item ".len() + 1);
    assert!(item_props.iter().any(|item| item.label == "history"));
    assert!(item_props.iter().any(|item| item.label == "onClick"));

    let group_props = complete_document(root, &document, 20, "  group ".len() + 1);
    assert!(group_props.iter().any(|item| item.label == "label"));
}

#[test]
fn completions_include_rich_control_map_component_props_and_values() {
    let source = [
        "page componentsPage",
        "  RichText ",
        "    mark ",
        "  RichText size:",
        "  RichText title:",
        "  Record ",
        "  Record variant:",
        "  ToggleGroup ",
        "    item ",
        "  ToggleGroup size:",
        "  Collapsible ",
        "  Countdown ",
        "  Countdown size:",
        "  Map ",
        "    marker ",
        "    waypoint ",
        "  Map scheme:",
    ]
    .join("\n");
    let document = LanguageDocument {
        path: Path::new("/project/pages/components.dowe").to_path_buf(),
        source,
    };
    let root = Path::new("/project");

    let base = complete_document(root, &document, 1, 1);
    for label in [
        "RichText",
        "Record",
        "ToggleGroup",
        "Collapsible",
        "Countdown",
        "Map",
    ] {
        assert!(base.iter().any(|item| item.label == label));
    }

    let rich_text_props = complete_document(root, &document, 2, "  RichText ".len() + 1);
    assert!(rich_text_props.iter().any(|item| item.label == "i18n"));
    assert!(rich_text_props.iter().any(|item| item.label == "weight"));
    assert!(rich_text_props.iter().any(|item| item.label == "title"));

    let mark_props = complete_document(root, &document, 3, "    mark ".len() + 1);
    assert!(mark_props.iter().any(|item| item.label == "text"));
    assert!(mark_props.iter().any(|item| item.label == "style"));
    assert!(mark_props.iter().any(|item| item.label == "scheme"));

    let rich_text_size = complete_document(root, &document, 4, "  RichText size:".len() + 1);
    assert!(rich_text_size.iter().any(|item| item.label == "\"xl\""));

    let rich_text_title = complete_document(root, &document, 5, "  RichText title:".len() + 1);
    assert!(rich_text_title.iter().any(|item| item.label == "true"));
    assert!(rich_text_title.iter().any(|item| item.label == "false"));

    let record_props = complete_document(root, &document, 6, "  Record ".len() + 1);
    assert!(record_props.iter().any(|item| item.label == "name"));
    assert!(record_props.iter().any(|item| item.label == "maxDuration"));
    assert!(record_props.iter().any(|item| item.label == "onConfirm"));
    assert!(!record_props.iter().any(|item| item.label == "color"));

    let record_variant = complete_document(root, &document, 7, "  Record variant:".len() + 1);
    assert!(record_variant.iter().any(|item| item.label == "\"solid\""));
    assert!(!record_variant.iter().any(|item| item.label == "\"ghost\""));

    let toggle_props = complete_document(root, &document, 8, "  ToggleGroup ".len() + 1);
    assert!(toggle_props.iter().any(|item| item.label == "bind"));
    assert!(toggle_props.iter().any(|item| item.label == "selected"));
    assert!(toggle_props.iter().any(|item| item.label == "onChange"));

    let item_props = complete_document(root, &document, 9, "    item ".len() + 1);
    assert!(item_props.iter().any(|item| item.label == "id"));
    assert!(item_props.iter().any(|item| item.label == "label"));
    assert!(item_props.iter().any(|item| item.label == "icon"));

    let toggle_size = complete_document(root, &document, 10, "  ToggleGroup size:".len() + 1);
    assert!(toggle_size.iter().any(|item| item.label == "\"sm\""));

    let collapsible_props = complete_document(root, &document, 11, "  Collapsible ".len() + 1);
    assert!(collapsible_props.iter().any(|item| item.label == "label"));
    assert!(
        collapsible_props
            .iter()
            .any(|item| item.label == "defaultOpen")
    );

    let countdown_props = complete_document(root, &document, 12, "  Countdown ".len() + 1);
    assert!(countdown_props.iter().any(|item| item.label == "target"));
    assert!(
        countdown_props
            .iter()
            .any(|item| item.label == "showSeconds")
    );
    assert!(
        countdown_props
            .iter()
            .any(|item| item.label == "onComplete")
    );

    let countdown_size = complete_document(root, &document, 13, "  Countdown size:".len() + 1);
    assert!(countdown_size.iter().any(|item| item.label == "\"xl\""));

    let map_props = complete_document(root, &document, 14, "  Map ".len() + 1);
    assert!(map_props.iter().any(|item| item.label == "centerLat"));
    assert!(
        map_props
            .iter()
            .any(|item| item.label == "showLocationControl")
    );
    assert!(map_props.iter().any(|item| item.label == "onRoute"));

    let marker_props = complete_document(root, &document, 15, "    marker ".len() + 1);
    assert!(marker_props.iter().any(|item| item.label == "lat"));
    assert!(marker_props.iter().any(|item| item.label == "popup"));
    assert!(marker_props.iter().any(|item| item.label == "onClick"));

    let waypoint_props = complete_document(root, &document, 16, "    waypoint ".len() + 1);
    assert!(waypoint_props.iter().any(|item| item.label == "lat"));
    assert!(waypoint_props.iter().any(|item| item.label == "lng"));

    let map_scheme = complete_document(root, &document, 17, "  Map scheme:".len() + 1);
    assert!(map_scheme.iter().any(|item| item.label == "\"primary\""));
    assert!(!map_scheme.iter().any(|item| item.label == "\"surface\""));
}

#[test]
fn completions_include_flex_direction_prop_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/layout.dowe").to_path_buf(),
        source: "page layoutPage\n  Flex \n  Flex direction:\n".to_string(),
    };

    let props = complete_document(Path::new("/project"), &document, 2, "  Flex ".len() + 1);
    assert!(props.iter().any(|item| item.label == "direction"));
    assert!(props.iter().any(|item| item.label == "wrap"));

    let values = complete_document(
        Path::new("/project"),
        &document,
        3,
        "  Flex direction:".len() + 1,
    );
    assert!(values.iter().any(|item| item.label == "\"row\""));
    assert!(values.iter().any(|item| item.label == "\"column\""));
}

#[test]
fn completions_include_flex_item_prop_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/layout.dowe").to_path_buf(),
        source: "page layoutPage\n  Box \n  Section \n  Flex \n  Grid \n  Card \n  Box flex:\n"
            .to_string(),
    };

    for (line, column) in [(2, 7), (3, 11), (4, 8), (5, 8), (6, 8)] {
        let props = complete_document(Path::new("/project"), &document, line, column);
        assert!(props.iter().any(|item| item.label == "flex"));
    }

    let values = complete_document(Path::new("/project"), &document, 7, "  Box flex:".len() + 1);
    assert!(values.iter().any(|item| item.label == "\"initial\""));
    assert!(values.iter().any(|item| item.label == "\"auto\""));
    assert!(values.iter().any(|item| item.label == "\"none\""));
    assert!(values.iter().any(|item| item.label == "1"));
}

#[test]
fn diagnostics_accept_flex_item_prop_on_grid() {
    let root = tempdir().expect("root");
    fs::create_dir_all(root.path().join("pages")).expect("pages");
    let document = LanguageDocument {
        path: root.path().join("pages/layout.dowe"),
        source: "page layoutPage\n  Grid flex:1\n    Text\n      \"Grid\"\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn completions_include_view_animation_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/login.dowe").to_path_buf(),
        source: "page loginPage\n  Box animation:\n  Section animation:\n  Section background:\n  Card animation:\n  Flex animation:\n  Chip transition:\n  Chip gesture:\n  Chip \n"
            .to_string(),
    };

    let box_animation = complete_document(Path::new("/project"), &document, 2, 17);
    assert!(box_animation.iter().any(|item| item.label == "\"fadeIn\""));
    assert!(
        box_animation
            .iter()
            .any(|item| item.label == "\"slideRight\"")
    );

    let section_animation = complete_document(Path::new("/project"), &document, 3, 21);
    assert!(
        section_animation
            .iter()
            .any(|item| item.label == "\"fadeIn\"")
    );

    let section_background = complete_document(Path::new("/project"), &document, 4, 22);
    assert!(
        section_background
            .iter()
            .any(|item| item.label == "\"aurora\"")
    );
    assert!(
        section_background
            .iter()
            .any(|item| item.label == "\"slate\"")
    );

    let card_animation = complete_document(Path::new("/project"), &document, 5, 18);
    assert!(
        card_animation
            .iter()
            .any(|item| item.label == "\"scaleIn\"")
    );

    let flex_animation = complete_document(Path::new("/project"), &document, 6, 18);
    assert!(flex_animation.iter().any(|item| item.label == "\"fadeIn\""));

    let chip_transition = complete_document(Path::new("/project"), &document, 7, 19);
    assert!(
        chip_transition
            .iter()
            .any(|item| item.label == "\"spring\"")
    );

    let chip_gesture = complete_document(Path::new("/project"), &document, 8, 16);
    assert!(chip_gesture.iter().any(|item| item.label == "\"tilt\""));

    let chip_props = complete_document(Path::new("/project"), &document, 9, 8);
    for prop in [
        "animation",
        "rotate",
        "scale",
        "translateX",
        "translateY",
        "transition",
        "gesture",
        "onClick",
    ] {
        assert!(chip_props.iter().any(|item| item.label == prop));
    }
}

#[test]
fn completions_include_code_component_props_and_languages() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/login.dowe").to_path_buf(),
        source: "page loginPage\n  Code \n  Code language:\n  Code scheme:\n".to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "Code"));

    let props = complete_document(Path::new("/project"), &document, 2, 8);
    assert!(props.iter().any(|item| item.label == "content"));
    assert!(props.iter().any(|item| item.label == "language"));
    assert!(props.iter().any(|item| item.label == "copyLabel"));
    assert!(props.iter().any(|item| item.label == "copiedLabel"));

    let languages = complete_document(Path::new("/project"), &document, 3, 17);
    assert!(languages.iter().any(|item| item.label == "\"dowe\""));
    assert!(languages.iter().any(|item| item.label == "\"typescript\""));
    assert!(languages.iter().any(|item| item.label == "\"javascript\""));
    assert!(languages.iter().any(|item| item.label == "\"go\""));
    assert!(languages.iter().any(|item| item.label == "\"rust\""));
    assert!(languages.iter().any(|item| item.label == "\"python\""));

    let schemes = complete_document(Path::new("/project"), &document, 4, 15);
    assert!(schemes.iter().any(|item| item.label == "\"surface\""));
    assert!(schemes.iter().any(|item| item.label == "\"danger\""));
}

#[test]
fn completions_include_video_component_props_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/login.dowe").to_path_buf(),
        source: "page loginPage\n  Video \n  Video aspect:\n  Video scheme:\n".to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "Video"));

    let props = complete_document(Path::new("/project"), &document, 2, 9);
    assert!(props.iter().any(|item| item.label == "src"));
    assert!(props.iter().any(|item| item.label == "poster"));
    assert!(props.iter().any(|item| item.label == "autoplay"));
    assert!(props.iter().any(|item| item.label == "aspect"));
    assert!(props.iter().any(|item| item.label == "scheme"));

    let aspects = complete_document(Path::new("/project"), &document, 3, 16);
    assert!(aspects.iter().any(|item| item.label == "\"horizontal\""));
    assert!(aspects.iter().any(|item| item.label == "\"vertical\""));
    assert!(aspects.iter().any(|item| item.label == "\"square\""));

    let schemes = complete_document(Path::new("/project"), &document, 4, 16);
    assert!(schemes.iter().any(|item| item.label == "\"surface\""));
    assert!(schemes.iter().any(|item| item.label == "\"accent\""));
}

#[test]
fn completions_include_canvas_component_props_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/canvas.dowe").to_path_buf(),
        source: "page canvasPage\n  signal scene value:[]\n  fn capture\n    set scene value:item\n  Canvas \n  Canvas scene:\n  Canvas fit:\n  Canvas background:\n  Canvas onPointer:\n".to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "Canvas"));
    let props = complete_document(Path::new("/project"), &document, 5, 10);
    for prop in [
        "scene",
        "viewWidth",
        "viewHeight",
        "fit",
        "fps",
        "autoplay",
        "background",
        "pixelated",
        "label",
        "onPointer",
        "onKey",
        "onMotion",
        "motionRate",
    ] {
        assert!(props.iter().any(|item| item.label == prop), "{prop}");
    }
    let scene = complete_document(Path::new("/project"), &document, 6, 16);
    assert!(scene.iter().any(|item| item.label == "scene"));
    let fits = complete_document(Path::new("/project"), &document, 7, 14);
    assert!(fits.iter().any(|item| item.label == "\"contain\""));
    assert!(fits.iter().any(|item| item.label == "\"cover\""));
    assert!(fits.iter().any(|item| item.label == "\"stretch\""));
    let backgrounds = complete_document(Path::new("/project"), &document, 8, 21);
    assert!(backgrounds.iter().any(|item| item.label == "\"surface\""));
    assert!(
        backgrounds
            .iter()
            .any(|item| item.label == "\"transparent\"")
    );
    let actions = complete_document(Path::new("/project"), &document, 9, 21);
    assert!(actions.iter().any(|item| item.label == "capture"));
}

#[test]
fn completions_include_candlestick_component_props_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/market.dowe").to_path_buf(),
        source: "page marketPage\n  signal candles value:[]\n  Candlestick \n  Candlestick data:\n  Candlestick scheme:\n  Candlestick upColor:\n"
            .to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "Candlestick"));

    let props = complete_document(Path::new("/project"), &document, 3, 15);
    assert!(props.iter().any(|item| item.label == "data"));
    assert!(props.iter().any(|item| item.label == "stream"));
    assert!(props.iter().any(|item| item.label == "upColor"));
    assert!(props.iter().any(|item| item.label == "downColor"));
    assert!(props.iter().any(|item| item.label == "emptyLabel"));
    assert!(props.iter().any(|item| item.label == "maxPoints"));

    let data = complete_document(Path::new("/project"), &document, 4, 20);
    assert!(data.iter().any(|item| item.label == "candles"));

    let schemes = complete_document(Path::new("/project"), &document, 5, 22);
    assert!(schemes.iter().any(|item| item.label == "\"surface\""));
    assert!(schemes.iter().any(|item| item.label == "\"accent\""));

    let colors = complete_document(Path::new("/project"), &document, 6, 23);
    assert!(colors.iter().any(|item| item.label == "\"success\""));
    assert!(colors.iter().any(|item| item.label == "\"danger\""));
}

#[test]
fn completions_include_chart_component_props_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/charts.dowe").to_path_buf(),
        source: "page chartPage\n  signal points value:[]\n  LineChart \n  LineChart data:\n  LineChart palette:\n  LineChart legendPosition:\n  LineChart curve:\n  PieChart \n"
            .to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    for label in ["ArcChart", "AreaChart", "BarChart", "LineChart", "PieChart"] {
        assert!(base.iter().any(|item| item.label == label));
    }

    let line_props = complete_document(Path::new("/project"), &document, 3, 13);
    assert!(line_props.iter().any(|item| item.label == "data"));
    assert!(line_props.iter().any(|item| item.label == "series"));
    assert!(line_props.iter().any(|item| item.label == "curve"));
    assert!(line_props.iter().any(|item| item.label == "palette"));

    let data = complete_document(Path::new("/project"), &document, 4, 18);
    assert!(data.iter().any(|item| item.label == "points"));

    let palettes = complete_document(Path::new("/project"), &document, 5, 21);
    assert!(palettes.iter().any(|item| item.label == "\"ocean\""));
    assert!(palettes.iter().any(|item| item.label == "\"forest\""));

    let legends = complete_document(Path::new("/project"), &document, 6, 28);
    assert!(legends.iter().any(|item| item.label == "\"bottom\""));
    assert!(legends.iter().any(|item| item.label == "\"right\""));

    let curves = complete_document(Path::new("/project"), &document, 7, 20);
    assert!(curves.iter().any(|item| item.label == "\"smooth\""));

    let pie_props = complete_document(Path::new("/project"), &document, 8, 12);
    assert!(pie_props.iter().any(|item| item.label == "donut"));
    assert!(pie_props.iter().any(|item| item.label == "donutWidth"));
}

#[test]
fn completions_include_table_component_and_column_props() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/users.dowe").to_path_buf(),
        source: "page usersPage\n  signal users value:[]\n  Table \n    column \n  Table data:\n  Table size:\n  Table scheme:\n    column align:\n"
            .to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "Table"));

    let props = complete_document(Path::new("/project"), &document, 3, 9);
    assert!(props.iter().any(|item| item.label == "data"));
    assert!(props.iter().any(|item| item.label == "scheme"));
    assert!(props.iter().any(|item| item.label == "emptyTitle"));
    assert!(props.iter().any(|item| item.label == "dividers"));

    let column_props = complete_document(Path::new("/project"), &document, 4, 12);
    assert!(column_props.iter().any(|item| item.label == "field"));
    assert!(column_props.iter().any(|item| item.label == "label"));
    assert!(column_props.iter().any(|item| item.label == "align"));
    assert!(column_props.iter().any(|item| item.label == "width"));

    let data = complete_document(Path::new("/project"), &document, 5, 14);
    assert!(data.iter().any(|item| item.label == "users"));

    let sizes = complete_document(Path::new("/project"), &document, 6, 14);
    assert!(sizes.iter().any(|item| item.label == "\"lg\""));

    let schemes = complete_document(Path::new("/project"), &document, 7, 16);
    assert!(schemes.iter().any(|item| item.label == "\"surface\""));

    let align = complete_document(Path::new("/project"), &document, 8, 19);
    assert!(align.iter().any(|item| item.label == "\"end\""));
}

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

#[test]
fn completes_form_validation_children_props_and_rule_values() {
    let document = LanguageDocument {
        path: PathBuf::from("/project/pages/form.dowe"),
        source: "page FormPage\n  Input \n    validate \n    validate rule:\n".to_string(),
    };

    let input_props = complete_document(Path::new("/project"), &document, 2, "  Input ".len() + 1);
    let validation_props = complete_document(
        Path::new("/project"),
        &document,
        3,
        "    validate ".len() + 1,
    );
    let rule_values = complete_document(
        Path::new("/project"),
        &document,
        4,
        "    validate rule:".len() + 1,
    );

    assert!(input_props.iter().any(|item| item.label == "helpText"));
    assert!(input_props.iter().any(|item| item.label == "errorText"));
    assert!(validation_props.iter().any(|item| item.label == "rule"));
    assert!(validation_props.iter().any(|item| item.label == "message"));
    assert!(rule_values.iter().any(|item| item.label == "\"required\""));
    assert!(
        rule_values
            .iter()
            .any(|item| item.label == "\"matches:form.confirmation\"")
    );
}
