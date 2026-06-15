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
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });

    assert!(html.contains(r#"class="avatar is-soft is-success avatar-lg is-bordered""#));
    assert!(html.contains(r#"class="badge is-solid is-danger is-bottom-right""#));
    assert!(html.contains(r#"class="chip is-outlined is-info chip-sm has-close""#));
    assert!(html.contains(r#"class="skeleton"#));
    assert!(html.contains("is-pulse"));
    assert!(html.contains("is-rounded"));
    assert!(html.contains(r#"data-dowe-modal data-dowe-modal-open="modal01""#));
    assert!(html.contains(r#"class="alert-dialog-actions""#));
    assert!(html.contains(r#"class="tooltip-popover is-solid is-muted position-end""#));
    assert!(html.contains(r#"class="toast"#));
    assert!(html.contains("is-solid"));
    assert!(html.contains("is-success"));
    assert!(html.contains(r#"class="dropdown-popover is-solid is-surface""#));
    assert!(html.contains(r#"data-dowe-command-open="modal01""#));
    assert!(page.css_content.contains(".avatar.is-soft.is-success"));
    assert!(page.css_content.contains(".modal.is-solid.is-surface"));
    assert!(
        page.css_content
            .contains(".dropdown-popover.is-solid.is-surface")
    );
    assert!(css.contains(".tooltip-popover{position:fixed;"));
    assert!(css.contains("@keyframes dowe-skeleton-pulse"));
    assert!(router.contains("function renderModals(root,state,scope)"));
    assert!(router.contains("function renderToasts(root,state,scope)"));
    assert!(router.contains("function openCommand(command)"));
    assert!(router.contains("data-dowe-dropdown-trigger"));
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
    assert!(html.contains(r#"class="marquee"#));
    assert!(html.contains("is-horizontal"));
    assert!(html.contains("is-fast"));
    assert!(html.contains("pause-on-hover"));
    assert!(html.contains("is-reverse"));
    assert!(html.contains("has-fade"));
    assert!(html.contains(r#"class="typewriter""#));
    assert!(page.css_content.contains(".avatar-group"));
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
    assert!(html.contains("rich-mark-grad"));
    assert!(html.contains(r#"data-dowe-record"#));
    assert!(html.contains(r#"data-dowe-toggle-group"#));
    assert!(html.contains(r#"data-dowe-collapsible"#));
    assert!(html.contains(r#"data-dowe-countdown-target="2030-01-01T00:00:00Z""#));
    assert!(html.contains(r#"data-dowe-map"#));
    assert!(html.contains(r#"data-dowe-map-marker="office""#));
    assert!(page.css_content.contains(".media.is-soft.is-primary"));
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
    assert!(css.contains(".record-wave"));
    assert!(css.contains(".toggle-group-item"));
    assert!(css.contains(".collapsible-content"));
    assert!(css.contains(".countdown-box"));
    assert!(css.contains(".map-grid"));
    assert!(router.contains("function hydrateRecords(root)"));
    assert!(router.contains("function renderToggleGroups(root,state,scope)"));
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
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });

    assert!(html.contains(r#"class="media is-soft is-primary""#));
    assert!(html.contains(r#"data-dowe-audio"#));
    assert!(html.contains(r#"class="image is-solid is-secondary square fit-contain""#));
    assert!(html.contains(r#"data-dowe-image"#));
    assert!(html.contains(r#"data-dowe-accordion data-dowe-accordion-multiple="true""#));
    assert!(html.contains(r#"data-dowe-carousel data-dowe-carousel-index="0""#));
    assert!(html.contains(r#"class="checkbox-input is-success""#));
    assert!(html.contains(r#"data-dowe-bind="accepted""#));
    assert!(html.contains(r#"class="color-input" type="color""#));
    assert!(html.contains(r#"class="date-input" type="date""#));
    assert!(html.contains(r#"class="radio is-muted is-lg""#));
    assert!(html.contains(r#"class="toggle-input is-secondary""#));

    assert!(page.css_content.contains(".media.is-soft.is-primary"));
    assert!(
        page.css_content
            .contains(".accordion.is-outlined.is-surface")
    );
    assert!(page.css_content.contains(".carousel.is-solid.is-info"));
    assert!(css.contains(".checkbox-input{position:relative;"));
    assert!(css.contains(".toggle-input{position:relative;"));
    assert!(css.contains(".date-range-inputs{display:flex;"));
    assert!(router.contains("function hydrateAudios(root)"));
    assert!(router.contains("function toggleAccordion(trigger)"));
    assert!(router.contains("function renderCarousel(root)"));
    assert!(router.contains("function downloadImage(root)"));
}

#[test]
fn renders_advanced_form_components_markup_runtime_and_css() {
    let root = Path::new("/project");
    let page_tree = advanced_form_tree();
    let page = build_page_chunk(
        root,
        Path::new("/project/src/pages/advanced.dowe"),
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

    assert!(html.contains(r#"class="combo-box"#));
    assert!(html.contains(r#"data-dowe-combo-box"#));
    assert!(html.contains(r#"data-dowe-bind="profile.role""#));
    assert!(html.contains(r#"data-dowe-combo-value="admin""#));
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
    assert!(html.contains(r#"class="password-field"#));
    assert!(html.contains(r#"data-dowe-password-input"#));
    assert!(html.contains(r#"class="phone-field"#));
    assert!(html.contains(r#"data-dowe-phone-field"#));
    assert!(html.contains(r#"class="pin-field"#));
    assert!(html.contains(r#"data-dowe-pin-field"#));
    assert!(html.contains(r#"inputmode="numeric""#));
    assert!(html.contains(r#"class="textarea"#));
    assert!(html.contains(r#"maxlength="160""#));
    assert!(
        page.css_content
            .contains(".control.is-outlined.is-primary")
    );
    assert!(page.css_content.contains(".button.is-outlined.is-primary"));
    assert!(page.css_content.contains(".drag-drop.is-soft.is-primary"));
    assert!(css.contains(".combo-box-options"));
    assert!(css.contains(".csv-field-modal"));
    assert!(css.contains(".drag-drop-item"));
    assert!(css.contains(".editor-toolbar"));
    assert!(css.contains(".password-strength"));
    assert!(router.contains("function hydrateAdvancedForms(root)"));
    assert!(router.contains("function filterCombo(root)"));
    assert!(router.contains("function handleCsvFile(input)"));
    assert!(router.contains("function renderPasswordStrength(input)"));
    assert!(router.contains("function updatePin(root"));
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
            ".control{--dowe-component-display:flex;position:relative;display:var(--dowe-show,var(--dowe-component-display));align-items:center;width:100%;min-height:2.5rem;"
        ));
    assert!(css.contains("min-height:2.5rem;padding:0 0.75rem;"));
    assert!(css.contains(".field{display:flex;flex-direction:column;"));
    assert!(css.contains(".select-popover{position:fixed;"));
    assert!(
        css.contains("transition:opacity 160ms ease,transform 160ms ease,visibility 160ms ease;")
    );
    assert!(css.contains(".select-popover.is-active{opacity:1;visibility:visible;pointer-events:auto;transform:translateY(0) scale(1);"));
    assert!(css.contains(".select-arrow{width:1em;height:1em;"));
    assert!(css.contains(".alert{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));align-items:center;justify-content:space-between;gap:0.75rem;padding:0.625rem 0.875rem;border-radius:var(--dowe-radiusUi);}"));
    assert!(css.contains(
            ".select-control.is-floating:not(.is-open):not(.has-value) .select-value{visibility:hidden;}"
        ));
    assert!(css.contains("font-size:clamp(0.875rem, 0.82rem + 0.25vw, 1rem)"));
    assert!(css.contains(".grid>[data-dowe-each],.flex>[data-dowe-each]{display:contents;}"));
    assert!(page.css_content.contains(
            ".control.is-outlined.is-secondary{background-color:var(--dowe-background);color:var(--dowe-secondary);border:1px solid rgba(127,127,127,0.36);}"
        ));
    assert!(page.css_content.contains(
        ".control.is-outlined.is-secondary:focus-within{border-color:var(--dowe-secondary);"
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
                    style: advanced_style("Tasks", None, ComponentVariant::Soft),
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
            ViewNode::PasswordField {
                props: PasswordFieldProps {
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
            ViewNode::PhoneField {
                props: PhoneFieldProps {
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
            ViewNode::PinField {
                props: PinFieldProps {
                    style: bound_style("profile.pin", "Code", ""),
                    value: None,
                    length: 6,
                    kind: PinFieldKind::Number,
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
                    ..Default::default()
                },
            },
            ViewNode::Select {
                props: VariantProps {
                    label: Some("Role".to_string()),
                    placeholder: Some("Choose role".to_string()),
                    label_floating: true,
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
    assert!(page.content.contains(r#"placeholder=\"Full name\""#));
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
    assert!(page.content.contains("color-tertiary"));
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
            .contains(r#"<path d=\"M22 12c0-5.523-4.477-10-10-10\" fill=\"currentColor\"></path>"#)
    );
    assert!(page.css_content.contains(".svg"));
    assert!(
        page.css_content
            .contains(".color-tertiary{color:var(--dowe-tertiary);}")
    );
    assert!(page.css_content.contains(".w-8{width:2rem;}"));
    assert!(page.css_content.contains(".h-8{height:2rem;}"));
}
