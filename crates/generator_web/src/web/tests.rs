use super::{
    ChunkKind, build_layout_chunk, build_page_chunk, build_translation_chunks, render_page_body,
    svg_path_attributes, web_artifacts,
};
use dowe_components::{
    AccordionItem, AccordionProps, AlertDialogProps, AudioProps, AvatarGroupItem,
    AvatarGroupProps, AvatarProps, AvatarStatus, BadgeProps, BarPosition, BarProps, BottomBarTab, Breakpoint,
    BannerProps, BoxPosition, BrandProps, ButtonSize,
    CarouselIndicatorType, CarouselOrientation, CarouselProps, CarouselSlide, CarouselVariant,
    ChatBoxMode,
    ChatBoxProps, CheckboxProps, ChipProps, ColorFamily, ColorProps, ColorToken, ComboBoxProps,
    ComboOption, CommandEntry, CollapsibleProps, CommandProps, ComponentProp, ComponentVariant,
    CountdownProps, CountdownSize, CoverSource, CsvColumn, CsvFieldProps, DateProps,
    DateRangeProps, DesignConfig, DividerOrientation, DividerProps, DragDropDirection,
    DragDropProps, DragGroup, DragItem, DrawerPosition, DrawerProps, DropdownProps, EditorProps,
    ElementProps, EmptyKind, EmptyProps, FabAction, FabProps, FontConfig, GapSize, GapValue, GridAlignment, GridProps,
    GridSpan, GridTracks, ImageAspect, ImageCropperProps, ImageCropperShape, ImageLoading,
    ImageObjectFit, ImageProps, MapMarker, MapMarkerIcon, MapProps,
    MapWaypoint, MarqueeOrientation, MarqueeProps, MarqueeSpeed, ModalProps, NavMenuItem,
    NavMenuItemProps, NavMenuProps, NavigationAction, NavigationOperation, OverlayCornerPosition,
    OverlayEntry, OverlayItemProps, OverlayPaint, OverlayPosition, PasswordFieldProps,
    PhoneFieldProps, PinFieldKind, PinFieldProps, PropValue, RadioGroupOrientation,
    RadioGroupProps, RadioOption, ReactiveVariantProps, RecordProps, ResponsiveEntry, ResponsiveValue, RichTextMark,
    RailNavItem, RailNavItemProps, RailNavProps, RichTextMarkStyle, RoundedSize, ScaleValue, ScaffoldProps, SectionBackground, SelectOption, SelectOptionEach,
    SideNavItem, SideNavItemProps,
    SideNavProps, SidebarProps, SideNavSize, SkeletonAnimation, SkeletonProps, SkeletonVariant, StyleProps,
    SvgLineCap, SvgLineJoin, SvgPath, SvgPathFill, SvgProps, SvgTransform, SvgViewBox, TabItem, TabsPosition, TabsProps, TabsVariant,
    TextProps, TextWeight, TextareaProps, ToastKind, ToastProps, ToggleGroupItem, ToggleGroupKind,
    ToggleGroupProps, ToggleProps, TooltipProps, TranslationCatalog, TranslationLocale,
    TranslationValue,
    TypeWriterItem, TypeWriterProps, VariantProps, VideoAspect, VideoProps, ViewAction,
    ViewActionKind, ViewAnimation, ViewAssignAction, ViewFunctionStatement, ViewIcon, ViewNode,
    ViewResetAction, ViewSignal, ViewSignalValue,
    VisibilityCondition, icon_component_node, solar_control_icon, svg_spinner_control_icon,
};
use std::path::{Path, PathBuf};

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
