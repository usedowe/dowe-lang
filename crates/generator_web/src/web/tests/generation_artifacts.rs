#[test]
fn emits_web_manifest_and_html_artifacts() {
    let root = Path::new("/project");
    let layout_tree = layout_tree();
    let page_tree = media_display_form_tree();
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
        runtime_chunks: Vec::new(),
        design_file_name: "design.css".to_string(),
        router_file_name: "router-test.js".to_string(),
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
        chunks: vec![Arc::new(layout), Arc::new(page)],
        pages: vec![Arc::new(view_page)],
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    };
    web.router_js = super::router_js(&web);
    let router_file_name = web.router_file_name();
    Arc::make_mut(&mut web.pages[0])
        .router_file_name
        .clone_from(&router_file_name);
    super::prepare_design_asset(&mut web, &FontConfig::default(), &DesignConfig::default());
    let router_file_name = web.router_file_name();
    let design_file_name = web.design_file_name().to_string();
    let artifacts = web_artifacts(&web, &FontConfig::default(), &DesignConfig::default());
    let style_chunks = web.pages[0]
        .css_chunks
        .iter()
        .filter(|path| path.starts_with("chunks/design/"))
        .cloned()
        .collect::<Vec<_>>();

    assert!(!style_chunks.is_empty());
    assert!(
        web.pages[0]
            .css_chunks
            .iter()
            .position(|path| path.starts_with("chunks/design/"))
            < web.pages[0]
                .css_chunks
                .iter()
                .position(|path| path.starts_with("chunks/layouts/"))
    );
    for path in &style_chunks {
        assert!(web.pages[0].html_document.contains(&format!(
            r#"data-dowe-css="{path}" rel="stylesheet" href="/{path}""#
        )));
        assert!(
            artifacts
                .iter()
                .any(|artifact| artifact.relative_path == Path::new("web").join(path))
        );
    }

    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.relative_path == Path::new("web/manifest.json"))
    );
    let service_worker = artifacts
        .iter()
        .find(|artifact| artifact.relative_path == Path::new("web/sw.js"))
        .expect("notification service worker");
    assert!(service_worker.content.contains("showNotification"));
    assert!(service_worker.content.contains("notificationclick"));
    assert!(
        service_worker
            .content
            .contains("payload.category !== \"process\"")
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
    assert!(
        index
            .content
            .contains(&format!(r#"href="{design_file_name}""#))
    );
    assert!(index.content.contains("data-dowe-router"));
    assert!(
        index
            .content
            .contains(&format!(r#"src="{router_file_name}""#))
    );
    assert!(index.content.contains(r#"src="chunks/layouts/"#));
    assert!(
        index
            .content
            .contains(r#"document.documentElement.classList.add("dowe-entrance-pending")"#)
    );
    let page = artifacts
        .iter()
        .find(|artifact| artifact.relative_path == Path::new("web/pages/login.html"))
        .expect("page");
    assert!(
        page.content
            .contains(&format!(r#"href="../{design_file_name}""#))
    );
    assert!(page.content.contains(r#"src="../chunks/layouts/"#));
    assert!(
        web.pages[0]
            .html_document
            .contains(&format!(r#"href="/{design_file_name}""#))
    );
    assert!(
        web.pages[0]
            .html_document
            .contains(&format!(r#"src="/{router_file_name}""#))
    );
    assert!(
        artifacts
            .iter()
            .any(|artifact| { artifact.relative_path == Path::new("web").join(&router_file_name) })
    );
    assert!(
        artifacts
            .iter()
            .any(|artifact| { artifact.relative_path == Path::new("web").join(&design_file_name) })
    );
    assert!(
        !artifacts
            .iter()
            .any(|artifact| artifact.relative_path == Path::new("web/router.js"))
    );
    assert!(super::manifest(&web).contains(r#""staticFile":"web/pages/login.html""#));
    assert!(super::manifest(&web).contains("chunks/design/"));
    let controls = super::controls_runtime_chunk().content;
    assert!(web.router_js.contains("staticMode"));
    assert!(web.router_js.contains("doweHref"));
    assert!(controls.contains("function positionSelect(control)"));
    assert!(controls.contains("function mountSelectPopover(control)"));
    assert!(controls.contains("document.body.appendChild(popover)"));
    assert!(controls.contains("popover.__doweControl"));
    assert!(controls.contains("const above=bottom<Math.min(height,224)&&top>bottom"));
    assert!(
        web.router_js
            .contains("scrollIntoView({behavior:reduce?\"auto\":\"smooth\",block:\"start\",})")
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
            .contains("document.head.insertBefore(replacement,current)")
    );
    assert!(
        web.router_js
            .contains("fetch(versionedAsset(\"manifest.json\",version)")
    );
    assert!(
        web.router_js
            .contains("prepareHydration(route,modules,preserveLayouts,true)")
    );
    assert!(web.router_js.contains("previous.state[signal.id]"));
    assert!(
        web.router_js
            .contains("Object.entries(activeView.globalIds)")
    );
    assert!(web.router_js.contains("key===name"));
    assert!(
        web.router_js
            .contains("const boundState=captureBoundState(app)")
    );
    assert!(web.router_js.contains("restoreBoundState(boundState)"));
    assert!(
        web.router_js
            .contains("function releaseEntranceAnimations()")
    );
    assert!(
        web.router_js
            .contains("function pageEntranceBoundary(root)")
    );
    assert!(
        web.router_js
            .contains("function clearPageEntranceAnimations(root)")
    );
    assert!(
        web.router_js
            .contains("function preparePageTransitionFallback(root)")
    );
    assert!(
        web.router_js
            .contains("function runCssPageTransition(update,afterReady=null)")
    );
    assert!(web.router_js.contains("page-transition-fallback"));
    assert!(
        web.router_js
            .contains("function waitForPageTransition(target)")
    );
    assert!(web.router_js.contains("function nextAnimationFrame()"));
    assert!(
        web.router_js
            .contains("function suppressPageEntranceAnimations(root)")
    );
    assert!(
        !web.router_js
            .contains("function prepareEntranceAnimations()")
    );
    assert!(web.router_js.contains(
        "requestAnimationFrame(()=>requestAnimationFrame(()=>document.documentElement.classList.remove(entranceMotionClass)))"
    ));
    assert!(
        web.router_js
            .contains("let navigationQueue=Promise.resolve()")
    );
    assert!(
        web.router_js
            .contains("const routeModulePromises=new Map()")
    );
    assert!(web.router_js.contains("const runtime=Promise.all"));
    assert!(web.router_js.contains("const modules=Promise.all"));
    assert!(
        web.router_js
            .contains("await Promise.all([runtime,modules])")
    );
    assert!(web.router_js.contains("function preloadRoute(route)"));
    assert!(web.router_js.contains(
        "function prepareHydration(route,modules,preserveLayouts=false,preserveState=false)"
    ));
    assert!(web.router_js.contains("function finishHydration(prepared)"));
    assert!(
        web.router_js
            .contains("function preloadNavigationTarget(anchor)")
    );
    assert!(web.router_js.contains("addEventListener(\"pointerover\""));
    assert!(web.router_js.contains("afterReady(target)"));
    assert!(web.router_js.contains("afterReady()"));
    assert!(web.router_js.contains("cachedRouteModules(route)"));
    assert!(
        web.router_js
            .contains("function navigateRoute(value,options={})")
    );
    assert!(
        web.router_js
            .contains("await Promise.all([loadRouteCss(route),cachedRouteModules(route),])")
    );
    assert!(web.router_js.contains("function preloadRouteCss(route)"));
    assert!(
        web.router_js
            .contains("link.rel=\"preload\";link.as=\"style\"")
    );
    assert!(
        web.router_js
            .contains("preloadRouteCss(route);return cachedRouteModules(route)")
    );
    assert!(
        !web.router_js
            .contains("const modules=await preloadRoute(route)")
    );
    assert!(web.router_js.contains("suppressPageEntranceAnimations"));
    assert!(
        web.router_js
            .contains("hydrate(route,modules,preserveLayouts)")
    );
    assert!(
        web.router_js
            .contains("runPageTransition(updateRoute,()=>finishHydration(prepared))")
    );
    assert!(
        web.router_js
            .contains("else{await updateRoute();hydrate(route,modules,preserveLayouts);}")
    );
    assert!(
        web.router_js
            .contains("compatibleSignalValue(previous.state[signal.id]")
    );
    assert!(
        web.router_js
            .contains("if(current&&!version)return waitForCss(current)")
    );
    assert!(!web.router_js.contains("document.head.appendChild(current)"));
    assert!(
        web.router_js
            .contains("document.head.insertBefore(link,next||null)")
    );
    assert!(web.router_js.contains("route.cssChunks"));
    assert!(web.router_js.contains("loadCss(route"));
    assert!(
        web.router_js
            .contains("if(!document.querySelector('script[src=\"/_dowe/dev/client.js\"]'))return")
    );
    assert!(
        web.router_js
            .contains("if(!route){await syncDevRoutes();route=routes[destination.path]")
    );
    assert_eq!(web.router_js.matches("await loadRouteCss(route").count(), 1);
    assert_eq!(web.router_js.matches("pruneCss(route)").count(), 3);
    assert!(web.router_js.contains("function pruneCss(route)"));
    assert!(
        web.router_js
            .contains("reject(new Error(\"Dowe CSS chunk failed: \"+link.href))")
    );
    assert!(web.router_js.contains(
        "if(options.writeHistory===false||options.replace)location.replace(destination.href)"
    ));
    assert!(web.router_js.contains("if(current)current.remove()"));
    assert!(web.router_js.contains("history.pushState"));
    assert!(web.router_js.len() < 228_495);
    assert!(
        !web.router_js
            .contains("navigator.mediaDevices.getUserMedia")
    );
    assert!(!web.router_js.contains("function renderCharts"));
    assert!(super::runtime_chunks_for_trees(&ViewNode::Children, &text("Basic")).is_empty());

    let design_css =
        super::prepare_dev_design_asset(&mut web, &FontConfig::default(), &DesignConfig::default());
    let initial_update = super::web_artifact_update(&web, None, design_css.clone());
    assert!(
        initial_update
            .files
            .iter()
            .any(|artifact| artifact.relative_path == Path::new("web/router.js"))
    );
    assert_eq!(web.pages[0].router_file_name, "router.js");

    let previous = web.clone();
    let unchanged_update = super::web_artifact_update(&web, Some(&previous), design_css);
    assert_eq!(unchanged_update.files.len(), 1);
    assert_eq!(
        unchanged_update.files[0].relative_path,
        Path::new("web/manifest.json")
    );

    let previous = web.clone();
    let page = Arc::make_mut(&mut web.pages[0]);
    page.html_document = page.html_document.replacen(
        r#"data-dowe-router type="module" src="/router.js""#,
        r#"data-dowe-router type="module" src="/""#,
        1,
    );
    assert!(
        page.html_document
            .contains(r#"data-dowe-router type="module" src="/""#)
    );
    super::prepare_incremental_dev_design_asset(
        &mut web,
        &previous,
        &FontConfig::default(),
        &DesignConfig::default(),
    );
    assert!(
        web.pages[0]
            .html_document
            .contains(r#"data-dowe-router type="module" src="/router.js""#)
    );

    let previous = web.clone();
    let previous_router = previous.router_js.clone();
    Arc::make_mut(&mut web.pages[0]).page_chunk_id = "changed-page".to_string();
    super::prepare_incremental_dev_design_asset(
        &mut web,
        &previous,
        &FontConfig::default(),
        &DesignConfig::default(),
    );
    assert_eq!(web.router_js, previous_router);
    assert!(web.router_js.contains("async function startRouter()"));
    assert!(
        web.router_js
            .contains("if(!currentRoute){await syncDevRoutes()")
    );
}

