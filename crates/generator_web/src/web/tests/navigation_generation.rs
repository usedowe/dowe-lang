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
    assert!(html.contains(r#"<nav class="sidebar w-96 is-soft is-surface sidebar-md is-wide""#));
    assert!(html.contains(r#"data-dowe-sidebar-href="/""#));
    assert!(css.contains(".navmenu{--dowe-component-display:flex"));
    assert!(css.contains(".sidebar{--dowe-component-display:flex"));
    assert!(css.contains(".sidebar.is-wide .sidebar-entry"));
    assert!(!css.contains(".sidebar.is-wide{width:100%;}"));
    assert!(css.contains(".scaffold{--dowe-component-display:flex"));
    assert!(page.css_content.contains(".w-96{width:24rem;}"));
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
    assert!(
        page.css_content
            .contains(".tabs-list.is-line.is-primary .tab.on-active")
    );
    assert!(
        page.css_content
            .contains(".tabs.is-start .tabs-list.is-line.is-primary .tab.on-active")
    );
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
