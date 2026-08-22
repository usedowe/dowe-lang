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
        r#"<header class="appbar is-soft is-surface position-fixed is-bordered is-blurred is-floating is-dock-on-scroll"><div class="appbar-top">"#
    ));
    assert!(
        html.contains(r#"</div><div class="appbar-content is-boxed"><div class="appbar-start">"#)
    );
    assert!(html.contains(r#"</div></div><div class="appbar-bottom">"#));
    assert!(html.contains(r#"<div class="appbar-start">"#));
    assert!(html.contains(r#"<footer class="footer is-soft is-surface is-bordered is-blurred">"#));
    assert!(html.contains(
        r#"<footer class="footer is-soft is-surface is-bordered is-blurred"><div class="footer-inner is-boxed"><div class="footer-top">"#
    ));
    assert!(html.contains(r#"</div><div class="footer-content"><div class="footer-start">"#));
    assert!(html.contains(r#"</div></div><div class="footer-bottom">"#));
    assert!(html.contains(r#"<nav class="bottombar is-soft is-surface is-bordered is-blurred">"#));
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
    assert!(page.css_content.contains(".appbar.is-soft.is-surface"));
    assert!(page.css_content.contains(".bottombar.is-soft.is-surface"));

    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
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
    assert!(page.css_content.contains(".sidenav.is-soft.is-primary .sidenav-entry:hover{background-color:color-mix(in srgb,var(--dowe-primary) 50%,transparent);color:var(--dowe-primary);}"));
    assert!(page.css_content.contains(".sidenav.is-soft.is-danger .sidenav-entry.is-active{background-color:var(--dowe-danger);color:var(--dowe-dangerText);border-color:transparent;font-weight:600;}"));
    assert!(page.css_content.contains(".sidenav.is-solid.is-primary .sidenav-entry.is-active{background-color:var(--dowe-primary);color:var(--dowe-primaryText);border-color:var(--dowe-primary);font-weight:600;}"));
    assert!(page.css_content.contains(".sidenav.is-outlined.is-primary .sidenav-entry.is-active{background-color:transparent;color:var(--dowe-primary);border-color:var(--dowe-primary);font-weight:600;}"));
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
    assert!(
        super::router_js(&super::WebOutput {
            chunks: Vec::new(),
            pages: Vec::new(),
            translation_chunks: Vec::new(),
            default_locale: None,
            router_js: String::new(),
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
                variant: Some(ComponentVariant::Soft),
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
    });

    assert!(html.contains(r#"<nav class="railnav is-soft is-primary railnav-md""#));
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
            .contains(".railnav.is-soft.is-primary .railnav-item.is-active")
    );
    assert!(router.contains("[data-dowe-railnav-href]"));
    assert!(router.contains("document.addEventListener(\"focusin\""));
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
    assert!(html.contains(r#"<aside class="scaffold-start">"#));
    assert!(html.contains(r#"<main class="scaffold-main">"#));
    assert!(html.contains(r#"<aside class="scaffold-end">"#));
    assert!(html.contains(r#"<div class="scaffold-overlays">"#));
    assert!(html.contains("Shell overlay"));
    assert!(html.contains(r#"<nav class="navmenu is-ghost is-muted navmenu-md""#));
    assert!(html.contains(r#"data-dowe-navmenu-trigger="1""#));
    assert!(html.contains(r#"<span class="navmenu-arrow" aria-hidden="true"><svg"#));
    assert!(!html.contains(r#"aria-hidden="true">⌄"#));
    assert!(html.contains(r#"data-dowe-navmenu-popover="2""#));
    assert!(html.contains(r#"data-dowe-navmenu-href="/docs""#));
    assert!(html.contains(r#"data-dowe-i18n="home.hero.title""#));
    assert!(html.contains("Resource hub"));
    assert!(html.contains(r#"<aside class="sidebar w-96 is-soft is-surface""#));
    assert!(html.contains(r#"<div class="sidebar-body">"#));
    assert!(html.contains(r#"<nav class="sidenav is-ghost is-muted sidenav-md is-wide""#));
    assert!(html.contains(r#"data-dowe-sidenav-href="/""#));
    assert!(css.contains(".navmenu{--dowe-component-display:flex"));
    assert!(css.contains(".navmenu-arrow svg{display:block;width:100%;height:100%;}"));
    assert!(css.contains(".sidebar{--dowe-component-display:flex"));
    assert!(css.contains(".scaffold-body{position:relative;display:flex;width:100%;min-width:0;min-height:0;flex:1 1 auto;align-items:stretch;padding-top:var(--dowe-scaffold-top-inset,0px);}"));
    assert!(css.contains(".scaffold-content{position:sticky;top:var(--dowe-scaffold-top-inset,0px);display:flex;height:100%;min-height:0;max-height:calc(100vh - var(--dowe-scaffold-top-inset,0px));overflow:hidden;}"));
    assert!(css.contains(".sidebar-body{display:flex;min-height:0;flex:1 1 auto;flex-direction:column;overflow:auto;overscroll-behavior:contain;}"));
    assert!(css.contains(".scaffold-content>.sidebar{height:100%;max-height:100%;}"));
    assert!(
        !css.contains(".scaffold-content{position:sticky;top:0;max-height:100vh;overflow:auto;")
    );
    assert!(!css.contains(".sidebar-entry"));
    assert!(css.contains(".scaffold{--dowe-component-display:flex"));
    assert!(css.contains(".scaffold.is-boxed>.scaffold-body{max-width:96rem;margin-inline:auto;}"));
    assert!(css.contains(".scaffold-overlays{position:relative;z-index:40;}"));
    assert!(page.css_content.contains(".w-96{width:24rem;}"));
    assert!(
        page.css_content
            .contains(".navmenu.is-ghost.is-muted .navmenu-item.is-active")
    );
    assert!(
        page.css_content
            .contains(".sidenav.is-ghost.is-muted .sidenav-entry.is-active")
    );
    assert!(router.contains("openNavMenu"));
    assert!(router.contains("if(open){closeNavMenus();return true;}closeNavMenus(root);"));
    assert!(router.contains("if(target.closest(\"[data-dowe-navmenu-popover]\"))closeNavMenus();"));
    assert!(router.contains("hydrateNavTreeSubmenus(root,\"sidenav\")"));
    assert!(router.contains("function hydrateScaffoldInsets(root)"));
    assert!(router.contains("appBar.getBoundingClientRect().bottom"));
    assert!(router.contains("new ResizeObserver"));
    assert!(router.contains("hydrateScaffoldInsets(view.root)"));
    assert!(!router.contains("data-dowe-sidebar-href"));
    assert!(router.contains("data-dowe-navmenu-href"));
}

#[test]
fn overlays_main_under_sticky_floating_appbar() {
    let tree = ViewNode::Scaffold {
        props: ScaffoldProps::default(),
        app_bar: vec![ViewNode::AppBar {
            props: BarProps {
                position: BarPosition::Sticky,
                floating: true,
                ..Default::default()
            },
            top: Vec::new(),
            start: vec![text("Navigation")],
            center: Vec::new(),
            end: Vec::new(),
            bottom: Vec::new(),
        }],
        start: Vec::new(),
        main: vec![text("Main content")],
        end: Vec::new(),
        bottom_bar: Vec::new(),
        overlays: Vec::new(),
    };
    let html = render_page_body(&ViewNode::Children, &tree);
    let css = super::design_css();

    assert!(
        html.contains(r#"<header class="appbar is-solid is-primary position-sticky is-floating">"#)
    );
    assert!(html.contains(r#"<main class="scaffold-main">"#));
    assert!(html.contains("Main content"));
    assert!(css.contains(
        ".scaffold:has(>.appbar.position-sticky.is-floating){--dowe-component-display:grid;grid-template-columns:minmax(0,1fr);grid-template-rows:minmax(min-content,1fr) auto;}"
    ));
    assert!(css.contains(
        ".scaffold:has(>.appbar.position-sticky.is-floating)>.appbar{grid-column:1;grid-row:1;align-self:start;}"
    ));
    assert!(css.contains(
        ".scaffold:has(>.appbar.position-sticky.is-floating)>.scaffold-body{grid-column:1;grid-row:1;}"
    ));
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
    let router = full_runtime_for_test();

    assert!(html.contains(r#"<div class="tabs is-start" data-dowe-tabs>"#));
    assert!(html.contains(r#"<div class="tabs-list is-line is-primary" role="tablist">"#));
    assert!(html.contains(r#"role="tab" id="tab-overview-button" aria-selected="true""#));
    assert!(html.contains(r#"tabindex="-1" data-dowe-tab="details""#));
    assert!(html.contains(r#"role="tabpanel" aria-labelledby="tab-details-button" data-dowe-tab-panel="details" hidden"#));
    assert!(html.contains("Overview content"));
    assert!(
        page.css_content
            .contains(".tabs-list.is-line.is-primary .tab.on-active")
    );
    assert!(
        page.css_content
            .contains(".tabs.is-start .tabs-list.is-line.is-primary .tab.on-active")
    );
    assert!(super::design_css().contains(
        ".tabs-list{display:flex;width:max-content;max-width:100%;flex:0 0 auto;align-self:flex-start;overflow-x:auto"
    ));
    assert!(router.contains("function setActiveTab(root,id)"));
    assert!(router.contains("[data-dowe-tab]"));
}

#[test]
fn renders_responsive_stepper_markup_and_css() {
    let root = Path::new("/project");
    let mut page_tree = tabs_tree();
    let ViewNode::Tabs { props, .. } = &mut page_tree else {
        panic!("stepper");
    };
    props.variant = TabsVariant::Stepper;
    props.position = TabsPosition::Top;
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/stepper.dowe"),
        "stepper",
        &page_tree,
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);

    assert!(html.contains(r#"class="tabs is-top stepper" data-dowe-tabs"#));
    assert!(html.contains(r#"class="tabs-list is-stepper is-primary" role="tablist""#));
    assert!(html.contains(r#"class="step-indicator" aria-hidden="true">1</span>"#));
    assert!(html.contains(r#"aria-current="step""#));
    assert!(
        page.css_content
            .contains(".tabs-list.is-stepper.is-primary")
    );
    assert!(page.css_content.contains("overflow-x:auto"));
    assert!(page.css_content.contains("scroll-snap-type:x proximity"));
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
        header: vec![text("Menu")],
        body: vec![text("Navigation")],
        footer: vec![text("Footer")],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);
    let css = super::design_css();
    let router = full_runtime_for_test();

    assert!(html.contains(r#"class="drawer-panel" data-dowe-drawer data-dowe-drawer-open="drawerOpen" data-dowe-drawer-disable-overlay-close="true" hidden"#));
    assert!(
        html.contains(
            r#"class="drawer is-soft is-surface is-end" role="dialog" aria-modal="true""#
        )
    );
    assert!(html.contains(r#"data-dowe-drawer-close"#));
    assert!(html.contains(r#"class="drawer-close is-end""#));
    let close_button_index = html.find("data-dowe-drawer-close").expect("close button");
    let dialog_end_index = html
        .find("aria-modal=\"true\"")
        .map(|index| html[index..].find(">").map(|offset| index + offset + 1).unwrap())
        .expect("dialog");
    assert!(
        close_button_index > dialog_end_index,
        "close button must render outside the dialog panel"
    );
    assert!(html.contains(r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24" aria-hidden="true" focusable="false">"#));
    assert!(html.contains(r#"d="m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073l.084.073L12 10.939l6.47-6.47a.75.75 0 1 1 1.06 1.061L13.061 12l6.47 6.47a.75.75 0 0 1 .072.976l-.073.084a.75.75 0 0 1-.976.073l-.084-.073L12 13.061l-6.47 6.47a.75.75 0 0 1-1.06-1.061L10.939 12l-6.47-6.47a.75.75 0 0 1-.072-.976l.073-.084z""#));
    assert!(html.contains(r#"class="drawer-header""#));
    assert!(html.contains(r#"class="drawer-body""#));
    assert!(html.contains(r#"class="drawer-footer""#));
    assert!(css.contains(
        ".drawer-panel{--dowe-component-display:flex;position:fixed;inset:0;z-index:50;"
    ));
    assert!(css.contains(".drawer{position:absolute;display:flex;max-width:100vw;max-height:100vh;min-height:0;flex-direction:column;overflow:hidden;"));
    assert!(css.contains(".drawer-body{display:flex;min-height:0;flex:1 1 auto;flex-direction:column;overflow:auto;overscroll-behavior:contain;}"));
    assert!(css.contains(".drawer.is-end{inset-block:0;inset-inline-end:0;width:min(20rem,100vw);border-start-end-radius:0;border-end-end-radius:0;transform:translateX(100%);"));
    assert!(css.contains(".drawer.is-start{inset-block:0;inset-inline-start:0;width:min(20rem,100vw);border-start-start-radius:0;border-end-start-radius:0;"));
    assert!(css.contains(".drawer.is-top{inset-inline:0;top:0;max-height:min(20rem,100vh);border-start-start-radius:0;border-start-end-radius:0;"));
    assert!(css.contains(".drawer.is-bottom{inset-inline:0;bottom:0;max-height:min(20rem,100vh);border-end-start-radius:0;border-end-end-radius:0;"));
    assert!(css.contains(".drawer-close svg{display:block;width:1em;height:1em;}"));
    assert!(css.contains(
        ".drawer-panel>.drawer-close.is-start{top:.5rem;right:.5rem;bottom:auto;left:auto;}"
    ));
    assert!(css.contains(
        ".drawer-panel>.drawer-close.is-end{top:.5rem;left:.5rem;bottom:auto;right:auto;}"
    ));
    assert!(css.contains(
        ".drawer-panel>.drawer-close.is-top{top:auto;right:.5rem;bottom:.5rem;left:auto;}"
    ));
    assert!(css.contains(
        ".drawer-panel>.drawer-close.is-bottom{top:.5rem;right:.5rem;bottom:auto;left:auto;}"
    ));
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
            header: Vec::new(),
            body: vec![text("Navigation")],
            footer: Vec::new(),
        },
    );
    assert!(rounded_html.contains("drawer rounded-lg is-soft is-surface is-start"));
}
