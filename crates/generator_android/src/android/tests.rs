use super::{
    generate_android, generate_android_with_app_and_translations,
    generate_android_with_translations,
};
use dowe_components::{
    AccordionItem, AccordionProps, Align, AlertDialogProps, AudioProps, AvatarGroupItem,
    AvatarGroupProps, AvatarProps, AvatarStatus, BadgeProps, BarProps, Breakpoint, ButtonSize,
    CarouselIndicatorType, CarouselOrientation, CarouselProps, CarouselSlide, ChatBoxMode,
    ChatBoxProps, CheckboxProps, ChipProps, ColorFamily, ColorProps, ColorToken, CommandEntry,
    CollapsibleProps, CommandProps, ComponentProp, ComponentVariant, CountdownProps,
    CountdownSize, CoverSource, DateProps, DateRangeProps, DesignConfig, DividerOrientation,
    DividerProps, DrawerPosition, DrawerProps, DropdownProps, ElementProps, EmptyKind,
    EmptyProps, FontConfig, GapSize, GapValue, GridProps, GridTracks, ImageAspect, ImageLoading,
    ImageObjectFit, ImageProps, Justify, LayoutProps, MapMarker, MapMarkerIcon, MapProps,
    MapWaypoint, MarqueeOrientation, MarqueeProps, MarqueeSpeed, ModalProps, NavMenuItem,
    NavMenuItemProps, NavMenuProps, NavigationAction, NavigationOperation, OverlayCornerPosition,
    OverlayEntry, OverlayItemProps, OverlayPaint, OverlayPosition, PropValue, RadioGroupProps,
    RadioOption, RecordProps, ResponsiveEntry, ResponsiveValue, RichTextMark, RichTextMarkStyle,
    RoundedSize, ScaleValue, ScaffoldProps, SectionBackground, SelectOption, SideNavIcon,
    SideNavItem, SideNavItemProps, SideNavProps, SideNavSize, SizeValue, SkeletonAnimation,
    SkeletonProps, SkeletonVariant, StyleProps, SvgPath, SvgPathFill, SvgProps, SvgViewBox,
    TabItem, TabsPosition, TabsProps, TabsVariant, TextProps, TextSize, TextWeight, ToastKind,
    ToastProps, ToggleGroupItem, ToggleGroupProps, ToggleProps, TooltipProps, TranslationCatalog,
    TranslationLocale, TranslationValue, TypeWriterItem, TypeWriterProps, VariantProps,
    ViewAnimation, ViewNode, ViewRoute, ViewSection, ViewSignal, ViewSignalValue, VisibilityCondition,
};
use std::path::PathBuf;


include!("tests/core_generation.rs");
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
