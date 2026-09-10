use dowe_components::{
    AccordionItem, AccordionProps, AlertDialogProps, Align, ArcChartProps, AudioProps,
    AvatarGroupItem, AvatarGroupProps, AvatarProps, AvatarSize, BadgeProps, BarPosition, BarProps,
    BorderWidth, BottomBarTab, BoxPosition, Breakpoint, ButtonSize, CameraProps, CanvasBackground,
    CarouselOrientation, CarouselProps, CarouselSlide, CarouselVariant, ChartCommonProps,
    ChatBoxProps, CheckboxProps, ChipProps, CodeTemplateSegment, CodeToken, CodeTokenKind,
    CollapsibleProps, ColorFamily, ColorProps, ColorToken, ComboBoxProps, ComboOption,
    CommandEntry, CommandProps, ComponentVariant, CountdownProps, CoverSource, CsvColumn,
    DateProps, DateRangeProps, DesignConfig, DesignTheme, DividerOrientation, DividerProps,
    DragGroup, DragItem, DrawerPosition, DrawerProps, DropdownProps, DropzoneProps, ElementProps,
    EmptyProps, FabAction, FabProps, FlexDirection, FlexItem, FontConfig, FontFamily,
    FormValidationRuleKind, GapSize, GapValue, GridAlignment, GridProps, GridTracks,
    INPUT_HORIZONTAL_PADDING, INPUT_MIN_HEIGHT, INPUT_TEXT_SIZE, ImageProps, Justify, LayoutProps,
    MapMarker, MapProps, MapWaypoint, MarqueeProps, MicrophoneProps, ModalProps, NavMenuItem,
    NavMenuItemProps, NavMenuProps, NavigationAction, OverlayCornerPosition, OverlayEntry,
    OverlayItemProps, OverlayPaint, PhoneProps, PieChartProps, PositionProps,
    RadioGroupOrientation, RadioGroupPresentation, RadioGroupProps, RadioOption, RailNavItem,
    RailNavProps, RecordProps,
    ResponsiveValue, RichTextMark, RoundedSize, SIDE_NAV_SUBMENU_ARROW_PATH, ScaffoldProps,
    ScaleValue, SectionBackground, SelectOption, SelectOptionEach, ShadowSize, SideNavIcon,
    SideNavItem, SideNavItemProps, SideNavProps, SideNavSize, SidebarProps, SizeValue,
    SkeletonProps, SliderProps, StyleProps, SvgLineCap, SvgLineJoin, SvgPath, SvgPathFill,
    SvgProps, SvgTransform, SvgViewBox, TabItem, TableColumn, TableColumnAlign, TableProps,
    TableSize, TabsProps, TabsVariant, TextAlign, TextProps, TextSize, TextSpacing, TextWeight,
    ThemeSelectProps, ThemeToggleProps, ToastProps, ToggleGroupItem, ToggleGroupKind,
    ToggleGroupProps, ToggleProps, TooltipProps, TranslationCatalog, TypeWriterItem,
    TypeWriterProps, VariantProps, ViewAction, ViewActionKind, ViewAnimation, ViewConstant,
    ViewForm, ViewFormFieldKind, ViewGesture, ViewNode, ViewRequestAction, ViewRoute, ViewSignal,
    ViewSignalValue, ViewTransition, VisibilityCondition, collect_route_font_families,
    collect_view_forms, compose_tree, empty_icon, fixed_box_nodes, fixed_fab_nodes,
    form_control_min_height, form_control_text_size, node_child_groups, node_element_props,
    phone_countries, phone_country, phone_country_flag_icon, side_nav_memory_key,
    side_nav_submenu_arrow_icon, solar_control_icon, text_spacing_em, text_template_bindings,
    text_template_segments, text_typography, translation_resource_name, view_icon,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidOutput {
    pub files: Vec<AndroidArtifact>,
    pub render_report: dowe_components::RenderReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidArtifact {
    pub relative_path: PathBuf,
    pub content: String,
    pub kind: AndroidArtifactKind,
    pub target: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidArtifactKind {
    ProjectFile,
    Manifest,
    Entrypoint,
    GeneratedView,
    Routing,
    Layouts,
    Pages,
    Theme,
    Responsive,
    DevEntrypoint,
    Localization,
}

pub fn generate_android(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
) -> AndroidOutput {
    generate_android_with_app_and_translations(
        routes,
        font_config,
        design_config,
        environment,
        &TranslationCatalog::default(),
        "Dowe Dev",
        "dev.dowe.generated",
    )
}

pub fn generate_android_with_translations(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    translations: &TranslationCatalog,
) -> AndroidOutput {
    generate_android_with_app_and_translations(
        routes,
        font_config,
        design_config,
        environment,
        translations,
        "Dowe Dev",
        "dev.dowe.generated",
    )
}

pub fn generate_android_with_app_and_translations(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    translations: &TranslationCatalog,
    app_name: &str,
    app_bundle: &str,
) -> AndroidOutput {
    generate_android_with_app_translations_and_icons(
        routes,
        font_config,
        design_config,
        environment,
        translations,
        app_name,
        app_bundle,
        false,
    )
}

pub fn generate_android_with_app_translations_and_icons(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    translations: &TranslationCatalog,
    app_name: &str,
    app_bundle: &str,
    has_app_icon: bool,
) -> AndroidOutput {
    let font_families = font_config.effective_families(&collect_route_font_families(routes));
    let uses_camera = routes.iter().any(|route| {
        android_tree_has_camera(&route.layout_tree) || android_tree_has_camera(&route.page_tree)
    });
    let uses_microphone = routes.iter().any(|route| {
        android_tree_has_microphone(&route.layout_tree)
            || android_tree_has_microphone(&route.page_tree)
    });
    let dev_sources = dev_activity_sources(
        routes,
        font_config,
        &font_families,
        design_config,
        environment,
        app_bundle,
    );
    let mut files = vec![
        AndroidArtifact {
            relative_path: PathBuf::from("apps/android/settings.gradle.kts"),
            content: settings_gradle(),
            kind: AndroidArtifactKind::ProjectFile,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from("apps/android/build.gradle.kts"),
            content: root_gradle(),
            kind: AndroidArtifactKind::ProjectFile,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from("apps/android/gradle.properties"),
            content: gradle_properties(),
            kind: AndroidArtifactKind::ProjectFile,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from("apps/android/app/build.gradle.kts"),
            content: app_gradle(app_bundle),
            kind: AndroidArtifactKind::ProjectFile,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from("apps/android/app/src/main/AndroidManifest.xml"),
            content: android_manifest(app_name, has_app_icon, uses_camera, uses_microphone),
            kind: AndroidArtifactKind::Manifest,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from("apps/android/app/src/main/res/values/styles.xml"),
            content: styles_xml(),
            kind: AndroidArtifactKind::ProjectFile,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/MainActivity.kt",
            ),
            content: main_activity(),
            kind: AndroidArtifactKind::Entrypoint,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/DoweNotifications.kt",
            ),
            content: android_notifications(),
            kind: AndroidArtifactKind::ProjectFile,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/GeneratedViews.kt",
            ),
            content: generated_views_index(),
            kind: AndroidArtifactKind::GeneratedView,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/DoweRouting.kt",
            ),
            content: android_routing(routes),
            kind: AndroidArtifactKind::Routing,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/DoweLayouts.kt",
            ),
            content: android_layouts(),
            kind: AndroidArtifactKind::Layouts,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt",
            ),
            content: generated_views(routes, font_config, &font_families, design_config),
            kind: AndroidArtifactKind::Pages,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/DoweEnvironment.kt",
            ),
            content: android_environment(environment),
            kind: AndroidArtifactKind::ProjectFile,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/DoweTheme.kt",
            ),
            content: android_theme(design_config),
            kind: AndroidArtifactKind::Theme,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/DoweResponsive.kt",
            ),
            content: android_responsive(),
            kind: AndroidArtifactKind::Responsive,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from("apps/android/dev/AndroidManifest.xml"),
            content: dev_manifest(
                app_name,
                app_bundle,
                has_app_icon,
                uses_camera,
                uses_microphone,
            ),
            kind: AndroidArtifactKind::Manifest,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java",
            ),
            content: dev_sources.core,
            kind: AndroidArtifactKind::DevEntrypoint,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/dev/src/dev/dowe/generated/DoweDevHostActivity.java",
            ),
            content: dev_hot_host_activity(),
            kind: AndroidArtifactKind::DevEntrypoint,
            target: "android",
        },
    ];
    files.extend(
        dev_sources
            .shards
            .into_iter()
            .map(|source| AndroidArtifact {
                relative_path: PathBuf::from("apps/android/dev/src/dev/dowe/generated")
                    .join(source.file_name),
                content: source.content,
                kind: AndroidArtifactKind::DevEntrypoint,
                target: "android",
            }),
    );
    files.extend(android_translation_artifacts(translations));
    files.push(AndroidArtifact {
        relative_path: PathBuf::from("apps/android/view-consumption.json"),
        content: android_view_consumption_manifest(routes),
        kind: AndroidArtifactKind::Manifest,
        target: "android",
    });
    files.push(AndroidArtifact {
        relative_path: PathBuf::from("apps/android/dev/view-consumption.json"),
        content: android_view_consumption_manifest(routes),
        kind: AndroidArtifactKind::Manifest,
        target: "android",
    });
    AndroidOutput {
        files,
        render_report: render_report_for_routes(routes),
    }
}

include!("android_artifact_reports.rs");
include!("android_artifact_gradle.rs");
include!("android_artifact_manifests.rs");
include!("android_artifact_helpers.rs");
