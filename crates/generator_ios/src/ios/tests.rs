use super::{
    IosOutput, generate_ios, generate_ios_with_app_and_translations,
    generate_ios_with_app_translations_and_icons, generate_ios_with_translations,
    swift_runtime_svg_runtime, swift_svg_fill,
};
use dowe_components::{
    AccordionItem, AccordionProps, AlertDialogProps, AudioProps, AvatarGroupItem, AvatarGroupProps,
    AvatarProps, AvatarStatus, BadgeProps, BarPosition, BarProps, BorderWidth, BottomBarTab,
    BannerProps, BoxPosition, BrandProps,
    Breakpoint, ButtonSize, CarouselIndicatorType, CarouselOrientation, CarouselProps,
    CarouselSlide, CarouselVariant, ChatBoxMode, ChatBoxProps, CheckboxProps, ChipProps,
    CollapsibleProps, ColorFamily, ColorProps, ColorToken, ComboBoxProps, ComboOption,
    CommandEntry, CommandProps, ComponentProp, ComponentVariant, CountdownProps, CountdownSize,
    CoverSource, CsvColumn, CsvFieldProps, DateProps, DateRangeProps, DesignConfig,
    DividerOrientation, DividerProps, DragDropDirection, DragDropProps, DragGroup, DragItem,
    DrawerPosition, DrawerProps, DropdownProps, DropzoneProps, EditorProps, ElementProps,
    EmptyKind, EmptyProps, FabAction, FabProps, FontConfig, GapSize, GapValue, GridProps,
    GridTracks, ImageAspect, ImageCropperProps, ImageCropperShape, ImageLoading, ImageObjectFit,
    ImageProps, MapMarker, MapMarkerIcon, MapProps, MapWaypoint, MarqueeOrientation, MarqueeProps,
    MarqueeSpeed, ModalProps, NavMenuItem, NavMenuItemProps, NavMenuProps, NavigationAction,
    NavigationOperation, OverlayCornerPosition, OverlayEntry, OverlayItemProps, OverlayPaint,
    OverlayPosition, PasswordProps, PhoneProps, PinKind, PinProps, PropValue,
    RadioGroupOrientation, RadioGroupProps, RadioOption, RailNavItem, RailNavItemProps,
    RailNavProps, ReactiveVariantProps, RecordProps, ResponsiveEntry, ResponsiveValue,
    RichTextMark, RichTextMarkStyle, RoundedSize, ScaffoldProps, ScaleValue, SectionBackground,
    SelectOption, SelectOptionEach, ShadowSize, SideNavIcon, SideNavItem, SideNavItemProps,
    SideNavProps, SideNavSize, SidebarProps, SkeletonAnimation, SkeletonProps, SkeletonVariant,
    SizeValue, SizingProps, SpacingProps, StyleExtras, StyleProps, SvgLineCap, SvgLineJoin, SvgPath, SvgPathFill, SvgProps,
    SvgTransform, SvgViewBox, TabItem, TabsPosition, TabsProps, TabsVariant, TextProps, TextSize,
    TextWeight, TextareaProps, ToastKind, ToastProps, ToggleGroupItem, ToggleGroupKind,
    ToggleGroupProps, ToggleProps, TooltipProps, TranslationCatalog, TranslationLocale,
    TranslationValue, TypeWriterItem, TypeWriterProps, VariantProps, ViewAction, ViewActionKind,
    ViewAnimation, ViewAssignAction, ViewFunctionStatement, ViewGesture, ViewIcon, ViewMotionStyle, ViewNode,
    ViewRequestAction, ViewRequestMethod, ViewRotation, ViewRoute, ViewScale, ViewSection,
    ViewSignal, ViewSignalValue, ViewToastAction, ViewTransition, ViewTranslation,
    VisibilityCondition, icon_component_node, solar_control_icon, svg_spinner_control_icon,
};
use std::path::PathBuf;

include!("tests/core_generation.rs");
include!("tests/data_generation.rs");
include!("tests/navigation_generation.rs");
include!("tests/component_display_generation.rs");
include!("tests/fixtures_routes_core.rs");
include!("tests/fixtures_routes_navigation.rs");
include!("tests/fixtures_routes_forms.rs");
include!("tests/fixtures_routes_media.rs");
include!("tests/fixtures_components_overlay.rs");
include!("tests/fixtures_components_rich.rs");
include!("tests/fixtures_shared.rs");
