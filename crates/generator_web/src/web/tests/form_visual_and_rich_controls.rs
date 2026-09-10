#[test]
fn renders_reactive_form_visual_props_and_input_events() {
    let input = ViewNode::Input {
        props: VariantProps {
            size: Some(ButtonSize::Sm),
            reactive: ReactiveVariantProps {
                variant: Some("fieldVariant".to_string()),
                scheme: Some("fieldScheme".to_string()),
                size: Some("fieldSize".to_string()),
                rounded: Some("fieldRounded".to_string()),
                ..Default::default()
            },
            element: ElementProps {
                on_change: Some("fieldChanged".to_string()),
                on_input: Some("fieldInput".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
    };
    let html = render_page_body(&ViewNode::Children, &input);
    let router = full_runtime_for_test();

    assert!(html.contains(r#"data-dowe-variant-binding="true""#));
    assert!(html.contains(r#"data-dowe-variant-size-prefix="is-""#));
    assert!(html.contains(r#"data-dowe-variant="fieldVariant""#));
    assert!(html.contains(r#"data-dowe-scheme="fieldScheme""#));
    assert!(html.contains(r#"data-dowe-size="fieldSize""#));
    assert!(html.contains(r#"data-dowe-rounded="fieldRounded""#));
    assert!(html.contains(r#"data-dowe-change="fieldChanged""#));
    assert!(html.contains(r#"data-dowe-input="fieldInput""#));
    assert!(router.contains("document.addEventListener(\"input\""));
    assert!(router.contains("document.addEventListener(\"change\""));
    assert!(router.contains("doweEditorDirty"));
}

#[test]
fn renders_reactive_button_disabled_visual_state() {
    let button = ViewNode::Button {
        props: VariantProps {
            variant: Some(ComponentVariant::Solid),
            color: Some(ColorFamily::Secondary),
            reactive: ReactiveVariantProps {
                disabled: Some("formInvalid".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Submit")],
    };
    let html = render_page_body(&ViewNode::Children, &button);
    let css = super::design_css();
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

    assert!(html.contains(r#"data-dowe-button-disabled="formInvalid""#));
    assert!(css.contains("user-select:none;-webkit-user-select:none"));
    assert!(css.contains(r#".button.is-disabled,.button[aria-disabled="true"]{opacity:0.5;}"#));
    assert!(router.contains("button.classList.toggle(\"is-disabled\",disabled)"));
}

#[test]
fn renders_display_chat_and_motion_components_markup_runtime_and_css() {
    let root = Path::new("/project");
    let page_tree = display_chat_motion_tree();
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/display.dowe"),
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
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    });

    assert!(html.contains(r#"class="avatar-group"#));
    assert!(html.contains("is-solid"));
    assert!(html.contains("is-primary"));
    assert!(html.contains("avatar-group-sm"));
    assert!(html.contains("is-auto-fit"));
    assert!(html.contains("is-bordered"));
    assert!(html.contains(r#"data-dowe-avatar-group-items="people""#));
    assert!(html.contains(r#"class="chat-box"#));
    assert!(html.contains("is-conversation"));
    assert!(html.contains(r#"data-dowe-chatbox-messages="messages""#));
    assert!(html.contains(r#"data-dowe-chatbox-action-visible="actionVisible""#));
    assert!(html.contains(r#"class="chat-box-action-row""#));
    assert!(html.contains("Continue"));
    assert!(html.contains(r#"data-dowe-chatbox-on-action=""#));
    assert!(html.contains(r#"class="empty"#));
    assert!(html.contains("is-result"));
    assert!(html.contains(r#"class="empty-icon"><svg class="svg"#));
    assert!(html.contains(r#"viewBox="0 0 24 24"#));
    assert!(!html.contains(r#"viewBox="0 0 120 100"#));
    assert!(html.contains(r#"class="marquee"#));
    assert!(html.contains("is-horizontal"));
    assert!(html.contains("is-fast"));
    assert!(html.contains("pause-on-hover"));
    assert!(html.contains("is-reverse"));
    assert!(html.contains("has-fade"));
    assert!(html.contains(r#"class="typewriter""#));
    assert!(page.css_content.contains(".avatar-group"));
    assert!(
        !page
            .css_content
            .contains(".avatar-group.is-solid.is-primary")
    );
    assert!(page.css_content.contains(".chat-box"));
    assert!(page.css_content.contains(".empty"));
    assert!(css.contains(".marquee"));
    assert!(css.contains(".chat-box-action"));
    assert!(css.contains(".chat-choice"));
    assert!(css.contains(".chat-message-questions"));
    assert!(css.contains(".typewriter"));
    assert!(router.contains("function renderAvatarGroups(root,state,scope)"));
    assert!(router.contains("function renderChatBoxes(root,state,scope)"));
    assert!(router.contains("data-dowe-chatbox-action-row"));
    assert!(router.contains("data-dowe-chatbox-choice"));
    assert!(router.contains("selectChatBoxChoice"));
    assert!(router.contains("chatAction"));
    assert!(router.contains("function chatMessageIsSpanish(item)"));
    assert!(router.contains("Supuestos de trabajo"));
    assert!(router.contains("function hydrateTypeWriters(root)"));
}

#[test]
fn renders_rich_control_map_components_markup_runtime_and_css() {
    let root = Path::new("/project");
    let page_tree = rich_control_map_tree();
    assert!(super::runtime_chunks_for_trees(&ViewNode::Children, &page_tree).is_empty());
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/rich-controls.dowe"),
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
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    });

    assert!(html.contains(r#"class="rich-text"#));
    assert!(html.contains(r#"data-dowe-rich-text"#));
    assert!(html.contains(r#"data-dowe-rich-mark"#));
    assert!(html.contains("title-md"));
    assert!(html.contains("rich-mark-grad"));
    assert!(html.contains(r#"data-dowe-record"#));
    assert!(html.contains(r#"data-dowe-toggle-group"#));
    assert!(html.contains(r#"data-dowe-pagination"#));
    assert!(html.contains(r#"data-dowe-pagination-total="total""#));
    assert!(html.contains(r#"data-dowe-pagination-page-size="60""#));
    assert!(html.contains(r#"data-dowe-pagination-step="-1""#));
    assert!(html.contains(r#"aria-label="Previous page""#));
    assert!(html.contains(r#"class="pagination-icon""#));
    assert!(html.contains(r#"data-dowe-collapsible"#));
    assert!(html.contains(r#"class="collapsible-arrow" aria-hidden="true"><svg"#));
    assert!(!html.contains(r#"class="collapsible-arrow" aria-hidden="true">⌄"#));
    assert!(html.contains(r#"data-dowe-countdown-target="2030-01-01T00:00:00Z""#));
    assert!(html.contains(r#"data-dowe-map"#));
    assert!(html.contains(r#"data-dowe-map-marker="office""#));
    assert!(page.css_content.contains(".media.is-solid.is-primary"));
    assert!(
        page.css_content
            .contains(".toggle-group.is-solid.is-secondary")
    );
    assert!(
        page.css_content
            .contains(".collapsible.is-solid.is-surface")
    );
    assert!(
        page.css_content
            .contains(".countdown-box.is-outlined.is-primary")
    );
    assert!(page.css_content.contains(".map.is-solid.is-surface"));
    assert!(css.contains(".rich-mark-grad"));
    assert!(css.contains("max-width:100%;width:100%;text-align:center;line-height:inherit"));
    assert!(css.contains("background:var(--rich-accent);color:var(--rich-on-accent)"));
    assert!(css.contains("padding:.125rem .5rem"));
    assert!(css.contains("display:inline-block;box-sizing:border-box;max-width:100%;text-align:center;white-space:normal;overflow-wrap:normal;word-break:normal"));
    assert!(css.contains("dowe-rich-neon-flicker"));
    assert!(css.contains(".rich-mark-slant::before"));
    assert!(css.contains(".record-wave"));
    assert!(css.contains(".toggle-group-item"));
    assert!(page.css_content.contains(
        ".toggle-group.is-solid.is-primary{--dowe-content-text:var(--dowe-primaryText);--dowe-content-title:var(--dowe-primaryTitle);background-color:var(--dowe-primary);color:var(--dowe-primaryText);border-color:transparent;}"
    ));
    assert!(!page.css_content.contains(".toggle-group.is-solid.is-primary{--dowe-content-text:var(--dowe-primaryText);--dowe-content-title:var(--dowe-primaryTitle);background-color:var(--dowe-primary);color:var(--dowe-primaryText);border-color:var(--dowe-primary);}"));
    assert!(css.contains(".pagination-nav"));
    assert!(css.contains(".collapsible-content"));
    assert!(css.contains(".collapsible-arrow>svg{width:100%;height:100%;}"));
    assert!(css.contains(".countdown-box"));
    assert!(css.contains("overflow-x:auto"));
    assert!(css.contains("min-width:3.5rem"));
    assert!(css.contains("padding-inline:.5rem"));
    assert!(css.contains("container-type:inline-size"));
    assert!(css.contains("@container(max-width:30rem)"));
    assert!(css.contains(".countdown-lg .countdown-box,.countdown-xl .countdown-box{min-width:2.5rem;height:3rem;padding-inline:.375rem;}"));
    assert!(css.contains(".map-grid"));
    assert!(router.contains("function hydrateRecords(root)"));
    assert!(router.contains("function fitRichTextMark(mark,availableWidth)"));
    assert!(router.contains("range.getClientRects()"));
    assert!(router.contains("function hydrateRichTexts(root)"));
    assert!(router.contains("new ResizeObserver(fit)"));
    assert!(router.contains("hydrateRichTexts(root)"));
    assert!(router.contains("function renderToggleGroups(root,state,scope)"));
    assert!(router.contains("function paginationPages(group,state,scope)"));
    assert!(router.contains("group.dataset.dowePagination"));
    assert!(router.contains("function hydrateCountdowns(root)"));
    assert!(router.contains("function toggleCollapsible"));
}

