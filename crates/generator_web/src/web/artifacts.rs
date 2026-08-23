use dowe_components::{
    AccordionItem, AccordionProps, AlertDialogProps, AlertProps, Align, ArcChartProps,
    AreaChartProps, AudioProps, AvatarGroupItem, AvatarGroupProps, AvatarProps, BadgeProps,
    BannerProps, BarChartProps, BarProps, BottomBarTab, BoxPosition, BrandProps, Breakpoint,
    ButtonSize, CameraProps, CandlestickProps, CanvasBackground, CanvasProps,
    CarouselIndicatorType, CarouselOrientation, CarouselProps, CarouselSlide, CarouselVariant,
    ChartCommonProps, ChatBoxProps, CheckboxProps, ChipProps, CodeProps, CodeTemplateSegment,
    CollapsibleProps, ColorFamily, ColorProps, ColorToken, ComboBoxProps, ComboOption,
    CommandEntry, CommandProps, ComponentVariant, ContainerSize, CountdownProps, CoverSource, CsvColumn,
    CsvFieldProps, DateProps, DateRangeProps, DesignConfig, DesignTheme, DeviceProps, DividerProps,
    DragDropProps, DragGroup, DragItem, DrawerProps, DropdownProps, DropzoneProps, EditorProps,
    ElementProps, EmptyKind, EmptyProps, FORM_CONTROL_FLOATING_HEIGHT_INCREMENT, FabAction,
    FabProps, FlexDirection, FlexItem, FontConfig, FontFamily, FormValidationRuleKind, GapSize, GapValue,
    GridAlignment, GridProps, INPUT_HORIZONTAL_PADDING, IframeProps, ImageCropperProps, ImageProps,
    Justify, LayoutProps, LineChartProps, MapMarker, MapProps, MapWaypoint, MarqueeProps,
    MicrophoneProps, ModalProps, NativeExternalMode, NavMenuItem, NavMenuItemProps, NavMenuProps,
    NavigationAction, NavigationOperation, OverlayEntry, OverlayItemProps, OverlayPaint,
    PasswordProps, PhoneProps, PieChartProps, PinKind, PinProps, RadioGroupProps, RadioOption,
    RailNavItem, RailNavItemProps, RailNavProps, RecordProps, ResponsiveValue, RichTextMark,
    ScaffoldProps, ScaleValue, SectionBackground, SelectOption, SelectOptionEach, ShadowSize,
    SideNavIcon, SideNavItem, SideNavItemProps, SideNavProps, SidebarProps, SizeValue,
    SkeletonProps, SliderProps, StyleProps, SvgLineCap, SvgLineJoin, SvgPath, SvgPathFill,
    SvgProps, TabItem, TableColumn, TableColumnAlign, TableProps, TabsProps, TabsVariant,
    TextAlign, TextProps, TextSize, TextSpacing, TextWeight, TextareaProps, ThemeSelectProps,
    ThemeToggleProps, ToastKind, ToastProps, ToggleGroupItem, ToggleGroupKind, ToggleGroupProps,
    ToggleProps, TooltipProps, TranslationCatalog, TypeWriterItem, TypeWriterProps, VariantProps,
    VideoProps, ViewAction, ViewActionKind, ViewAnimation, ViewAssignAction, ViewConstant,
    ViewForm, ViewFormFieldKind, ViewFunctionStatement, ViewGesture, ViewIcon, ViewMetadata,
    ViewNavigationAction, ViewNode, ViewRequestAction, ViewResetAction, ViewSection, ViewSignal,
    ViewSignalValue, VisibilityCondition, WebTarget, collect_node_font_families,
    collect_view_forms, empty_icon, form_control_min_height, form_control_text_size,
    ordered_phone_countries, phone_countries, phone_country, phone_country_flag_icon,
    side_nav_memory_key, side_nav_submenu_arrow_icon, solar_control_icon, text_binding_path,
    text_spacing_em, text_typography, text_weight_number,
};
use dowe_minifier::{minify_css, minify_js};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;


include!("artifacts/types.rs");
include!("artifacts/chunks.rs");
include!("artifacts/rendering.rs");
include!("artifacts/design.rs");
include!("artifacts/update.rs");
include!("artifacts/files.rs");
include!("artifacts/manifests.rs");
