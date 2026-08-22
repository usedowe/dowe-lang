const CONTROLS_RUNTIME_MODULES: &[&str] = &[
    include_str!("router_runtime/color_picker.js"),
    include_str!("router_runtime/forms_1.js"),
    include_str!("router_runtime/forms_2.js"),
    include_str!("router_runtime/forms_3.js"),
];

const MEDIA_RUNTIME_MODULES: &[&str] = &[
    include_str!("router_runtime/overlays_media_1.js"),
    include_str!("router_runtime/overlays_media_2.js"),
    include_str!("router_runtime/capture_media.js"),
    include_str!("router_runtime/video_media.js"),
];

const VISUALIZATION_RUNTIME_MODULES: &[&str] = &[
    include_str!("router_runtime/visualization_1.js"),
    include_str!("router_runtime/visualization_2.js"),
    include_str!("router_runtime/visualization_3.js"),
    include_str!("router_runtime/visualization_4.js"),
];

const CONTROLS_RUNTIME_EXPORTS: &[&str] = &[
    "renderDoweColors",
    "cropperState",
    "cropperDraw",
    "cropperReset",
    "cropperOpen",
    "cropperCancel",
    "cropperApply",
    "cropperRemove",
    "cropperMove",
    "handleCropperFile",
    "selectOptions",
    "closeSelect",
    "closeSelects",
    "positionSelect",
    "openSelect",
    "renderSelect",
    "renderSelects",
    "dateRootFromTarget",
    "closeDatePicker",
    "closeDatePickers",
    "positionDatePicker",
    "openDatePicker",
    "renderDateFields",
    "changeDateMonth",
    "selectDateValue",
    "comboHost",
    "closeCombos",
    "positionCombo",
    "renderCombo",
    "renderCombos",
    "filterCombo",
    "renderPasswordStrength",
    "filterPhone",
    "sanitizePhoneInput",
    "closePhones",
    "positionPhone",
    "setPhoneCountry",
    "updatePin",
    "hydrateAdvancedForms",
    "handleCsvFile",
];

const MEDIA_RUNTIME_EXPORTS: &[&str] = &[
    "renderCarouselEffects",
    "renderCarousel",
    "goToCarousel",
    "moveCarousel",
    "hydrateCarousels",
    "downloadImage",
    "toggleImageFullscreen",
    "renderDrawers",
    "closeDrawer",
    "closeDrawers",
    "renderModals",
    "closeModal",
    "closeModals",
    "closeDropdowns",
    "positionDropdown",
    "openDropdown",
    "tooltipPosition",
    "closeTooltips",
    "renderToasts",
    "closeToast",
    "openCommand",
    "closeCommand",
    "filterCommand",
    "setActiveTab",
    "moveActiveTab",
    "edgeActiveTab",
    "audioTime",
    "updateAudio",
    "hydrateAudios",
    "toggleAccordion",
    "closeCameraFrames",
    "closeMicrophoneFrames",
    "hydrateCameras",
    "hydrateMicrophones",
    "hydrateVideos",
];

pub fn runtime_chunks_for_page(page: &ViewPage) -> Vec<GeneratedRuntimeChunk> {
    runtime_chunks_for_trees(&page.layout_tree, &page.page_tree)
}

pub fn runtime_chunks_for_trees(
    layout_tree: &ViewNode,
    page_tree: &ViewNode,
) -> Vec<GeneratedRuntimeChunk> {
    let features = runtime_features([layout_tree, page_tree]);
    let mut chunks = Vec::new();
    if features.controls {
        chunks.push(controls_runtime_chunk());
    }
    if features.media {
        chunks.push(media_runtime_chunk());
    }
    if features.visualization {
        chunks.push(visualization_runtime_chunk());
    }
    chunks
}

fn controls_runtime_chunk() -> GeneratedRuntimeChunk {
    static CHUNK: std::sync::OnceLock<GeneratedRuntimeChunk> = std::sync::OnceLock::new();
    CHUNK
        .get_or_init(|| {
            capability_runtime_chunk(
                "controls",
                CONTROLS_RUNTIME_MODULES,
                CONTROLS_RUNTIME_EXPORTS,
            )
        })
        .clone()
}

fn media_runtime_chunk() -> GeneratedRuntimeChunk {
    static CHUNK: std::sync::OnceLock<GeneratedRuntimeChunk> = std::sync::OnceLock::new();
    CHUNK
        .get_or_init(|| {
            capability_runtime_chunk("media", MEDIA_RUNTIME_MODULES, MEDIA_RUNTIME_EXPORTS)
        })
        .clone()
}

fn capability_runtime_chunk(
    name: &'static str,
    modules: &[&str],
    exported_functions: &[&str],
) -> GeneratedRuntimeChunk {
    let mut source = format!(
        r#"window.__doweRegisterRuntimeCapability("{name}",api=>{{const{{readPath,writePath,runAction,scopeFor,renderReactive,touchFormValidation,onViewportResize,onViewportScroll}}=api;let activeView=null;"#,
    );
    source.reserve(modules.iter().map(|module| module.len()).sum());
    for module in modules {
        source.push_str(module);
        source.push('\n');
    }
    source.push_str("return{setActiveView(view){activeView=view},");
    source.push_str(&exported_functions.join(","));
    source.push_str("};});");
    GeneratedRuntimeChunk::new(name, minify_js(&source))
}

fn visualization_runtime_chunk() -> GeneratedRuntimeChunk {
    static CHUNK: std::sync::OnceLock<GeneratedRuntimeChunk> = std::sync::OnceLock::new();
    CHUNK
        .get_or_init(|| {
            let mut source = String::from(
                r#"window.__doweRegisterRuntimeCapability("visualization",api=>{const{readPath,writePath,runAction,scopeFor,renderReactive,getActiveView,prefersReducedMotion}=api;"#,
            );
            source.reserve(
                VISUALIZATION_RUNTIME_MODULES
                    .iter()
                    .map(|module| module.len())
                    .sum(),
            );
            for module in VISUALIZATION_RUNTIME_MODULES {
                source.push_str(module);
                source.push('\n');
            }
            source.push_str(
                "return{renderCharts,renderCanvases,renderCandlesticks,closeCandlestickStreams,closeCanvasFrames,hydrateCanvases,hydrateCandlesticks};});",
            );
            GeneratedRuntimeChunk::new("visualization", minify_js(&source))
        })
        .clone()
}

#[derive(Default)]
struct RuntimeFeatures {
    controls: bool,
    media: bool,
    visualization: bool,
}

fn runtime_features<'a>(roots: impl IntoIterator<Item = &'a ViewNode>) -> RuntimeFeatures {
    let mut features = RuntimeFeatures::default();
    let mut pending = roots.into_iter().collect::<Vec<_>>();
    while let Some(node) = pending.pop() {
        features.controls |= node_uses_controls(node);
        features.media |= node_uses_media(node);
        features.media |= node_actions_use_media(node);
        features.visualization |= node_uses_visualization(node);
        for group in dowe_components::node_child_groups(node) {
            pending.extend(group);
        }
    }
    features
}

fn node_actions_use_media(tree: &ViewNode) -> bool {
    match tree {
        ViewNode::Scope { actions, .. } => actions.iter().any(action_uses_media),
        _ => false,
    }
}

fn action_uses_media(action: &ViewAction) -> bool {
    match &action.kind {
        ViewActionKind::Sequence(statements) => statements_use_media(statements),
        ViewActionKind::Request(_) | ViewActionKind::Assign(_) | ViewActionKind::Reset(_) => false,
    }
}

fn statements_use_media(statements: &[ViewFunctionStatement]) -> bool {
    statements.iter().any(|statement| match statement {
        ViewFunctionStatement::Toast(_) => true,
        ViewFunctionStatement::If { success, error, .. } => {
            statements_use_media(success) || statements_use_media(error)
        }
        ViewFunctionStatement::Validate { .. }
        | ViewFunctionStatement::Request { .. }
        | ViewFunctionStatement::Assign(_)
        | ViewFunctionStatement::Reset(_)
        | ViewFunctionStatement::Redirect { .. } => false,
    })
}

fn node_uses_visualization(tree: &ViewNode) -> bool {
    matches!(
        tree,
        ViewNode::Canvas { .. }
            | ViewNode::Candlestick { .. }
            | ViewNode::ArcChart { .. }
            | ViewNode::AreaChart { .. }
            | ViewNode::BarChart { .. }
            | ViewNode::LineChart { .. }
            | ViewNode::PieChart { .. }
    )
}

fn node_uses_controls(tree: &ViewNode) -> bool {
    matches!(
        tree,
        ViewNode::SelectTheme { .. }
            | ViewNode::Select { .. }
            | ViewNode::ComboBox { .. }
            | ViewNode::CsvField { .. }
            | ViewNode::Editor { .. }
            | ViewNode::ImageCropper { .. }
            | ViewNode::Password { .. }
            | ViewNode::Phone { .. }
            | ViewNode::Pin { .. }
            | ViewNode::Color { .. }
            | ViewNode::Date { .. }
            | ViewNode::DateRange { .. }
    )
}

fn node_uses_media(tree: &ViewNode) -> bool {
    match tree {
        ViewNode::Image { props } => !props.hide_controls,
        ViewNode::Audio { .. }
        | ViewNode::Camera { .. }
        | ViewNode::Microphone { .. }
        | ViewNode::Video { .. }
        | ViewNode::Tabs { .. }
        | ViewNode::Drawer { .. }
        | ViewNode::Modal { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Tooltip { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Dropdown { .. }
        | ViewNode::Command { .. }
        | ViewNode::Accordion { .. }
        | ViewNode::Carousel { .. } => true,
        _ => false,
    }
}
