#[test]
fn emits_persistent_view_store_metadata() {
    let tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "session01".to_string(),
            name: "session".to_string(),
            storage_key: "views/store/session:session".to_string(),
            scope: dowe_components::ViewSignalScope::Global,
            storage: dowe_components::ViewSignalStorage::Local,
            initial: ViewSignalValue::Object(vec![(
                "token".to_string(),
                ViewSignalValue::String(String::new()),
            )]),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![text("Session")],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/auth.dowe"),
        "page authPage",
        &tree,
    );

    assert!(
        page.content
            .contains(r#""storageKey":"views/store/session:session""#)
    );
    assert!(
        page.content
            .contains(r#""scope":"global","storage":"local""#)
    );
}

#[test]
fn inherits_container_foreground_and_preserves_text_overrides() {
    let tree = container_foreground_tree();
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/views/pages/colors.dowe"),
        "page ColorsPage",
        &tree,
    );
    let html = render_page_body(&ViewNode::Children, &tree);

    assert!(html.contains(
        "<div class=\"box color-onPrimary\"><p class=\"text-md\">Box inherited</p><p class=\"text-md color-danger\">Box override</p></div>"
    ));
    assert!(html.contains("<article class=\"card"));
    assert!(html.contains("is-soft is-muted"));
    assert!(html.contains("<p class=\"text-md\">Card inherited</p>"));
    assert!(html.contains("<p class=\"title-md color-warning\">Card override</p>"));
    assert!(page
        .css_content
        .contains(".color-onPrimary{color:var(--dowe-onPrimary);}"));
    assert!(page.css_content.contains(
        ".card.is-soft.is-muted{background-color:var(--dowe-softMuted);color:var(--dowe-onSoftMuted);border-color:var(--dowe-softMuted);}"
    ));
}

#[test]
fn rejects_incompatible_persisted_view_store_shapes() {
    let web = super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    };
    let router = super::router_js(&web);

    assert!(router.contains("Object.keys(initial).every"));
    assert!(router.contains(
        "stored===undefined||!compatibleSignalValue(stored,signal.initial)?signal.initial:stored"
    ));
}

#[test]
fn fills_request_path_placeholders_from_signal_names() {
    let web = super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    };
    let router = super::router_js(&web);

    assert!(router.contains("activeView?.signalNames?.[name]||name"));
    assert!(router.contains("readPath(state,binding,scope)"));
}

#[test]
fn emits_constants_outside_web_signal_state() {
    let tree = ViewNode::Scope {
        constants: vec![dowe_components::ViewConstant {
            id: "plans01".to_string(),
            name: "plans".to_string(),
            value: ViewSignalValue::Array(vec![ViewSignalValue::String("Starter".to_string())]),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![text("Plans")],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/plans.dowe"),
        "page plans",
        &tree,
    );
    assert!(page.content.contains(r#""constants":[{"id":"plans01""#));
    assert!(page.content.contains(r#""signals":[]"#));
}

#[test]
fn emits_select_options_from_constant_each() {
    let tree = ViewNode::Scope {
        constants: vec![dowe_components::ViewConstant {
            id: "options01".to_string(),
            name: "options".to_string(),
            value: ViewSignalValue::Array(Vec::new()),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![ViewNode::Select {
            props: Default::default(),
            options: Vec::new(),
            option_each: Some(SelectOptionEach {
                item: "option".to_string(),
                collection: "options".to_string(),
                key: "option.id".to_string(),
                value: "option.value".to_string(),
                label: "option.label".to_string(),
                description: None,
            }),
        }],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/options.dowe"),
        "page options",
        &tree,
    );
    assert!(page.content.contains("data-dowe-each=\\\"options01\\\""));
    assert!(
        page.content
            .contains("data-dowe-option-value-path=\\\"option.value\\\"")
    );
}

#[test]
fn emits_init_and_reactive_splash_boundary() {
    let tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "loading01".to_string(),
            name: "isLoading".to_string(),
            storage_key: "isLoading".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::Bool(true),
            schema: None,
        }],
        actions: vec![ViewAction::init(
            "init01".to_string(),
            vec![ViewFunctionStatement::Assign(ViewAssignAction {
                target: "isLoading".to_string(),
                source: "$dowe:bool:false".to_string(),
                literal: None,
                call: None,
            })],
        )],
        children: vec![ViewNode::Splash {
            binding: "isLoading".to_string(),
            initial: true,
            content: vec![text("Users")],
            children: vec![text("Loading users")],
        }],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/views/pages/users.dowe"),
        "page UsersPage",
        &tree,
    );
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });

    assert!(page.content.contains("data-dowe-splash=\\\"loading01\\\""));
    assert!(page.content.contains("data-dowe-splash-main hidden"));
    assert!(page.content.contains("data-dowe-splash-content"));
    assert!(page.content.contains("\"autoload\":true"));
    assert!(page.content.contains("\"init\":true"));
    assert!(router.contains("renderSplashes"));
    assert!(router.contains("data-dowe-splash-main"));
}

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
    };
    let router = super::router_js(&web);

    assert!(router.contains("function stdSvgConvert("));
    assert!(router.contains(r#"path.transform?" transform:\""+path.transform+"\"":"""#));
    assert!(!router.contains(r#"path.transform?`transform:"#));
    assert!(router.contains("case\"parse.svg\":return stdSvgConvert(a.value,a.fallback)"));
}

#[test]
fn preserves_manifest_path_prefix_regex_in_minified_router() {
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
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

#[test]
fn renders_box_and_text_as_div_and_paragraph() {
    assert_eq!(
        render_page_body(&layout_tree(), &page_tree()),
        r#"<div class="box"><p class="text-md">Layout</p><div class="box"><p class="text-md">Login</p></div></div>"#
    );
}

#[test]
fn renders_section_markup_and_background_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Section {
        props: StyleProps {
            boxed: true,
            text: Some(ResponsiveValue::scalar(ColorToken::OnBackground)),
            background: Some(ResponsiveValue::ordered(vec![
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xs,
                    value: SectionBackground::Soft,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Md,
                    value: SectionBackground::Aurora,
                },
            ])),
            ..Default::default()
        },
        children: vec![text("Hero")],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    let html = render_page_body(&ViewNode::Children, &page_tree);

    assert!(html.contains("<section"));
    assert!(html.contains(
        "<div class=\"section-body is-boxed px-4 md:px-6 py-10 md:py-16\"><p class=\"text-md\">Hero</p></div>"
    ));
    assert!(!html.contains("<section class=\"section is-boxed"));
    assert!(html.contains(
        "section color-onBackground has-background background-soft md:background-aurora"
    ));
    assert!(page.css_content.contains(
        "background-image:linear-gradient(135deg,var(--dowe-surface),var(--dowe-background));"
    ));
    assert!(page.css_content.contains("background-image:linear-gradient(135deg,var(--dowe-softPrimary),var(--dowe-softSecondary),var(--dowe-softTertiary));"));
    assert!(page.css_content.contains("@media (min-width:768px)"));
    let base_vertical_padding = page
        .css_content
        .find(".py-10{padding-top:2.5rem;padding-bottom:2.5rem;}")
        .expect("base section vertical padding");
    let responsive_vertical_padding = page
        .css_content
        .rfind("@media (min-width:768px){.md\\:py-16{padding-top:4rem;padding-bottom:4rem;}}")
        .expect("responsive section vertical padding");
    assert!(base_vertical_padding < responsive_vertical_padding);
    let design_css = super::design_css();
    assert!(design_css.contains(".section-body{width:100%;}"));
    assert!(design_css.contains(".section-body.is-boxed{max-width:96rem;margin-inline:auto;}"));
}

#[test]
fn emits_explicit_section_body_padding_css() {
    let page_tree = ViewNode::Section {
        props: StyleProps {
            spacing: dowe_components::SpacingProps {
                p: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(24))),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Content")],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);

    assert!(html.contains("<section class=\"section\"><div class=\"section-body p-12\">"));
    assert!(page.css_content.contains(".p-12{padding:3rem;}"));
}

#[test]
fn preserves_default_section_horizontal_padding_with_vertical_override() {
    let page_tree = ViewNode::Section {
        props: StyleProps {
            spacing: dowe_components::SpacingProps {
                py: Some(ResponsiveValue::ordered(vec![
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Xs,
                        value: ScaleValue::from_half_steps(12),
                    },
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: ScaleValue::from_half_steps(20),
                    },
                ])),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Content")],
    };
    let html = render_page_body(&ViewNode::Children, &page_tree);

    assert!(html.contains(
        "<section class=\"section\"><div class=\"section-body px-4 md:px-6 py-6 md:py-10\">"
    ));
}

#[test]
fn scopes_layout_and_page_reactivity_by_generated_id() {
    let root = Path::new("/project");
    let layout_tree = reactive_tree("layout01", "action01", true);
    let page_tree = reactive_tree("page0001", "action02", false);
    let layout = build_layout_chunk(
        root,
        Path::new("/project/src/layouts/auth.dowe"),
        "layout reactive",
        &layout_tree,
    );
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/login.dowe"),
        "page reactive",
        &page_tree,
    );
    let html = render_page_body(&layout_tree, &page_tree);

    assert!(layout.content.contains("doweLayout"));
    assert!(layout.content.contains(r#""id":"layout01""#));
    assert!(layout.content.contains(r#""target":"layout01""#));
    assert!(page.content.contains(r#""id":"page0001""#));
    assert!(page.content.contains(r#""target":"page0001""#));
    assert!(html.contains(r#"data-dowe-bind="layout01.message""#));
    assert!(html.contains(r#"data-dowe-bind="page0001.message""#));
    assert!(html.contains(r#"data-dowe-click="action01""#));
    assert!(html.contains(r#"data-dowe-click="action02""#));
}

#[test]
fn emits_web_manifest_and_html_artifacts() {
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
    let layout_js = strip_web_for_test(&layout.relative_path);
    let page_js = strip_web_for_test(&page.relative_path);
    let layout_css = strip_web_for_test(&layout.css_relative_path);
    let page_css = strip_web_for_test(&page.css_relative_path);
    let body_html =
        super::render_routed_page_body(&layout_tree, &page_tree, &[layout.id.clone()], &page.id);
    let mut view_page = super::ViewPage {
        id: "login".to_string(),
        route_path: "/login".to_string(),
        source_path: Path::new("/project/src/pages/login.dowe").to_path_buf(),
        layout_tree,
        page_tree,
        body_html,
        html_document: String::new(),
        layout_text: "Layout".to_string(),
        page_text: "Login".to_string(),
        layout_chunk_id: layout.id.clone(),
        page_chunk_id: page.id.clone(),
        layout_chunk_ids: vec![layout.id.clone()],
        js_chunks: vec![layout_js, page_js],
        css_chunks: vec![layout_css, page_css],
        boundaries: vec![format!("layout:{}", layout.id), format!("page:{}", page.id)],
        sections: Vec::new(),
        navigation_actions: Vec::new(),
        metadata: Vec::new(),
    };
    view_page.html_document = super::render_page_document(&view_page);
    assert!(
        view_page
            .html_document
            .contains(r#"<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=5, viewport-fit=cover, interactive-widget=resizes-content">"#)
    );
    assert!(
        view_page
            .html_document
            .contains(r#"<link rel="icon" href="data:image/svg+xml,"#)
    );
    let mut web = super::WebOutput {
        chunks: vec![layout, page],
        pages: vec![view_page],
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    };
    web.router_js = super::router_js(&web);
    let artifacts = web_artifacts(&web, &FontConfig::default(), &DesignConfig::default());

    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.relative_path == Path::new("web/manifest.json"))
    );
    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.relative_path == Path::new("web/pages/login.html"))
    );
    let index = artifacts
        .iter()
        .find(|artifact| artifact.relative_path == Path::new("web/index.html"))
        .expect("index");
    assert!(index.content.contains(r#"href="design.css""#));
    assert!(index.content.contains(r#"src="chunks/layouts/"#));
    let page = artifacts
        .iter()
        .find(|artifact| artifact.relative_path == Path::new("web/pages/login.html"))
        .expect("page");
    assert!(page.content.contains(r#"href="../design.css""#));
    assert!(page.content.contains(r#"src="../chunks/layouts/"#));
    assert!(web.pages[0].html_document.contains(r#"href="/design.css""#));
    assert!(super::manifest(&web).contains(r#""staticFile":"web/pages/login.html""#));
    assert!(web.router_js.contains("staticMode"));
    assert!(web.router_js.contains("doweHref"));
    assert!(web.router_js.contains("function positionSelect(control)"));
    assert!(
        web.router_js
            .contains("function mountSelectPopover(control)")
    );
    assert!(web.router_js.contains("document.body.appendChild(popover)"));
    assert!(web.router_js.contains("popover.__doweControl"));
    assert!(
        web.router_js
            .contains("const above=bottom<Math.min(height,224)&&top>bottom")
    );
    assert!(
        web.router_js
            .contains("scrollIntoView({behavior:reduce?\"auto\":\"smooth\",block:\"start\"})")
    );
    assert!(
        web.router_js
            .contains("if(\"scrollRestoration\"in history)history.scrollRestoration=\"manual\"")
    );
    assert!(web.router_js.contains("function pageScrollViewport()"));
    assert!(
        web.router_js
            .contains("viewport.scrollTop=0;viewport.scrollLeft=0")
    );
    assert!(
        web.router_js
            .contains("viewport.style.scrollBehavior=\"auto\"")
    );
    assert!(
        web.router_js
            .contains("viewport.style.scrollBehavior=behavior")
    );
    assert!(
        web.router_js
            .contains("scrollToPageDestination(currentFragment)")
    );
    assert!(
        web.router_js
            .contains("new RegExp(\"^https?:/{2}\",\"i\").test(source)")
    );
    assert!(
        web.router_js
            .contains("const boundary=document.querySelector('[data-dowe-boundary^=\"page:\"]')")
    );
    assert!(
        web.router_js
            .contains("boundary.outerHTML=wrapPage(route,page.render())")
    );
    assert!(web.router_js.contains("window.__doweHotUpdate=hotUpdate"));
    assert!(
        web.router_js
            .contains("fetch(versionedAsset(\"manifest.json\",version)")
    );
    assert!(
        web.router_js
            .contains("hydrate(route,modules,preserveLayouts,true)")
    );
    assert!(web.router_js.contains("previous.state[signal.id]"));
    assert!(
        web.router_js
            .contains("const boundState=captureBoundState(app)")
    );
    assert!(web.router_js.contains("restoreBoundState(boundState)"));
    assert!(
        web.router_js
            .contains("compatibleSignalValue(previous.state[signal.id]")
    );
    assert!(web
        .router_js
        .contains("if(current&&!version){document.head.appendChild(current)"));
    assert!(web
        .router_js
        .contains("return Promise.all(route.cssChunks.map(path=>loadCss(path"));
    assert!(web.router_js.contains(
        "if(!document.querySelector('script[src=\"/_dowe/dev/client.js\"]'))return"
    ));
    assert!(web
        .router_js
        .contains("await syncDevRoutes();const route=routes[destination.path]"));
    assert_eq!(
        web.router_js.matches("await loadRouteCss(route").count(),
        2
    );
    assert!(web
        .router_js
        .contains("reject(new Error(\"Dowe CSS chunk failed: \"+link.href))"));
    assert!(web.router_js.contains(
        "if(options.writeHistory===false||options.replace)location.replace(destination.href)"
    ));
    assert!(web.router_js.contains("if(current)current.remove()"));
    assert!(web.router_js.contains("history.pushState"));
}

#[test]
fn emits_container_refactor_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Grid {
        props: GridProps {
            columns: Some(ResponsiveValue::scalar(GridTracks::Count(3))),
            rows: Some(ResponsiveValue::scalar(GridTracks::Template(
                "100px auto".to_string(),
            ))),
            justify: Some(ResponsiveValue::scalar(GridAlignment::Center)),
            gap: Some(ResponsiveValue::scalar(GapValue::Pair(
                GapSize::Px(10),
                GapSize::Px(20),
            ))),
            ..Default::default()
        },
        children: vec![
            ViewNode::Box {
                props: StyleProps {
                    cover: Some(ResponsiveValue::ordered(vec![
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Xs,
                            value: CoverSource("/mobile.jpg".to_string()),
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Md,
                            value: CoverSource("/desktop.jpg".to_string()),
                        },
                    ])),
                    overlay: Some(ResponsiveValue::scalar(OverlayPaint::BlackOpacity(
                        "0.6".to_string(),
                    ))),
                    grid_item: Some(Box::new(dowe_components::GridItemProps {
                        col_span: Some(ResponsiveValue::scalar(GridSpan(2))),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                children: vec![text("Hero")],
            },
            ViewNode::Card {
                props: VariantProps {
                    variant: Some(ComponentVariant::Soft),
                    color: Some(dowe_components::ColorFamily::Surface),
                    ..Default::default()
                },
                children: vec![text("Card")],
            },
        ],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("grid-cols-3"));
    assert!(page.content.contains("grid-justify-center"));
    assert!(page.content.contains("col-span-2"));
    assert!(page.content.contains("has-cover"));
    assert!(page.content.contains("has-overlay"));
    assert!(
        page.css_content
            .contains("grid-template-columns:repeat(3,minmax(0,1fr));")
    );
    assert!(page.css_content.contains("grid-template-rows:100px auto;"));
    assert!(page.css_content.contains("row-gap:10px;column-gap:20px;"));
    assert!(
        page.css_content
            .contains("background-image:url(\"/mobile.jpg\")")
    );
    assert!(page.css_content.contains("@media (min-width:768px)"));
    assert!(page.css_content.contains("rgba(0,0,0,0.6)"));
    assert!(page.css_content.contains(".card.is-soft.is-surface"));
}

#[test]
fn emits_portable_box_positioning_css() {
    let page_tree = ViewNode::Box {
        props: StyleProps {
            position: Some(Box::new(dowe_components::PositionProps {
                mode: BoxPosition::Relative,
                ..Default::default()
            })),
            ..Default::default()
        },
        children: vec![
            ViewNode::Box {
                props: StyleProps {
                    position: Some(Box::new(dowe_components::PositionProps {
                        mode: BoxPosition::Absolute,
                        top: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                        right: Some(ResponsiveValue::ordered(vec![
                            ResponsiveEntry {
                                breakpoint: Breakpoint::Xs,
                                value: ScaleValue::from_half_steps(8),
                            },
                            ResponsiveEntry {
                                breakpoint: Breakpoint::Md,
                                value: ScaleValue::from_half_steps(12),
                            },
                        ])),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                children: vec![text("Proof")],
            },
            ViewNode::Box {
                props: StyleProps {
                    position: Some(Box::new(dowe_components::PositionProps {
                        mode: BoxPosition::Fixed,
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                children: vec![text("Persistent")],
            },
        ],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/positioning.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("position-relative"));
    assert!(page.content.contains("position-absolute"));
    assert!(page.content.contains("position-fixed top-0 left-0"));
    assert!(page.content.contains("top-4"));
    assert!(page.content.contains("right-4 md:right-6"));
    assert!(page.css_content.contains(".position-relative{position:relative;}"));
    assert!(page.css_content.contains(".position-absolute{position:absolute;}"));
    assert!(page.css_content.contains(".position-fixed{position:fixed;}"));
    assert!(page.css_content.contains(".top-4{top:1rem;}"));
    assert!(page.css_content.contains(".top-0{top:0rem;}"));
    assert!(page.css_content.contains(".left-0{left:0rem;}"));
    assert!(page.css_content.contains(".right-4{right:1rem;}"));
    assert!(page.css_content.contains(
        "@media (min-width:768px){.md\\:right-6{right:1.5rem;}}"
    ));
}

#[test]
fn emits_reset_and_font_css() {
    let css = super::design_css();

    assert!(css.contains("body{margin:0;"));
    assert!(css.contains("p,h1,h2,h3,h4,h5,h6{margin:0;"));
    assert!(css.contains("a{color:inherit;text-decoration:inherit;}"));
    assert!(css.contains("button,input,textarea,select{font:inherit;color:inherit;margin:0;}"));
    assert!(css.contains("--dowe-font-inter"));
    assert!(css.contains("@font-face{font-family:\"Dowe Inter\""));
    assert!(css.contains("src:url(\"/fonts/inter/inter-regular.ttf\") format(\"truetype\")"));
}

#[test]
fn rewrites_static_route_hrefs_for_desktop_fallback() {
    let document = r##"<a class="button" href="/signup#join" data-dowe-nav="push" data-dowe-href="/signup#join">Signup</a><a class="button" href="/" data-dowe-nav="push" data-dowe-href="/">Home</a><link rel="stylesheet" href="/design.css">"##;
    let index = super::static_html_document(document, "");
    let page = super::static_html_document(document, "../");

    assert!(index.contains(
        r##"href="pages/signup.html#join" data-dowe-nav="push" data-dowe-href="/signup#join""##
    ));
    assert!(index.contains(r##"href="index.html" data-dowe-nav="push" data-dowe-href="/""##));
    assert!(index.contains(r#"href="design.css""#));
    assert!(page.contains(
        r##"href="signup.html#join" data-dowe-nav="push" data-dowe-href="/signup#join""##
    ));
    assert!(page.contains(r##"href="../index.html" data-dowe-nav="push" data-dowe-href="/""##));
    assert!(page.contains(r#"href="../design.css""#));
}
