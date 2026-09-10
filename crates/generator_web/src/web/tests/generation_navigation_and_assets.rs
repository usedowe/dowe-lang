#[test]
fn emits_fab_actions_as_intrinsic_colored_capsules() {
    let tree = ViewNode::Fab {
        props: FabProps {
            style: VariantProps {
                color: Some(ColorFamily::Primary),
                variant: Some(ComponentVariant::Solid),
                size: Some(ButtonSize::Lg),
                ..Default::default()
            },
            position: OverlayCornerPosition::BottomRight,
            fixed: true,
            offset_x: ScaleValue::from_half_steps(8),
            offset_y: ScaleValue::from_half_steps(8),
            icon: ViewIcon::Plus,
            label: "Open actions".to_string(),
        },
        actions: vec![FabAction {
            label: "View Button".to_string(),
            icon: ViewIcon::Link,
            color: ColorFamily::Info,
            on_click: None,
            navigation: Some(NavigationAction::Internal {
                path: "/views/button".to_string(),
                fragment: None,
                operation: NavigationOperation::Push,
            }),
        }],
    };
    let body = render_page_body(&ViewNode::Children, &tree);
    let css = show_design_css();

    assert!(
        body.contains(
            "data-dowe-fab-action><span class=\"fab-action-label\">View Button</span><svg"
        )
    );
    assert!(!body.contains("</span><a"));
    assert!(css.contains(
        ".fab-action-button{width:auto;min-width:0;height:auto;padding:.5rem .75rem;gap:.75rem"
    ));
    assert!(css.contains(".fab-trigger.is-open{transform:rotate(45deg);}"));
}

#[test]
fn renders_brand_navigation_without_button_chrome() {
    let tree = ViewNode::Brand {
        props: BrandProps {
            style: StyleProps {
                sizing: dowe_components::SizingProps {
                    w: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        ScaleValue::from_half_steps(64),
                    ))),
                    h: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        ScaleValue::from_half_steps(16),
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            },
            navigation: Some(NavigationAction::Internal {
                path: "/".to_string(),
                fragment: None,
                operation: NavigationOperation::Push,
            }),
            label: Some("Dowe home".to_string()),
        },
        children: vec![text("Dowe")],
    };
    let body = render_page_body(&ViewNode::Children, &tree);
    let css = super::design_css();

    assert!(body.contains(r#"<a class="brand w-32 h-8""#));
    assert!(body.contains(r#"href="/" data-dowe-nav="push" data-dowe-href="/""#));
    assert!(body.contains(r#"aria-label="Dowe home""#));
    assert!(!body.contains(r#"class="button"#));
    assert!(css.contains(".brand{--dowe-component-display:inline-flex;"));

    let static_body = render_page_body(
        &ViewNode::Children,
        &ViewNode::Brand {
            props: BrandProps {
                label: Some("Dowe symbol".to_string()),
                ..Default::default()
            },
            children: vec![text("Dowe")],
        },
    );
    assert!(static_body.contains(r#"<div class="brand" role="img" aria-label="Dowe symbol">"#));
    assert!(!static_body.contains("data-dowe-nav"));
}

#[test]
fn renders_banner_as_safe_external_block_link() {
    let tree = ViewNode::Banner {
        props: BannerProps {
            style: StyleProps {
                spacing: dowe_components::SpacingProps {
                    p: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(12))),
                    ..Default::default()
                },
                ..Default::default()
            },
            navigation: NavigationAction::External {
                url: "https://dowe.dev/cloud".to_string(),
                web_target: dowe_components::WebTarget::Blank,
                native_external_mode: dowe_components::NativeExternalMode::System,
            },
            label: Some("Explore Dowe Cloud".to_string()),
        },
        children: vec![text("Build beyond code")],
    };
    let body = render_page_body(&ViewNode::Children, &tree);
    let css = super::design_css();

    assert!(body.contains(r#"<a class="banner p-6""#));
    assert!(body.contains(r#"href="https://dowe.dev/cloud""#));
    assert!(body.contains(r#"target="_blank" rel="noopener noreferrer""#));
    assert!(body.contains(r#"aria-label="Explore Dowe Cloud""#));
    assert!(!body.contains(r#"class="button"#));
    assert!(css.contains(".banner{--dowe-component-display:block;"));
}

#[test]
fn creates_stable_chunk_ids() {
    let root = Path::new("/project");
    let source = "page loginPage\n  Text\n    Login";
    let page_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![text("Login")],
    };
    let first = build_page_chunk(
        root,
        Path::new("/project/src/pages/login.dowe"),
        source,
        &page_tree,
    );
    let second = build_page_chunk(
        root,
        Path::new("/project/src/pages/login.dowe"),
        source,
        &page_tree,
    );

    assert_eq!(first.id, second.id);
    assert_eq!(first.id.len(), 8);
    assert!(
        first
            .id
            .chars()
            .all(|value| value.is_ascii_lowercase() || value.is_ascii_digit())
    );
}

#[test]
fn creates_locale_chunks_and_browser_translation_runtime() {
    let catalog = translations();
    let first = build_translation_chunks(Path::new("/project"), &catalog);
    let second = build_translation_chunks(Path::new("/project"), &catalog);
    assert_eq!(first, second);
    assert_eq!(first.len(), 2);
    assert!(first[0].relative_path.starts_with("web/chunks/i18n"));
    assert!(
        first
            .iter()
            .any(|chunk| chunk.content.contains("Dowe construye sistemas."))
    );

    let tree = ViewNode::Title {
        props: TextProps {
            i18n: Some("home.hero.title".to_string()),
            ..Default::default()
        },
        value: "Dowe builds systems.".to_string(),
    };
    assert!(
        render_page_body(&ViewNode::Children, &tree)
            .contains(r#"data-dowe-i18n="home.hero.title""#)
    );

    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: first,
        default_locale: Some("en".to_string()),
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    });
    assert!(router.contains("navigator.languages"));
    assert!(router.contains("localeChunks"));
    assert!(router.contains("hydrateTranslations"));
}

#[test]
fn emits_portable_svg_import_runtime() {
    let web = super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    };
    let router = super::router_js(&web);

    assert!(router.contains("function stdSvgConvert("));
    assert!(router.contains("function stdSvgRect("));
    assert!(router.contains("function stdSvgColorEqual("));
    assert!(router.contains("function stdSvgOriginalFill("));
    assert!(router.contains("name===\"rect\""));
    assert!(router.contains(r#"fillRule:\"evenodd\""#));
    assert!(router.contains("evenOdd:path.evenOdd===true"));
    assert!(router.contains(r#"path.transform?" transform:\""+path.transform+"\"":"""#));
    assert!(!router.contains(r#"path.transform?`transform:"#));
    assert!(router.contains("case\"parse.svg\":return stdSvgConvert(a.value,a.fallback,a.colors||\"tokens\",a.format||\"source\")"));
    assert!(router.contains("if(format===\"data\")return JSON.stringify"));
}

#[test]
fn preserves_manifest_path_prefix_regex_in_minified_router() {
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    });

    assert!(router.contains(r#"path.replace(/^web\//,"")"#));
    assert!(!router.contains(r#"/^web\function"#));
}

#[test]
fn separates_layout_and_page_chunks() {
    let root = Path::new("/project");
    let layout_tree = layout_tree();
    let page_tree = page_tree();
    let layout = build_layout_chunk(
        root,
        Path::new("/project/src/layouts/auth.dowe"),
        "layout",
        &layout_tree,
    );
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/login.dowe"),
        "page",
        &page_tree,
    );

    assert_eq!(layout.kind, ChunkKind::Layout);
    assert_eq!(page.kind, ChunkKind::Page);
    assert_ne!(layout.relative_path, page.relative_path);
}

