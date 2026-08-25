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
    let router = full_runtime_for_test();

    assert!(html.contains(r#"class="avatar is-soft is-success avatar-lg is-bordered""#));
    assert!(html.contains(r#"class="badge is-bottom-right""#));
    assert!(html.contains(r#"class="badge-content is-solid is-danger""#));
    assert!(html.contains(r#"class="chip is-outlined is-info chip-sm has-close""#));
    assert!(html.contains(r#"<span class="chip-icon"><svg"#));
    assert!(html.contains(r#"class="skeleton"#));
    assert!(html.contains("is-pulse"));
    assert!(html.contains("is-rounded"));
    assert!(html.contains(r#"data-dowe-modal data-dowe-modal-open="modal01""#));
    assert!(html.contains(r#"aria-label="Close modal" data-dowe-modal-close><svg"#));
    assert!(html.contains("m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073"));
    assert!(!html.contains("data-dowe-modal-close>&times;"));
    assert!(html.contains(r#"class="alert-dialog-actions""#));
    assert!(html.contains(r#"class="modal is-solid is-surface" role="alertdialog""#));
    assert!(html.contains(r#"class="button button-md is-solid is-danger""#));
    assert!(html.contains(r#"class="tooltip-popover is-solid is-muted position-end""#));
    assert!(html.contains(r#"class="toast is-outlined is-warning is-top-right"#));
    assert!(html.contains(r#"<span class="toast-icon" aria-hidden="true">✓</span>"#));
    assert!(html.contains(r#"aria-label="Close toast" data-dowe-toast-close><svg"#));
    assert!(!html.contains("data-dowe-toast-close>&times;"));
    assert!(html.contains(r#"class="dropdown-popover is-solid is-surface""#));
    assert!(html.contains(r#"data-dowe-command-open="modal01""#));
    assert!(page.css_content.contains(".avatar.is-soft.is-success"));
    assert!(page.css_content.contains(".w-4{width:1rem;}"));
    assert!(page.css_content.contains(".h-4{height:1rem;}"));
    assert!(
        page.css_content
            .contains(".badge-content.is-solid.is-danger")
    );
    assert!(!page.css_content.contains(".badge.is-solid.is-danger"));
    assert!(page.css_content.contains(".modal.is-solid.is-surface"));
    assert!(page.css_content.contains(
        ".toast.is-outlined.is-warning{--dowe-content-text:var(--dowe-surfaceText);--dowe-content-title:var(--dowe-surfaceTitle);background-color:var(--dowe-surface);color:var(--dowe-surfaceText);border:1px solid var(--dowe-warning);}"
    ));
    assert!(
        page.css_content
            .contains(".dropdown-popover.is-solid.is-surface")
    );
    assert!(css.contains(".tooltip-popover{position:fixed;"));
    assert!(css.contains(".toast-icon{flex:0 0 auto;"));
    assert!(css.contains(".tooltip-arrow{background-color:inherit;}"));
    assert!(css.contains("@keyframes dowe-skeleton-pulse"));
    assert!(css.contains(".modal{position:relative;display:flex;max-width:min(100%,35rem);max-height:calc(100vh - 2rem);flex-direction:column;gap:1rem;overflow:hidden;padding:1.25rem;"));
    assert!(css.contains(
        ".drawer-close,.modal-close,.toast-close{display:inline-flex;width:1.75rem;height:1.75rem;"
    ));
    assert!(css.contains(".drawer-close svg,.modal-close svg,.toast-close svg{display:block;width:1.125rem;height:1.125rem;}"));
    assert!(router.contains("function renderModals(root,state,scope)"));
    assert!(router.contains("function renderToasts(root,state,scope)"));
    assert!(router.contains("data-dowe-toast-close"));
    assert!(router.contains("toastClose.closest(\"[data-dowe-toast],#dowe-global-toast\")"));
    assert!(router.contains("is-${toast.variant||\"solid\"}"));
    assert!(router.contains("function openCommand(command)"));
    assert!(router.contains("data-dowe-dropdown-trigger"));
}

#[test]
fn resolves_modal_and_alert_dialog_panels_like_card_surfaces() {
    let root = Path::new("/project");
    let modal = ViewNode::Modal {
        props: ModalProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Outlined),
                color: Some(ColorFamily::Warning),
                ..Default::default()
            },
            open: "modal01".to_string(),
            on_close: None,
            disable_overlay_close: false,
            hide_close_button: false,
        },
        header: Vec::new(),
        body: vec![text("Body")],
        footer: Vec::new(),
    };
    let modal_page = build_page_chunk(
        root,
        Path::new("/project/src/pages/modal.dowe"),
        "modal",
        &modal,
    );
    assert!(modal_page.css_content.contains(
        ".modal.is-outlined.is-warning{--dowe-content-text:var(--dowe-surfaceText);--dowe-content-title:var(--dowe-surfaceTitle);background-color:var(--dowe-surface);color:var(--dowe-surfaceText);border:1px solid var(--dowe-warning);}"
    ));

    let alert = ViewNode::AlertDialog {
        props: AlertDialogProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Solid),
                color: Some(ColorFamily::Warning),
                ..Default::default()
            },
            open: "alert01".to_string(),
            title: "Archive?".to_string(),
            description: "Archive this project.".to_string(),
            confirm_text: "Archive".to_string(),
            cancel_text: "Cancel".to_string(),
            on_confirm: None,
            on_cancel: None,
            loading: false,
        },
    };
    let alert_page = build_page_chunk(
        root,
        Path::new("/project/src/pages/alert.dowe"),
        "alert",
        &alert,
    );
    assert!(alert_page.css_content.contains(".modal.is-soft.is-surface"));
    assert!(
        alert_page
            .css_content
            .contains(".button.is-solid.is-warning")
    );
}

#[test]
fn emits_global_toast_action_surface_rules() {
    let tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: Vec::new(),
        actions: vec![ViewAction {
            id: "notify01".to_string(),
            name: "notify".to_string(),
            params: Vec::new(),
            return_type: None,
            kind: ViewActionKind::Sequence(vec![ViewFunctionStatement::Toast(ViewToastAction {
                kind: "warning".to_string(),
                title: "Review".to_string(),
                message: "Check the changes".to_string(),
                duration: Some(4000),
                scheme: Some("warning".to_string()),
                variant: Some("outlined".to_string()),
                position: Some("top-right".to_string()),
            })]),
        }],
        children: vec![text("Notify")],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/views/pages/notify.dowe"),
        "notify",
        &tree,
    );

    assert!(page.css_content.contains(
        ".toast.is-outlined.is-warning{--dowe-content-text:var(--dowe-surfaceText);--dowe-content-title:var(--dowe-surfaceTitle);background-color:var(--dowe-surface);color:var(--dowe-surfaceText);border:1px solid var(--dowe-warning);}"
    ));
}

#[test]
fn renders_solar_icon_fill_and_stroke_attributes() {
    let fill = svg_path_attributes(SvgPathFill::Fill {
        color: Some(ColorToken::Secondary),
        opacity: 128,
        even_odd: true,
    });
    assert!(fill.contains("fill=\"var(--dowe-secondary)\""));
    assert!(fill.contains("opacity=\"0.502\""));
    assert!(fill.contains("fill-rule=\"evenodd\""));
    let stroke = svg_path_attributes(SvgPathFill::Stroke {
        color: Some(ColorToken::Accent),
        opacity: 255,
        width: 150,
        line_cap: SvgLineCap::Round,
        line_join: SvgLineJoin::Round,
    });
    assert!(stroke.contains("stroke=\"var(--dowe-accent)\""));
    assert!(stroke.contains("stroke-width=\"1.50\""));
}

#[test]
fn renders_svg_spinner_css_and_reduced_motion_behavior() {
    let spinner = icon_component_node(vec![
        ComponentProp {
            name: "name".to_string(),
            value: PropValue::String("svg-spinners:3-dots-bounce".to_string()),
        },
        ComponentProp {
            name: "fill".to_string(),
            value: PropValue::String("primary".to_string()),
        },
    ])
    .expect("spinner");
    let html = render_page_body(&ViewNode::Children, &spinner);

    assert!(html.contains("is-svg-spinner"));
    assert!(html.contains("@keyframes spinner_8HQG"));
    assert!(html.contains("fill=\"var(--dowe-primary)\""));
    assert!(html.contains("@media (prefers-reduced-motion:reduce)"));
    assert!(html.contains("dowe-svg-spinner-fallback"));
    assert!(!html.contains("spinner_Pcrv"));
}

#[test]
fn renders_svg_logo_as_an_isolated_bundled_data_resource() {
    let logo = icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String("svg-logos:github-icon".to_string()),
    }])
    .expect("SVG logo");
    let html = render_page_body(&ViewNode::Children, &logo);

    assert!(html.contains("data:image/svg+xml,"));
    assert!(html.contains("%3Csvg"));
    assert!(html.contains(r#"<image width="100%" height="100%""#));
    assert!(!html.contains("is-svg-spinner"));
}

#[test]
fn renders_reactive_button_loading_spinner_and_runtime_binding() {
    let button = ViewNode::Button {
        props: VariantProps {
            loading_icon: Some(svg_spinner_control_icon("3-dots-move").expect("button spinner")),
            reactive: ReactiveVariantProps {
                loading: Some("saving".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Save")],
    };
    let html = render_page_body(&ViewNode::Children, &button);
    let css = super::design_css();
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });

    assert!(html.contains(r#"data-dowe-button-loading hidden aria-hidden="true""#));
    assert!(html.contains("data-dowe-button-content"));
    assert!(html.contains("is-svg-spinner"));
    assert!(html.contains(r#"data-dowe-button-loading="saving""#));
    assert!(css.contains(".button.is-loading>[data-dowe-button-content]"));
    assert!(router.contains("doweButtonLoading"));
    assert!(router.contains("aria-busy"));
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
    });

    assert!(html.contains(r#"class="avatar-group"#));
    assert!(html.contains("is-soft"));
    assert!(html.contains("is-primary"));
    assert!(html.contains("avatar-group-sm"));
    assert!(html.contains("is-auto-fit"));
    assert!(html.contains("is-bordered"));
    assert!(html.contains(r#"data-dowe-avatar-group-items="people""#));
    assert!(html.contains(r#"class="chat-box"#));
    assert!(html.contains("is-conversation"));
    assert!(html.contains(r#"data-dowe-chatbox-messages="messages""#));
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
            .contains(".avatar-group.is-soft.is-primary")
    );
    assert!(page.css_content.contains(".chat-box"));
    assert!(page.css_content.contains(".empty"));
    assert!(css.contains(".marquee"));
    assert!(css.contains(".typewriter"));
    assert!(router.contains("function renderAvatarGroups(root,state,scope)"));
    assert!(router.contains("function renderChatBoxes(root,state,scope)"));
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
    assert!(page.css_content.contains(".media.is-soft.is-primary"));
    assert!(page
        .css_content
        .contains(".media.is-soft.is-primary .media-button{background-color:var(--dowe-primary);color:var(--dowe-primaryText);}"));
    assert!(
        page.css_content
            .contains(".toggle-group.is-soft.is-secondary")
    );
    assert!(
        page.css_content
            .contains(".collapsible.is-solid.is-surface")
    );
    assert!(
        page.css_content
            .contains(".countdown-box.is-outlined.is-primary")
    );
    assert!(page.css_content.contains(".map.is-soft.is-surface"));
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
    assert!(page.css_content.contains(
        ".toggle-group-item.is-active.is-solid.is-primary,.toggle-group-item.is-active.is-soft.is-primary{background-color:var(--dowe-primaryText);color:var(--dowe-primary);}"
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
    let router = full_runtime_for_test();

    assert!(html.contains(r#"class="media is-soft is-primary""#));
    assert!(html.contains(r#"data-dowe-audio"#));
    assert!(html.contains(r#"data-dowe-audio-play-icon"#));
    assert!(html.contains(r#"data-dowe-audio-pause-icon"#));
    assert!(html.contains(r#"data-dowe-audio-waveform"#));
    assert!(html.contains(r#"class="media-bars loaded"#));
    assert_eq!(
        html.matches(r#"class="media-bar" style="height:"#).count(),
        50
    );
    assert!(html.contains(r#"class="image is-solid is-secondary square fit-contain""#));
    assert!(html.contains(r#"data-dowe-image"#));
    assert!(html.contains(r#"data-dowe-image-download"#));
    assert!(html.contains(r#"data-dowe-image-fullscreen"#));

    let hidden_image = ViewNode::Image {
        props: ImageProps {
            style: VariantProps::default(),
            src: "https://example.com/photo.jpg".to_string(),
            reactive_src: None,
            alt: "Photo".to_string(),
            aspect: ImageAspect::Auto,
            object_fit: ImageObjectFit::Cover,
            loading: ImageLoading::Lazy,
            hide_controls: true,
        },
    };
    let hidden_image_html = render_page_body(&ViewNode::Children, &hidden_image);
    assert!(!hidden_image_html.contains(r#"data-dowe-image-download"#));
    assert!(!hidden_image_html.contains(r#"data-dowe-image-fullscreen"#));
    assert!(super::runtime_chunks_for_trees(&ViewNode::Children, &hidden_image).is_empty());
    let dynamic_image = ViewNode::Image {
        props: ImageProps {
            style: VariantProps::default(),
            src: String::new(),
            reactive_src: Some("feature.cover".to_string()),
            alt: "Feature cover".to_string(),
            aspect: ImageAspect::Auto,
            object_fit: ImageObjectFit::Cover,
            loading: ImageLoading::Lazy,
            hide_controls: true,
        },
    };
    let dynamic_image_html = render_page_body(&ViewNode::Children, &dynamic_image);
    assert!(dynamic_image_html.contains(r#"src="" data-dowe-image-src="feature.cover""#));
    assert!(router.contains("function renderReactiveImages"));
    assert!(html.contains(r#"data-dowe-accordion data-dowe-accordion-multiple="true""#));
    assert!(html.contains(r#"class="accordion-arrow" aria-hidden="true"><svg"#));
    assert!(html.contains(r#"d="m19.704 12l-8.491-8.727a.75.75 0 1 1 1.075-1.046l9 9.25a.75.75 0 0 1 0 1.046l-9 9.25a.75.75 0 1 1-1.075-1.046z""#));
    assert!(!html.contains(r#"class="accordion-arrow">⌄"#));
    assert!(html.contains(r#"data-dowe-carousel data-dowe-carousel-index="0""#));
    assert!(html.contains(r#"data-dowe-carousel-variant="coverFlow""#));
    assert!(html.contains("is-cover-flow"));
    assert!(html.contains(r#"class="checkbox-input is-success""#));
    assert!(html.contains(r#"data-dowe-bind="accepted""#));
    assert!(html.contains(r#"data-dowe-color-picker"#));
    assert!(html.contains("has-start-adornment"));
    assert!(html.contains("has-value"));
    assert!(html.contains(r#"data-dowe-color-sv role="slider""#));
    assert!(html.contains(r#"data-dowe-color-hue role="slider""#));
    assert!(!html.contains(r#"type="color""#));
    assert!(html.contains(r#"data-dowe-date-field"#));
    assert!(html.contains(r#"data-dowe-date-range"#));
    assert!(!html.contains(r#"type="date""#));
    assert!(html.contains(r#"class="radio-group is-horizontal""#));
    assert!(html.contains(r#"class="radio is-muted is-lg""#));
    assert!(html.contains(r#"class="toggle-input is-secondary""#));

    assert!(page.css_content.contains(".media.is-soft.is-primary"));
    assert!(
        page.css_content
            .contains(".accordion.is-outlined.is-surface")
    );
    assert!(page.css_content.contains(
        ".accordion.is-outlined.is-surface{--dowe-content-text:var(--dowe-surfaceText);--dowe-content-title:var(--dowe-surfaceTitle);background-color:var(--dowe-surface);color:var(--dowe-surfaceText);border:1px solid var(--dowe-surface);padding:.25rem;gap:.75rem;}"
    ));
    assert!(page.css_content.contains(
        ".accordion.is-outlined.is-surface .accordion-item{background-color:var(--dowe-surface);border:1px solid color-mix(in srgb,var(--dowe-surface) 24%,transparent);"
    ));
    assert!(page.css_content.contains(".carousel.is-solid.is-info"));
    assert!(css.contains(".checkbox-input{position:relative;"));
    assert!(css.contains("border-radius:.25rem"));
    assert!(css.contains(".radio-group.is-vertical{flex-direction:column;}"));
    assert!(css.contains(".radio-group.is-horizontal{flex-direction:row;flex-wrap:wrap;}"));
    assert!(css.contains(".toggle-input{position:relative;"));
    assert!(css.contains(".color-picker-popover{position:fixed;"));
    assert!(css.contains(".color-picker-canvas{position:relative;"));
    assert!(css.contains(".color-field.is-floating.is-lg{min-height:3.5rem}"));
    assert!(css.contains(".color-field.is-floating .color-control-trigger{min-height:var(--dowe-control-min-height);padding-top:.5rem;}"));
    assert!(css.contains(".date-control-trigger{display:flex;"));
    assert!(css.contains(".date-range-calendars{display:flex;"));
    assert!(css.contains(".accordion-arrow>svg{width:100%;height:100%;}"));
    assert!(!css.contains(".accordion-arrow{background-color:"));
    assert!(css.contains(".accordion-label{font-size:.9375rem;font-weight:700;line-height:1.35;}"));
    assert!(css.contains(".accordion-header.is-open .accordion-arrow{transform:rotate(90deg);}"));
    assert!(css.contains(".media-icon>svg{width:100%;height:100%;}"));
    assert!(css.contains("@keyframes dowe-media-wave-appear"));
    assert!(css.contains(".media-bars.loaded .media-bar.active"));
    assert!(router.contains("function hydrateAudios(root)"));
    assert!(router.contains("function startAudioFrame(root)"));
    assert!(router.contains("aria-pressed"));
    assert!(router.contains("function seekAudio(root,clientX)"));
    assert!(router.contains("pointerdown"));
    assert!(router.contains("aria-valuetext"));
    assert!(router.contains("function toggleAccordion(trigger)"));
    assert!(router.contains("function renderCarousel(root)"));
    assert!(router.contains("function renderCarouselEffects"));
    assert!(router.contains("case\"coverFlow\""));
    assert!(router.contains("touchmove"));
    assert!(router.contains("function renderDateField(root,state,scope)"));
    assert!(router.contains("function renderDoweColor(root,state,scope)"));
    assert!(router.contains("function doweColorOklch(rgb)"));
    assert!(router.contains("function updateDoweColorPointer(target,event)"));
    assert!(router.contains("function splitDestination"));
    assert!(router.contains("window.addEventListener(\"scroll\""));
    assert!(!router.contains(")window.addEventListener"));
    assert!(router.contains("function selectDateValue(root,value)"));
    assert!(router.contains("function syncCarousel(root)"));
    assert!(router.contains("maximum-position<=edge"));
    assert!(router.contains("function scrollCarouselSlide(root,slide"));
    assert!(router.contains("viewport.scrollTo({left,behavior})"));
    assert!(router.contains("button.disabled=disabled"));
    assert!(!router.contains("slides[index].scrollIntoView"));
    assert!(router.contains("pointerdown"));
    assert!(css.contains(".carousel-viewport::-webkit-scrollbar"));
    assert!(css.contains("scrollbar-width:none"));
    assert!(css.contains(".carousel.is-vertical{flex-direction:column"));
    assert!(css.contains(".carousel-nav:disabled,.carousel-control:disabled"));
    assert!(css.contains("-webkit-overflow-scrolling:touch"));
    assert!(css.contains("border:0;border-radius:1.25rem;background:transparent;box-shadow:none"));
    for variant in [
        "is-simple",
        "is-snapping",
        "is-masonry",
        "is-rtl",
        "is-sticky",
        "is-controls",
        "is-dots",
        "is-thumbnails",
        "is-cover-flow",
        "is-slideshow",
        "is-stories",
        "is-smart-stack",
        "is-card-stack",
        "is-flipbook",
    ] {
        assert!(css.contains(variant));
    }
    assert!(router.contains("function downloadImage(root)"));
}

#[test]
fn renders_advanced_form_components_markup_runtime_and_css() {
    let root = Path::new("/project");
    let page_tree = advanced_form_tree();
    let runtime_chunks = super::runtime_chunks_for_trees(&ViewNode::Children, &page_tree);
    assert_eq!(
        runtime_chunks
            .iter()
            .map(|chunk| chunk.name)
            .collect::<Vec<_>>(),
        vec!["controls"]
    );
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/advanced.dowe"),
        "page",
        &page_tree,
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);
    let css = super::design_css();
    let router = full_runtime_for_test();

    assert!(html.contains(r#"class="combo-box"#));
    assert!(html.contains(r#"data-dowe-combo-box"#));
    assert!(html.contains(r#"data-dowe-bind="profile.role""#));
    assert!(html.contains(r#"data-dowe-combo-value="admin""#));
    assert!(html.contains(r#"data-dowe-combo-search"#));
    assert!(html.contains(r#"data-dowe-combo-clear"#));
    assert!(html.contains(r#"class="csv-field"#));
    assert!(html.contains(r#"data-dowe-csv"#));
    assert!(html.contains("Email"));
    assert!(html.contains(r#"class="drag-drop"#));
    assert!(html.contains(r#"data-dowe-drag-drop"#));
    assert!(html.contains(r#"data-dowe-drag-item="draft""#));
    assert!(html.contains(r#"class="editor"#));
    assert!(html.contains(r#"data-dowe-editor"#));
    assert!(html.contains("image-cropper"));
    assert!(html.contains("is-circle"));
    assert!(html.contains(r#"data-dowe-image-cropper"#));
    assert!(html.contains(r#"data-dowe-cropper-stage"#));
    assert!(html.contains(r#"data-dowe-cropper-zoom"#));
    assert!(html.contains("Apply"));
    assert!(router.contains("cropperApply"));
    assert!(router.contains("toDataURL"));
    assert!(router.contains(r#"value.match(/^data:(image\/[^;]+)/)"#));
    assert!(!router.contains(r#"value.match(/^data:(image\\/[^;]+)/)"#));
    assert!(router.contains(
        "closeCanvasFrames(previous);closeCameraFrames(previous);closeMicrophoneFrames(previous);"
    ));
    assert!(!router.contains("closeCameraFrames(view);closeMicrophoneFrames(view);"));
    assert!(html.contains(r#"class="password"#));
    assert!(html.contains(r#"data-dowe-password-input"#));
    assert!(html.contains(r#"data-dowe-password-toggle"#));
    assert!(html.contains(r#"data-dowe-password-show-icon"#));
    assert!(html.contains(r#"data-dowe-password-hide-icon hidden"#));
    assert!(html.contains(r#"aria-label="Show password""#));
    assert!(html.contains(r#"class="password-strength"#));
    assert!(router.contains("passwordToggle.setAttribute(\"aria-label\""));
    assert!(router.contains("[data-dowe-password-show-icon]"));
    assert!(router.contains("[data-dowe-password-hide-icon]"));
    assert!(html.contains(r#"class="phone"#));
    assert!(html.contains(r#"data-dowe-phone"#));
    assert!(html.contains(r#"data-dowe-phone-option"#));
    assert!(html.contains(r#"viewBox="0 0 512 512""#));
    assert!(html.contains(r##"fill="#d80027""##));
    assert!(html.contains(r#"aria-expanded="false""#));
    assert!(html.contains(r#"inputmode="numeric""#));
    assert!(html.contains(r#"pattern="[0-9]*""#));
    assert!(router.contains("flag.innerHTML=source?.innerHTML"));
    assert!(router.contains("[data-dowe-phone-option]"));
    assert!(router.contains("function sanitizePhoneInput(input)"));
    assert!(router.contains("function positionPhone(root)"));
    assert!(router.contains("popover.hidden=open"));
    assert!(css.contains(".phone-popover{position:fixed;"));
    assert!(css.contains(".phone-country-trigger{display:inline-flex"));
    assert!(css.contains("padding:0 .75rem;"));
    assert!(css.contains(".phone-search-wrap{display:flex"));
    assert!(css.contains(".phone-country{gap:.625rem;padding:.5rem .75rem;"));
    assert!(html.contains(r#"class="pin"#));
    assert!(html.contains(r#"data-dowe-pin"#));
    assert!(!html.contains('�'));
    assert!(html.contains("pin-cell"));
    assert!(html.contains(r#"class="pin-input"#));
    assert!(!html.contains(r#"class="pin is-outlined"#));
    assert!(!html.contains(r#"<input type="number""#));
    assert!(html.contains(r#"inputmode="numeric""#));
    assert!(html.contains(r#"class="textarea"#));
    assert!(html.contains(r#"maxlength="160""#));
    assert!(page.css_content.contains(".control.is-outlined.is-primary"));
    assert!(page.css_content.contains(".button.is-outlined.is-primary"));
    assert!(page.css_content.contains(".drag-drop.is-soft.is-primary"));
    assert!(css.contains(".combo-box-options"));
    assert!(css.contains(".combo-box-popover{position:fixed;"));
    assert!(css.contains(".combo-box-option:disabled"));
    assert!(css.contains(".password-toggle .svg{width:1.25rem;height:1.25rem;}"));
    assert!(css.contains(".csv-field-modal"));
    assert!(css.contains(".drag-drop-item"));
    assert!(css.contains(".pin-cell.control"));
    assert!(css.contains(".editor-toolbar"));
    assert!(css.contains(".password-strength"));
    assert!(css.contains(
        ".password-strength{display:flex;width:100%;flex-direction:column;gap:.35rem;padding:0;}"
    ));
    assert!(css.contains(
        ".textarea-field.is-floating .control-label{top:.25rem;transform:translateY(0);font-size:.75rem;}"
    ));
    assert!(css.contains(
        ".textarea-field.is-floating:not(:focus-within) .textarea-control:placeholder-shown::placeholder{opacity:0;}"
    ));
    assert!(router.contains("function hydrateAdvancedForms(root)"));
    assert!(router.contains("function filterCombo(root)"));
    assert!(router.contains("function openCombo(control)"));
    assert!(router.contains("function positionCombo(control)"));
    assert!(router.contains("function hydrateCombo(control)"));
    assert!(router.contains("data-dowe-combo-popover"));
    assert!(router.contains("function handleCsvFile(input)"));
    assert!(router.contains("function renderPasswordStrength(input)"));
    assert!(router.contains("clipboardData"));
    assert!(router.contains("event.key===\"Backspace\""));
    assert!(router.contains("target.value=next.slice(0,1)"));
    assert!(router.contains(
        "const rootIndex=Array.from(document.querySelectorAll(\"[data-dowe-pin]\")).indexOf(root)"
    ));
    assert!(router.contains(
        "nextRoot=Array.from(document.querySelectorAll(\"[data-dowe-pin]\"))[rootIndex]||root"
    ));
    assert!(router.contains("requestAnimationFrame(()=>Array.from(nextRoot.querySelectorAll(\"[data-dowe-pin-cell]\"))[focusIndex]?.focus())"));
    assert!(
        router.contains("updatePin(root,true,target.value&&index+1<cells.length?index+1:null)")
    );
    assert!(router.contains("input.closest(\".field\")"));
    assert!(router.contains("function updatePin(root"));
    assert!(css.contains(".pin-cell.control.is-sm{flex-basis:2.5rem;width:2.5rem;min-width:2.5rem;height:2rem;min-height:2rem;}"));
    assert!(css.contains(".pin-cell.control.is-md{flex-basis:2.75rem;width:2.75rem;min-width:2.75rem;height:2.5rem;min-height:2.5rem;}"));
    assert!(css.contains(".pin-cell.control.is-lg{flex-basis:3.25rem;width:3.25rem;min-width:3.25rem;height:3rem;min-height:3rem;}"));
    assert!(css.contains("font-size:var(--dowe-control-font-size);line-height:var(--dowe-control-line-height);font-weight:800"));
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
            ".control{--dowe-component-display:flex;--dowe-control-min-height:var(--dowe-form-control-min-md);--dowe-control-height:var(--dowe-control-min-height);--dowe-control-font-size:var(--dowe-form-control-text-md);--dowe-control-line-height:var(--dowe-form-control-line-md);position:relative;display:var(--dowe-show,var(--dowe-component-display));align-items:center;width:100%;height:var(--dowe-control-height);min-height:var(--dowe-control-height);"
        ));
    assert!(css.contains(".control.is-sm{--dowe-control-min-height:var(--dowe-form-control-min-sm);--dowe-control-font-size:var(--dowe-form-control-text-sm);--dowe-control-line-height:var(--dowe-form-control-line-sm);}"));
    assert!(css.contains(".control.is-md{--dowe-control-min-height:var(--dowe-form-control-min-md);--dowe-control-font-size:var(--dowe-form-control-text-md);--dowe-control-line-height:var(--dowe-form-control-line-md);}"));
    assert!(css.contains(".control.is-lg{--dowe-control-min-height:var(--dowe-form-control-min-lg);--dowe-control-font-size:var(--dowe-form-control-text-lg);--dowe-control-line-height:var(--dowe-form-control-line-lg);}"));
    assert!(css.contains(
        ".control.is-floating{--dowe-control-height:calc(var(--dowe-control-min-height) + var(--dowe-form-control-floating));padding-top:var(--dowe-form-control-floating);}"
    ));
    assert!(css.contains(".textarea-field{align-items:stretch;height:auto;"));
    assert!(css.contains(
        "min-height:var(--dowe-control-min-height);padding:0 var(--dowe-form-control-padding);"
    ));

    assert!(css.contains(".control.is-floating"));
    assert!(css.contains(
        ".color-field.is-floating.is-sm.has-start-adornment.has-value>.control-label{left:2.5rem;max-width:calc(100% - 3.25rem);}",
    ));
    assert!(css.contains(
        ".color-field.is-floating.is-md.has-start-adornment.has-value>.control-label{left:2.75rem;max-width:calc(100% - 3.5rem);}",
    ));
    assert!(css.contains(
        ".color-field.is-floating.is-lg.has-start-adornment.has-value>.control-label{left:3.25rem;max-width:calc(100% - 4rem);}",
    ));
    assert!(!css.contains(".control-icon:first-child"));
    assert!(css.contains(".field{display:flex;flex-direction:column;"));
    assert!(css.contains(".select-popover{position:fixed;"));
    assert!(
        css.contains("transition:opacity 160ms ease,transform 160ms ease,visibility 160ms ease;")
    );
    assert!(css.contains(".select-popover.is-active{opacity:1;visibility:visible;pointer-events:auto;transform:translateY(0) scale(1);"));
    assert!(css.contains(".select-arrow{width:1em;height:1em;"));
    assert!(css.contains(".alert{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));align-items:center;justify-content:space-between;gap:0.75rem;padding:0.625rem 0.875rem;border-radius:var(--dowe-radius);}"));
    assert!(css.contains(
            ".select-control.is-floating:not(.is-open):not(.has-value) .select-value{visibility:hidden;}"
        ));
    assert!(css.contains("--dowe-form-control-text-md:clamp(0.875rem,0.82rem + 0.25vw,1rem);"));
    assert!(css.contains(
        "font-size:var(--dowe-control-font-size);line-height:var(--dowe-control-line-height);"
    ));
    assert!(css.contains(".color-field-value{font-size:var(--dowe-control-font-size);line-height:var(--dowe-control-line-height);}"));
    assert!(css.contains(
        ".grid>[data-dowe-each],.flex>[data-dowe-each],[data-dowe-each-row]{display:contents;}"
    ));
    assert!(page.css_content.contains(
            ".control.is-outlined.is-secondary{background-color:var(--dowe-background);color:var(--dowe-secondary);border:1px solid rgba(127,127,127,0.36);}"
        ));
    assert!(page.css_content.contains(
        ".control.is-outlined.is-secondary:focus-within{border-color:var(--dowe-secondary);"
    ));
}

#[test]
fn emits_form_validation_metadata_runtime_and_accessibility_hooks() {
    let mut props = VariantProps {
        label: Some("Email".to_string()),
        variant: Some(ComponentVariant::Outlined),
        ..Default::default()
    };
    let validation = props.element.form_validation_mut();
    validation.help_text = Some("Use your work email".to_string());
    validation.rules = vec![
        dowe_components::form_validation_rule("required", "Email is required").expect("rule"),
        dowe_components::form_validation_rule("email", "Enter a valid email").expect("rule"),
    ];
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &ViewNode::Input { props },
    );
    let web = super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    };
    let router = super::router_js(&web);

    assert!(
        page.content
            .contains("data-dowe-validation-kind=\\\"string\\\"")
    );
    assert!(page.content.contains("Email is required"));
    assert!(page.content.contains("data-dowe-validation-feedback"));
    assert!(page.content.contains("data-dowe-validation-control"));
    assert!(router.contains("function formValidationInvalid"));
    assert!(router.contains(r#"#?&\/=]*)$/.test(text);if(rule.kind==="phone")"#));
    assert!(router.contains("aria-invalid"));
    assert!(router.contains("touchFormValidation"));
    assert_eq!(router.matches("function formDefinition").count(), 1);
    assert_eq!(router.matches("async function runSteps").count(), 1);
    assert_eq!(router.matches("function renderReactiveButtons").count(), 1);
    assert!(!router.contains("DoweDesign"));
}

#[test]
fn emits_readable_outlined_surface_controls() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Select {
        props: VariantProps {
            variant: Some(ComponentVariant::Outlined),
            color: Some(ColorFamily::Surface),
            ..Default::default()
        },
        options: vec![SelectOption {
            value: "dark".to_string(),
            label: "Dark".to_string(),
            description: None,
        }],
        option_each: None,
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.css_content.contains(
        ".control.is-outlined.is-surface{background-color:var(--dowe-surface);color:var(--dowe-surfaceText);border:1px solid rgba(127,127,127,0.36);}"
    ));
    assert!(page.css_content.contains(
        ".control.is-outlined.is-surface:focus-within{border-color:var(--dowe-surfaceText);"
    ));
}

fn advanced_form_tree() -> ViewNode {
    ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::ComboBox {
                props: ComboBoxProps {
                    style: bound_style("profile.role", "Role", "Choose role"),
                    value: Some("editor".to_string()),
                    search_placeholder: "Search roles".to_string(),
                    empty_text: "No roles".to_string(),
                    loading_text: "Loading".to_string(),
                    loading_more_text: "Loading more".to_string(),
                    clearable: true,
                    disabled: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
                options: vec![
                    ComboOption {
                        value: "admin".to_string(),
                        label: "Admin".to_string(),
                        description: Some("Full access".to_string()),
                        src: None,
                        icon: None,
                        disabled: false,
                    },
                    ComboOption {
                        value: "editor".to_string(),
                        label: "Editor".to_string(),
                        description: None,
                        src: None,
                        icon: None,
                        disabled: false,
                    },
                ],
            },
            ViewNode::CsvField {
                props: CsvFieldProps {
                    style: advanced_style("Import", None, ComponentVariant::Outlined),
                    button_text: "Upload CSV".to_string(),
                    modal_title: "Review import".to_string(),
                    instructions: "Columns are checked".to_string(),
                    cancel_text: "Cancel".to_string(),
                    confirm_text: "Import".to_string(),
                    clear_text: "Clear".to_string(),
                    preview_title: "Preview".to_string(),
                    multiple: false,
                    show_preview: true,
                    preview_rows: 3,
                    preview_page_size: 10,
                    error_text: None,
                },
                columns: vec![CsvColumn {
                    name: "email".to_string(),
                    label: Some("Email".to_string()),
                }],
            },
            ViewNode::DragDrop {
                props: DragDropProps {
                    style: advanced_style("Tasks", None, ComponentVariant::Solid),
                    empty_text: "No tasks".to_string(),
                    direction: DragDropDirection::Horizontal,
                    allow_group_transfer: true,
                    disabled: false,
                    size: ButtonSize::Md,
                },
                items: Vec::new(),
                groups: vec![DragGroup {
                    id: "todo".to_string(),
                    title: Some("Todo".to_string()),
                    items: vec![DragItem {
                        id: "draft".to_string(),
                        label: Some("Draft".to_string()),
                        description: Some("Prepare".to_string()),
                        disabled: false,
                    }],
                }],
            },
            ViewNode::Editor {
                props: EditorProps {
                    style: bound_style("profile.notes", "Notes", "Write notes"),
                    value: None,
                    min_height: 180,
                    hide_toolbar: false,
                    disabled: false,
                    readonly: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::ImageCropper {
                props: ImageCropperProps {
                    style: bound_style("profile.avatar", "Avatar", "Upload avatar"),
                    src: None,
                    alt: "Avatar".to_string(),
                    accept: "image/*".to_string(),
                    aspect_ratio: None,
                    min_width: 128,
                    min_height: 128,
                    max_width: None,
                    max_height: None,
                    shape: ImageCropperShape::Circle,
                    disabled: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::Password {
                props: PasswordProps {
                    style: bound_style("profile.password", "Password", "Create password"),
                    value: None,
                    hide_strength: false,
                    weak_label: "Weak".to_string(),
                    medium_label: "Medium".to_string(),
                    strong_label: "Strong".to_string(),
                    disabled: false,
                    readonly: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::Phone {
                props: PhoneProps {
                    style: bound_style("profile.phone", "Phone", "Phone number"),
                    value: None,
                    country: Some("US".to_string()),
                    dial_code_name: "dialCode".to_string(),
                    search_placeholder: "Search countries".to_string(),
                    empty_text: "No countries".to_string(),
                    loading_text: "Loading".to_string(),
                    priority_countries: vec!["US".to_string()],
                    disabled: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::Pin {
                props: PinProps {
                    style: bound_style("profile.pin", "Code", ""),
                    value: None,
                    length: 6,
                    kind: PinKind::Number,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::Textarea {
                props: TextareaProps {
                    style: bound_style("profile.bio", "Bio", "Short bio"),
                    value: None,
                    rows: 4,
                    cols: None,
                    max_length: Some(160),
                    resize: true,
                    disabled: false,
                    readonly: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
        ],
    }
}

fn bound_style(bind: &str, label: &str, placeholder: &str) -> VariantProps {
    let mut style = advanced_style(label, Some(placeholder), ComponentVariant::Outlined);
    style.element.bind = Some(bind.to_string());
    style.label_floating = true;
    style
}

fn advanced_style(
    label: &str,
    placeholder: Option<&str>,
    variant: ComponentVariant,
) -> VariantProps {
    VariantProps {
        label: Some(label.to_string()),
        placeholder: placeholder.map(str::to_string),
        variant: Some(variant),
        color: Some(ColorFamily::Primary),
        ..Default::default()
    }
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
                    size: Some(ButtonSize::Sm),
                    icon_start: Some(solar_control_icon("magnifier").expect("start icon")),
                    icon_end: Some(solar_control_icon("close-circle").expect("end icon")),
                    ..Default::default()
                },
            },
            ViewNode::Input {
                props: VariantProps {
                    label: Some("Email".to_string()),
                    element: ElementProps {
                        show: Some(VisibilityCondition::Signal("showEmail".to_string())),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ViewNode::Select {
                props: VariantProps {
                    label: Some("Role".to_string()),
                    placeholder: Some("Choose role".to_string()),
                    label_floating: true,
                    size: Some(ButtonSize::Lg),
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
                option_each: None,
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
    assert!(
        page.content
            .contains(r#"class=\"field\" data-dowe-show=\"showEmail\""#)
    );
    assert!(page.content.contains(r#"class=\"control is-sm"#));
    assert!(page.content.contains(r#"class=\"control is-lg"#));
    assert!(page.content.contains("has-start-adornment"));
    assert!(page.content.contains(r#"placeholder=\"Full name\""#));
    assert!(
        page.content
            .contains(r#"class=\"control-icon icon-start\""#)
    );
    assert!(page.content.contains(r#"class=\"control-icon icon-end\""#));
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
fn renders_phone_floating_label_inside_number_input_shell() {
    let tree = ViewNode::Phone {
        props: PhoneProps {
            style: bound_style("profile.phone", "Phone number", "Enter phone number"),
            value: None,
            country: Some("US".to_string()),
            dial_code_name: "dialCode".to_string(),
            search_placeholder: "Search countries".to_string(),
            empty_text: "No countries".to_string(),
            loading_text: "Loading".to_string(),
            priority_countries: vec!["US".to_string()],
            disabled: false,
            name: None,
            help_text: None,
            error_text: None,
        },
    };
    let html = render_page_body(&ViewNode::Children, &tree);
    let trigger = html
        .find(r#"class="phone-country-trigger""#)
        .expect("country trigger");
    let shell = html
        .find(r#"class="phone-input-shell""#)
        .expect("phone input shell");
    let label = html
        .find(r#"class="control-label">Phone number</span>"#)
        .expect("floating phone label");
    let input = html
        .find(r#"class="phone-input input""#)
        .expect("phone input");

    assert!(trigger < shell);
    assert!(shell < label);
    assert!(label < input);
    assert!(
        super::design_css().contains(
            ".phone-input-shell>.control-label{left:.75rem;max-width:calc(100% - 1.5rem);}"
        )
    );
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
    assert!(page.content.contains("color-accent"));
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
            .contains(r#"<path d=\"M22 12c0-5.523-4.477-10-10-10\" fill=\"currentColor\" fill-rule=\"evenodd\" clip-rule=\"evenodd\" transform=\"matrix(2 0 0 2 4 6)\"></path>"#)
    );
    assert!(page.css_content.contains(".svg"));
    assert!(
        page.css_content
            .contains(".color-accent{--dowe-content-text:var(--dowe-accent);--dowe-content-title:var(--dowe-accent);color:var(--dowe-accent);}")
    );
    assert!(page.css_content.contains(".w-8{width:2rem;}"));
    assert!(page.css_content.contains(".h-8{height:2rem;}"));
}

#[test]
fn preserves_svg_intrinsic_ratio_when_web_dimension_is_omitted() {
    let mut tree = svg_tree();
    let ViewNode::Svg { props, .. } = &mut tree else {
        panic!("svg tree");
    };
    props.style.sizing.w = None;

    let html = render_page_body(&ViewNode::Children, &tree);

    assert!(html.contains(r#"class="svg color-accent h-8""#));
    assert!(!html.contains("w-8"));
}

#[test]
fn renders_runtime_svg_as_safe_data_surface() {
    let root = Path::new("/project");
    let tree = dowe_components::svg_component_node(
        vec![ComponentProp {
            name: "data".to_string(),
            value: PropValue::String("icon.svg".to_string()),
        }],
        Vec::new(),
    )
    .expect("runtime Svg");
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &tree,
    );

    assert!(page.content.contains("data-dowe-svg-data=\\\"icon.svg\\\""));
    assert!(!page.content.contains("<path"));
}

#[test]
fn renders_dynamic_icon_binding_surface() {
    let root = Path::new("/project");
    let tree = dowe_components::icon_component_node(vec![
        ComponentProp {
            name: "name".to_string(),
            value: PropValue::String("@icon-binding:iconName".to_string()),
        },
        ComponentProp {
            name: "fill".to_string(),
            value: PropValue::String("muted".to_string()),
        },
    ])
    .expect("dynamic Icon");
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &tree,
    );

    assert!(
        page.content
            .contains("data-dowe-icon-name=\\\"iconName\\\"")
    );
    assert!(page.content.contains("color-muted"));
    assert!(page.content.contains("viewBox=\\\"0 0 24 24\\\""));
    assert!(!page.content.contains("<path"));
}

#[test]
fn renders_viewport_minus_height_classes() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Box {
        props: StyleProps {
            sizing: dowe_components::SizingProps {
                h: Some(ResponsiveValue::scalar(
                    dowe_components::SizeValue::ViewportMinus(ScaleValue::from_half_steps(32)),
                )),
                min_h: Some(ResponsiveValue::scalar(
                    dowe_components::SizeValue::ViewportMinus(ScaleValue::from_half_steps(40)),
                )),
                max_w: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                    ScaleValue::from_half_steps(128),
                ))),
                max_h: Some(ResponsiveValue::scalar(
                    dowe_components::SizeValue::ViewportMinus(ScaleValue::from_half_steps(48)),
                )),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(page.content.contains("vh-16"));
    assert!(page.content.contains("min-h-vh-20"));
    assert!(page.content.contains("max-w-64"));
    assert!(page.content.contains("max-h-vh-24"));
    assert!(
        page.css_content
            .contains(".vh-16{height:calc(100vh - 4rem);}")
    );
    assert!(
        page.css_content
            .contains(".min-h-vh-20{min-height:calc(100vh - 5rem);}")
    );
    assert!(page.css_content.contains(".max-w-64{max-width:16rem;}"));
    assert!(
        page.css_content
            .contains(".max-h-vh-24{max-height:calc(100vh - 6rem);}")
    );
}

#[test]
fn renders_percentage_width_classes() {
    let root = Path::new("/project");
    let page_tree = ViewNode::Box {
        props: StyleProps {
            sizing: dowe_components::SizingProps {
                w: Some(ResponsiveValue::scalar(SizeValue::Percent(30))),
                min_w: Some(ResponsiveValue {
                    entries: vec![
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Xs,
                            value: SizeValue::Percent(40),
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Md,
                            value: SizeValue::Percent(100),
                        },
                    ],
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![],
    };
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/index.dowe"),
        "page",
        &page_tree,
    );

    assert!(
        page.content
            .contains("w-pct-30 min-w-pct-40 md:min-w-pct-100")
    );
    assert!(page.css_content.contains(".w-pct-30{width:30%;}"));
    assert!(page.css_content.contains(".min-w-pct-40{min-width:40%;}"));
    assert!(
        page.css_content
            .contains(".md\\:min-w-pct-100{min-width:100%;}")
    );
}

#[test]
fn renders_camera_and_microphone_capture_contract() {
    let page_tree = ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::Camera {
                props: CameraProps {
                    style: VariantProps::default(),
                    facing: CameraFacing::User,
                    label: "Take photo".to_string(),
                    disabled: false,
                    on_start: Some("cameraStart".to_string()),
                    on_capture: Some("cameraCapture".to_string()),
                    on_error: Some("cameraError".to_string()),
                },
            },
            ViewNode::Microphone {
                props: MicrophoneProps {
                    style: VariantProps::default(),
                    label: "Record audio".to_string(),
                    max_duration: Some(30),
                    disabled: false,
                    on_start: Some("microphoneStart".to_string()),
                    on_stop: Some("microphoneStop".to_string()),
                    on_error: Some("microphoneError".to_string()),
                },
            },
        ],
    };
    let page = build_page_chunk(
        Path::new("/project"),
        Path::new("/project/src/pages/capture.dowe"),
        "capture",
        &page_tree,
    );
    let runtime_chunks = super::runtime_chunks_for_trees(&ViewNode::Children, &page_tree);
    assert_eq!(
        runtime_chunks
            .iter()
            .map(|chunk| chunk.name)
            .collect::<Vec<_>>(),
        vec!["media"]
    );
    let html = render_page_body(&ViewNode::Children, &page_tree);
    let router = full_runtime_for_test();

    assert!(html.contains("data-dowe-camera"));
    assert!(html.contains("data-dowe-camera-facing=\"user\""));
    assert!(html.contains("data-dowe-camera-on-capture=\"cameraCapture\""));
    assert!(html.contains("data-dowe-microphone"));
    assert!(html.contains("data-dowe-microphone-max-duration=\"30\""));
    assert!(page.css_content.contains(".camera"));
    assert!(page.css_content.contains(".microphone"));
    assert!(router.contains("function hydrateCameras(root)"));
    assert!(router.contains("navigator.mediaDevices.getUserMedia"));
    assert!(router.contains("function hydrateMicrophones(root)"));
    assert!(router.contains("new MediaRecorder(microphone.__doweMicrophoneStream)"));
    assert!(router.contains("function closeCameraFrames(view)"));
    assert!(router.contains("function closeMicrophoneFrames(view)"));
}
