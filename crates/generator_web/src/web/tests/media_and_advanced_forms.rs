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

    assert!(html.contains(r#"class="media is-solid is-primary""#));
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

    let card = ViewNode::RadioGroup {
        props: RadioGroupProps {
            style: VariantProps {
                variant: Some(ComponentVariant::Outlined),
                color: Some(ColorFamily::Primary),
                element: ElementProps {
                    bind: Some("workspace".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            size: ButtonSize::Md,
            orientation: RadioGroupOrientation::Horizontal,
            presentation: RadioGroupPresentation::Card,
            name: Some("workspace".to_string()),
            info: None,
            error: None,
        },
        options: vec![RadioOption {
            value: "local".to_string(),
            label: "Local".to_string(),
            description: Some("Edit, run, and test files on your computer".to_string()),
            icon: Some(solar_control_icon("laptop").expect("laptop icon")),
            disabled: false,
        }],
    };
    let card_html = render_page_body(&ViewNode::Children, &card);
    assert!(card_html.contains(r#"class="radio-card-group is-outlined is-horizontal""#));
    assert!(card_html.contains(r#"class="radio-card is-outlined is-primary is-md""#));
    assert!(card_html.contains(r#"class="radio-card-control""#));
    assert!(card_html.contains(r#"class="radio-card-icon""#));
    assert!(card_html.contains("Edit, run, and test files on your computer"));
    assert!(css.contains(".radio-card-group"));
    assert!(css.contains(".radio-card-control:checked"));
    assert!(css.contains(".radio-card-indicator"));

    assert!(page.css_content.contains(".media.is-solid.is-primary"));
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
    assert!(html.contains(r#"data-dowe-editor-language="dowe""#));
    assert!(html.contains(r#"contenteditable="true""#));
    assert!(html.contains(r#"data-dowe-editor-save="saveEditor""#));
    assert!(html.contains("editor-highlight"));
    assert!(html.contains("image-cropper"));
    assert!(html.contains("is-circle"));
    assert!(html.contains(r#"data-dowe-image-cropper"#));
    assert!(html.contains(r#"data-dowe-bind="profile.avatar""#));
    assert!(html.contains(r#"data-dowe-cropper-change hidden"#));
    assert!(html.contains(r#"image-cropper-empty-icon"#));
    assert!(html.contains(r#"class="image-cropper-label""#));
    assert!(html.contains(r#"data-dowe-cropper-stage"#));
    assert!(html.contains(r#"data-dowe-cropper-zoom"#));
    assert!(html.contains("Apply"));
    assert!(router.contains("dragDropInsertionTarget"));
    assert!(router.contains("doweDirection"));
    assert!(router.contains("is-drag-over"));
    assert!(router.contains("doweAllowGroupTransfer"));
    assert!(router.contains("clientX"));
    assert!(router.contains("cropperApply"));
    assert!(router.contains("hasBinding"));
    assert!(router.contains("!!bound&&!!state"));
    assert!(router.contains("function cropperCommit"));
    assert!(router.contains("function cropperClampOffset"));
    assert!(router.contains("current.editing&&value===appliedValue"));
    assert!(router.contains("change.hidden=!hasValue"));
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
    assert!(page.css_content.contains(".drag-drop.is-solid.is-primary"));
    assert!(css.contains(".combo-box-options"));
    assert!(css.contains(".combo-box-popover{position:fixed;"));
    assert!(css.contains(".combo-box-option:disabled"));
    assert!(css.contains(".password-toggle .svg{width:1.25rem;height:1.25rem;}"));
    assert!(css.contains(".csv-field-modal"));
    assert!(css.contains(".drag-drop-item"));
    assert!(css.contains(".drag-drop-list.is-drag-over"));
    assert!(css.contains(".pin-cell.control"));
    assert!(css.contains(".editor-toolbar"));
    assert!(css.contains(".editor-highlight .code-token-keyword"));
    assert!(css.contains(".editor-highlight .code-token-string"));
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
    assert!(router.contains("function highlightEditor(root)"));
    assert!(router.contains("runtimeCall(\"controls\",\"highlightEditor\""));
    assert!(router.contains("doweEditorSave"));
    assert!(router.contains("target.innerText"));
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

