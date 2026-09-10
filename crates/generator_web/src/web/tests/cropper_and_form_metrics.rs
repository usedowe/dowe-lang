#[test]
fn image_cropper_keeps_empty_preview_icon_when_initial_value_exists() {
    let mut page_tree = advanced_form_tree();
    let ViewNode::Box { children, .. } = &mut page_tree else {
        panic!("advanced form root");
    };
    let Some(ViewNode::ImageCropper { props }) = children
        .iter_mut()
        .find(|node| matches!(node, ViewNode::ImageCropper { .. }))
    else {
        panic!("image cropper");
    };
    props.src = Some("data:image/png;base64,AAAA".to_string());

    let html = render_page_body(&ViewNode::Children, &page_tree);

    assert!(html.contains(r#"<svg hidden class="image-cropper-empty-icon""#));
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
        render_report: dowe_components::RenderReport::new(
            dowe_components::RenderTarget::Web,
            Vec::new(),
        ),
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

