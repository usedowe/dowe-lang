use super::{
    LanguageCompletionKind, LanguageDiagnosticSeverity, LanguageDocument, LanguageRange,
    complete_document, definition_at, document_symbols, format_document, hover_at,
};
use crate::language::analyze_document;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn formatter_normalizes_spacing_and_newline() {
    let source = "page loginPage   \n  Box   p:4\n    Text   size:\"md\"\n      Login";
    let formatted = format_document(
        Path::new("/project"),
        Path::new("/project/src/pages/login.dowe"),
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
            Path::new("/project/src/pages/login.dowe"),
            &formatted,
        )
        .expect("formatted again"),
        formatted
    );
}

#[test]
fn formatter_rejects_unsafe_parse() {
    let error = format_document(
        Path::new("/project"),
        Path::new("/project/src/pages/login.dowe"),
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
fn diagnostics_report_invalid_import() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src/pages")).expect("src");
    let path = root.path().join("src/views.dowe");
    let document = LanguageDocument {
        path,
        source: "import Missing from \"./pages/missing\"\nviews\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == LanguageDiagnosticSeverity::Error
            && diagnostic.message.contains("does not exist")
    }));
}

#[test]
fn diagnostics_resolve_imports_from_nearest_nested_src() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src")).expect("root src");
    let example_src = root.path().join("examples/commerce-ops/src");
    fs::create_dir_all(example_src.join("layouts")).expect("layouts");
    fs::create_dir_all(example_src.join("pages")).expect("pages");
    fs::write(
        example_src.join("layouts/ops.dowe"),
        "layout OpsLayout\n  Box\n    children\n",
    )
    .expect("layout");
    fs::write(
        example_src.join("pages/dashboard.dowe"),
        "page dashboardPage\n  Text\n    Dashboard\n",
    )
    .expect("dashboard");
    fs::write(
        example_src.join("pages/inventory.dowe"),
        "page inventoryPage\n  Text\n    Inventory\n",
    )
    .expect("inventory");
    let document = LanguageDocument {
        path: example_src.join("views.dowe"),
        source: concat!(
            "import OpsLayout from \"./layouts/ops\"\n",
            "import dashboardPage from \"./pages/dashboard\"\n",
            "import inventoryPage from \"./pages/inventory\"\n\n",
            "views\n",
            "  route path:\"/\" layout:OpsLayout\n",
            "    page path:\"\" component:dashboardPage\n",
            "    page path:\"inventory\" component:inventoryPage\n"
        )
        .to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
    let import_location = definition_at(root.path(), &document, 1, 8).expect("definition");
    assert_eq!(import_location.path, example_src.join("layouts/ops.dowe"));
}

#[test]
fn diagnostics_report_legacy_server_entry() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("src/server.dowe"),
        source: "main\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == LanguageDiagnosticSeverity::Error
            && diagnostic.message.contains("src/main.dowe")
    }));
}

#[test]
fn diagnostics_validate_view_actions_and_bindings() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src/pages")).expect("src");
    let path = root.path().join("src/pages/blogs.dowe");
    let document = LanguageDocument {
        path,
        source: "page blogsPage\n  signal blog value:{ title:\"\" }\n  Button onClick:missing\n    Save\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("unknown action `missing`"))
    );
}

#[test]
fn diagnostics_reject_unknown_view_signal_fields() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src/pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("src/pages/blogs.dowe"),
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
    fs::create_dir_all(root.path().join("src/handlers")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("src/handlers/blogs.dowe"),
        source: "handler createBlog\n  let db = store database:\"app\"\n  let created = db.insert table:\"blogs\" value:{ title:\"\" }\n  log created.content\n  return response json:created\n"
            .to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("unknown field `created.content`")
    }));

    let response_document = LanguageDocument {
        path: root.path().join("src/handlers/blogs.dowe"),
        source: "handler createBlog\n  let db = store database:\"app\"\n  let created = db.insert table:\"blogs\" value:{ title:\"\" }\n  return response json:{ data:created.content }\n"
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
    fs::create_dir_all(root.path().join("src/handlers")).expect("src");
    fs::write(
        root.path().join("src/config.dowe"),
        concat!(
            "config\n",
            "  env\n",
            "    variable name:\"STORE_HOST\" visibility:\"server\" required:true\n",
            "    variable name:\"STORE_TOKEN\" visibility:\"server\" required:true\n"
        ),
    )
    .expect("config");
    let document = LanguageDocument {
        path: root.path().join("src/handlers/appointments.dowe"),
        source: concat!(
            "handler listAppointments req\n",
            "  let db = store database:\"clinic\" host:env.STORE_HOST user:\"clinic-api\" token:env.STORE_TOKEN\n",
            "  let appointments = db.list table:\"appointments\"\n",
            "  return response json:{ ok:true data:appointments }\n"
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
fn diagnostics_accept_text_typography_props() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src/pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("src/pages/login.dowe"),
        source: "page loginPage\n  Text size:\"md\" color:\"onPrimary\" i18n:\"auth.login.title\"\n    Login\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn diagnostics_validate_translation_catalogs() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src/i18n")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("src/i18n/en.dowe"),
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
    fs::create_dir_all(root.path().join("src/pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("src/pages/login.dowe"),
        source: "page loginPage\n  Svg viewBox:\"0 0 24 24\" color:\"tertiary\" w:8 h:8\n    Path d:\"M0 0h24v24H0z\" fill:\"none\"\n    Path d:\"M3.5 12a8.5 8.5 0 1 1 17 0\" fill:\"currentColor\"\n"
            .to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn diagnostics_place_component_prop_errors_on_prop_token() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src/pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("src/pages/login.dowe"),
        source: "page loginPage\n  Input variant:\"soft\" unknownLabel:test\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "DOWE_PROP")
        .expect("prop diagnostic");

    assert_eq!(
        diagnostic.message,
        "2:24: unknown prop `unknownLabel` on `Input`"
    );
    assert_eq!(diagnostic.range, LanguageRange::single_line(2, 24, 12));
}

#[test]
fn diagnostics_report_unquoted_static_component_strings() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src/pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("src/pages/login.dowe"),
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
        path: root.path().join("src/pages/login.dowe"),
        source: "page loginPage\n  Input variant:outlined scheme:primary\n".to_string(),
    };
    let enum_diagnostics = analyze_document(root.path(), &enum_document);
    let enum_diagnostic = enum_diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "DOWE_PROP")
        .expect("unquoted enum diagnostic");
    assert!(
        enum_diagnostic
            .message
            .contains("invalid value for prop `variant`: expected quoted static string literal")
    );
    assert_eq!(enum_diagnostic.range.start.line, 2);
}

#[test]
fn diagnostics_report_unquoted_static_config_strings() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("src/config.dowe"),
        source: "config\n  fonts default:inter install:[\"inter\"]\n".to_string(),
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
    fs::create_dir_all(root.path().join("src/pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("src/pages/login.dowe"),
        source: "page loginPage\n  signal profile value:{ name:\"\" role:\"admin\" }\n  Box\n    Input bind:profile.name label:\"Name\" placeholder:\"Full name\" labelFloating:true\n    Select bind:profile.role label:\"Role\" placeholder:\"Choose role\" labelFloating:true\n      Option value:\"admin\" label:\"Admin\"\n      Option value:\"viewer\" label:\"Viewer\"\n".to_string(),
    };

    let diagnostics = analyze_document(root.path(), &document);

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn completions_include_actions_signals_and_env() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src/pages")).expect("src");
    fs::write(
        root.path().join("src/config.dowe"),
        "config\n  env\n    variable name:\"BACKEND_URL\" visibility:\"client\" required:false default:\"\"\n",
    )
    .expect("config");
    let document = LanguageDocument {
        path: root.path().join("src/pages/blogs.dowe"),
        source: "page blogsPage\n  signal blog value:{ title:\"\" }\n  action saveBlog\n    reset blog\n  Button onClick:\n    Save\n  Input bind:\n  Text\n    env.\n  Text\n    blog.\n".to_string(),
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
fn completions_include_show_booleans_and_signals() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src/pages")).expect("src");
    let document = LanguageDocument {
        path: root.path().join("src/pages/ready.dowe"),
        source: "page readyPage\n  signal isReady value:false\n  Text show:\n    Ready\n  Drawer open:\n    Text\n      Menu\n"
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
fn completions_and_diagnostics_include_server_middlewares() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src/middlewares")).expect("middlewares");
    fs::write(
        root.path().join("src/config.dowe"),
        "config\n  env\n    variable name:\"JWT_SECRET\" visibility:\"server\" required:false\n",
    )
    .expect("config");
    fs::write(
        root.path().join("src/middlewares/auth.dowe"),
        "middleware requireBearer async req\n  let authorization = req.header name:\"Authorization\"\n  let token = bearer authorization\n  let verified = jwt.verify token secret:env.JWT_SECRET algorithm:\"HS256\"\n  if verified.valid\n    return continue context:{ auth:{ subject:verified.claims.sub } }\n  return response status:401 json:{ ok:false }\n",
    )
    .expect("middleware");
    let document = LanguageDocument {
        path: root.path().join("src/main.dowe"),
        source: "import requireBearer from \"./middlewares/auth\"\nmain\n  server port:8080\n    route \"/users/:id\" middleware:[requireBearer]\n      handler req\n        return response text:\"Hello\"\n".to_string(),
    };

    let completions = complete_document(root.path(), &document, 4, 35);
    assert!(completions.iter().any(|item| item.label == "requireBearer"));

    let bad_middleware = LanguageDocument {
        path: root.path().join("src/middlewares/bad.dowe"),
        source: "middleware bad req\n  return continue\n".to_string(),
    };
    let diagnostics = analyze_document(root.path(), &bad_middleware);
    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );

    let bad_page = LanguageDocument {
        path: root.path().join("src/pages/bad.dowe"),
        source: "middleware bad req\n  return continue\n".to_string(),
    };
    let page_diagnostics = analyze_document(root.path(), &bad_page);
    assert!(page_diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("must live under `src/middlewares`")
    }));
}

#[test]
fn completions_and_hover_include_inferred_handler_fields() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src/handlers")).expect("src");
    let completion_document = LanguageDocument {
        path: root.path().join("src/handlers/blogs.dowe"),
        source: "handler createBlog\n  let db = store database:\"app\"\n  let created = db.insert table:\"blogs\" value:{ title:\"\" content:\"\" }\n  log created.\n  return response json:created\n"
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
        path: root.path().join("src/handlers/blogs.dowe"),
        source: "handler createBlog\n  let db = store database:\"app\"\n  let created = db.insert table:\"blogs\" value:{ title:\"\" }\n  log created.title\n  return response json:created\n"
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
    fs::create_dir_all(root.path().join("src/handlers")).expect("src");
    let completion_document = LanguageDocument {
        path: root.path().join("src/handlers/cache.dowe"),
        source: "handler cacheAppointment\n  let db = kv database:\"clinic\"\n  let saved = db.set key:\"appointment:1\" value:{ patientName:\"Ana\" }\n  log saved.\n  return response json:saved\n"
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
    fs::create_dir_all(root.path().join("src/handlers")).expect("handlers");
    fs::create_dir_all(root.path().join("src/pages")).expect("pages");
    let handler_document = LanguageDocument {
        path: root.path().join("src/handlers/users.dowe"),
        source: "type User\n  name:string\n  age:number\n\nhandler createUser async req\n  let body:User = await req.json()\n  log body.\n  let db = store database:\"app\"\n  let created = db.insert table:\"users\" value:{ name:body.email }\n  return response json:created\n".to_string(),
    };

    let completions = complete_document(root.path(), &handler_document, 7, "  log body.".len() + 1);
    assert!(completions.iter().any(|item| item.label == "name"));
    assert!(completions.iter().any(|item| item.label == "age"));

    let diagnostic_document = LanguageDocument {
        path: root.path().join("src/handlers/users.dowe"),
        source: "type User\n  name:string\n  age:number\n\nhandler createUser async req\n  let body:User = await req.json()\n  let db = store database:\"app\"\n  let created = db.insert table:\"users\" value:{ name:body.email }\n  return response json:created\n".to_string(),
    };
    let diagnostics = analyze_document(root.path(), &diagnostic_document);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("unknown field `body.email`"))
    );

    let view_document = LanguageDocument {
        path: root.path().join("src/pages/blogs.dowe"),
        source: "type BlogItem\n  id:string\n  title:string\n\npage blogsPage\n  signal blogs type:BlogItem[] value:[]\n  Grid\n    each item in blogs key:item.id\n      Text\n        item.\n".to_string(),
    };
    let item_completions =
        complete_document(root.path(), &view_document, 10, "        item.".len() + 1);
    assert!(item_completions.iter().any(|item| item.label == "id"));
    assert!(item_completions.iter().any(|item| item.label == "title"));
}

#[test]
fn completions_include_current_view_component_props() {
    let document = LanguageDocument {
        path: Path::new("/project/src/pages/login.dowe").to_path_buf(),
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
    assert!(base.iter().any(|item| item.label == "Tabs"));
    assert!(base.iter().any(|item| item.label == "Video"));
    assert!(base.iter().any(|item| item.label == "Divider"));
    assert!(base.iter().any(|item| item.label == "Drawer"));
    assert!(!base.iter().any(|item| item.label == "Body"));

    let box_props = complete_document(Path::new("/project"), &document, 2, 7);
    assert!(box_props.iter().any(|item| item.label == "color"));
    assert!(box_props.iter().any(|item| item.label == "animation"));
    assert!(!box_props.iter().any(|item| item.label == "text"));

    let section_props = complete_document(Path::new("/project"), &document, 3, 11);
    assert!(section_props.iter().any(|item| item.label == "background"));
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
    for (line, column) in [(5, 8), (7, 9), (8, 9)] {
        let props = complete_document(Path::new("/project"), &document, line, column);
        assert!(!props.iter().any(|item| item.label == "size"));
    }
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
    assert!(drawer_props.iter().any(|item| item.label == "open"));
    assert!(drawer_props.iter().any(|item| item.label == "position"));
    assert!(drawer_props.iter().any(|item| item.label == "scheme"));
    assert!(drawer_props.iter().any(|item| item.label == "show"));

    let bar_document = LanguageDocument {
        path: Path::new("/project/src/pages/bars.dowe").to_path_buf(),
        source: "page barsPage\n  AppBar \n  Footer \n  BottomBar \n  SideNav \n".to_string(),
    };
    let appbar_props = complete_document(Path::new("/project"), &bar_document, 2, 11);
    assert!(appbar_props.iter().any(|item| item.label == "floating"));
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
}

#[test]
fn completions_include_translation_keys() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src/i18n")).expect("i18n");
    fs::write(
        root.path().join("src/i18n/en.dowe"),
        "translations default:true\n  home\n    hero\n      title \"Dowe builds systems.\"\n",
    )
    .expect("english");
    fs::write(
        root.path().join("src/i18n/es.dowe"),
        "translations\n  home\n    hero\n      title \"Dowe construye sistemas.\"\n",
    )
    .expect("spanish");
    let document = LanguageDocument {
        path: root.path().join("src/pages/home.dowe"),
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
        path: Path::new("/project/src/pages/login.dowe").to_path_buf(),
        source: "page loginPage\n  Input scheme:\n  Input variant:\n  Path fill:\n  Button navigate:\n  Alert type:\n  AppBar scheme:\n  Text weight:\n  SideNav size:\n  SideNav scheme:\n  Divider orientation:\n  Divider scheme:\n  Drawer position:\n  Drawer scheme:\n  Tabs variant:\n  Tabs position:\n  Tabs scheme:\n"
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
    assert!(variant.iter().any(|item| item.label == "\"soft\""));

    let fill = complete_document(Path::new("/project"), &document, 4, 13);
    assert!(fill.iter().any(|item| item.label == "\"none\""));
    assert!(fill.iter().any(|item| item.label == "\"currentColor\""));
    assert!(fill.iter().any(|item| item.label == "\"tertiary\""));

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
    assert!(
        side_nav_scheme
            .iter()
            .any(|item| item.label == "\"surface\"")
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
    assert!(!tabs_variant.iter().any(|item| item.label == "\"soft\""));

    let tabs_position = complete_document(Path::new("/project"), &document, 16, 18);
    assert!(tabs_position.iter().any(|item| item.label == "\"start\""));
    assert!(tabs_position.iter().any(|item| item.label == "\"bottom\""));

    let tabs_scheme = complete_document(Path::new("/project"), &document, 17, 17);
    assert!(tabs_scheme.iter().any(|item| item.label == "\"surface\""));
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
        "  Skeleton variant:",
        "  Skeleton animation:",
        "  Modal scheme:",
        "  AlertDialog variant:",
        "  Tooltip position:",
        "  Toast type:",
        "  Dropdown ",
        "  Command ",
        "  Command variant:",
        "  item ",
        "  group ",
    ]
    .join("\n");
    let document = LanguageDocument {
        path: Path::new("/project/src/pages/overlay.dowe").to_path_buf(),
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

    let skeleton_variant = complete_document(root, &document, 7, "  Skeleton variant:".len() + 1);
    assert!(
        skeleton_variant
            .iter()
            .any(|item| item.label == "\"circular\"")
    );

    let skeleton_animation =
        complete_document(root, &document, 8, "  Skeleton animation:".len() + 1);
    assert!(
        skeleton_animation
            .iter()
            .any(|item| item.label == "\"none\"")
    );

    let modal_scheme = complete_document(root, &document, 9, "  Modal scheme:".len() + 1);
    assert!(modal_scheme.iter().any(|item| item.label == "\"surface\""));

    let dialog_variant = complete_document(root, &document, 10, "  AlertDialog variant:".len() + 1);
    assert!(dialog_variant.iter().any(|item| item.label == "\"ghost\""));

    let tooltip_position = complete_document(root, &document, 11, "  Tooltip position:".len() + 1);
    assert!(tooltip_position.iter().any(|item| item.label == "\"end\""));

    let toast_type = complete_document(root, &document, 12, "  Toast type:".len() + 1);
    assert!(toast_type.iter().any(|item| item.label == "\"success\""));
    assert!(toast_type.iter().any(|item| item.label == "\"error\""));

    let dropdown_props = complete_document(root, &document, 13, "  Dropdown ".len() + 1);
    assert!(dropdown_props.iter().any(|item| item.label == "scheme"));
    assert!(!dropdown_props.iter().any(|item| item.label == "variant"));

    let command_props = complete_document(root, &document, 14, "  Command ".len() + 1);
    assert!(command_props.iter().any(|item| item.label == "shortcut"));
    assert!(command_props.iter().any(|item| item.label == "scheme"));

    let command_variant = complete_document(root, &document, 15, "  Command variant:".len() + 1);
    assert!(command_variant.iter().any(|item| item.label == "\"ghost\""));

    let item_props = complete_document(root, &document, 16, "  item ".len() + 1);
    assert!(item_props.iter().any(|item| item.label == "history"));
    assert!(item_props.iter().any(|item| item.label == "onClick"));

    let group_props = complete_document(root, &document, 17, "  group ".len() + 1);
    assert!(group_props.iter().any(|item| item.label == "label"));
}

#[test]
fn completions_include_view_animation_values() {
    let document = LanguageDocument {
        path: Path::new("/project/src/pages/login.dowe").to_path_buf(),
        source: "page loginPage\n  Box animation:\n  Section animation:\n  Section background:\n  Card animation:\n  Flex animation:\n"
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
    assert!(!flex_animation.iter().any(|item| item.label == "\"fadeIn\""));
}

#[test]
fn completions_include_code_component_props_and_languages() {
    let document = LanguageDocument {
        path: Path::new("/project/src/pages/login.dowe").to_path_buf(),
        source: "page loginPage\n  Code \n  Code language:\n  Code scheme:\n".to_string(),
    };

    let base = complete_document(Path::new("/project"), &document, 1, 1);
    assert!(base.iter().any(|item| item.label == "Code"));

    let props = complete_document(Path::new("/project"), &document, 2, 8);
    assert!(props.iter().any(|item| item.label == "lines"));
    assert!(props.iter().any(|item| item.label == "language"));
    assert!(props.iter().any(|item| item.label == "copyLabel"));
    assert!(props.iter().any(|item| item.label == "copiedLabel"));

    let languages = complete_document(Path::new("/project"), &document, 3, 17);
    assert!(languages.iter().any(|item| item.label == "\"dowe\""));
    assert!(languages.iter().any(|item| item.label == "\"typescript\""));
    assert!(languages.iter().any(|item| item.label == "\"go\""));
    assert!(languages.iter().any(|item| item.label == "\"rust\""));

    let schemes = complete_document(Path::new("/project"), &document, 4, 15);
    assert!(schemes.iter().any(|item| item.label == "\"surface\""));
    assert!(schemes.iter().any(|item| item.label == "\"danger\""));
}

#[test]
fn completions_include_video_component_props_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/src/pages/login.dowe").to_path_buf(),
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
    assert!(schemes.iter().any(|item| item.label == "\"tertiary\""));
}

#[test]
fn completions_include_candlestick_component_props_and_values() {
    let document = LanguageDocument {
        path: Path::new("/project/src/pages/market.dowe").to_path_buf(),
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
    assert!(schemes.iter().any(|item| item.label == "\"tertiary\""));

    let colors = complete_document(Path::new("/project"), &document, 6, 23);
    assert!(colors.iter().any(|item| item.label == "\"success\""));
    assert!(colors.iter().any(|item| item.label == "\"danger\""));
}

#[test]
fn completions_include_table_component_and_column_props() {
    let document = LanguageDocument {
        path: Path::new("/project/src/pages/users.dowe").to_path_buf(),
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
fn definition_resolves_imports_and_env() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src/pages")).expect("src");
    fs::write(
        root.path().join("src/pages/blogs.dowe"),
        "page blogsPage\n  Box\n",
    )
    .expect("page");
    fs::write(
        root.path().join("src/config.dowe"),
        "config\n  env\n    variable name:\"BACKEND_URL\" visibility:\"client\" required:false default:\"\"\n",
    )
    .expect("config");
    let document = LanguageDocument {
        path: root.path().join("src/views.dowe"),
        source: "import blogsPage from \"./pages/blogs\"\nviews\n  page path:\"blogs\" component:blogsPage\n".to_string(),
    };

    let import_location = definition_at(root.path(), &document, 1, 9).expect("definition");
    assert_eq!(
        import_location.path,
        root.path().join("src/pages/blogs.dowe")
    );

    let page = LanguageDocument {
        path: root.path().join("src/pages/blogs.dowe"),
        source: "page blogsPage\n  Text\n    env.BACKEND_URL\n".to_string(),
    };
    let env_location = definition_at(root.path(), &page, 3, 18).expect("env definition");
    assert_eq!(env_location.path, root.path().join("src/config.dowe"));
}

#[test]
fn document_symbols_include_routes_and_handlers() {
    let document = LanguageDocument {
        path: Path::new("/project/src/main.dowe").to_path_buf(),
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
