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
        render_report: dowe_components::RenderReport::new(dowe_components::RenderTarget::Web, Vec::new()),
    });

    assert!(html.contains(r#"<div class="scaffold is-boxed">"#));
    assert!(html.contains(r#"<div class="scaffold-body">"#));
    assert!(html.contains("--dowe-scaffold-top-inset"));
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
    assert!(html.contains(r#"<aside class="sidebar w-96 is-solid is-surface""#));
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
                mobile_menu: None,
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
    props.style.element.bind = Some("selectedStep".to_string());
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/stepper.dowe"),
        "stepper",
        &page_tree,
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);
    let router = full_runtime_for_test();

    assert!(html.contains(r#"class="tabs is-top stepper" data-dowe-tabs"#));
    assert!(html.contains(r#"data-dowe-tabs-bind="selectedStep""#));
    assert!(html.contains(r#"class="tabs-list is-stepper is-primary" role="tablist""#));
    assert!(html.contains(r#"class="step-indicator" aria-hidden="true">1</span>"#));
    assert!(html.contains(r#"aria-current="step""#));
    assert!(
        page.css_content
            .contains(".tabs-list.is-stepper.is-primary")
    );
    assert!(page.css_content.contains("overflow-x:auto"));
    assert!(page.css_content.contains("scroll-snap-type:x proximity"));
    assert!(router.contains("writePath(activeView.state,bind,id)"));
}

