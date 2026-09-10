#[test]
fn renders_drawer_markup_runtime_and_css() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Drawer {
        props: DrawerProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Solid),
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
            r#"class="drawer is-solid is-surface is-end" role="dialog" aria-modal="true""#
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
    assert!(page.css_content.contains(".drawer.is-solid.is-surface"));
    assert!(router.contains("function closeDrawer(drawer)"));
    assert!(router.contains("data-dowe-drawer-overlay"));

    let rounded_html = render_page_body(
        &ViewNode::Children,
        &ViewNode::Drawer {
            props: DrawerProps {
                style: VariantProps {
                    variant: Some(ComponentVariant::Solid),
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
    assert!(rounded_html.contains("drawer rounded-lg is-solid is-surface is-start"));
}
