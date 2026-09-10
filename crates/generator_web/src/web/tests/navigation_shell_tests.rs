#[test]
fn emits_smooth_page_fade_transition_css() {
    let css = super::design_css();

    assert!(css.contains(
        "html.page-transitioning::view-transition-group(root){animation:none;}"
    ));
    assert!(css.contains(
        "html.page-transitioning::view-transition-old(root),html.page-transitioning::view-transition-new(root){animation:none;}"
    ));
    assert!(css.contains(
        "html.page-transitioning::view-transition-group(dowe-page){animation:none;}"
    ));
    assert!(css.contains(
        "html.page-transitioning::view-transition-old(dowe-page){animation:none;}"
    ));
    assert!(!css.contains("dowe-page-fade-out"));
    assert!(!css.contains("dowe-page-slide-out"));
    assert!(!css.contains("dowe-page-scale-out"));
    assert!(css.contains(
        "html.page-transitioning[data-dowe-page-transition='fade']::view-transition-new(dowe-page){animation:dowe-page-fade-in 280ms cubic-bezier(.22,.61,.36,1) both;}"
    ));
    assert!(css.contains(
        ".dowe-page-enter{opacity:0;transition:opacity 280ms cubic-bezier(.22,.61,.36,1);}.dowe-page-enter-active{opacity:1;}"
    ));
}

#[test]
fn renders_layout_bars_markup_and_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::AppBar {
                props: BarProps {
                    position: BarPosition::Fixed,
                    dock_on_scroll: true,
                    ..bar_props(true)
                },
                top: vec![text("Notice")],
                start: vec![text("Menu")],
                center: vec![text("Brand")],
                end: vec![text("Account")],
                bottom: vec![text("Status")],
                mobile_menu: None,
            },
            ViewNode::Footer {
                props: bar_props(false),
                top: vec![text("Directory")],
                start: vec![text("Footer")],
                center: Vec::new(),
                end: vec![text("Legal")],
                bottom: vec![text("Copyright")],
            },
            ViewNode::BottomBar {
                props: bar_props(false),
                tabs: vec![BottomBarTab {
                    label: "Create".to_string(),
                    i18n: None,
                    featured: true,
                    icon: solar_control_icon("add-circle").expect("icon"),
                    navigation: NavigationAction::Internal {
                        path: "/create".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    },
                }],
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
        r#"<header class="appbar is-solid is-surface position-fixed is-bordered is-blurred is-floating is-dock-on-scroll"><div class="appbar-top">"#
    ));
    assert!(
        html.contains(r#"</div><div class="appbar-content is-boxed"><div class="appbar-start">"#)
    );
    assert!(html.contains(r#"</div></div><div class="appbar-bottom">"#));
    assert!(html.contains(r#"<div class="appbar-start">"#));
    assert!(html.contains(r#"<footer class="footer is-solid is-surface is-bordered is-blurred">"#));
    assert!(html.contains(
        r#"<footer class="footer is-solid is-surface is-bordered is-blurred"><div class="footer-inner is-boxed"><div class="footer-top">"#
    ));
    assert!(html.contains(r#"</div><div class="footer-content"><div class="footer-start">"#));
    assert!(html.contains(r#"</div></div><div class="footer-bottom">"#));
    assert!(html.contains(r#"<nav class="bottombar is-solid is-surface is-bordered is-blurred">"#));
    assert!(html.contains(r#"<div class="bottombar-tabs is-boxed">"#));
    assert!(html.contains(r#"class="bottombar-tab is-featured""#));
    assert!(html.contains(r#"data-dowe-bottombar-href="/create""#));
    assert!(html.contains("bottombar-tab-icon"));
    assert!(css.contains(".appbar,.footer,.bottombar{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));width:100%;"));
    assert!(css.contains(".appbar.position-sticky{position:sticky;top:0;}"));
    assert!(css.contains(".appbar.position-fixed{position:fixed;top:0;left:0;right:0;}"));
    assert!(css.contains(".appbar.is-dock-on-scroll:not(.is-floating){margin-top:0;border-bottom:1px solid var(--dowe-muted);border-radius:0;overflow:hidden;}"));
    assert!(
        css.contains(".appbar-top>*,.appbar-bottom>*,.footer-top>*,.footer-bottom>*{width:100%;}")
    );
    assert!(css.contains(".appbar{padding-top:0;}"));
    assert!(css.contains(
        ".appbar-content.is-boxed,.footer-inner.is-boxed,.bottombar-content.is-boxed{max-width:96rem;margin:0 auto;}"
    ));
    assert!(css.contains(".bottombar-tabs.is-boxed{max-width:96rem;margin:0 auto;}"));
    assert!(page.css_content.contains(".appbar.is-solid.is-surface"));
    assert!(page.css_content.contains(".bottombar.is-solid.is-surface"));

    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(dowe_components::RenderTarget::Web, Vec::new()),
    });
    assert!(router.contains("function hydrateScrollDockingAppBars(root)"));
    assert!(router.contains("const floating=(window.scrollY||0)<=100"));
    assert!(router.contains("bar.classList.toggle(\"is-floating\",floating)"));
    assert!(router.contains("bar.addEventListener(\"transitionend\""));
    assert!(router.contains("hydrateScrollDockingAppBars(root)"));
}

#[test]
fn renders_side_nav_markup_active_runtime_and_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::SideNav {
        props: SideNavProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Ghost),
                color: Some(ColorFamily::Muted),
                reactive: ReactiveVariantProps {
                    variant: Some("variantChoice".to_string()),
                    scheme: Some("schemeChoice".to_string()),
                    size: Some("sizeChoice".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            size: SideNavSize::Md,
            wide: true,
            reactive_wide: Some("wideEnabled".to_string()),
        },
        items: vec![
            SideNavItem::Header(SideNavItemProps {
                label: "Workspace".to_string(),
                i18n: None,
                description: Some("Account navigation".to_string()),
                description_i18n: None,
                status: None,
                status_i18n: None,
                icon: None,
                on_click: None,
                navigation: None,
            }),
            SideNavItem::Item(side_nav_item("Home", "/")),
            SideNavItem::Divider,
            SideNavItem::Submenu {
                props: SideNavItemProps {
                    label: "Content".to_string(),
                    i18n: None,
                    description: None,
                    description_i18n: None,
                    status: Some("2".to_string()),
                    status_i18n: None,
                    icon: None,
                    on_click: None,
                    navigation: None,
                },
                open: true,
                bordered: false,
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

    assert!(html.contains(r#"<nav class="sidenav is-ghost is-muted sidenav-md is-wide""#));
    assert!(html.contains(r#"data-dowe-nav-memory-key="structure:"#));
    assert!(html.contains(r#"data-dowe-sidenav-variant="variantChoice""#));
    assert!(html.contains(r#"data-dowe-sidenav-scheme="schemeChoice""#));
    assert!(html.contains(r#"data-dowe-sidenav-size="sizeChoice""#));
    assert!(html.contains(r#"data-dowe-sidenav-wide="wideEnabled""#));
    assert!(html.contains(r#"data-dowe-sidenav-href="/blogs""#));
    assert!(html.contains(
        r#"<details class="sidenav-submenu is-open is-unbordered" data-dowe-sidenav-submenu data-dowe-nav-submenu-key="3" open>"#
    ));
    assert!(html.contains(r#"<span class="sidenav-chevron" aria-hidden="true"><svg"#));
    assert!(html.contains(r#"<span class="sidenav-status">2</span>"#));
    assert!(html.contains(r#"d="m19.704 12l-8.491-8.727a.75.75 0 1 1 1.075-1.046l9 9.25a.75.75 0 0 1 0 1.046l-9 9.25a.75.75 0 1 1-1.075-1.046z""#));
    assert!(html.contains(r#"aria-expanded="true""#));
    assert!(html.contains(r#"<div class="sidenav-divider"></div>"#));
    assert!(html.contains(
        r#"<div class="sidenav-submenu-content"><div class="sidenav-submenu-content-inner">"#
    ));
    assert!(css.contains(".sidenav{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));flex-direction:column;"));
    assert!(css.contains(".sidenav.is-wide{width:100%;}"));
    assert!(css.contains(".sidenav-sm .sidenav-entry,.sidenav-sm .sidenav-header{gap:0.5rem;"));
    assert!(css.contains(".sidenav-md .sidenav-entry,.sidenav-md .sidenav-header{gap:0.625rem;"));
    assert!(css.contains(".sidenav-submenu-content{display:grid;grid-template-rows:0fr;"));
    assert!(css.contains(".sidenav-submenu-content-inner{display:flex;min-height:0;flex-direction:column;gap:0.125rem;overflow:hidden;}"));
    assert!(css.contains("transition:grid-template-rows 180ms ease,opacity 160ms ease;"));
    assert!(
        css.contains(".sidenav-submenu.is-unbordered>.sidenav-submenu-content{border-left:0;}")
    );
    assert!(css.contains(".sidenav-chevron svg{display:block;width:1em;height:1em;}"));
    assert!(css.contains(".sidenav-status{flex:0 0 auto;border-radius:999px;padding:0.125rem 0.5rem;background:var(--dowe-muted);color:var(--dowe-mutedText);"));
    assert!(css.contains(
        ".sidenav-submenu.is-open>.sidenav-submenu-content{grid-template-rows:1fr;opacity:1;"
    ));
    assert!(!css.contains("max-height:40rem"));
    assert!(
        page.css_content
            .contains(".sidenav.is-ghost.is-danger .sidenav-entry:hover{background-color:transparent;color:var(--dowe-danger);}")
    );
    assert!(page.css_content.contains(".sidenav.is-solid.is-primary .sidenav-entry.is-active{background-color:var(--dowe-primary);color:var(--dowe-primaryText);border-color:var(--dowe-primary);font-weight:600;}"));
    assert!(page.css_content.contains(".sidenav.is-outlined.is-primary .sidenav-entry.is-active{background-color:transparent;color:var(--dowe-primary);border-color:var(--dowe-primary);font-weight:600;}"));
    assert!(
        super::router_js(&super::WebOutput {
            chunks: Vec::new(),
            pages: Vec::new(),
            translation_chunks: Vec::new(),
            default_locale: None,
            router_js: String::new(),
            render_report: dowe_components::RenderReport::new(dowe_components::RenderTarget::Web, Vec::new()),
        })
        .contains("toggleNavTreeSubmenu(\"sidenav\"")
    );
    assert!(
        super::router_js(&super::WebOutput {
            chunks: Vec::new(),
            pages: Vec::new(),
            translation_chunks: Vec::new(),
            default_locale: None,
            router_js: String::new(),
            render_report: dowe_components::RenderReport::new(dowe_components::RenderTarget::Web, Vec::new()),
        })
        .contains(
            "event.stopPropagation();toggleNavTreeSubmenu(\"sidenav\",sideNavTrigger);},true)"
        )
    );
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(dowe_components::RenderTarget::Web, Vec::new()),
    });
    assert!(router.contains("const navTreeSubmenuMemory=new Map()"));
    assert!(router.contains("navTreeSubmenuMemory.set(memoryKey,open)"));
    assert!(router.contains("navTreeSubmenuMemory.has(memoryKey)"));
}

#[test]
fn renders_rail_nav_icons_tooltips_labels_and_active_state() {
    let root = Path::new("/project");
    let item = |label: &str, path: &str, icon: &str| {
        RailNavItem::Item(RailNavItemProps {
            label: label.to_string(),
            i18n: None,
            icon: solar_control_icon(icon).expect("icon"),
            on_click: None,
            navigation: Some(NavigationAction::Internal {
                path: path.to_string(),
                fragment: None,
                operation: NavigationOperation::Push,
            }),
        })
    };
    let icon_only = ViewNode::RailNav {
        props: RailNavProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Solid),
                color: Some(ColorFamily::Primary),
                ..Default::default()
            },
            size: SideNavSize::Md,
            show_labels: false,
        },
        items: vec![item("Home", "/", "home"), RailNavItem::Divider],
    };
    let labeled = ViewNode::RailNav {
        props: RailNavProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Ghost),
                color: Some(ColorFamily::Muted),
                ..Default::default()
            },
            size: SideNavSize::Sm,
            show_labels: true,
        },
        items: vec![item("Settings", "/settings", "settings")],
    };
    let tree = ViewNode::Flex {
        props: Default::default(),
        children: vec![icon_only, labeled],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &tree,
    );
    let html = render_page_body(&ViewNode::Children, &tree);
    let css = super::design_css();
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(dowe_components::RenderTarget::Web, Vec::new()),
    });

    assert!(html.contains(r#"<nav class="railnav is-solid is-primary railnav-md""#));
    assert!(html.contains(r#"class="tooltip railnav-tooltip" data-dowe-tooltip"#));
    assert!(
        html.contains(r#"class="tooltip-popover is-solid is-muted position-end" role="tooltip""#)
    );
    assert!(html.contains(r#"aria-label="Home""#));
    assert!(html.contains(r#"data-dowe-railnav-href="/""#));
    assert!(html.contains(r#"<div class="railnav-divider"></div>"#));
    assert!(html.contains(r#"railnav-sm has-labels"#));
    assert!(html.contains(r#"<span class="railnav-label">Settings</span>"#));
    assert_eq!(html.matches("data-dowe-tooltip").count(), 1);
    assert!(css.contains(".railnav{--dowe-component-display:flex;"));
    assert!(css.contains(".railnav-md{width:4rem;}"));
    assert!(
        page.css_content
            .contains(".railnav.is-solid.is-primary .railnav-item.is-active")
    );
    assert!(router.contains("[data-dowe-railnav-href]"));
    assert!(router.contains("document.addEventListener(\"focusin\""));
}

