use super::{
    IosOutput, generate_ios, generate_ios_with_app_and_translations, generate_ios_with_translations,
};
use dowe_components::{
    AccordionItem, AccordionProps, AlertDialogProps, AudioProps, AvatarGroupItem,
    AvatarGroupProps, AvatarProps, AvatarStatus, BadgeProps, BarProps, Breakpoint, ButtonSize,
    CarouselIndicatorType, CarouselOrientation, CarouselProps, CarouselSlide, ChatBoxMode,
    ChatBoxProps, CheckboxProps, ChipProps, ColorFamily, ColorProps, ColorToken, ComboBoxProps,
    ComboOption, CommandEntry, CollapsibleProps, CommandProps, ComponentProp, ComponentVariant,
    CountdownProps, CountdownSize, CoverSource, CsvColumn, CsvFieldProps, DateProps,
    DateRangeProps, DesignConfig, DividerOrientation, DividerProps, DragDropDirection,
    DragDropProps, DragGroup, DragItem, DrawerPosition, DrawerProps, DropdownProps, EditorProps,
    ElementProps, EmptyKind, EmptyProps, FontConfig, GapSize, GapValue, GridProps, GridTracks,
    ImageAspect, ImageCropperProps, ImageCropperShape, ImageLoading, ImageObjectFit, ImageProps,
    MapMarker, MapMarkerIcon, MapProps, MapWaypoint, MarqueeOrientation, MarqueeProps,
    MarqueeSpeed, ModalProps, NavMenuItem, NavMenuItemProps, NavMenuProps, NavigationAction,
    NavigationOperation, OverlayCornerPosition, OverlayEntry, OverlayItemProps, OverlayPaint,
    OverlayPosition, PasswordFieldProps, PhoneFieldProps, PinFieldKind, PinFieldProps, PropValue,
    RadioGroupProps, RadioOption, RecordProps, ResponsiveEntry, ResponsiveValue, RichTextMark,
    RichTextMarkStyle, RoundedSize, ScaleValue, ScaffoldProps, SectionBackground, SelectOption,
    SideNavItem, SideNavItemProps, SideNavProps, SideNavSize, SkeletonAnimation, SkeletonProps,
    SkeletonVariant, StyleProps, SvgPath, SvgPathFill, SvgProps, SvgViewBox, TabItem,
    TabsPosition, TabsProps, TabsVariant, TextProps, TextSize, TextWeight, TextareaProps,
    ToastKind, ToastProps, ToggleGroupItem, ToggleGroupProps, ToggleProps, TooltipProps,
    TranslationCatalog, TranslationLocale, TranslationValue, TypeWriterItem, TypeWriterProps,
    VariantProps, ViewAnimation, ViewNode, ViewRoute, ViewSection, ViewSignal, ViewSignalValue,
    VisibilityCondition,
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
