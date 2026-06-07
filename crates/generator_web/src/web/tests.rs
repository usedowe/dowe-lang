use super::{
    ChunkKind, build_layout_chunk, build_page_chunk, build_translation_chunks, render_page_body,
    web_artifacts,
};
use dowe_components::{
    AccordionItem, AccordionProps, AlertDialogProps, AudioProps, AvatarProps, AvatarStatus,
    BadgeProps, BarProps, Breakpoint, ButtonSize, CarouselIndicatorType, CarouselOrientation,
    CarouselProps, CarouselSlide, CheckboxProps, ChipProps, ColorFamily, ColorProps, ColorToken,
    CommandEntry, CommandProps, ComponentProp, ComponentVariant, CoverSource, DateProps,
    DateRangeProps, DesignConfig, DividerOrientation, DividerProps, DrawerPosition, DrawerProps,
    DropdownProps, ElementProps, FontConfig, GapSize, GapValue, GridAlignment, GridProps, GridSpan,
    GridTracks, ImageAspect, ImageLoading, ImageObjectFit, ImageProps, ModalProps, NavMenuItem,
    NavMenuItemProps, NavMenuProps, NavigationAction, NavigationOperation, OverlayCornerPosition,
    OverlayEntry, OverlayItemProps, OverlayPaint, OverlayPosition, PropValue, RadioGroupProps,
    RadioOption, ResponsiveEntry, ResponsiveValue, RoundedSize, ScaffoldProps, SectionBackground,
    SelectOption, SideNavItem, SideNavItemProps, SideNavProps, SideNavSize, SkeletonAnimation,
    SkeletonProps, SkeletonVariant, StyleProps, SvgPath, SvgPathFill, SvgProps, SvgViewBox,
    TabItem, TabsPosition, TabsProps, TabsVariant, TextProps, TextWeight, ToastKind, ToastProps,
    ToggleProps, TooltipProps, TranslationCatalog, TranslationLocale, TranslationValue,
    VariantProps, VideoAspect, VideoProps, ViewAction, ViewActionKind, ViewAnimation, ViewNode,
    ViewResetAction, ViewSignal, ViewSignalValue, VisibilityCondition,
};
use std::path::{Path, PathBuf};

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
        "section color-onBackground has-background background-soft md:background-aurora"
    ));
    assert!(page.css_content.contains(
        "background-image:linear-gradient(135deg,var(--dowe-surface),var(--dowe-background));"
    ));
    assert!(page.css_content.contains("background-image:linear-gradient(135deg,var(--dowe-softPrimary),var(--dowe-softSecondary),var(--dowe-softTertiary));"));
    assert!(page.css_content.contains("@media (min-width:768px)"));
}

#[test]
fn renders_code_markup_theme_classes_and_copy_runtime() {
    let tree = code_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains("data-dowe-code"));
    assert!(html.contains("data-dowe-code-copy"));
    assert!(html.contains("code-token-keyword"));
    assert!(html.contains("code-token-type"));
    assert!(html.contains("docsPage"));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/docs.dowe"),
        "page docsPage",
        &tree,
    );
    assert!(chunk.css_content.contains(".code-block.is-soft.is-surface"));
    assert!(
        super::router_js(&super::WebOutput {
            chunks: Vec::new(),
            pages: Vec::new(),
            translation_chunks: Vec::new(),
            default_locale: None,
            router_js: String::new(),
        })
        .contains("navigator.clipboard")
    );
}

#[test]
fn renders_video_markup_theme_classes_and_hls_runtime() {
    let tree = video_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains("data-dowe-video"));
    assert!(html.contains("controls playsinline"));
    assert!(html.contains(r#"poster="/images/video.jpg""#));
    assert!(html.contains(r#"class="video horizontal is-solid is-surface""#));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/video.dowe"),
        "page videoPage",
        &tree,
    );
    assert!(chunk.css_content.contains(".video.is-solid.is-surface"));
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });
    assert!(router.contains("application/vnd.apple.mpegurl"));
    assert!(router.contains("https://cdn.jsdelivr.net/npm/hls.js@1/dist/hls.min.js"));
    assert!(router.contains("hls.loadSource(source)"));
}

#[test]
fn renders_candlestick_markup_theme_classes_and_stream_runtime() {
    let tree = candlestick_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains("data-dowe-candlestick"));
    assert!(html.contains(r#"data-dowe-candlestick-data="candles""#));
    assert!(html.contains(r#"data-dowe-candlestick-stream="/api/candles""#));
    assert!(html.contains(r#"class="candlestick"#));
    assert!(html.contains("is-soft"));
    assert!(html.contains("is-surface"));
    assert!(html.contains("Market closed"));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/market.dowe"),
        "page marketPage",
        &tree,
    );
    assert!(
        chunk
            .css_content
            .contains(".candlestick.is-soft.is-surface")
    );
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });
    assert!(router.contains("new EventSource(stream)"));
    assert!(router.contains("upsertCandles"));
    assert!(router.contains("renderCandlestick"));
}

#[test]
fn renders_table_markup_css_and_runtime() {
    let tree = table_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(
        html.contains(
            r#"class="table is-lg is-soft is-surface is-striped is-bordered has-dividers""#
        )
    );
    assert!(html.contains(r#"data-dowe-table-data="users""#));
    assert!(html.contains(r#"data-dowe-table-field="status""#));
    assert!(html.contains(r#"data-dowe-table-align="end""#));
    assert!(html.contains(r#"style="text-align:end;width:8rem""#));
    assert!(html.contains("No users"));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/users.dowe"),
        "page usersPage",
        &tree,
    );
    assert!(chunk.css_content.contains(".table.is-soft.is-surface"));
    assert!(chunk.css_content.contains(".table-container"));
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });
    assert!(router.contains("renderTable"));
    assert!(router.contains("tableCellValue"));
}

#[test]
fn renders_divider_markup_orientation_and_scheme_css() {
    let tree = divider_tree();
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains(r#"class="divider divider-vertical is-primary""#));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/divider.dowe"),
        "page dividerPage",
        &tree,
    );
    assert!(super::design_css().contains(".divider{--dowe-component-display:block;"));
    assert!(chunk.css_content.contains(
        ".divider.is-primary{background-color:var(--dowe-primary);color:var(--dowe-primary);}"
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
    };
    view_page.html_document = super::render_page_document(&view_page);
    assert!(
        view_page
            .html_document
            .contains(r#"<meta name="viewport" content="width=device-width, initial-scale=1">"#)
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
                    grid_item: dowe_components::GridItemProps {
                        col_span: Some(ResponsiveValue::scalar(GridSpan(2))),
                        ..Default::default()
                    },
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
fn emits_view_motion_markup_and_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Box {
        props: StyleProps {
            animation: Some(ViewAnimation::FadeIn),
            ..Default::default()
        },
        children: vec![ViewNode::Card {
            props: VariantProps {
                style: StyleProps {
                    animation: Some(ViewAnimation::SlideUp),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Motion")],
        }],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/motion.dowe"),
        "page",
        &page_tree,
    );
    let css = super::design_css();

    assert!(page.content.contains("animate-fade-in"));
    assert!(page.content.contains("animate-slide-up"));
    assert!(
        page.css_content
            .contains(".animate-fade-in{animation:dowe-fade-in 220ms ease-out both;}")
    );
    assert!(
        page.css_content
            .contains(".animate-slide-up{animation:dowe-slide-up 220ms ease-out both;}")
    );
    assert!(css.contains("@keyframes dowe-fade-in"));
    assert!(css.contains("@media (prefers-reduced-motion:reduce)"));
}

#[test]
fn emits_button_size_and_variant_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Button {
        props: VariantProps {
            variant: Some(ComponentVariant::Soft),
            color: Some(ColorFamily::Warning),
            size: Some(ButtonSize::Lg),
            style: StyleProps {
                rounded: Some(ResponsiveValue::scalar(RoundedSize::Full)),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Warn")],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("button-lg"));
    assert!(page.content.contains("rounded-full"));
    assert!(page.content.contains("is-soft"));
    assert!(page.content.contains("is-warning"));
    assert!(
        page.css_content
            .contains(".button-lg{padding:0.75rem 1.25rem;min-height:2.75rem;}")
    );
    assert!(
        page.css_content
            .contains(".rounded-full{border-radius:9999px;}")
    );
    assert!(page.css_content.contains(".button.is-soft.is-warning"));
}

#[test]
fn emits_text_weight_override_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Text {
        props: TextProps {
            weight: Some(ResponsiveValue::ordered(vec![
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xs,
                    value: TextWeight::Thin,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Md,
                    value: TextWeight::Extralight,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Lg,
                    value: TextWeight::Black,
                },
            ])),
            ..Default::default()
        },
        value: "Weight".to_string(),
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("weight-thin"));
    assert!(page.content.contains("md:weight-extralight"));
    assert!(page.content.contains("lg:weight-black"));
    assert!(page.css_content.contains(".weight-thin{font-weight:100;}"));
    assert!(
        page.css_content
            .contains(".md\\:weight-extralight{font-weight:200;}")
    );
    assert!(
        page.css_content
            .contains(".lg\\:weight-black{font-weight:900;}")
    );
}

#[test]
fn emits_show_visibility_markup_and_css() {
    let root = Path::new("/project");
    let page_tree = show_tree();
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/ready.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("show-false md:show-true"));
    assert!(page.content.contains(r#"data-dowe-show=\"ready01\""#));
    assert!(!page.css_content.contains(".show-false{display:none;}"));
    assert!(!page.css_content.contains(".md\\:show-true"));
    let design_css = show_design_css();
    assert!(design_css.contains(".show-false{display:none;}"));
    assert!(design_css.contains(
        "@media (min-width:768px){.md\\:show-false{display:none;}.md\\:show-true{display:var(--dowe-component-display,revert);}}"
    ));
    assert!(
        super::router_js(&super::WebOutput {
            chunks: Vec::new(),
            pages: Vec::new(),
            translation_chunks: Vec::new(),
            default_locale: None,
            router_js: String::new(),
        })
        .contains("data-dowe-show")
    );
}

#[test]
fn keeps_nested_layout_visibility_rules_order_safe() {
    let root = Path::new("/project");
    let parent = build_layout_chunk(
        root,
        Path::new("/project/src/layouts/docs.dowe"),
        "docs",
        &show_tree(),
    );
    let child = build_layout_chunk(
        root,
        Path::new("/project/src/layouts/views.dowe"),
        "views",
        &ViewNode::Box {
            props: StyleProps {
                element: ElementProps {
                    show: Some(VisibilityCondition::Static(responsive_bool(&[
                        (Breakpoint::Xs, false),
                        (Breakpoint::Lg, true),
                    ]))),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Views")],
        },
    );
    let design_css = show_design_css();

    assert!(!parent.css_content.contains(".show-false"));
    assert!(!child.css_content.contains(".show-false"));
    assert!(design_css.contains(".show-false{display:none;}"));
    assert!(design_css.contains(".md\\:show-true{display:var(--dowe-component-display,revert);}"));
    assert!(design_css.contains(".lg\\:show-true{display:var(--dowe-component-display,revert);}"));
}

#[test]
fn renders_layout_bars_markup_and_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::AppBar {
                props: bar_props(true),
                start: vec![text("Menu")],
                center: vec![text("Brand")],
                end: vec![text("Account")],
            },
            ViewNode::Footer {
                props: bar_props(false),
                start: vec![text("Footer")],
                center: Vec::new(),
                end: vec![text("Legal")],
            },
            ViewNode::BottomBar {
                props: bar_props(false),
                start: vec![text("Home")],
                center: vec![text("Search")],
                end: vec![text("Profile")],
            },
        ],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);
    let css = super::design_css();

    assert!(html.contains(
            r#"<header class="appbar is-soft is-surface is-bordered is-blurred is-floating"><div class="appbar-content is-boxed">"#
        ));
    assert!(html.contains(r#"<div class="appbar-start">"#));
    assert!(html.contains(r#"<footer class="footer is-soft is-surface is-bordered is-blurred">"#));
    assert!(html.contains(r#"<nav class="bottombar is-soft is-surface is-bordered is-blurred">"#));
    assert!(css.contains(".appbar,.footer,.bottombar{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));width:100%;"));
    assert!(
        css.contains(
            ".appbar-content.is-boxed,.footer-content.is-boxed,.bottombar-content.is-boxed"
        )
    );
    assert!(page.css_content.contains(".appbar.is-soft.is-surface"));
    assert!(page.css_content.contains(".bottombar.is-soft.is-surface"));
}

#[test]
fn renders_side_nav_markup_active_runtime_and_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::SideNav {
        props: SideNavProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Ghost),
                color: Some(ColorFamily::Background),
                ..Default::default()
            },
            size: SideNavSize::Md,
            wide: true,
        },
        items: vec![
            SideNavItem::Header(SideNavItemProps {
                label: "Workspace".to_string(),
                description: Some("Account navigation".to_string()),
                status: None,
                icon: None,
                on_click: None,
                navigation: None,
            }),
            SideNavItem::Item(side_nav_item("Home", "/")),
            SideNavItem::Divider,
            SideNavItem::Submenu {
                props: SideNavItemProps {
                    label: "Content".to_string(),
                    description: None,
                    status: Some("2".to_string()),
                    icon: None,
                    on_click: None,
                    navigation: None,
                },
                open: true,
                items: vec![side_nav_item("Blogs", "/blogs")],
            },
        ],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);
    let css = super::design_css();

    assert!(html.contains(r#"<nav class="sidenav is-ghost is-background sidenav-md is-wide""#));
    assert!(html.contains(r#"data-dowe-sidenav-href="/blogs""#));
    assert!(
        html.contains(
            r#"<details class="sidenav-submenu is-open" data-dowe-sidenav-submenu open>"#
        )
    );
    assert!(html.contains(r#"aria-expanded="true""#));
    assert!(html.contains(r#"<div class="sidenav-divider"></div>"#));
    assert!(css.contains(".sidenav{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));flex-direction:column;"));
    assert!(css.contains(".sidenav-submenu-content{display:flex;flex-direction:column;gap:0.125rem;max-height:0;overflow:hidden;"));
    assert!(
        css.contains(
            ".sidenav-submenu.is-open>.sidenav-submenu-content{max-height:40rem;opacity:1;"
        )
    );
    assert!(
        page.css_content
            .contains(".sidenav.is-ghost.is-background .sidenav-entry.is-active{background-color:transparent;color:var(--dowe-onBackground);border-color:transparent;}")
    );
    assert!(
        super::router_js(&super::WebOutput {
            chunks: Vec::new(),
            pages: Vec::new(),
            translation_chunks: Vec::new(),
            default_locale: None,
            router_js: String::new(),
        })
        .contains("toggleNavTreeSubmenu(\"sidenav\"")
    );
}

#[test]
fn renders_navigation_shell_markup_runtime_and_css() {
    let root = Path::new("/project");
    let page_tree = navigation_shell_tree();
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);
    let css = super::design_css();
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });

    assert!(html.contains(r#"<div class="scaffold is-boxed">"#));
    assert!(html.contains(r#"<div class="scaffold-body">"#));
    assert!(html.contains(r#"<main class="scaffold-main">"#));
    assert!(html.contains(r#"<nav class="navmenu is-ghost is-muted navmenu-md""#));
    assert!(html.contains(r#"data-dowe-navmenu-trigger="1""#));
    assert!(html.contains(r#"data-dowe-navmenu-popover="2""#));
    assert!(html.contains(r#"data-dowe-navmenu-href="/docs""#));
    assert!(html.contains("Resource hub"));
    assert!(html.contains(r#"<nav class="sidebar is-soft is-surface sidebar-md is-wide""#));
    assert!(html.contains(r#"data-dowe-sidebar-href="/""#));
    assert!(css.contains(".navmenu{--dowe-component-display:flex"));
    assert!(css.contains(".sidebar{--dowe-component-display:flex"));
    assert!(css.contains(".scaffold{--dowe-component-display:flex"));
    assert!(
        page.css_content
            .contains(".navmenu.is-ghost.is-muted .navmenu-item.is-active")
    );
    assert!(
        page.css_content
            .contains(".sidebar.is-soft.is-surface .sidebar-entry.is-active")
    );
    assert!(router.contains("openNavMenu"));
    assert!(router.contains("hydrateNavTreeSubmenus(root,\"sidebar\")"));
    assert!(router.contains("data-dowe-navmenu-href"));
}

#[test]
fn renders_tabs_markup_runtime_and_css() {
    let root = Path::new("/project");
    let page_tree = tabs_tree();
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });

    assert!(html.contains(r#"<div class="tabs is-start" data-dowe-tabs>"#));
    assert!(html.contains(r#"<div class="tabs-list is-line is-primary" role="tablist">"#));
    assert!(html.contains(r#"role="tab" id="tab-overview-button" aria-selected="true""#));
    assert!(html.contains(r#"tabindex="-1" data-dowe-tab="details""#));
    assert!(html.contains(r#"role="tabpanel" aria-labelledby="tab-details-button" data-dowe-tab-panel="details" hidden"#));
    assert!(html.contains("Overview content"));
    assert!(page.css_content.contains(".tabs-list.is-line.is-primary .tab.on-active"));
    assert!(page.css_content.contains(".tabs.is-start .tabs-list.is-line.is-primary .tab.on-active"));
    assert!(router.contains("function setActiveTab(root,id)"));
    assert!(router.contains("[data-dowe-tab]"));
}

#[test]
fn renders_drawer_markup_runtime_and_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Drawer {
        props: DrawerProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Soft),
                color: Some(ColorFamily::Surface),
                ..Default::default()
            },
            open: "drawerOpen".to_string(),
            position: DrawerPosition::End,
            disable_overlay_close: true,
            hide_close_button: false,
        },
        children: vec![text("Navigation")],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);
    let css = super::design_css();
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });

    assert!(html.contains(r#"class="drawer-panel" data-dowe-drawer data-dowe-drawer-open="drawerOpen" data-dowe-drawer-disable-overlay-close="true" hidden"#));
    assert!(
        html.contains(
            r#"class="drawer is-soft is-surface is-end" role="dialog" aria-modal="true""#
        )
    );
    assert!(html.contains(r#"data-dowe-drawer-close"#));
    assert!(css.contains(
        ".drawer-panel{--dowe-component-display:flex;position:fixed;inset:0;z-index:50;"
    ));
    assert!(css.contains(".drawer.is-end{inset-block:0;inset-inline-end:0;width:min(20rem,100vw);border-start-end-radius:0;border-end-end-radius:0;transform:translateX(100%);"));
    assert!(css.contains(".drawer.is-start{inset-block:0;inset-inline-start:0;width:min(20rem,100vw);border-start-start-radius:0;border-end-start-radius:0;"));
    assert!(css.contains(".drawer.is-top{inset-inline:0;top:0;max-height:min(20rem,100vh);border-start-start-radius:0;border-start-end-radius:0;"));
    assert!(css.contains(".drawer.is-bottom{inset-inline:0;bottom:0;max-height:min(20rem,100vh);border-end-start-radius:0;border-end-end-radius:0;"));
    assert!(page.css_content.contains(".drawer.is-soft.is-surface"));
    assert!(router.contains("function closeDrawer(drawer)"));
    assert!(router.contains("data-dowe-drawer-overlay"));

    let rounded_html = render_page_body(
        &ViewNode::Children,
        &ViewNode::Drawer {
            props: DrawerProps {
                style: VariantProps {
                    variant: Some(ComponentVariant::Soft),
                    color: Some(ColorFamily::Surface),
                    style: StyleProps {
                        rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                open: "drawerOpen".to_string(),
                position: DrawerPosition::Start,
                disable_overlay_close: false,
                hide_close_button: false,
            },
            children: vec![text("Navigation")],
        },
    );
    assert!(rounded_html.contains("drawer rounded-lg is-soft is-surface is-start"));
}

#[test]
fn renders_display_and_overlay_components_markup_runtime_and_css() {
    let root = Path::new("/project");
    let page_tree = display_overlay_tree();
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/overlays.dowe"),
        "page",
        &page_tree,
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);
    let css = super::design_css();
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });

    assert!(html.contains(r#"class="avatar is-soft is-success avatar-lg is-bordered""#));
    assert!(html.contains(r#"class="badge is-solid is-danger is-bottom-right""#));
    assert!(html.contains(r#"class="chip is-outlined is-info chip-sm has-close""#));
    assert!(html.contains(r#"class="skeleton"#));
    assert!(html.contains("is-pulse"));
    assert!(html.contains("is-rounded"));
    assert!(html.contains(r#"data-dowe-modal data-dowe-modal-open="modal01""#));
    assert!(html.contains(r#"class="alert-dialog-actions""#));
    assert!(html.contains(r#"class="tooltip-popover is-solid is-muted position-end""#));
    assert!(html.contains(r#"class="toast"#));
    assert!(html.contains("is-solid"));
    assert!(html.contains("is-success"));
    assert!(html.contains(r#"class="dropdown-popover is-solid is-surface""#));
    assert!(html.contains(r#"data-dowe-command-open="modal01""#));
    assert!(page.css_content.contains(".avatar.is-soft.is-success"));
    assert!(page.css_content.contains(".modal.is-solid.is-surface"));
    assert!(page.css_content.contains(".dropdown-popover.is-solid.is-surface"));
    assert!(css.contains(".tooltip-popover{position:fixed;"));
    assert!(css.contains("@keyframes dowe-skeleton-pulse"));
    assert!(router.contains("function renderModals(root,state,scope)"));
    assert!(router.contains("function renderToasts(root,state,scope)"));
    assert!(router.contains("function openCommand(command)"));
    assert!(router.contains("data-dowe-dropdown-trigger"));
}

#[test]
fn renders_media_display_and_form_components_markup_runtime_and_css() {
    let root = Path::new("/project");
    let page_tree = media_display_form_tree();
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/components.dowe"),
        "page",
        &page_tree,
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);
    let css = super::design_css();
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });

    assert!(html.contains(r#"class="media is-soft is-primary""#));
    assert!(html.contains(r#"data-dowe-audio"#));
    assert!(html.contains(r#"class="image is-solid is-secondary square fit-contain""#));
    assert!(html.contains(r#"data-dowe-image"#));
    assert!(html.contains(r#"data-dowe-accordion data-dowe-accordion-multiple="true""#));
    assert!(html.contains(r#"data-dowe-carousel data-dowe-carousel-index="0""#));
    assert!(html.contains(r#"class="checkbox-input is-success""#));
    assert!(html.contains(r#"data-dowe-bind="accepted""#));
    assert!(html.contains(r#"class="color-input" type="color""#));
    assert!(html.contains(r#"class="date-input" type="date""#));
    assert!(html.contains(r#"class="radio is-muted is-lg""#));
    assert!(html.contains(r#"class="toggle-input is-secondary""#));

    assert!(page.css_content.contains(".media.is-soft.is-primary"));
    assert!(page.css_content.contains(".accordion.is-outlined.is-surface"));
    assert!(page.css_content.contains(".carousel.is-solid.is-info"));
    assert!(css.contains(".checkbox-input{position:relative;"));
    assert!(css.contains(".toggle-input{position:relative;"));
    assert!(css.contains(".date-range-inputs{display:flex;"));
    assert!(router.contains("function hydrateAudios(root)"));
    assert!(router.contains("function toggleAccordion(trigger)"));
    assert!(router.contains("function renderCarousel(root)"));
    assert!(router.contains("function downloadImage(root)"));
}

#[test]
fn emits_portable_input_metrics_and_outlined_colors() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Input {
        props: VariantProps {
            variant: Some(ComponentVariant::Outlined),
            color: Some(ColorFamily::Secondary),
            ..Default::default()
        },
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );
    let css = super::design_css();

    assert!(css.contains(
            ".control{--dowe-component-display:flex;position:relative;display:var(--dowe-show,var(--dowe-component-display));align-items:center;width:100%;min-height:2.5rem;"
        ));
    assert!(css.contains("min-height:2.5rem;padding:0 0.75rem;"));
    assert!(css.contains(".field{display:flex;flex-direction:column;"));
    assert!(css.contains(".select-popover{position:fixed;"));
    assert!(
        css.contains("transition:opacity 160ms ease,transform 160ms ease,visibility 160ms ease;")
    );
    assert!(css.contains(".select-popover.is-active{opacity:1;visibility:visible;pointer-events:auto;transform:translateY(0) scale(1);"));
    assert!(css.contains(".select-arrow{width:1em;height:1em;"));
    assert!(css.contains(".alert{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));align-items:center;justify-content:space-between;gap:0.75rem;padding:0.625rem 0.875rem;border-radius:var(--dowe-radiusUi);}"));
    assert!(css.contains(
            ".select-control.is-floating:not(.is-open):not(.has-value) .select-value{visibility:hidden;}"
        ));
    assert!(css.contains("font-size:clamp(0.875rem, 0.82rem + 0.25vw, 1rem)"));
    assert!(css.contains(".grid>[data-dowe-each],.flex>[data-dowe-each]{display:contents;}"));
    assert!(page.css_content.contains(
            ".control.is-outlined.is-secondary{background-color:var(--dowe-background);color:var(--dowe-secondary);border:1px solid rgba(127,127,127,0.36);}"
        ));
    assert!(page.css_content.contains(
        ".control.is-outlined.is-secondary:focus-within{border-color:var(--dowe-secondary);"
    ));
}

#[test]
fn renders_labeled_input_and_select_markup() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::Input {
                props: VariantProps {
                    label: Some("Name".to_string()),
                    placeholder: Some("Full name".to_string()),
                    label_floating: true,
                    ..Default::default()
                },
            },
            ViewNode::Select {
                props: VariantProps {
                    label: Some("Role".to_string()),
                    placeholder: Some("Choose role".to_string()),
                    label_floating: true,
                    ..Default::default()
                },
                options: vec![
                    SelectOption {
                        value: "admin".to_string(),
                        label: "Admin".to_string(),
                        description: None,
                    },
                    SelectOption {
                        value: "viewer".to_string(),
                        label: "Viewer".to_string(),
                        description: Some("Read only".to_string()),
                    },
                ],
            },
        ],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("is-floating"));
    assert!(page.content.contains(r#"placeholder=\"Full name\""#));
    assert!(page.content.contains("data-dowe-select"));
    assert!(page.content.contains(r#"<svg class=\"select-arrow\""#));
    assert!(
        page.content
            .contains("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4")
    );
    assert!(page.content.contains(r#"data-dowe-option-value=\"admin\""#));
    assert!(page.content.contains("select-option-description"));
    assert!(page.content.contains("Read only"));
}

#[test]
fn renders_svg_markup_and_color_classes() {
    let root = Path::new("/project");
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &svg_tree(),
    );

    assert!(page.content.contains(r#"<svg"#));
    assert!(page.content.contains(r#"class=\"svg"#));
    assert!(page.content.contains("color-tertiary"));
    assert!(page.content.contains("w-8"));
    assert!(page.content.contains("h-8"));
    assert!(
        page.content
            .contains(r#"xmlns=\"http://www.w3.org/2000/svg\""#)
    );
    assert!(page.content.contains(r#"viewBox=\"0 0 24 24\""#));
    assert!(page.content.contains(r#"aria-hidden=\"true\""#));
    assert!(
        page.content
            .contains(r#"<path d=\"M0 0h24v24H0z\" fill=\"none\"></path>"#)
    );
    assert!(
        page.content
            .contains(r#"<path d=\"M22 12c0-5.523-4.477-10-10-10\" fill=\"currentColor\"></path>"#)
    );
    assert!(page.css_content.contains(".svg"));
    assert!(
        page.css_content
            .contains(".color-tertiary{color:var(--dowe-tertiary);}")
    );
    assert!(page.css_content.contains(".w-8{width:2rem;}"));
    assert!(page.css_content.contains(".h-8{height:2rem;}"));
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

fn strip_web_for_test(path: &Path) -> String {
    path.strip_prefix("web")
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn layout_tree() -> ViewNode {
    ViewNode::Box {
        props: Default::default(),
        children: vec![text("Layout"), ViewNode::Children],
    }
}

fn page_tree() -> ViewNode {
    ViewNode::Box {
        props: Default::default(),
        children: vec![text("Login")],
    }
}

fn show_tree() -> ViewNode {
    ViewNode::Scope {
        signals: vec![ViewSignal {
            id: "ready01".to_string(),
            name: "isReady".to_string(),
            initial: ViewSignalValue::Bool(false),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![ViewNode::Box {
            props: StyleProps {
                element: ElementProps {
                    show: Some(VisibilityCondition::Static(responsive_bool(&[
                        (Breakpoint::Xs, false),
                        (Breakpoint::Md, true),
                    ]))),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![ViewNode::Text {
                props: TextProps {
                    style: StyleProps {
                        element: ElementProps {
                            show: Some(VisibilityCondition::Signal("isReady".to_string())),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                value: "Ready".to_string(),
            }],
        }],
    }
}

fn reactive_tree(signal_id: &str, action_id: &str, children: bool) -> ViewNode {
    let mut input = VariantProps::default();
    input.element.bind = Some("alert.message".to_string());
    let mut button = VariantProps::default();
    button.element.on_click = Some("close".to_string());
    let mut nodes = vec![
        ViewNode::Input { props: input },
        ViewNode::Button {
            props: button,
            children: vec![text("Close")],
        },
    ];
    if children {
        nodes.push(ViewNode::Children);
    }
    ViewNode::Scope {
        signals: vec![ViewSignal {
            id: signal_id.to_string(),
            name: "alert".to_string(),
            initial: ViewSignalValue::Object(vec![(
                "message".to_string(),
                ViewSignalValue::String(String::new()),
            )]),
            schema: None,
        }],
        actions: vec![ViewAction {
            id: action_id.to_string(),
            name: "close".to_string(),
            kind: ViewActionKind::Reset(ViewResetAction {
                target: "alert".to_string(),
            }),
        }],
        children: nodes,
    }
}

fn text(value: &str) -> ViewNode {
    ViewNode::Text {
        props: Default::default(),
        value: value.to_string(),
    }
}

fn show_design_css() -> String {
    let web = super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    };
    web_artifacts(&web, &FontConfig::default(), &DesignConfig::default())
        .into_iter()
        .find(|artifact| artifact.relative_path == Path::new("web/design.css"))
        .expect("design css")
        .content
}

fn responsive_bool(entries: &[(Breakpoint, bool)]) -> ResponsiveValue<bool> {
    ResponsiveValue::ordered(
        entries
            .iter()
            .map(|(breakpoint, value)| ResponsiveEntry {
                breakpoint: *breakpoint,
                value: *value,
            })
            .collect(),
    )
}

fn media_display_form_tree() -> ViewNode {
    ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::Audio {
                props: AudioProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Primary),
                        ..Default::default()
                    },
                    src: "https://cdn.pixabay.com/audio/2022/04/25/audio_5d61b5204f.mp3"
                        .to_string(),
                    subtitle: Some("Preview".to_string()),
                    avatar_src: Some("https://example.com/avatar.png".to_string()),
                },
            },
            ViewNode::Image {
                props: ImageProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Secondary),
                        ..Default::default()
                    },
                    src: "https://example.com/photo.jpg".to_string(),
                    alt: "Photo".to_string(),
                    aspect: ImageAspect::Square,
                    object_fit: ImageObjectFit::Contain,
                    loading: ImageLoading::Eager,
                    hide_controls: false,
                },
            },
            ViewNode::Accordion {
                props: AccordionProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    multiple: true,
                },
                items: vec![AccordionItem {
                    id: "intro".to_string(),
                    label: "Intro".to_string(),
                    disabled: false,
                    default_open: true,
                    children: vec![text("Intro body")],
                }],
            },
            ViewNode::Carousel {
                props: CarouselProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Info),
                        ..Default::default()
                    },
                    autoplay: true,
                    autoplay_interval: 4000,
                    disable_loop: false,
                    hide_controls: false,
                    hide_indicators: false,
                    show_navigation: true,
                    show_counter: true,
                    orientation: CarouselOrientation::Horizontal,
                    size: ButtonSize::Sm,
                    indicator_type: CarouselIndicatorType::Dot,
                    title: Some("Samples".to_string()),
                    slide_width: Some(240),
                    slide_height: None,
                    slides_per_view: 1,
                    gap: 12,
                },
                slides: vec![
                    CarouselSlide {
                        id: "one".to_string(),
                        children: vec![text("First")],
                    },
                    CarouselSlide {
                        id: "two".to_string(),
                        children: vec![text("Second")],
                    },
                ],
            },
            ViewNode::Checkbox {
                props: CheckboxProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Success),
                        label: Some("I accept".to_string()),
                        element: ElementProps {
                            bind: Some("accepted".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    checked: true,
                    disabled: false,
                    name: Some("accepted".to_string()),
                },
            },
            ViewNode::Color {
                props: ColorProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Primary),
                        label: Some("Theme".to_string()),
                        element: ElementProps {
                            bind: Some("themeColor".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    value: "#3366ff".to_string(),
                    size: ButtonSize::Md,
                    name: Some("themeColor".to_string()),
                    help_text: Some("Pick a color".to_string()),
                    error_text: None,
                    show_hex: true,
                    show_rgb: true,
                    show_cmyk: false,
                    show_oklch: false,
                },
            },
            ViewNode::Date {
                props: DateProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Warning),
                        label: Some("Ship date".to_string()),
                        element: ElementProps {
                            bind: Some("shipDate".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    value: Some("2026-06-05".to_string()),
                    size: ButtonSize::Md,
                    name: Some("shipDate".to_string()),
                    help_text: None,
                    error_text: None,
                    min: Some("2026-01-01".to_string()),
                    max: Some("2026-12-31".to_string()),
                },
            },
            ViewNode::DateRange {
                props: DateRangeProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Danger),
                        label: Some("Range".to_string()),
                        ..Default::default()
                    },
                    start: Some("startDate".to_string()),
                    end: Some("endDate".to_string()),
                    start_value: Some("2026-06-01".to_string()),
                    end_value: Some("2026-06-08".to_string()),
                    size: ButtonSize::Md,
                    name: Some("range".to_string()),
                    help_text: None,
                    error_text: None,
                    min: Some("2026-01-01".to_string()),
                    max: Some("2026-12-31".to_string()),
                },
            },
            ViewNode::RadioGroup {
                props: RadioGroupProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Muted),
                        label: Some("Plan".to_string()),
                        element: ElementProps {
                            bind: Some("choice".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    size: ButtonSize::Lg,
                    name: Some("plan".to_string()),
                    info: Some("Choose one".to_string()),
                    error: None,
                },
                options: vec![
                    RadioOption {
                        value: "basic".to_string(),
                        label: "Basic".to_string(),
                        disabled: false,
                    },
                    RadioOption {
                        value: "pro".to_string(),
                        label: "Pro".to_string(),
                        disabled: true,
                    },
                ],
            },
            ViewNode::Toggle {
                props: ToggleProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Secondary),
                        label: Some("Enabled".to_string()),
                        element: ElementProps {
                            bind: Some("accepted".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    checked: true,
                    disabled: false,
                    name: Some("enabled".to_string()),
                    label_left: Some("Off".to_string()),
                    label_right: Some("On".to_string()),
                },
            },
        ],
    }
}

fn bar_props(floating: bool) -> BarProps {
    BarProps {
        style: VariantProps {
            variant: Some(ComponentVariant::Soft),
            color: Some(ColorFamily::Surface),
            ..Default::default()
        },
        bordered: true,
        blurred: true,
        boxed: true,
        floating,
    }
}

fn side_nav_item(label: &str, path: &str) -> SideNavItemProps {
    SideNavItemProps {
        label: label.to_string(),
        description: None,
        status: None,
        icon: None,
        on_click: None,
        navigation: Some(NavigationAction::Internal {
            path: path.to_string(),
            fragment: None,
            operation: NavigationOperation::Push,
        }),
    }
}

fn navigation_shell_tree() -> ViewNode {
    ViewNode::Scaffold {
        props: ScaffoldProps {
            style: StyleProps::default(),
            boxed: true,
        },
        app_bar: vec![ViewNode::NavMenu {
            props: NavMenuProps {
                style: VariantProps {
                    variant: Some(ComponentVariant::Ghost),
                    color: Some(ColorFamily::Muted),
                    ..Default::default()
                },
                size: SideNavSize::Md,
            },
            items: vec![
                NavMenuItem::Item(NavMenuItemProps {
                    label: "Home".to_string(),
                    description: None,
                    icon: None,
                    on_click: None,
                    navigation: Some(NavigationAction::Internal {
                        path: "/".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                }),
                NavMenuItem::Submenu {
                    props: NavMenuItemProps {
                        label: "Docs".to_string(),
                        description: None,
                        icon: None,
                        on_click: None,
                        navigation: None,
                    },
                    items: vec![NavMenuItemProps {
                        label: "Guide".to_string(),
                        description: Some("Start here".to_string()),
                        icon: None,
                        on_click: None,
                        navigation: Some(NavigationAction::Internal {
                            path: "/docs".to_string(),
                            fragment: None,
                            operation: NavigationOperation::Push,
                        }),
                    }],
                },
                NavMenuItem::Megamenu {
                    props: NavMenuItemProps {
                        label: "Resources".to_string(),
                        description: None,
                        icon: None,
                        on_click: None,
                        navigation: None,
                    },
                    content: vec![text("Resource hub")],
                },
            ],
        }],
        start: vec![ViewNode::Sidebar {
            props: SideNavProps {
                style: VariantProps {
                    variant: Some(ComponentVariant::Soft),
                    color: Some(ColorFamily::Surface),
                    ..Default::default()
                },
                size: SideNavSize::Md,
                wide: true,
            },
            items: vec![SideNavItem::Item(SideNavItemProps {
                label: "Side Home".to_string(),
                description: None,
                status: None,
                icon: None,
                on_click: None,
                navigation: Some(NavigationAction::Internal {
                    path: "/".to_string(),
                    fragment: None,
                    operation: NavigationOperation::Push,
                }),
            })],
        }],
        main: vec![text("Main content")],
        end: Vec::new(),
        bottom_bar: vec![text("Bottom")],
    }
}

fn tabs_tree() -> ViewNode {
    ViewNode::Tabs {
        props: TabsProps {
            style: StyleProps::default(),
            variant: TabsVariant::Line,
            color: ColorFamily::Primary,
            position: TabsPosition::Start,
        },
        tabs: vec![
            TabItem {
                id: "overview".to_string(),
                label: "Overview".to_string(),
                children: vec![text("Overview content")],
            },
            TabItem {
                id: "details".to_string(),
                label: "Details".to_string(),
                children: vec![text("Details content")],
            },
        ],
    }
}

fn svg_tree() -> ViewNode {
    ViewNode::Svg {
        props: SvgProps {
            style: StyleProps {
                text: Some(ResponsiveValue::scalar(
                    dowe_components::ColorToken::Tertiary,
                )),
                sizing: dowe_components::SizingProps {
                    w: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        dowe_components::ScaleValue::from_half_steps(16),
                    ))),
                    h: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        dowe_components::ScaleValue::from_half_steps(16),
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            },
            view_box: SvgViewBox {
                min_x: "0".to_string(),
                min_y: "0".to_string(),
                width: "24".to_string(),
                height: "24".to_string(),
            },
        },
        paths: vec![
            SvgPath {
                data: "M0 0h24v24H0z".to_string(),
                fill: SvgPathFill::None,
            },
            SvgPath {
                data: "M22 12c0-5.523-4.477-10-10-10".to_string(),
                fill: SvgPathFill::CurrentColor,
            },
        ],
    }
}

fn code_tree() -> ViewNode {
    dowe_components::code_node(
        vec![
            ComponentProp {
                name: "language".to_string(),
                value: PropValue::String("dowe".to_string()),
            },
            ComponentProp {
                name: "scheme".to_string(),
                value: PropValue::String("surface".to_string()),
            },
        ],
        vec![
            "page docsPage".to_string(),
            "  Text".to_string(),
            "    Documentation".to_string(),
        ],
    )
    .expect("code")
}

fn video_tree() -> ViewNode {
    ViewNode::Video {
        props: VideoProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Solid),
                color: Some(ColorFamily::Surface),
                ..Default::default()
            },
            src: "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8".to_string(),
            poster: Some("/images/video.jpg".to_string()),
            autoplay: false,
            aspect: VideoAspect::Horizontal,
        },
    }
}

fn candlestick_tree() -> ViewNode {
    dowe_components::candlestick_node(vec![
        ComponentProp {
            name: "data".to_string(),
            value: PropValue::String("candles".to_string()),
        },
        ComponentProp {
            name: "stream".to_string(),
            value: PropValue::String("/api/candles".to_string()),
        },
        ComponentProp {
            name: "variant".to_string(),
            value: PropValue::String("soft".to_string()),
        },
        ComponentProp {
            name: "scheme".to_string(),
            value: PropValue::String("surface".to_string()),
        },
        ComponentProp {
            name: "emptyLabel".to_string(),
            value: PropValue::String("Market closed".to_string()),
        },
    ])
    .expect("candlestick")
}

fn table_tree() -> ViewNode {
    dowe_components::table_node(
        vec![
            ComponentProp {
                name: "data".to_string(),
                value: PropValue::String("users".to_string()),
            },
            ComponentProp {
                name: "variant".to_string(),
                value: PropValue::String("soft".to_string()),
            },
            ComponentProp {
                name: "scheme".to_string(),
                value: PropValue::String("surface".to_string()),
            },
            ComponentProp {
                name: "size".to_string(),
                value: PropValue::String("lg".to_string()),
            },
            ComponentProp {
                name: "striped".to_string(),
                value: PropValue::Boolean(true),
            },
            ComponentProp {
                name: "bordered".to_string(),
                value: PropValue::Boolean(true),
            },
            ComponentProp {
                name: "emptyTitle".to_string(),
                value: PropValue::String("No users".to_string()),
            },
        ],
        vec![
            dowe_components::table_column_component(vec![
                ComponentProp {
                    name: "field".to_string(),
                    value: PropValue::String("name".to_string()),
                },
                ComponentProp {
                    name: "label".to_string(),
                    value: PropValue::String("Name".to_string()),
                },
            ])
            .expect("name column"),
            dowe_components::table_column_component(vec![
                ComponentProp {
                    name: "field".to_string(),
                    value: PropValue::String("status".to_string()),
                },
                ComponentProp {
                    name: "label".to_string(),
                    value: PropValue::String("Status".to_string()),
                },
                ComponentProp {
                    name: "align".to_string(),
                    value: PropValue::String("end".to_string()),
                },
                ComponentProp {
                    name: "width".to_string(),
                    value: PropValue::String("8rem".to_string()),
                },
            ])
            .expect("status column"),
        ],
    )
    .expect("table")
}

fn translations() -> TranslationCatalog {
    TranslationCatalog {
        default_locale: Some("en".to_string()),
        locales: vec![
            TranslationLocale {
                locale: "en".to_string(),
                source_path: PathBuf::from("src/i18n/en.dowe"),
                values: vec![TranslationValue {
                    key: "home.hero.title".to_string(),
                    value: "Dowe builds systems.".to_string(),
                }],
            },
            TranslationLocale {
                locale: "es".to_string(),
                source_path: PathBuf::from("src/i18n/es.dowe"),
                values: vec![TranslationValue {
                    key: "home.hero.title".to_string(),
                    value: "Dowe construye sistemas.".to_string(),
                }],
            },
        ],
    }
}

fn display_overlay_tree() -> ViewNode {
    ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::Avatar {
                props: AvatarProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Success),
                        ..Default::default()
                    },
                    src: None,
                    name: Some("Ada".to_string()),
                    alt: "Ada Lovelace".to_string(),
                    size: ButtonSize::Lg,
                    status: Some(AvatarStatus::Online),
                    bordered: true,
                },
                icon: None,
            },
            ViewNode::Badge {
                props: BadgeProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Danger),
                        ..Default::default()
                    },
                    text: "3".to_string(),
                    position: OverlayCornerPosition::BottomRight,
                },
                children: vec![text("Inbox")],
            },
            ViewNode::Chip {
                props: ChipProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Info),
                        size: Some(ButtonSize::Sm),
                        ..Default::default()
                    },
                    on_close: Some("close".to_string()),
                },
                value: "Filter".to_string(),
                start: None,
                end: None,
            },
            ViewNode::Skeleton {
                props: SkeletonProps {
                    style: StyleProps::default(),
                    variant: SkeletonVariant::Rounded,
                    animation: SkeletonAnimation::Pulse,
                },
            },
            ViewNode::Modal {
                props: ModalProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    open: "modal01".to_string(),
                    on_close: Some("close".to_string()),
                    disable_overlay_close: false,
                    hide_close_button: false,
                },
                header: vec![text("Settings")],
                body: vec![text("Body")],
                footer: vec![text("Footer")],
            },
            ViewNode::AlertDialog {
                props: AlertDialogProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Danger),
                        ..Default::default()
                    },
                    open: "modal01".to_string(),
                    title: "Delete?".to_string(),
                    description: "Cannot undo.".to_string(),
                    confirm_text: "Delete".to_string(),
                    cancel_text: "Cancel".to_string(),
                    on_confirm: Some("confirm".to_string()),
                    on_cancel: Some("close".to_string()),
                    loading: false,
                },
            },
            ViewNode::Tooltip {
                props: TooltipProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Muted),
                        ..Default::default()
                    },
                    label: "More actions".to_string(),
                    position: OverlayPosition::End,
                },
                children: vec![text("Hover")],
            },
            ViewNode::Toast {
                props: ToastProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Success),
                        ..Default::default()
                    },
                    source: None,
                    kind: ToastKind::Success,
                    title: Some("Saved".to_string()),
                    description: "Profile updated".to_string(),
                    position: OverlayCornerPosition::TopRight,
                    show_icon: true,
                },
            },
            ViewNode::Dropdown {
                props: DropdownProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                },
                trigger: vec![text("Menu")],
                header: Vec::new(),
                entries: vec![
                    OverlayEntry::Item(OverlayItemProps {
                        label: "Profile".to_string(),
                        description: None,
                        icon: None,
                        on_click: Some("profile".to_string()),
                        navigation: None,
                        disabled: false,
                    }),
                    OverlayEntry::Divider,
                    OverlayEntry::Item(OverlayItemProps {
                        label: "Docs".to_string(),
                        description: Some("Open docs".to_string()),
                        icon: None,
                        on_click: None,
                        navigation: Some(NavigationAction::Internal {
                            path: "/docs".to_string(),
                            fragment: None,
                            operation: NavigationOperation::Push,
                        }),
                        disabled: false,
                    }),
                ],
                footer: Vec::new(),
            },
            ViewNode::Command {
                props: CommandProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Muted),
                        ..Default::default()
                    },
                    open: Some("modal01".to_string()),
                    placeholder: "Search".to_string(),
                    empty_text: "No results".to_string(),
                    close_text: "to close".to_string(),
                    navigate_text: "Navigate".to_string(),
                    select_text: "Select".to_string(),
                    toggle_text: "Toggle".to_string(),
                    shortcut: "p".to_string(),
                    disable_global_shortcut: false,
                    show_footer: true,
                },
                entries: vec![
                    CommandEntry::Item(OverlayItemProps {
                        label: "Home".to_string(),
                        description: None,
                        icon: None,
                        on_click: None,
                        navigation: Some(NavigationAction::Internal {
                            path: "/".to_string(),
                            fragment: None,
                            operation: NavigationOperation::Push,
                        }),
                        disabled: false,
                    }),
                    CommandEntry::Group {
                        label: "Admin".to_string(),
                        icon: None,
                        items: vec![OverlayItemProps {
                            label: "Users".to_string(),
                            description: None,
                            icon: None,
                            on_click: Some("users".to_string()),
                            navigation: None,
                            disabled: false,
                        }],
                    },
                ],
            },
        ],
    }
}

fn divider_tree() -> ViewNode {
    ViewNode::Divider {
        props: DividerProps {
            style: StyleProps::default(),
            orientation: DividerOrientation::Vertical,
            color: ColorFamily::Primary,
        },
    }
}
