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

    assert!(html.contains(r#"class="avatar is-solid is-success avatar-lg is-bordered""#));
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
    assert!(page.css_content.contains(".avatar.is-solid.is-success"));
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
    assert!(
        alert_page
            .css_content
            .contains(".modal.is-solid.is-surface")
    );
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
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
    });

    assert!(html.contains(r#"data-dowe-button-loading hidden aria-hidden="true""#));
    assert!(html.contains("data-dowe-button-content"));
    assert!(html.contains("is-svg-spinner"));
    assert!(html.contains(r#"data-dowe-button-loading="saving""#));
    assert!(css.contains(".button.is-loading>[data-dowe-button-content]"));
    assert!(css.contains(".button>[data-dowe-button-content]{display:inline-flex;"));
    assert!(router.contains("doweButtonLoading"));
    assert!(router.contains("aria-busy"));
}

