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
fn diagnostics_reject_unknown_possible_icon_names() {
    let root = tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("pages")).expect("pages");
    for (value, binding, expected) in [
        (
            "const icons value:[{ id:\"one\" name:\"home\" } { id:\"two\" name:\"not-a-dowe-icon\" }]",
            "each in:icons as:icon key:icon.id\n    Icon name:icon.name",
            "not-a-dowe-icon",
        ),
        (
            "signal chosen value:\"home\"",
            "Icon name:chosen",
            "must resolve to known names from a `const`",
        ),
    ] {
        let document = LanguageDocument {
            path: root.path().join("pages/icons.dowe"),
            source: format!("page iconPage\n  {value}\n  {binding}\n"),
        };
        let diagnostics = analyze_document(root.path(), &document);
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "expected {expected}: {diagnostics:?}"
        );
    }
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
fn diagnostics_accept_card_color_style_prop() {
    let document = LanguageDocument {
        path: Path::new("/project/pages/card.dowe").to_path_buf(),
        source: "page cardPage\n  Card:\n    cover:\"https://images.unsplash.com/photo-1497366754035-f200968a6e72?auto=format&fit=crop&w=1200&q=80\"\n    overlay:0.62\n    rounded:\"lg\"\n    minH:72\n    color:\"white\"\n    Text\n      \"Office\"\n"
            .to_string(),
    };

    let diagnostics = analyze_document(Path::new("/project"), &document);
    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
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
