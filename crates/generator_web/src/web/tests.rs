use super::{
    ChunkKind, build_layout_chunk, build_page_chunk, build_translation_chunks, render_page_body,
    svg_path_attributes, web_artifacts,
};
use dowe_components::{
    AccordionItem, AccordionProps, AlertDialogProps, AudioProps, AvatarGroupItem, AvatarGroupProps,
    AvatarProps, AvatarStatus, BadgeProps, BannerProps, BarPosition, BarProps, BottomBarTab,
    BorderWidth, BoxPosition, BrandProps, Breakpoint, ButtonSize, CameraFacing, CameraProps,
    CarouselIndicatorType, CarouselOrientation, CarouselProps, CarouselSlide, CarouselVariant,
    ChatBoxMode, ChatBoxProps, CheckboxProps, ChipProps, CollapsibleProps, ColorFamily, ColorProps,
    ColorToken, ComboBoxProps, ComboOption, CommandEntry, CommandProps, ComponentProp,
    ComponentVariant, CountdownProps, CountdownSize, CoverSource, CsvColumn, CsvFieldProps,
    DateProps, DateRangeProps, DesignConfig, DividerOrientation, DividerProps, DragDropDirection,
    DragDropProps, DragGroup, DragItem, DrawerPosition, DrawerProps, DropdownProps, EditorProps,
    ElementProps, EmptyKind, EmptyProps, FabAction, FabProps, FontConfig, GapSize, GapValue,
    GridAlignment, GridProps, GridSpan, GridTracks, ImageAspect, ImageCropperProps,
    ImageCropperShape, ImageLoading, ImageObjectFit, ImageProps, MapMarker, MapMarkerIcon,
    MapProps, MapWaypoint, MarqueeOrientation, MarqueeProps, MarqueeSpeed, MicrophoneProps,
    ModalProps, NavMenuItem, NavMenuItemProps, NavMenuProps, NavigationAction, NavigationOperation,
    OverlayCornerPosition, OverlayEntry, OverlayItemProps, OverlayPaint, OverlayPosition,
    PasswordProps, PhoneProps, PinKind, PinProps, PropValue, RadioGroupOrientation,
    RadioGroupProps, RadioOption, RailNavItem, RailNavItemProps, RailNavProps,
    ReactiveVariantProps, RecordProps, ResponsiveEntry, ResponsiveValue, RichTextMark,
    RichTextMarkStyle, RoundedSize, ScaffoldProps, ScaleValue, SectionBackground, SelectOption,
    SelectOptionEach, SideNavIcon, SideNavItem, SideNavItemProps, SideNavProps, SideNavSize,
    SidebarProps, SizeValue, SkeletonAnimation, SkeletonProps, SkeletonVariant, StyleExtras,
    StyleProps, SvgLineCap, SvgLineJoin, SvgPath, SvgPathFill, SvgProps, SvgTransform, SvgViewBox,
    TabItem, TabsPosition, TabsProps, TabsVariant, TextAlign, TextProps, TextWeight, TextareaProps,
    ToastKind, ToastProps, ToggleGroupItem, ToggleGroupKind, ToggleGroupProps, ToggleProps,
    TooltipProps, TranslationCatalog, TranslationLocale, TranslationValue, TypeWriterItem,
    TypeWriterProps, VariantProps, VideoAspect, VideoProps, ViewAction, ViewActionKind,
    ViewAnimation, ViewAssignAction, ViewConstant, ViewFunctionStatement, ViewGesture, ViewIcon,
    ViewMotionStyle, ViewNode, ViewResetAction, ViewRotation, ViewScale, ViewSignal,
    ViewSignalValue, ViewToastAction, ViewTransition, ViewTranslation, VisibilityCondition,
    icon_component_node, solar_control_icon, svg_spinner_control_icon,
};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;

fn full_runtime_for_test() -> String {
    [
        super::router_js(&super::WebOutput {
            chunks: Vec::new(),
            pages: Vec::new(),
            translation_chunks: Vec::new(),
            default_locale: None,
            router_js: String::new(),
        }),
        super::controls_runtime_chunk().content,
        super::media_runtime_chunk().content,
        super::visualization_runtime_chunk().content,
    ]
    .join("\n")
}

fn assert_javascript_syntax(source: &str) {
    let Ok(mut child) = Command::new("node")
        .args(["--input-type=module", "--check"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
    else {
        return;
    };
    child
        .stdin
        .take()
        .expect("node stdin")
        .write_all(source.as_bytes())
        .expect("write javascript");
    let output = child.wait_with_output().expect("javascript syntax check");
    assert!(
        output.status.success(),
        "invalid generated javascript: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn generated_runtime_javascript_has_valid_syntax() {
    let web = super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    };
    let core = super::router_js(&web);
    let controls = super::controls_runtime_chunk().content;
    let media = super::media_runtime_chunk().content;
    let visualization = super::visualization_runtime_chunk().content;
    assert_javascript_syntax(&core);
    assert_javascript_syntax(&controls);
    assert_javascript_syntax(&media);
    assert_javascript_syntax(&visualization);
    assert!(core.len() <= 133_000, "router core is {} bytes", core.len());
    assert!(controls.len() <= 50_000);
    assert!(media.len() <= 40_000);
    assert!(visualization.len() <= 35_000);
    assert_eq!(
        core.matches("window.addEventListener(\"resize\"").count(),
        1
    );
    assert_eq!(
        core.matches("window.addEventListener(\"scroll\"").count(),
        1
    );
    assert!(controls.contains("onViewportResize"));
    assert!(controls.contains("onViewportScroll"));
}

#[test]
fn dynamic_icon_router_embeds_only_static_constant_names() {
    let icon = icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String("@icon-binding:platform.icon".to_string()),
    }])
    .expect("dynamic icon");
    let tree = ViewNode::Scope {
        constants: vec![ViewConstant {
            id: "platforms".to_string(),
            name: "platforms".to_string(),
            value: ViewSignalValue::Array(vec![
                ViewSignalValue::Object(vec![(
                    "icon".to_string(),
                    ViewSignalValue::String("route-bold-duotone".to_string()),
                )]),
                ViewSignalValue::Object(vec![(
                    "icon".to_string(),
                    ViewSignalValue::String("svg-logos:apple".to_string()),
                )]),
            ]),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![
            ViewNode::Each {
                item: "platform".to_string(),
                collection: "platforms".to_string(),
                key: "platform.icon".to_string(),
                children: vec![icon],
            },
            ViewNode::Each {
                item: "entry".to_string(),
                collection: "runtimeEntries".to_string(),
                key: "entry.id".to_string(),
                children: vec![ViewNode::Text {
                    props: Default::default(),
                    value: "Catalog entry".to_string(),
                }],
            },
        ],
    };
    let page = super::ViewPage {
        id: "platforms".to_string(),
        route_path: "/".to_string(),
        source_path: PathBuf::from("/project/views/pages/home.dowe"),
        layout_tree: ViewNode::Children,
        page_tree: tree,
        body_html: String::new(),
        html_document: String::new(),
        layout_text: String::new(),
        page_text: String::new(),
        layout_chunk_id: String::new(),
        page_chunk_id: String::new(),
        layout_chunk_ids: Vec::new(),
        js_chunks: Vec::new(),
        css_chunks: Vec::new(),
        runtime_chunks: Vec::new(),
        design_file_name: "design.css".to_string(),
        router_file_name: String::new(),
        boundaries: Vec::new(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
        metadata: Vec::new(),
    };
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: vec![Arc::new(page)],
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
    });

    assert!(router.contains("\"route-bold-duotone\""));
    assert!(router.contains("\"svg-logos:apple\""));
    assert!(!router.contains("\"country-flags:CO\""));
}

#[test]
fn basic_view_css_excludes_unused_component_domains() {
    let tree = ViewNode::Box {
        props: Default::default(),
        children: vec![ViewNode::Text {
            props: Default::default(),
            value: "Basic".to_string(),
        }],
    };
    let css =
        super::design_css_for_trees([&tree], &FontConfig::default(), &DesignConfig::default());

    assert!(css.contains(".box{"));
    assert!(!css.contains(".color-picker-popover{"));
    assert!(!css.contains(".camera-preview{"));
    assert!(!css.contains(".arc-chart-container{"));
    assert!(!css.contains(".drawer-panel{"));
    assert!(css.len() < 50_000);
}

#[test]
fn global_toast_actions_include_overlay_css() {
    let tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: Vec::new(),
        actions: vec![ViewAction {
            id: "save".to_string(),
            name: "save".to_string(),
            params: Vec::new(),
            return_type: None,
            kind: ViewActionKind::Sequence(vec![ViewFunctionStatement::Toast(ViewToastAction {
                kind: "info".to_string(),
                title: String::new(),
                message: "Saved".to_string(),
                duration: None,
                scheme: None,
                variant: None,
                position: None,
            })]),
        }],
        children: Vec::new(),
    };
    let css =
        super::design_css_for_trees([&tree], &FontConfig::default(), &DesignConfig::default());

    assert!(css.contains(".toast{"));
    assert!(css.contains(".toast.is-bottom-right{"));
}

#[test]
fn global_toast_actions_include_media_runtime() {
    let tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: Vec::new(),
        actions: vec![ViewAction {
            id: "save".to_string(),
            name: "save".to_string(),
            params: Vec::new(),
            return_type: None,
            kind: ViewActionKind::Sequence(vec![ViewFunctionStatement::Toast(ViewToastAction {
                kind: "info".to_string(),
                title: String::new(),
                message: "Saved".to_string(),
                duration: None,
                scheme: None,
                variant: None,
                position: None,
            })]),
        }],
        children: Vec::new(),
    };
    let chunks = super::runtime_chunks_for_trees(&tree, &ViewNode::Children);

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].name, "media");
    assert!(chunks[0].content.contains("closeToast"));
}

#[test]
fn view_css_includes_each_used_component_domain_once() {
    let roots = [
        media_display_form_tree(),
        charts_tree(),
        display_overlay_tree(),
        navigation_shell_tree(),
        code_tree(),
    ];
    let css = super::design_css_for_trees(
        roots.iter(),
        &FontConfig::default(),
        &DesignConfig::default(),
    );

    for selector in [
        ".color-picker-popover{",
        ".camera-preview{",
        ".canvas{",
        ".drawer-panel{",
        ".rich-text{",
        ".carousel-viewport{",
        ".avatar-group{",
        ".navmenu{",
    ] {
        assert!(css.contains(selector), "missing selector {selector}");
    }
}

#[test]
fn design_asset_name_is_content_addressed_and_deterministic() {
    let first = super::design_css_file_name(".box{display:block;}");
    let second = super::design_css_file_name(".box{display:block;}");
    let changed = super::design_css_file_name(".box{display:flex;}");

    assert_eq!(first, second);
    assert_ne!(first, changed);
    assert!(first.starts_with("design-"));
    assert!(first.ends_with(".css"));
}

#[test]
fn reactive_variant_and_style_capability_are_selected_for_bound_props() {
    let tree = ViewNode::Card {
        props: VariantProps { style: StyleProps { bg_binding: Some(dowe_components::PropBinding::string("theme.color")), ..Default::default() }, variant_binding: Some(dowe_components::PropBinding::string("theme.variant")), ..Default::default() },
        children: Vec::new(),
    };
    let chunks = super::runtime_chunks_for_trees(&ViewNode::Children, &tree);
    assert_eq!(chunks.iter().map(|chunk| chunk.name).collect::<Vec<_>>(), vec!["styles"]);
    assert!(chunks[0].content.contains("renderStyles"));
}

#[test]
fn style_capability_chunks_are_selected_and_content_addressed() {
    let basic = ViewNode::Box {
        props: Default::default(),
        children: vec![ViewNode::Text {
            props: Default::default(),
            value: "Basic".to_string(),
        }],
    };
    let basic_features = super::DesignCssFeatures::collect([&basic]);
    assert!(super::design_css_chunks(basic_features).is_empty());

    let roots = [media_display_form_tree(), charts_tree()];
    let features = super::DesignCssFeatures::collect(roots.iter());
    let chunks = super::design_css_chunks(features);
    let names = chunks.iter().map(|chunk| chunk.name).collect::<Vec<_>>();

    assert_eq!(names, ["forms", "media", "visualization", "disclosure"]);
    for chunk in chunks {
        let browser_path = chunk.browser_path();
        assert!(browser_path.starts_with(&format!("chunks/design/{}-", chunk.name)));
        assert!(browser_path.ends_with(".css"));
        assert!(!chunk.content.contains('\n'));
    }
}

include!("tests/core_generation.rs");
include!("tests/data_generation.rs");
include!("tests/navigation_generation.rs");
include!("tests/component_display_generation.rs");
include!("tests/fixtures_core.rs");
include!("tests/fixtures_media_forms.rs");
include!("tests/fixtures_navigation.rs");
include!("tests/fixtures_data.rs");
include!("tests/fixtures_display_overlay.rs");
include!("tests/fixtures_display_chat.rs");
include!("tests/fixtures_rich_controls.rs");
include!("tests/stdlib_generation.rs");
