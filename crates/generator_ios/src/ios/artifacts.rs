use dowe_components::{
    AccordionItem, AccordionProps, AlertDialogProps, Align, ArcChartProps, AudioProps,
    AvatarGroupItem, AvatarGroupProps, AvatarProps, BadgeProps, BarPosition, BarProps, BorderWidth,
    BottomBarTab, BoxPosition, Breakpoint, ButtonSize, CanvasBackground, CarouselProps,
    CarouselSlide, ChartCommonProps, ChatBoxProps, CheckboxProps, ChipProps, CodeTemplateSegment,
    CodeToken, CodeTokenKind, CollapsibleProps, ColorFamily, ColorProps, ColorToken, ComboBoxProps,
    ComboOption, CommandEntry, CommandProps, ComponentVariant, CountdownProps, CoverSource,
    CsvColumn, CsvFieldProps, DateProps, DateRangeProps, DesignConfig, DesignTheme,
    DividerOrientation, DividerProps, DragDropProps, DragGroup, DragItem, DrawerProps,
    DropdownProps, DropzoneProps, EditorProps, ElementProps,
    EmptyProps, FabAction, FabProps, FlexDirection, FlexItem, FontConfig, FontFamily,
    FormValidationRuleKind, GapSize, GapValue, GridAlignment, GridProps, GridTracks,
    INPUT_HORIZONTAL_PADDING, INPUT_MIN_HEIGHT, INPUT_TEXT_SIZE, ImageProps, Justify, LayoutProps,
    ImageCropperProps, MapMarker, MapProps, MapWaypoint, MarqueeProps, ModalProps, NavMenuItem,
    NavMenuItemProps,
    NavMenuProps, NavigationAction, OverlayCornerPosition, OverlayEntry, OverlayItemProps,
    OverlayPaint, PasswordProps, PhoneProps, PinProps, PieChartProps, PositionProps,
    RadioGroupPresentation, RadioGroupProps,
    RadioOption, RailNavItem,
    RailNavProps, RecordProps, ResponsiveValue, RichTextMark, RoundedSize,
    SIDE_NAV_SUBMENU_ARROW_PATH, ScaffoldProps, ScaleValue, SectionBackground, SelectOption,
    SelectOptionEach, ShadowSize, SideNavIcon, SideNavItem, SideNavItemProps, SideNavProps,
    SideNavSize, SidebarProps, SizeValue, SkeletonProps, SliderProps, StyleProps, SvgLineCap,
    SvgLineJoin, SvgPath, SvgPathFill, SvgProps, SvgViewBox, TabItem, TableColumn,
    TableColumnAlign, TableSize, TabsProps, TabsVariant, TextAlign, TextProps, TextSize,
    TextWeight, TextareaProps, ThemeSelectProps, ThemeToggleProps, ToastProps, ToggleGroupItem,
    ToggleGroupKind,
    ToggleGroupProps, ToggleProps, TooltipProps, TranslationCatalog, TypeWriterItem,
    TypeWriterProps, VariantProps, ViewAction, ViewActionKind, ViewAnimation, ViewConstant,
    ViewForm, ViewFormFieldKind, ViewGesture, ViewNode, ViewRequestAction, ViewRoute, ViewSignal,
    ViewSignalValue, ViewTransition, VisibilityCondition, collect_route_font_families,
    collect_view_forms, compose_tree, empty_icon, fixed_box_nodes, fixed_fab_nodes,
    form_control_min_height, form_control_text_size, node_child_groups, node_element_props,
    phone_countries, phone_country_flag_icon, side_nav_memory_key, side_nav_submenu_arrow_icon,
    solar_control_icon, text_spacing_em, text_template_bindings, text_template_segments,
    text_typography, view_icon,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IosOutput {
    pub files: Vec<IosArtifact>,
    pub render_report: dowe_components::RenderReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IosArtifact {
    pub relative_path: PathBuf,
    pub content: String,
    pub kind: IosArtifactKind,
    pub target: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IosArtifactKind {
    Entrypoint,
    GeneratedView,
    Routing,
    Layouts,
    Pages,
    Theme,
    Responsive,
    Manifest,
    Localization,
    DevEntrypoint,
}

pub fn generate_ios(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
) -> IosOutput {
    generate_ios_with_app_and_translations(
        routes,
        font_config,
        design_config,
        environment,
        &TranslationCatalog::default(),
        "Dowe Dev",
        "dev.dowe.generated",
    )
}

pub fn generate_ios_with_translations(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    translations: &TranslationCatalog,
) -> IosOutput {
    generate_ios_with_app_and_translations(
        routes,
        font_config,
        design_config,
        environment,
        translations,
        "Dowe Dev",
        "dev.dowe.generated",
    )
}

pub fn generate_ios_with_app_and_translations(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    translations: &TranslationCatalog,
    app_name: &str,
    app_bundle: &str,
) -> IosOutput {
    generate_ios_with_app_translations_and_icons(
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

pub fn generate_ios_with_app_translations_and_icons(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    translations: &TranslationCatalog,
    app_name: &str,
    app_bundle: &str,
    has_app_icon: bool,
) -> IosOutput {
    let font_families = font_config.effective_families(&collect_route_font_families(routes));
    let (layouts, route_layouts) = reusable_ios_layouts(routes);
    let mut files = vec![
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DoweIosApp.swift"),
            content: ios_app(),
            kind: IosArtifactKind::Entrypoint,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DoweNotifications.swift"),
            content: ios_notifications(),
            kind: IosArtifactKind::GeneratedView,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/Dowe.entitlements"),
            content: ios_entitlements(),
            kind: IosArtifactKind::Manifest,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/GeneratedViews.swift"),
            content: generated_views_index(),
            kind: IosArtifactKind::GeneratedView,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DoweRouting.swift"),
            content: ios_routing(routes),
            kind: IosArtifactKind::Routing,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DoweLayouts.swift"),
            content: ios_layouts_index(),
            kind: IosArtifactKind::Layouts,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DowePages.swift"),
            content: generated_views(routes, font_config, &font_families, design_config),
            kind: IosArtifactKind::Pages,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DoweEnvironment.swift"),
            content: ios_environment(environment),
            kind: IosArtifactKind::GeneratedView,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DoweTheme.swift"),
            content: ios_theme(design_config),
            kind: IosArtifactKind::Theme,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DoweResponsive.swift"),
            content: ios_responsive(),
            kind: IosArtifactKind::Responsive,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/Info.plist"),
            content: info_plist(
                &font_families,
                translations.default_locale.as_deref(),
                app_name,
                app_bundle,
                routes.iter().any(|route| {
                    ios_canvas_motion(&route.layout_tree) || ios_canvas_motion(&route.page_tree)
                }),
                routes.iter().any(|route| {
                    ios_video_playback(&route.layout_tree) || ios_video_playback(&route.page_tree)
                }),
                routes.iter().any(|route| {
                    ios_tree_has_camera(&route.layout_tree) || ios_tree_has_camera(&route.page_tree)
                }),
                routes.iter().any(|route| {
                    ios_tree_has_microphone(&route.layout_tree)
                        || ios_tree_has_microphone(&route.page_tree)
                }),
                has_app_icon,
            ),
            kind: IosArtifactKind::Manifest,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/dev/DoweIosDevHost.swift"),
            content: ios_dev_host(),
            kind: IosArtifactKind::DevEntrypoint,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/dev/DoweIosViewModule.swift"),
            content: ios_dev_module_factory(),
            kind: IosArtifactKind::DevEntrypoint,
            target: "ios",
        },
    ];
    files.extend(ios_layout_artifacts(&layouts, font_config));
    files.extend(ios_route_artifacts(routes, font_config, &route_layouts));
    if ios_has_dynamic_icon(routes) {
        files.extend(ios_dynamic_icon_catalog_artifacts(routes));
    }
    if routes
        .iter()
        .any(|route| ios_tree_has_phone(&route.layout_tree) || ios_tree_has_phone(&route.page_tree))
    {
        files.extend(ios_phone_catalog_artifacts());
    }
    files.extend(ios_translation_artifacts(translations));
    files.push(IosArtifact {
        relative_path: PathBuf::from("apps/ios/view-consumption.json"),
        content: ios_view_consumption_manifest(routes),
        kind: IosArtifactKind::Manifest,
        target: "ios",
    });
    files.push(IosArtifact {
        relative_path: PathBuf::from("apps/ios/dev/view-consumption.json"),
        content: ios_view_consumption_manifest(routes),
        kind: IosArtifactKind::Manifest,
        target: "ios",
    });
    IosOutput {
        files,
        render_report: render_report_for_routes(routes),
    }
}

include!("ios_reports_and_routes.rs");
include!("ios_localization_artifacts.rs");
include!("ios_routing.rs");
include!("ios_layout_artifacts.rs");
include!("ios_theme_and_app.rs");
include!("ios_notifications.rs");
include!("ios_dev_host.rs");
include!("ios_plist_and_helpers.rs");
