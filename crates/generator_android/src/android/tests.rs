use super::{
    AndroidOutput, android_runtime_data_code_svg, compose_svg_fill, dev_activity_svg_view,
    dev_java_payload_expression, dev_svg_path_color, dev_svg_path_details, generate_android,
    generate_android_with_app_and_translations, generate_android_with_translations,
};
use dowe_components::{
    AccordionItem, AccordionProps, AlertDialogProps, Align, AudioProps, AvatarGroupItem,
    AvatarGroupProps, AvatarProps, AvatarStatus, BadgeProps, BannerProps, BarPosition, BarProps,
    BorderWidth, BottomBarTab, BoxPosition, BrandProps, Breakpoint, ButtonSize, CameraFacing,
    CameraProps, CarouselIndicatorType, CarouselOrientation, CarouselProps, CarouselSlide,
    CarouselVariant, ChatBoxMode, ChatBoxProps, CheckboxProps, ChipProps, CollapsibleProps,
    ColorFamily, ColorProps, ColorToken, ComboBoxProps, ComboOption, CommandEntry, CommandProps,
    ComponentProp, ComponentVariant, CountdownProps, CountdownSize, CoverSource, CsvColumn,
    CsvFieldProps, DateProps, DateRangeProps, DesignConfig, DividerOrientation, DividerProps,
    DragDropDirection, DragDropProps, DragGroup, DragItem, DrawerPosition, DrawerProps,
    DropdownProps, DropzoneProps, EditorProps, ElementProps, EmptyKind, EmptyProps, FabAction,
    FlexItem,
    FabProps, FontConfig, GapSize, GapValue, GridProps, GridTracks, ImageAspect, ImageCropperProps,
    ImageCropperShape, ImageLoading, ImageObjectFit, ImageProps, Justify, LayoutProps, MapMarker,
    MapMarkerIcon, MapProps, MapWaypoint, MarqueeOrientation, MarqueeProps, MarqueeSpeed,
    MicrophoneProps, ModalProps, NavMenuItem, NavMenuItemProps, NavMenuProps, NavigationAction,
    NavigationOperation, OverlayCornerPosition, OverlayEntry, OverlayItemProps, OverlayPaint,
    OverlayPosition, PasswordProps, PhoneProps, PinKind, PinProps, PropValue,
    RadioGroupOrientation, RadioGroupProps, RadioOption, RailNavItem, RailNavItemProps,
    RailNavProps, ReactiveVariantProps, RecordProps, ResponsiveEntry, ResponsiveValue,
    RichTextMark, RichTextMarkStyle, RoundedSize, ScaffoldProps, ScaleValue, SectionBackground,
    SelectOption, SelectOptionEach, ShadowSize, SideNavIcon, SideNavItem, SideNavItemProps,
    SideNavProps, SideNavSize, SidebarProps, SizeValue, SizingProps, SkeletonAnimation,
    SkeletonProps, SkeletonVariant, SliderProps, SpacingProps, StyleExtras, StyleProps, SvgLineCap,
    SvgLineJoin, SvgPath, SvgPathFill, SvgProps, SvgTransform, SvgViewBox, TabItem, TabsPosition,
    TabsProps, TabsVariant, TextAlign, TextProps, TextSize, TextWeight, TextareaProps,
    ThemeSelectProps, ToastKind, ToastProps, ToggleGroupItem, ToggleGroupKind, ToggleGroupProps,
    ToggleProps, TooltipProps, TranslationCatalog, TranslationLocale, TranslationValue,
    TypeWriterItem, TypeWriterProps, VariantProps, ViewAction, ViewActionKind, ViewAnimation,
    ViewAssignAction, ViewFunctionStatement, ViewGesture, ViewIcon, ViewMotionStyle, ViewNode,
    ViewRequestAction, ViewRequestMethod, ViewRotation, ViewRoute, ViewScale, ViewSection,
    ViewSignal, ViewSignalValue, ViewToastAction, ViewTransition, ViewTranslation,
    VisibilityCondition, icon_component_node, solar_control_icon, svg_spinner_control_icon,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

struct DevJavaSource {
    content: String,
}

fn dev_java_source(output: &AndroidOutput) -> DevJavaSource {
    let core = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    let mut content = core
        .content
        .lines()
        .map(|line| {
            if let Some(declaration) = line.strip_prefix("    ")
                && !declaration.starts_with(' ')
            {
                format!("    private {declaration}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    content.push('\n');
    for shard in output.files.iter().filter(|file| {
        file.relative_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                (name.starts_with("DoweDevRoute") || name.starts_with("DoweDevLayout"))
                    && name.ends_with(".java")
            })
    }) {
        content.push('\n');
        content.push_str(
            &shard
                .content
                .replace(
                    "int viewportWidth = runtime.viewportWidth;",
                    "int viewportWidth = this.viewportWidth;",
                )
                .replace("runtime.", "")
                .replace("runtime", "this")
                .replace("DoweDevActivity.", ""),
        );
    }
    DevJavaSource { content }
}

fn all_android_source(output: &AndroidOutput) -> String {
    let mut content = output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<String>();
    content.push_str(&dev_java_source(output).content);
    content
}

include!("tests/core_generation.rs");
include!("tests/dev_sharding.rs");
include!("tests/navigation_generation.rs");
include!("tests/component_data_generation.rs");
include!("tests/component_navigation_generation.rs");
include!("tests/component_display_generation.rs");
include!("tests/fixtures_routes_core.rs");
include!("tests/fixtures_routes_navigation.rs");
include!("tests/fixtures_routes_forms.rs");
include!("tests/fixtures_components_media.rs");
include!("tests/fixtures_components_rich.rs");
include!("tests/fixtures_components_shared.rs");
include!("tests/fixtures_display.rs");
include!("tests/stdlib_generation.rs");
