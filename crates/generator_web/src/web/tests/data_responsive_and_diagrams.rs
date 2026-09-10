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
    assert!(
        page.content
            .contains(r#"data-dowe-show=\"ready01\" hidden"#)
    );
    assert!(!page.css_content.contains(".show-false:not([hidden])"));
    assert!(!page.css_content.contains(".md\\:show-true"));
    let design_css = show_design_css();
    assert!(design_css.contains(".show-false:not([hidden]){display:none;}"));
    assert!(design_css.contains(
        "@media (min-width:768px){.md\\:show-false:not([hidden]){display:none;}.md\\:show-true:not([hidden]){display:var(--dowe-component-display,revert);}"
    ));
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(dowe_components::RenderTarget::Web, Vec::new()),
    });
    assert!(router.contains("data-dowe-show"));
    assert!(router.contains("element.hidden=!visible"));
    assert!(router.contains("if(!scoped&&button.closest(\"[data-dowe-each-row]\"))continue"));
}

#[test]
fn emits_design_responsive_css_in_ascending_breakpoint_blocks() {
    let css = super::compose_design_base_css(
        &Default::default(),
        &FontConfig::default(),
        &DesignConfig::default(),
        false,
        false,
        false,
    );
    let base_end = css.find("@keyframes dowe-scale-in").unwrap();
    let sm = css.find("@media (min-width:640px)").unwrap();
    let md = css.find("@media (min-width:768px)").unwrap();
    let lg = css.find("@media (min-width:1024px)").unwrap();
    let xl = css.find("@media (min-width:1280px)").unwrap();

    assert!(base_end < sm && sm < md && md < lg && lg < xl);
    assert_eq!(css.matches("@media (min-width:").count(), 4);
    for width in [640, 768, 1024, 1280] {
        assert_eq!(
            css.matches(&format!("@media (min-width:{width}px)"))
                .count(),
            1
        );
    }
    let css = show_design_css();
    assert!(css.contains(".pie-chart-container.legend-left{flex-direction:row-reverse;}"));
    assert!(css.contains(".pie-chart-container .dowe-chart-viewport{flex:0 1 20rem;"));
    assert!(css.contains(".pie-chart-container .dowe-chart-svg{min-height:0;aspect-ratio:1;"));
    assert!(css.contains(".pie-chart-container.has-glow .dowe-chart-slice"));
    assert!(css.contains(".command-kbd{display:flex;}"));
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
    assert!(design_css.contains(".show-false:not([hidden]){display:none;}"));
    assert!(
        design_css.contains(
            ".md\\:show-true:not([hidden]){display:var(--dowe-component-display,revert);}"
        )
    );
    assert!(
        design_css.contains(
            ".lg\\:show-true:not([hidden]){display:var(--dowe-component-display,revert);}"
        )
    );
    assert!(
        design_css
            .find(".show-false:not([hidden]){display:none;}")
            .unwrap()
            < design_css
                .find(".box{--dowe-component-display:flex;")
                .unwrap()
    );
}

#[test]
fn renders_diagram_markup_css_and_runtime() {
    let tree = diagram_tree();
    let runtime_chunks = super::runtime_chunks_for_trees(&ViewNode::Children, &tree);
    assert_eq!(runtime_chunks.len(), 1);
    assert_eq!(runtime_chunks[0].name, "visualization");
    let html = render_page_body(&ViewNode::Children, &tree);
    assert!(html.contains(r#"data-dowe-diagram data-dowe-diagram-fit-view="true""#));
    assert!(html.contains(r#"data-dowe-diagram-nodes="flowNodes""#));
    assert!(html.contains(r#"data-dowe-diagram-edges="flowEdges""#));
    assert!(html.contains(r#"data-dowe-diagram-minimap="true""#));
    assert!(html.contains(r#"data-dowe-diagram-empty-label="No flow yet""#));
    assert!(html.contains(r#"data-dowe-diagram-on-node-click="selectNode""#));
    assert!(html.contains(r#"data-dowe-diagram-on-connect="connectNodes""#));
    assert!(html.contains("diagram-canvas"));
    assert!(html.contains("diagram-nodes-layer"));
    assert!(html.contains("diagram-minimap-svg"));

    let chunk = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/diagram.dowe"),
        "page diagramPage",
        &tree,
    );
    assert!(chunk.css_content.contains(".diagram.is-solid.is-surface"));
    let design_css = super::design_css();
    assert!(design_css.contains(".diagram .diagram-canvas"));
    assert!(design_css.contains(".diagram .diagram-node-port"));
    assert!(design_css.contains(".diagram .diagram-edge-preview"));
    assert!(design_css.contains(".diagram .diagram-minimap-viewport"));
    assert!(design_css.contains(".diagram .diagram-node.is-connect-target"));
    let runtime = super::visualization_runtime_chunk().content;
    assert!(runtime.contains("function renderDiagrams"));
    assert!(runtime.contains("function hydrateDiagrams"));
    assert!(runtime.contains("doweDiagramOnNodeClick"));
    assert!(runtime.contains("doweDiagramOnConnect"));
    assert!(runtime.contains("data-dowe-diagram-node-id"));
    assert!(runtime.contains("persistDiagramConnection"));
    assert!(runtime.contains("diagramEdgeGeometry"));
    assert!(runtime.contains("diagram-edge-preview"));
    assert!(runtime.contains("diagram-minimap-viewport"));
    assert!(runtime.contains("backgroundPosition"));
    assert!(runtime.contains("is-connect-target"));
    assert!(runtime.contains("function diagramPosition"));
}
