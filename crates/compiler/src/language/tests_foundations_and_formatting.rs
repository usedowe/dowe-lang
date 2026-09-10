#[test]
fn completions_offer_draw_erase_only_for_draw() {
    for (component, expected) in [("Draw", true), ("Canvas", false)] {
        let line = format!("  {component} drawMode:");
        let document = LanguageDocument {
            path: PathBuf::from("/project/views/pages/draw.dowe"),
            source: format!("page drawPage\n{line}"),
        };
        let values = complete_document(Path::new("/project"), &document, 2, line.len() + 1);
        for mode in ["pen", "rect", "circle"] {
            assert!(
                values
                    .iter()
                    .any(|item| item.label == format!("\"{mode}\""))
            );
        }
        for mode in ["select", "erase"] {
            assert_eq!(
                values
                    .iter()
                    .any(|item| item.label == format!("\"{mode}\"")),
                expected
            );
        }
    }
}

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

