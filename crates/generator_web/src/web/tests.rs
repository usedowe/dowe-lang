use super::{
    ChunkKind, build_layout_chunk, build_page_chunk, build_translation_chunks, render_page_body,
    web_artifacts,
};
use dowe_components::{
    AccordionItem, AccordionProps, AlertDialogProps, AudioProps, AvatarGroupItem,
    AvatarGroupProps, AvatarProps, AvatarStatus, BadgeProps, BarProps, Breakpoint, ButtonSize,
    CarouselIndicatorType, CarouselOrientation, CarouselProps, CarouselSlide, ChatBoxMode,
    ChatBoxProps, CheckboxProps, ChipProps, ColorFamily, ColorProps, ColorToken, CommandEntry,
    CollapsibleProps, CommandProps, ComponentProp, ComponentVariant, CountdownProps,
    CountdownSize, CoverSource, DateProps, DateRangeProps, DesignConfig, DividerOrientation,
    DividerProps, DrawerPosition, DrawerProps, DropdownProps, ElementProps, EmptyKind,
    EmptyProps, FontConfig, GapSize, GapValue, GridAlignment, GridProps, GridSpan, GridTracks,
    ImageAspect, ImageLoading, ImageObjectFit, ImageProps, MapMarker, MapMarkerIcon, MapProps,
    MapWaypoint, MarqueeOrientation, MarqueeProps, MarqueeSpeed, ModalProps, NavMenuItem, NavMenuItemProps,
    NavMenuProps, NavigationAction, NavigationOperation, OverlayCornerPosition, OverlayEntry,
    OverlayItemProps, OverlayPaint, OverlayPosition, PropValue, RadioGroupProps, RadioOption,
    RecordProps, ResponsiveEntry, ResponsiveValue, RichTextMark, RichTextMarkStyle, RoundedSize,
    ScaleValue, ScaffoldProps, SectionBackground, SelectOption, SideNavItem, SideNavItemProps,
    SideNavProps, SideNavSize, SkeletonAnimation, SkeletonProps, SkeletonVariant, StyleProps,
    SvgPath, SvgPathFill, SvgProps, SvgViewBox, TabItem, TabsPosition, TabsProps, TabsVariant,
    TextProps, TextWeight, ToastKind, ToastProps, ToggleGroupItem, ToggleGroupProps, ToggleProps,
    TooltipProps, TranslationCatalog, TranslationLocale, TranslationValue, TypeWriterItem,
    TypeWriterProps, VariantProps, VideoAspect, VideoProps, ViewAction, ViewActionKind,
    ViewAnimation, ViewNode, ViewResetAction, ViewSignal, ViewSignalValue, VisibilityCondition,
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
