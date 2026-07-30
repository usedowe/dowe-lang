use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

pub type ComponentResult<T> = Result<T, ComponentError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewNode {
    Scope {
        constants: Vec<ViewConstant>,
        signals: Vec<ViewSignal>,
        actions: Vec<ViewAction>,
        children: Vec<ViewNode>,
    },
    Splash {
        binding: String,
        initial: bool,
        content: Vec<ViewNode>,
        children: Vec<ViewNode>,
    },
    Box {
        props: StyleProps,
        children: Vec<ViewNode>,
    },
    Section {
        props: StyleProps,
        children: Vec<ViewNode>,
    },
    Flex {
        props: LayoutProps,
        children: Vec<ViewNode>,
    },
    Grid {
        props: GridProps,
        children: Vec<ViewNode>,
    },
    Card {
        props: VariantProps,
        children: Vec<ViewNode>,
    },
    Tabs {
        props: TabsProps,
        tabs: Vec<TabItem>,
    },
    NavMenu {
        props: NavMenuProps,
        items: Vec<NavMenuItem>,
    },
    Button {
        props: VariantProps,
        children: Vec<ViewNode>,
    },
    Brand {
        props: BrandProps,
        children: Vec<ViewNode>,
    },
    Banner {
        props: BannerProps,
        children: Vec<ViewNode>,
    },
    ToggleTheme {
        props: ThemeToggleProps,
    },
    SelectTheme {
        props: ThemeSelectProps,
    },
    Fab {
        props: FabProps,
        actions: Vec<FabAction>,
    },
    Input {
        props: VariantProps,
    },
    Slider {
        props: SliderProps,
    },
    Dropzone {
        props: DropzoneProps,
    },
    Select {
        props: VariantProps,
        options: Vec<SelectOption>,
        option_each: Option<SelectOptionEach>,
    },
    ComboBox {
        props: ComboBoxProps,
        options: Vec<ComboOption>,
    },
    CsvField {
        props: CsvFieldProps,
        columns: Vec<CsvColumn>,
    },
    DragDrop {
        props: DragDropProps,
        items: Vec<DragItem>,
        groups: Vec<DragGroup>,
    },
    Editor {
        props: EditorProps,
    },
    ImageCropper {
        props: ImageCropperProps,
    },
    PasswordField {
        props: PasswordFieldProps,
    },
    PhoneField {
        props: PhoneFieldProps,
    },
    PinField {
        props: PinFieldProps,
    },
    Textarea {
        props: TextareaProps,
    },
    Audio {
        props: AudioProps,
    },
    Image {
        props: ImageProps,
    },
    Code {
        props: CodeProps,
    },
    Video {
        props: VideoProps,
    },
    Iframe {
        props: IframeProps,
    },
    Device {
        props: DeviceProps,
        iframe: IframeProps,
    },
    Canvas {
        props: CanvasProps,
    },
    Candlestick {
        props: CandlestickProps,
    },
    ArcChart {
        props: ArcChartProps,
    },
    AreaChart {
        props: AreaChartProps,
    },
    BarChart {
        props: BarChartProps,
    },
    LineChart {
        props: LineChartProps,
    },
    PieChart {
        props: PieChartProps,
    },
    Table {
        props: TableProps,
    },
    Divider {
        props: DividerProps,
    },
    Title {
        props: TextProps,
        value: String,
    },
    Text {
        props: TextProps,
        value: String,
    },
    Alert {
        props: AlertProps,
    },
    Svg {
        props: SvgProps,
        paths: Vec<SvgPath>,
    },
    AppBar {
        props: BarProps,
        top: Vec<ViewNode>,
        start: Vec<ViewNode>,
        center: Vec<ViewNode>,
        end: Vec<ViewNode>,
        bottom: Vec<ViewNode>,
    },
    Footer {
        props: BarProps,
        top: Vec<ViewNode>,
        start: Vec<ViewNode>,
        center: Vec<ViewNode>,
        end: Vec<ViewNode>,
        bottom: Vec<ViewNode>,
    },
    BottomBar {
        props: BarProps,
        tabs: Vec<BottomBarTab>,
    },
    SideNav {
        props: SideNavProps,
        items: Vec<SideNavItem>,
    },
    RailNav {
        props: RailNavProps,
        items: Vec<RailNavItem>,
    },
    Sidebar {
        props: SidebarProps,
        header: Vec<ViewNode>,
        body: Vec<ViewNode>,
        footer: Vec<ViewNode>,
    },
    Scaffold {
        props: ScaffoldProps,
        app_bar: Vec<ViewNode>,
        start: Vec<ViewNode>,
        main: Vec<ViewNode>,
        end: Vec<ViewNode>,
        bottom_bar: Vec<ViewNode>,
        overlays: Vec<ViewNode>,
    },
    Drawer {
        props: DrawerProps,
        header: Vec<ViewNode>,
        body: Vec<ViewNode>,
        footer: Vec<ViewNode>,
    },
    Avatar {
        props: AvatarProps,
        icon: Option<SideNavIcon>,
    },
    Badge {
        props: BadgeProps,
        children: Vec<ViewNode>,
    },
    Chip {
        props: ChipProps,
        value: String,
        start: Option<SideNavIcon>,
        end: Option<SideNavIcon>,
    },
    Skeleton {
        props: SkeletonProps,
    },
    Modal {
        props: ModalProps,
        header: Vec<ViewNode>,
        body: Vec<ViewNode>,
        footer: Vec<ViewNode>,
    },
    AlertDialog {
        props: AlertDialogProps,
    },
    Tooltip {
        props: TooltipProps,
        children: Vec<ViewNode>,
    },
    Toast {
        props: ToastProps,
    },
    Dropdown {
        props: DropdownProps,
        trigger: Vec<ViewNode>,
        header: Vec<ViewNode>,
        entries: Vec<OverlayEntry>,
        footer: Vec<ViewNode>,
    },
    Command {
        props: CommandProps,
        entries: Vec<CommandEntry>,
    },
    AvatarGroup {
        props: AvatarGroupProps,
        items: Vec<AvatarGroupItem>,
    },
    ChatBox {
        props: ChatBoxProps,
    },
    Empty {
        props: EmptyProps,
    },
    Marquee {
        props: MarqueeProps,
        children: Vec<ViewNode>,
    },
    TypeWriter {
        props: TypeWriterProps,
        items: Vec<TypeWriterItem>,
    },
    RichText {
        props: TextProps,
        marks: Vec<RichTextMark>,
    },
    Record {
        props: RecordProps,
    },
    ToggleGroup {
        props: ToggleGroupProps,
        items: Vec<ToggleGroupItem>,
    },
    Collapsible {
        props: CollapsibleProps,
        children: Vec<ViewNode>,
    },
    Countdown {
        props: CountdownProps,
    },
    Map {
        props: MapProps,
        markers: Vec<MapMarker>,
        waypoints: Vec<MapWaypoint>,
    },
    Accordion {
        props: AccordionProps,
        items: Vec<AccordionItem>,
    },
    Carousel {
        props: CarouselProps,
        slides: Vec<CarouselSlide>,
    },
    Checkbox {
        props: CheckboxProps,
    },
    Color {
        props: ColorProps,
    },
    Date {
        props: DateProps,
    },
    DateRange {
        props: DateRangeProps,
    },
    RadioGroup {
        props: RadioGroupProps,
        options: Vec<RadioOption>,
    },
    Toggle {
        props: ToggleProps,
    },
    Each {
        item: String,
        collection: String,
        key: String,
        children: Vec<ViewNode>,
    },
    Children,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewConstant {
    pub id: String,
    pub name: String,
    pub value: ViewSignalValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewSignal {
    pub id: String,
    pub name: String,
    pub storage_key: String,
    pub scope: ViewSignalScope,
    pub storage: ViewSignalStorage,
    pub initial: ViewSignalValue,
    pub schema: Option<ViewSignalValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewSignalScope {
    Page,
    Global,
}

impl ViewSignalScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Global => "global",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewSignalStorage {
    None,
    Local,
}

impl ViewSignalStorage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Local => "local",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewSignalValue {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<ViewSignalValue>),
    Object(Vec<(String, ViewSignalValue)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewAction {
    pub id: String,
    pub name: String,
    pub params: Vec<ViewFunctionParameter>,
    pub return_type: Option<ViewFunctionReturn>,
    pub kind: ViewActionKind,
}

impl ViewAction {
    pub fn init(id: String, statements: Vec<ViewFunctionStatement>) -> Self {
        Self {
            id,
            name: "$dowe:init".to_string(),
            params: Vec::new(),
            return_type: None,
            kind: ViewActionKind::Sequence(statements),
        }
    }

    pub fn is_init(&self) -> bool {
        self.name == "$dowe:init"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewFunctionParameter {
    pub name: String,
    pub type_name: String,
    pub schema: ViewSignalValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewFunctionReturn {
    pub type_name: String,
    pub schema: ViewSignalValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewActionKind {
    Sequence(Vec<ViewFunctionStatement>),
    Request(ViewRequestAction),
    Assign(ViewAssignAction),
    Reset(ViewResetAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewFunctionStatement {
    Request {
        result: String,
        action: ViewRequestAction,
    },
    If {
        result: String,
        success: Vec<ViewFunctionStatement>,
        error: Vec<ViewFunctionStatement>,
    },
    Assign(ViewAssignAction),
    Reset(ViewResetAction),
    Toast(ViewToastAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewToastAction {
    pub kind: String,
    pub title: String,
    pub message: String,
    pub duration: Option<u32>,
    pub scheme: Option<String>,
    pub variant: Option<String>,
    pub position: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewRequestAction {
    pub method: ViewRequestMethod,
    pub path: String,
    pub base_env: Option<String>,
    pub headers: Vec<ViewRequestHeader>,
    pub body: Option<String>,
    pub update: Option<String>,
    pub reset: Option<String>,
    pub success_alert: Option<String>,
    pub success_message: Option<String>,
    pub error_alert: Option<String>,
    pub error_message: Option<String>,
    pub autoload: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewRequestHeader {
    pub name: String,
    pub value: ViewRequestHeaderValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewRequestHeaderValue {
    Static(String),
    Signal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewRequestMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl ViewRequestMethod {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "PUT" => Some(Self::Put),
            "PATCH" => Some(Self::Patch),
            "DELETE" => Some(Self::Delete),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewAssignAction {
    pub target: String,
    pub source: String,
    pub literal: Option<ViewSignalValue>,
    pub call: Option<StdlibCall>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewResetAction {
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewRoute {
    pub id: String,
    pub route_path: String,
    pub layout_tree: ViewNode,
    pub page_tree: ViewNode,
    pub sections: Vec<ViewSection>,
    pub navigation_actions: Vec<ViewNavigationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewSection {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewNavigationAction {
    pub id: String,
    pub action: NavigationAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewMetadata {
    pub name: String,
    pub content: String,
}

pub const VIEW_META_NAMES: &[&str] = &[
    "title",
    "description",
    "keywords",
    "robots",
    "canonical",
    "og:title",
    "og:description",
    "og:image",
    "og:image:alt",
    "og:type",
    "og:url",
    "og:site_name",
    "twitter:card",
    "twitter:title",
    "twitter:description",
    "twitter:image",
    "twitter:image:alt",
    "twitter:site",
    "twitter:creator",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationAction {
    Internal {
        path: String,
        fragment: Option<String>,
        operation: NavigationOperation,
    },
    Section {
        fragment: String,
        operation: NavigationOperation,
    },
    External {
        url: String,
        web_target: WebTarget,
        native_external_mode: NativeExternalMode,
    },
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationOperation {
    Push,
    Replace,
}

impl NavigationOperation {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "push" => Some(Self::Push),
            "replace" => Some(Self::Replace),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Push => "push",
            Self::Replace => "replace",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Push, Self::Replace]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebTarget {
    SelfTarget,
    Blank,
}

impl WebTarget {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "self" => Some(Self::SelfTarget),
            "blank" => Some(Self::Blank),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SelfTarget => "self",
            Self::Blank => "blank",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::SelfTarget, Self::Blank]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeExternalMode {
    System,
    Webview,
}

impl NativeExternalMode {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Self::System),
            "webview" => Some(Self::Webview),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Webview => "webview",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::System, Self::Webview]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ElementProps {
    pub id: Option<String>,
    pub font: Option<ResponsiveValue<FontFamily>>,
    pub bind: Option<String>,
    pub on_click: Option<String>,
    pub show: Option<VisibilityCondition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StyleProps {
    pub element: ElementProps,
    pub font: Option<ResponsiveValue<FontFamily>>,
    pub bg: Option<ResponsiveValue<ColorToken>>,
    pub text: Option<ResponsiveValue<ColorToken>>,
    pub cover: Option<ResponsiveValue<CoverSource>>,
    pub overlay: Option<ResponsiveValue<OverlayPaint>>,
    pub background: Option<ResponsiveValue<SectionBackground>>,
    pub boxed: bool,
    pub animation: Option<ViewAnimation>,
    pub spacing: SpacingProps,
    pub sizing: SizingProps,
    pub rounded: Option<ResponsiveValue<RoundedSize>>,
    pub border: Option<ResponsiveValue<BorderWidth>>,
    pub border_color: Option<ColorFamily>,
    pub shadow: Option<ResponsiveValue<ShadowSize>>,
    pub shadow_color: Option<ColorFamily>,
    pub grid_item: Option<Box<GridItemProps>>,
    pub position: Option<Box<PositionProps>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoxPosition {
    #[default]
    Static,
    Relative,
    Absolute,
    Fixed,
}

impl BoxPosition {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "static" => Some(Self::Static),
            "relative" => Some(Self::Relative),
            "absolute" => Some(Self::Absolute),
            "fixed" => Some(Self::Fixed),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Relative => "relative",
            Self::Absolute => "absolute",
            Self::Fixed => "fixed",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Static, Self::Relative, Self::Absolute, Self::Fixed]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PositionProps {
    pub mode: BoxPosition,
    pub top: Option<ResponsiveValue<ScaleValue>>,
    pub right: Option<ResponsiveValue<ScaleValue>>,
    pub bottom: Option<ResponsiveValue<ScaleValue>>,
    pub left: Option<ResponsiveValue<ScaleValue>>,
}

impl StyleProps {
    pub fn grid_item(&self) -> &GridItemProps {
        self.grid_item.as_deref().unwrap_or(&GridItemProps::EMPTY)
    }

    pub fn grid_item_mut(&mut self) -> &mut GridItemProps {
        self.grid_item
            .get_or_insert_with(|| Box::new(GridItemProps::default()))
    }

    pub fn position(&self) -> &PositionProps {
        self.position
            .as_deref()
            .unwrap_or(&PositionProps::STATIC)
    }

    pub fn position_mut(&mut self) -> &mut PositionProps {
        self.position
            .get_or_insert_with(|| Box::new(PositionProps::default()))
    }
}

impl GridItemProps {
    pub const EMPTY: Self = Self {
        col_span: None,
        row_span: None,
    };
}

impl PositionProps {
    pub const STATIC: Self = Self {
        mode: BoxPosition::Static,
        top: None,
        right: None,
        bottom: None,
        left: None,
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutProps {
    pub style: StyleProps,
    pub direction: ResponsiveValue<FlexDirection>,
    pub wrap: bool,
    pub justify: Option<ResponsiveValue<Justify>>,
    pub align: Option<ResponsiveValue<Align>>,
    pub gap: Option<ResponsiveValue<GapValue>>,
}

impl Default for LayoutProps {
    fn default() -> Self {
        Self {
            style: StyleProps::default(),
            direction: ResponsiveValue::scalar(FlexDirection::Row),
            wrap: false,
            justify: None,
            align: None,
            gap: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GridProps {
    pub style: StyleProps,
    pub columns: Option<ResponsiveValue<GridTracks>>,
    pub rows: Option<ResponsiveValue<GridTracks>>,
    pub justify: Option<ResponsiveValue<GridAlignment>>,
    pub align: Option<ResponsiveValue<GridAlignment>>,
    pub gap: Option<ResponsiveValue<GapValue>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VariantProps {
    pub element: ElementProps,
    pub style: StyleProps,
    pub variant: Option<ComponentVariant>,
    pub color: Option<ColorFamily>,
    pub size: Option<ButtonSize>,
    pub label: Option<String>,
    pub i18n: Option<String>,
    pub placeholder: Option<String>,
    pub label_floating: bool,
    pub icon_start: Option<SideNavIcon>,
    pub icon_end: Option<SideNavIcon>,
    pub loading_icon: Option<SideNavIcon>,
    pub icon_only: bool,
    pub navigation: Option<NavigationAction>,
    pub reactive: ReactiveVariantProps,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BrandProps {
    pub style: StyleProps,
    pub navigation: Option<NavigationAction>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerProps {
    pub style: StyleProps,
    pub navigation: NavigationAction,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReactiveVariantProps {
    pub variant: Option<String>,
    pub scheme: Option<String>,
    pub size: Option<String>,
    pub rounded: Option<String>,
    pub loading: Option<String>,
    pub icon_start_when: Option<String>,
    pub icon_end_when: Option<String>,
    pub icon_start_comparison: Option<ReactiveNumberComparison>,
    pub icon_end_comparison: Option<ReactiveNumberComparison>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactiveNumberComparison {
    pub operator: NumberComparisonOperator,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberComparisonOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl NumberComparisonOperator {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextProps {
    pub style: StyleProps,
    pub size: Option<ResponsiveValue<TextSize>>,
    pub weight: Option<ResponsiveValue<TextWeight>>,
    pub letter_spacing: Option<ResponsiveValue<TextSpacing>>,
    pub i18n: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertProps {
    pub style: VariantProps,
    pub kind: AlertKind,
    pub message: String,
    pub visible: Option<String>,
    pub on_close: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgProps {
    pub style: StyleProps,
    pub view_box: SvgViewBox,
    pub data: Option<String>,
    pub motion: Option<SvgMotion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgMotion {
    pub source: &'static str,
    pub fill: Option<ColorToken>,
    pub stroke: Option<ColorToken>,
    pub animated: bool,
}

impl SvgProps {
    pub fn is_animated(&self) -> bool {
        self.motion
            .as_ref()
            .is_some_and(|source| source.animated)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BarProps {
    pub style: VariantProps,
    pub bordered: bool,
    pub blurred: bool,
    pub boxed: bool,
    pub floating: bool,
    pub position: BarPosition,
    pub hide_on_scroll: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BottomBarTab {
    pub label: String,
    pub i18n: Option<String>,
    pub featured: bool,
    pub icon: SideNavIcon,
    pub navigation: NavigationAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarPosition {
    #[default]
    Static,
    Sticky,
    Fixed,
}

impl BarPosition {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "static" => Some(Self::Static),
            "sticky" => Some(Self::Sticky),
            "fixed" => Some(Self::Fixed),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Sticky => "sticky",
            Self::Fixed => "fixed",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Static, Self::Sticky, Self::Fixed]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideNavProps {
    pub style: VariantProps,
    pub size: SideNavSize,
    pub wide: bool,
    pub reactive_wide: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RailNavProps {
    pub style: VariantProps,
    pub size: SideNavSize,
    pub show_labels: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarProps {
    pub style: VariantProps,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavMenuProps {
    pub style: VariantProps,
    pub size: SideNavSize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScaffoldProps {
    pub style: StyleProps,
    pub boxed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawerProps {
    pub style: VariantProps,
    pub open: String,
    pub position: DrawerPosition,
    pub disable_overlay_close: bool,
    pub hide_close_button: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvatarProps {
    pub style: VariantProps,
    pub src: Option<String>,
    pub name: Option<String>,
    pub alt: String,
    pub size: ButtonSize,
    pub status: Option<AvatarStatus>,
    pub bordered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadgeProps {
    pub style: VariantProps,
    pub text: String,
    pub position: OverlayCornerPosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChipProps {
    pub style: VariantProps,
    pub on_close: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkeletonProps {
    pub style: StyleProps,
    pub variant: SkeletonVariant,
    pub animation: SkeletonAnimation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalProps {
    pub style: VariantProps,
    pub open: String,
    pub on_close: Option<String>,
    pub disable_overlay_close: bool,
    pub hide_close_button: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertDialogProps {
    pub style: VariantProps,
    pub open: String,
    pub title: String,
    pub description: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub on_confirm: Option<String>,
    pub on_cancel: Option<String>,
    pub loading: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TooltipProps {
    pub style: VariantProps,
    pub label: String,
    pub position: OverlayPosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastProps {
    pub style: VariantProps,
    pub source: Option<String>,
    pub kind: ToastKind,
    pub title: Option<String>,
    pub description: String,
    pub position: OverlayCornerPosition,
    pub show_icon: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropdownProps {
    pub style: VariantProps,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandProps {
    pub style: VariantProps,
    pub open: Option<String>,
    pub placeholder: String,
    pub empty_text: String,
    pub close_text: String,
    pub navigate_text: String,
    pub select_text: String,
    pub toggle_text: String,
    pub shortcut: String,
    pub disable_global_shortcut: bool,
    pub show_footer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvatarGroupProps {
    pub style: VariantProps,
    pub items: Option<String>,
    pub size: ButtonSize,
    pub max: Option<u16>,
    pub auto_fit: bool,
    pub inline: bool,
    pub bordered: bool,
}

impl AvatarGroupProps {
    pub fn visible_item_count(&self, item_count: usize) -> usize {
        self.max
            .map(|max| usize::from(max).min(item_count))
            .unwrap_or(item_count)
    }

    pub fn overflow_count(&self, item_count: usize) -> usize {
        item_count.saturating_sub(self.visible_item_count(item_count))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvatarGroupItem {
    pub src: Option<String>,
    pub name: Option<String>,
    pub alt: Option<String>,
    pub on_click: Option<String>,
    pub navigation: Option<NavigationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatBoxProps {
    pub style: VariantProps,
    pub messages: String,
    pub mode: ChatBoxMode,
    pub current_user_id: String,
    pub user_name: String,
    pub user_avatar: Option<String>,
    pub user_status: String,
    pub assistant_name: String,
    pub assistant_avatar: Option<String>,
    pub show_header: bool,
    pub placeholder: String,
    pub show_attachments: bool,
    pub show_voice_note: bool,
    pub show_camera: bool,
    pub loading: Option<String>,
    pub sending: Option<String>,
    pub streaming: Option<String>,
    pub has_more: Option<String>,
    pub on_send: Option<String>,
    pub on_load_more: Option<String>,
    pub on_stop: Option<String>,
    pub on_voice_note: Option<String>,
    pub on_file_attach: Option<String>,
    pub on_camera_capture: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptyProps {
    pub style: VariantProps,
    pub kind: EmptyKind,
    pub title: Option<String>,
    pub description: Option<String>,
    pub action_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarqueeProps {
    pub style: StyleProps,
    pub speed: MarqueeSpeed,
    pub pause_on_hover: bool,
    pub reverse: bool,
    pub orientation: MarqueeOrientation,
    pub fade: bool,
    pub fade_color: ColorToken,
    pub gap: ScaleValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeWriterProps {
    pub style: StyleProps,
    pub type_speed: u64,
    pub delete_speed: u64,
    pub after_typed: u64,
    pub after_deleted: u64,
    pub repeat: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeWriterItem {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichTextMark {
    pub text: String,
    pub style: RichTextMarkStyle,
    pub color: ColorFamily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RichTextMarkStyle {
    Mark,
    Grad,
    Pill,
    Slant,
    Glow,
    Under,
    Strike,
    Box,
    Wave,
    Neon,
    Pop,
    Tag,
}

impl RichTextMarkStyle {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "mark" => Some(Self::Mark),
            "grad" => Some(Self::Grad),
            "pill" => Some(Self::Pill),
            "slant" => Some(Self::Slant),
            "glow" => Some(Self::Glow),
            "under" => Some(Self::Under),
            "strike" => Some(Self::Strike),
            "box" => Some(Self::Box),
            "wave" => Some(Self::Wave),
            "neon" => Some(Self::Neon),
            "pop" => Some(Self::Pop),
            "tag" => Some(Self::Tag),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mark => "mark",
            Self::Grad => "grad",
            Self::Pill => "pill",
            Self::Slant => "slant",
            Self::Glow => "glow",
            Self::Under => "under",
            Self::Strike => "strike",
            Self::Box => "box",
            Self::Wave => "wave",
            Self::Neon => "neon",
            Self::Pop => "pop",
            Self::Tag => "tag",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Mark,
            Self::Grad,
            Self::Pill,
            Self::Slant,
            Self::Glow,
            Self::Under,
            Self::Strike,
            Self::Box,
            Self::Wave,
            Self::Neon,
            Self::Pop,
            Self::Tag,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordProps {
    pub style: VariantProps,
    pub name: String,
    pub url: Option<String>,
    pub disabled: bool,
    pub max_duration: Option<u16>,
    pub on_start: Option<String>,
    pub on_pause: Option<String>,
    pub on_resume: Option<String>,
    pub on_stop: Option<String>,
    pub on_discard: Option<String>,
    pub on_confirm: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleGroupProps {
    pub style: VariantProps,
    pub kind: ToggleGroupKind,
    pub pagination: Option<PaginationProps>,
    pub value: Option<String>,
    pub selected: String,
    pub size: ButtonSize,
    pub wide: bool,
    pub vertical: bool,
    pub disabled: bool,
    pub aria_label: Option<String>,
    pub on_change: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginationProps {
    pub total: PaginationTotal,
    pub page_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaginationTotal {
    Static(u32),
    Signal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleGroupKind {
    #[default]
    Selection,
    Pagination,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleGroupItem {
    pub id: String,
    pub label: String,
    pub icon: Option<ViewIcon>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollapsibleProps {
    pub style: VariantProps,
    pub label: String,
    pub default_open: bool,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountdownProps {
    pub style: VariantProps,
    pub target: String,
    pub show_days: bool,
    pub show_hours: bool,
    pub show_minutes: bool,
    pub show_seconds: bool,
    pub size: CountdownSize,
    pub days_label: String,
    pub hours_label: String,
    pub minutes_label: String,
    pub seconds_label: String,
    pub on_complete: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountdownSize {
    Sm,
    Md,
    Lg,
    Xl,
}

impl CountdownSize {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            "xl" => Some(Self::Xl),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Sm, Self::Md, Self::Lg, Self::Xl]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapProps {
    pub style: VariantProps,
    pub center_lat: String,
    pub center_lng: String,
    pub zoom: u16,
    pub height: String,
    pub width: String,
    pub show_controls: bool,
    pub show_scale: bool,
    pub show_location_control: bool,
    pub interactive: bool,
    pub route_start_lat: Option<String>,
    pub route_start_lng: Option<String>,
    pub route_end_lat: Option<String>,
    pub route_end_lng: Option<String>,
    pub on_location: Option<String>,
    pub on_location_error: Option<String>,
    pub on_route: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapMarker {
    pub id: String,
    pub lat: String,
    pub lng: String,
    pub label: Option<String>,
    pub popup: Option<String>,
    pub icon: MapMarkerIcon,
    pub on_click: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapWaypoint {
    pub lat: String,
    pub lng: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapMarkerIcon {
    Default,
    Start,
    End,
    Waypoint,
}

impl MapMarkerIcon {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "default" => Some(Self::Default),
            "start" => Some(Self::Start),
            "end" => Some(Self::End),
            "waypoint" => Some(Self::Waypoint),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Start => "start",
            Self::End => "end",
            Self::Waypoint => "waypoint",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioProps {
    pub style: VariantProps,
    pub src: String,
    pub subtitle: Option<String>,
    pub avatar_src: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageProps {
    pub style: VariantProps,
    pub src: String,
    pub alt: String,
    pub aspect: ImageAspect,
    pub object_fit: ImageObjectFit,
    pub loading: ImageLoading,
    pub hide_controls: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccordionProps {
    pub style: VariantProps,
    pub multiple: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccordionItem {
    pub id: String,
    pub label: String,
    pub disabled: bool,
    pub default_open: bool,
    pub children: Vec<ViewNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarouselProps {
    pub style: VariantProps,
    pub variant: CarouselVariant,
    pub autoplay: bool,
    pub autoplay_interval: u16,
    pub disable_loop: bool,
    pub hide_controls: bool,
    pub hide_indicators: bool,
    pub show_navigation: bool,
    pub show_counter: bool,
    pub orientation: CarouselOrientation,
    pub size: ButtonSize,
    pub indicator_type: CarouselIndicatorType,
    pub title: Option<String>,
    pub slide_width: Option<u16>,
    pub slide_height: Option<u16>,
    pub slides_per_view: u16,
    pub gap: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarouselSlide {
    pub id: String,
    pub children: Vec<ViewNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckboxProps {
    pub style: VariantProps,
    pub checked: bool,
    pub disabled: bool,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorProps {
    pub style: VariantProps,
    pub value: String,
    pub size: ButtonSize,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
    pub show_hex: bool,
    pub show_rgb: bool,
    pub show_cmyk: bool,
    pub show_oklch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateProps {
    pub style: VariantProps,
    pub value: Option<String>,
    pub size: ButtonSize,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
    pub min: Option<String>,
    pub max: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRangeProps {
    pub style: VariantProps,
    pub start: Option<String>,
    pub end: Option<String>,
    pub start_value: Option<String>,
    pub end_value: Option<String>,
    pub size: ButtonSize,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
    pub min: Option<String>,
    pub max: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioGroupProps {
    pub style: VariantProps,
    pub size: ButtonSize,
    pub orientation: RadioGroupOrientation,
    pub name: Option<String>,
    pub info: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadioGroupOrientation {
    Vertical,
    Horizontal,
}

impl RadioGroupOrientation {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "vertical" => Some(Self::Vertical),
            "horizontal" => Some(Self::Horizontal),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleProps {
    pub style: VariantProps,
    pub checked: bool,
    pub disabled: bool,
    pub name: Option<String>,
    pub label_left: Option<String>,
    pub label_right: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeToggleProps {
    pub style: VariantProps,
    pub light_label: String,
    pub dark_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeSelectProps {
    pub style: VariantProps,
    pub label: String,
    pub placeholder: String,
    pub themes: Vec<String>,
    pub default_theme: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabProps {
    pub style: VariantProps,
    pub position: OverlayCornerPosition,
    pub fixed: bool,
    pub offset_x: ScaleValue,
    pub offset_y: ScaleValue,
    pub icon: ViewIcon,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabAction {
    pub label: String,
    pub icon: ViewIcon,
    pub color: ColorFamily,
    pub on_click: Option<String>,
    pub navigation: Option<NavigationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliderProps {
    pub style: VariantProps,
    pub value: String,
    pub min: String,
    pub max: String,
    pub step: Option<String>,
    pub size: ButtonSize,
    pub name: Option<String>,
    pub hide_label: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropzoneProps {
    pub style: VariantProps,
    pub accept: Option<String>,
    pub multiple: bool,
    pub max_size: Option<u64>,
    pub size: ButtonSize,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComboBoxProps {
    pub style: VariantProps,
    pub value: Option<String>,
    pub search_placeholder: String,
    pub empty_text: String,
    pub loading_text: String,
    pub loading_more_text: String,
    pub clearable: bool,
    pub disabled: bool,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComboOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
    pub src: Option<String>,
    pub icon: Option<ViewIcon>,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvFieldProps {
    pub style: VariantProps,
    pub button_text: String,
    pub modal_title: String,
    pub instructions: String,
    pub cancel_text: String,
    pub confirm_text: String,
    pub clear_text: String,
    pub preview_title: String,
    pub multiple: bool,
    pub show_preview: bool,
    pub preview_rows: u16,
    pub preview_page_size: u16,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvColumn {
    pub name: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragDropProps {
    pub style: VariantProps,
    pub empty_text: String,
    pub direction: DragDropDirection,
    pub allow_group_transfer: bool,
    pub disabled: bool,
    pub size: ButtonSize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragItem {
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragGroup {
    pub id: String,
    pub title: Option<String>,
    pub items: Vec<DragItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragDropDirection {
    Horizontal,
    Vertical,
}

impl DragDropDirection {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "horizontal" => Some(Self::Horizontal),
            "vertical" => Some(Self::Vertical),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorProps {
    pub style: VariantProps,
    pub value: Option<String>,
    pub min_height: u16,
    pub hide_toolbar: bool,
    pub disabled: bool,
    pub readonly: bool,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCropperProps {
    pub style: VariantProps,
    pub src: Option<String>,
    pub alt: String,
    pub accept: String,
    pub aspect_ratio: Option<String>,
    pub min_width: u16,
    pub min_height: u16,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
    pub shape: ImageCropperShape,
    pub disabled: bool,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageCropperShape {
    Circle,
    Square,
}

impl ImageCropperShape {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "circle" => Some(Self::Circle),
            "square" => Some(Self::Square),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Circle => "circle",
            Self::Square => "square",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordFieldProps {
    pub style: VariantProps,
    pub value: Option<String>,
    pub hide_strength: bool,
    pub weak_label: String,
    pub medium_label: String,
    pub strong_label: String,
    pub disabled: bool,
    pub readonly: bool,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneFieldProps {
    pub style: VariantProps,
    pub value: Option<String>,
    pub country: Option<String>,
    pub dial_code_name: String,
    pub search_placeholder: String,
    pub empty_text: String,
    pub loading_text: String,
    pub priority_countries: Vec<String>,
    pub disabled: bool,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinFieldProps {
    pub style: VariantProps,
    pub value: Option<String>,
    pub length: u8,
    pub kind: PinFieldKind,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinFieldKind {
    Text,
    Password,
    Number,
}

impl PinFieldKind {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "text" => Some(Self::Text),
            "password" => Some(Self::Password),
            "number" => Some(Self::Number),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Password => "password",
            Self::Number => "number",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextareaProps {
    pub style: VariantProps,
    pub value: Option<String>,
    pub rows: u16,
    pub cols: Option<u16>,
    pub max_length: Option<u16>,
    pub resize: bool,
    pub disabled: bool,
    pub readonly: bool,
    pub name: Option<String>,
    pub help_text: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlayEntry {
    Item(OverlayItemProps),
    Divider,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayItemProps {
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<SideNavIcon>,
    pub on_click: Option<String>,
    pub navigation: Option<NavigationAction>,
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandEntry {
    Item(OverlayItemProps),
    Group {
        label: String,
        icon: Option<SideNavIcon>,
        items: Vec<OverlayItemProps>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabsProps {
    pub style: StyleProps,
    pub variant: TabsVariant,
    pub color: ColorFamily,
    pub position: TabsPosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabItem {
    pub id: String,
    pub label: String,
    pub i18n: Option<String>,
    pub children: Vec<ViewNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavMenuItem {
    Item(NavMenuItemProps),
    Submenu {
        props: NavMenuItemProps,
        items: Vec<NavMenuItemProps>,
    },
    Megamenu {
        props: NavMenuItemProps,
        content: Vec<ViewNode>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavMenuItemProps {
    pub label: String,
    pub i18n: Option<String>,
    pub description: Option<String>,
    pub description_i18n: Option<String>,
    pub icon: Option<SideNavIcon>,
    pub on_click: Option<String>,
    pub navigation: Option<NavigationAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsPosition {
    Top,
    Bottom,
    Start,
    End,
}

impl TabsPosition {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "top" => Some(Self::Top),
            "bottom" => Some(Self::Bottom),
            "start" => Some(Self::Start),
            "end" => Some(Self::End),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Start => "start",
            Self::End => "end",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Top, Self::Bottom, Self::Start, Self::End]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarStatus {
    Online,
    Offline,
    Busy,
    Away,
}

impl AvatarStatus {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "online" => Some(Self::Online),
            "offline" => Some(Self::Offline),
            "busy" => Some(Self::Busy),
            "away" => Some(Self::Away),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Offline => "offline",
            Self::Busy => "busy",
            Self::Away => "away",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Online, Self::Offline, Self::Busy, Self::Away]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayCornerPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl OverlayCornerPosition {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "top-left" => Some(Self::TopLeft),
            "top-right" => Some(Self::TopRight),
            "bottom-left" => Some(Self::BottomLeft),
            "bottom-right" => Some(Self::BottomRight),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::TopLeft => "top-left",
            Self::TopRight => "top-right",
            Self::BottomLeft => "bottom-left",
            Self::BottomRight => "bottom-right",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::TopLeft,
            Self::TopRight,
            Self::BottomLeft,
            Self::BottomRight,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayPosition {
    Top,
    Bottom,
    Start,
    End,
}

impl OverlayPosition {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "top" => Some(Self::Top),
            "bottom" => Some(Self::Bottom),
            "start" => Some(Self::Start),
            "end" => Some(Self::End),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Start => "start",
            Self::End => "end",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Top, Self::Bottom, Self::Start, Self::End]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewIcon {
    Plus,
    Link,
    Edit,
    Trash,
    Search,
    Settings,
    Upload,
    File,
    Dismiss,
    Moon,
    Sun,
}

impl ViewIcon {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "plus" => Some(Self::Plus),
            "link" => Some(Self::Link),
            "edit" => Some(Self::Edit),
            "trash" => Some(Self::Trash),
            "search" => Some(Self::Search),
            "settings" => Some(Self::Settings),
            "upload" => Some(Self::Upload),
            "file" => Some(Self::File),
            "dismiss" => Some(Self::Dismiss),
            "moon" => Some(Self::Moon),
            "sun" => Some(Self::Sun),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plus => "plus",
            Self::Link => "link",
            Self::Edit => "edit",
            Self::Trash => "trash",
            Self::Search => "search",
            Self::Settings => "settings",
            Self::Upload => "upload",
            Self::File => "file",
            Self::Dismiss => "dismiss",
            Self::Moon => "moon",
            Self::Sun => "sun",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Plus,
            Self::Link,
            Self::Edit,
            Self::Trash,
            Self::Search,
            Self::Settings,
            Self::Upload,
            Self::File,
            Self::Dismiss,
            Self::Moon,
            Self::Sun,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkeletonVariant {
    Text,
    Circular,
    Rectangular,
    Rounded,
}

impl SkeletonVariant {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "text" => Some(Self::Text),
            "circular" => Some(Self::Circular),
            "rectangular" => Some(Self::Rectangular),
            "rounded" => Some(Self::Rounded),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Circular => "circular",
            Self::Rectangular => "rectangular",
            Self::Rounded => "rounded",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Text,
            Self::Circular,
            Self::Rectangular,
            Self::Rounded,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkeletonAnimation {
    Pulse,
    Wave,
    None,
}

impl SkeletonAnimation {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "pulse" => Some(Self::Pulse),
            "wave" => Some(Self::Wave),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pulse => "pulse",
            Self::Wave => "wave",
            Self::None => "none",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Pulse, Self::Wave, Self::None]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Primary,
    Secondary,
    Muted,
    Success,
    Info,
    Warning,
    Danger,
    Error,
}

impl ToastKind {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "primary" => Some(Self::Primary),
            "secondary" => Some(Self::Secondary),
            "muted" => Some(Self::Muted),
            "success" => Some(Self::Success),
            "info" => Some(Self::Info),
            "warning" => Some(Self::Warning),
            "danger" => Some(Self::Danger),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Muted => "muted",
            Self::Success => "success",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Danger => "danger",
            Self::Error => "error",
        }
    }

    pub fn color(self) -> ColorFamily {
        match self {
            Self::Primary => ColorFamily::Primary,
            Self::Secondary => ColorFamily::Secondary,
            Self::Muted => ColorFamily::Muted,
            Self::Success => ColorFamily::Success,
            Self::Info => ColorFamily::Info,
            Self::Warning => ColorFamily::Warning,
            Self::Danger | Self::Error => ColorFamily::Danger,
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Primary,
            Self::Secondary,
            Self::Muted,
            Self::Success,
            Self::Info,
            Self::Warning,
            Self::Danger,
            Self::Error,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatBoxMode {
    Conversation,
    Prompt,
}

impl ChatBoxMode {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "conversation" => Some(Self::Conversation),
            "prompt" => Some(Self::Prompt),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Conversation => "conversation",
            Self::Prompt => "prompt",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Conversation, Self::Prompt]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyKind {
    Playlist,
    Result,
    Data,
    Template,
}

impl EmptyKind {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "playlist" => Some(Self::Playlist),
            "result" => Some(Self::Result),
            "data" => Some(Self::Data),
            "template" => Some(Self::Template),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Playlist => "playlist",
            Self::Result => "result",
            Self::Data => "data",
            Self::Template => "template",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Playlist, Self::Result, Self::Data, Self::Template]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarqueeSpeed {
    Slow,
    Normal,
    Fast,
}

impl MarqueeSpeed {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "slow" => Some(Self::Slow),
            "normal" => Some(Self::Normal),
            "fast" => Some(Self::Fast),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Slow => "slow",
            Self::Normal => "normal",
            Self::Fast => "fast",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Slow, Self::Normal, Self::Fast]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarqueeOrientation {
    Horizontal,
    Vertical,
}

impl MarqueeOrientation {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "horizontal" => Some(Self::Horizontal),
            "vertical" => Some(Self::Vertical),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Horizontal, Self::Vertical]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsVariant {
    Solid,
    Outlined,
    Line,
    Ghost,
    Pills,
}

impl TabsVariant {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "solid" => Some(Self::Solid),
            "outlined" | "outline" => Some(Self::Outlined),
            "line" => Some(Self::Line),
            "ghost" => Some(Self::Ghost),
            "pills" => Some(Self::Pills),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Outlined => "outlined",
            Self::Line => "line",
            Self::Ghost => "ghost",
            Self::Pills => "pills",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Solid,
            Self::Outlined,
            Self::Line,
            Self::Ghost,
            Self::Pills,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SideNavItem {
    Header(SideNavItemProps),
    Item(SideNavItemProps),
    Divider,
    Submenu {
        props: SideNavItemProps,
        open: bool,
        bordered: bool,
        items: Vec<SideNavItemProps>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideNavItemProps {
    pub label: String,
    pub i18n: Option<String>,
    pub description: Option<String>,
    pub description_i18n: Option<String>,
    pub status: Option<String>,
    pub status_i18n: Option<String>,
    pub icon: Option<SideNavIcon>,
    pub on_click: Option<String>,
    pub navigation: Option<NavigationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RailNavItem {
    Item(RailNavItemProps),
    Divider,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RailNavItemProps {
    pub label: String,
    pub i18n: Option<String>,
    pub icon: SideNavIcon,
    pub on_click: Option<String>,
    pub navigation: Option<NavigationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideNavIcon {
    pub props: SvgProps,
    pub paths: Vec<SvgPath>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgViewBox {
    pub min_x: String,
    pub min_y: String,
    pub width: String,
    pub height: String,
}

impl SvgViewBox {
    pub fn as_str(&self) -> String {
        format!(
            "{} {} {} {}",
            self.min_x, self.min_y, self.width, self.height
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgPath {
    pub data: String,
    pub fill: SvgPathFill,
    pub transform: Option<SvgTransform>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgTransform {
    pub a: String,
    pub b: String,
    pub c: String,
    pub d: String,
    pub e: String,
    pub f: String,
}

impl SvgTransform {
    pub fn as_str(&self) -> String {
        format!(
            "matrix({} {} {} {} {} {})",
            self.a, self.b, self.c, self.d, self.e, self.f
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOptionEach {
    pub item: String,
    pub collection: String,
    pub key: String,
    pub value: String,
    pub label: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeProps {
    pub style: VariantProps,
    pub language: CodeLanguage,
    pub source: String,
    pub tokens: Vec<CodeToken>,
    pub copy_label: String,
    pub copied_label: String,
    pub template_segments: Vec<CodeTemplateSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeTemplateSegment {
    Static {
        text: String,
        tokens: Vec<CodeToken>,
    },
    Binding(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoProps {
    pub style: VariantProps,
    pub src: String,
    pub poster: Option<String>,
    pub autoplay: bool,
    pub aspect: VideoAspect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IframeProps {
    pub style: StyleProps,
    pub src: String,
    pub title: String,
    pub loading: IframeLoading,
    pub allow: Vec<String>,
    pub sandbox: Option<Vec<String>>,
    pub allow_fullscreen: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceProps {
    pub style: StyleProps,
    pub device: DeviceProfile,
    pub options: Vec<DeviceOption>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceOption {
    pub profile: DeviceProfile,
    pub icon: SideNavIcon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceProfile {
    Mobile,
    Tablet,
    Laptop,
    Monitor,
}

impl DeviceProfile {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "mobile" => Some(Self::Mobile),
            "tablet" => Some(Self::Tablet),
            "laptop" => Some(Self::Laptop),
            "monitor" => Some(Self::Monitor),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mobile => "mobile",
            Self::Tablet => "tablet",
            Self::Laptop => "laptop",
            Self::Monitor => "monitor",
        }
    }

    pub fn dimensions(self) -> (u16, u16) {
        match self {
            Self::Mobile => (390, 844),
            Self::Tablet => (768, 1024),
            Self::Laptop => (1440, 900),
            Self::Monitor => (1920, 1080),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IframeLoading {
    Lazy,
    Eager,
}

impl IframeLoading {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "lazy" => Some(Self::Lazy),
            "eager" => Some(Self::Eager),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lazy => "lazy",
            Self::Eager => "eager",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Lazy, Self::Eager]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanvasProps {
    pub style: StyleProps,
    pub scene: String,
    pub view_width: u16,
    pub view_height: u16,
    pub fit: CanvasFit,
    pub fps: u8,
    pub autoplay: bool,
    pub background: CanvasBackground,
    pub pixelated: bool,
    pub label: String,
    pub on_pointer: Option<String>,
    pub on_key: Option<String>,
    pub on_motion: Option<String>,
    pub motion_rate: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasFit {
    Contain,
    Cover,
    Stretch,
}

impl CanvasFit {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "contain" => Some(Self::Contain),
            "cover" => Some(Self::Cover),
            "stretch" => Some(Self::Stretch),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Contain => "contain",
            Self::Cover => "cover",
            Self::Stretch => "stretch",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasBackground {
    Transparent,
    Color(ColorToken),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandlestickProps {
    pub style: VariantProps,
    pub data: String,
    pub stream: Option<String>,
    pub up_color: ColorToken,
    pub down_color: ColorToken,
    pub empty_label: String,
    pub max_points: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartCommonProps {
    pub style: VariantProps,
    pub data: Option<String>,
    pub series: Option<String>,
    pub size: ChartSize,
    pub palette: ChartPalette,
    pub legend_position: ChartLegendPosition,
    pub empty_label: String,
    pub loading: bool,
    pub hide_legend: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcChartProps {
    pub common: ChartCommonProps,
    pub center_text: Option<String>,
    pub center_value: Option<String>,
    pub thickness: u16,
    pub gap: u16,
    pub start_angle: i16,
    pub end_angle: i16,
    pub show_inline_labels: bool,
    pub hide_values: bool,
    pub show_glow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AreaChartProps {
    pub common: ChartCommonProps,
    pub curve: ChartCurve,
    pub stroke_width: u16,
    pub fill_opacity: u16,
    pub stacked: bool,
    pub hide_line: bool,
    pub show_points: bool,
    pub hide_grid: bool,
    pub hide_x_axis: bool,
    pub hide_y_axis: bool,
    pub show_glow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarChartProps {
    pub common: ChartCommonProps,
    pub grouped: bool,
    pub stacked: bool,
    pub show_values: bool,
    pub bar_radius: u16,
    pub hide_grid: bool,
    pub show_glow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineChartProps {
    pub common: ChartCommonProps,
    pub curve: ChartCurve,
    pub stroke_width: u16,
    pub point_radius: u16,
    pub hide_points: bool,
    pub hide_grid: bool,
    pub hide_x_axis: bool,
    pub hide_y_axis: bool,
    pub show_gradient_fill: bool,
    pub show_glow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PieChartProps {
    pub common: ChartCommonProps,
    pub donut: bool,
    pub donut_width: u16,
    pub center_label: Option<String>,
    pub center_value: Option<String>,
    pub start_angle: i16,
    pub pad_angle: u16,
    pub hide_labels: bool,
    pub hide_values: bool,
    pub hide_percentages: bool,
    pub show_glow: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartSize {
    Sm,
    Md,
    Lg,
    Xl,
}

impl ChartSize {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            "xl" => Some(Self::Xl),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }

    pub fn circular_height(self) -> ScaleValue {
        match self {
            Self::Sm => ScaleValue::from_half_steps(40),
            Self::Md => ScaleValue::from_half_steps(56),
            Self::Lg => ScaleValue::from_half_steps(75),
            Self::Xl => ScaleValue::from_half_steps(100),
        }
    }

    pub fn cartesian_height(self) -> ScaleValue {
        match self {
            Self::Sm => ScaleValue::from_half_steps(50),
            Self::Md => ScaleValue::from_half_steps(75),
            Self::Lg => ScaleValue::from_half_steps(100),
            Self::Xl => ScaleValue::from_half_steps(125),
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Sm, Self::Md, Self::Lg, Self::Xl]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartPalette {
    Default,
    Rainbow,
    Ocean,
    Sunset,
    Forest,
    Neon,
}

impl ChartPalette {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "default" => Some(Self::Default),
            "rainbow" => Some(Self::Rainbow),
            "ocean" => Some(Self::Ocean),
            "sunset" => Some(Self::Sunset),
            "forest" => Some(Self::Forest),
            "neon" => Some(Self::Neon),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Rainbow => "rainbow",
            Self::Ocean => "ocean",
            Self::Sunset => "sunset",
            Self::Forest => "forest",
            Self::Neon => "neon",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Default,
            Self::Rainbow,
            Self::Ocean,
            Self::Sunset,
            Self::Forest,
            Self::Neon,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartLegendPosition {
    Top,
    Right,
    Bottom,
    Left,
    None,
}

impl ChartLegendPosition {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "top" => Some(Self::Top),
            "right" => Some(Self::Right),
            "bottom" => Some(Self::Bottom),
            "left" => Some(Self::Left),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Right => "right",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::None => "none",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Top, Self::Right, Self::Bottom, Self::Left, Self::None]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartCurve {
    Linear,
    Smooth,
}

impl ChartCurve {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "linear" => Some(Self::Linear),
            "smooth" => Some(Self::Smooth),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Smooth => "smooth",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Linear, Self::Smooth]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableProps {
    pub style: VariantProps,
    pub data: String,
    pub columns: Vec<TableColumn>,
    pub size: TableSize,
    pub striped: bool,
    pub bordered: bool,
    pub dividers: bool,
    pub empty_title: String,
    pub empty_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableColumn {
    pub field: String,
    pub label: String,
    pub align: TableColumnAlign,
    pub width: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DividerProps {
    pub style: StyleProps,
    pub orientation: DividerOrientation,
    pub color: ColorFamily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DividerOrientation {
    Horizontal,
    Vertical,
}

impl DividerOrientation {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "horizontal" => Some(Self::Horizontal),
            "vertical" => Some(Self::Vertical),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Horizontal, Self::Vertical]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageAspect {
    Horizontal,
    Vertical,
    Square,
    Auto,
}

impl ImageAspect {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "horizontal" => Some(Self::Horizontal),
            "vertical" => Some(Self::Vertical),
            "square" => Some(Self::Square),
            "auto" => Some(Self::Auto),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
            Self::Square => "square",
            Self::Auto => "auto",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Horizontal, Self::Vertical, Self::Square, Self::Auto]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageObjectFit {
    Cover,
    Contain,
    Fill,
    None,
}

impl ImageObjectFit {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "cover" => Some(Self::Cover),
            "contain" => Some(Self::Contain),
            "fill" => Some(Self::Fill),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cover => "cover",
            Self::Contain => "contain",
            Self::Fill => "fill",
            Self::None => "none",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Cover, Self::Contain, Self::Fill, Self::None]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageLoading {
    Lazy,
    Eager,
}

impl ImageLoading {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "lazy" => Some(Self::Lazy),
            "eager" => Some(Self::Eager),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lazy => "lazy",
            Self::Eager => "eager",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Lazy, Self::Eager]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselOrientation {
    Horizontal,
    Vertical,
}

impl CarouselOrientation {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "horizontal" => Some(Self::Horizontal),
            "vertical" => Some(Self::Vertical),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Horizontal, Self::Vertical]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselVariant {
    Simple,
    Snapping,
    Masonry,
    Rtl,
    Sticky,
    Controls,
    Dots,
    Thumbnails,
    CoverFlow,
    Slideshow,
    Stories,
    SmartStack,
    CardStack,
    Flipbook,
}

impl CarouselVariant {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "simple" => Some(Self::Simple),
            "snapping" => Some(Self::Snapping),
            "masonry" => Some(Self::Masonry),
            "rtl" => Some(Self::Rtl),
            "sticky" => Some(Self::Sticky),
            "controls" => Some(Self::Controls),
            "dots" => Some(Self::Dots),
            "thumbnails" => Some(Self::Thumbnails),
            "coverFlow" => Some(Self::CoverFlow),
            "slideshow" => Some(Self::Slideshow),
            "stories" => Some(Self::Stories),
            "smartStack" => Some(Self::SmartStack),
            "cardStack" => Some(Self::CardStack),
            "flipbook" => Some(Self::Flipbook),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Snapping => "snapping",
            Self::Masonry => "masonry",
            Self::Rtl => "rtl",
            Self::Sticky => "sticky",
            Self::Controls => "controls",
            Self::Dots => "dots",
            Self::Thumbnails => "thumbnails",
            Self::CoverFlow => "coverFlow",
            Self::Slideshow => "slideshow",
            Self::Stories => "stories",
            Self::SmartStack => "smartStack",
            Self::CardStack => "cardStack",
            Self::Flipbook => "flipbook",
        }
    }

    pub fn class_name(self) -> &'static str {
        match self {
            Self::CoverFlow => "cover-flow",
            Self::SmartStack => "smart-stack",
            Self::CardStack => "card-stack",
            _ => self.as_str(),
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Simple,
            Self::Snapping,
            Self::Masonry,
            Self::Rtl,
            Self::Sticky,
            Self::Controls,
            Self::Dots,
            Self::Thumbnails,
            Self::CoverFlow,
            Self::Slideshow,
            Self::Stories,
            Self::SmartStack,
            Self::CardStack,
            Self::Flipbook,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselIndicatorType {
    Bar,
    Dot,
}

impl CarouselIndicatorType {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "bar" => Some(Self::Bar),
            "dot" => Some(Self::Dot),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bar => "bar",
            Self::Dot => "dot",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Bar, Self::Dot]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoAspect {
    Horizontal,
    Vertical,
    Square,
}

impl VideoAspect {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "horizontal" => Some(Self::Horizontal),
            "vertical" => Some(Self::Vertical),
            "square" => Some(Self::Square),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
            Self::Square => "square",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Horizontal, Self::Vertical, Self::Square]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeToken {
    pub kind: CodeTokenKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeTokenKind {
    Plain,
    Keyword,
    Type,
    String,
    Number,
    Attribute,
    Comment,
    Punctuation,
}

impl CodeTokenKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Keyword => "keyword",
            Self::Type => "type",
            Self::String => "string",
            Self::Number => "number",
            Self::Attribute => "attribute",
            Self::Comment => "comment",
            Self::Punctuation => "punctuation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeLanguage {
    Dowe,
    TypeScript,
    JavaScript,
    Go,
    Rust,
    Python,
}

impl CodeLanguage {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "dowe" => Some(Self::Dowe),
            "typescript" => Some(Self::TypeScript),
            "javascript" => Some(Self::JavaScript),
            "go" => Some(Self::Go),
            "rust" => Some(Self::Rust),
            "python" => Some(Self::Python),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dowe => "dowe",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Go => "go",
            Self::Rust => "rust",
            Self::Python => "python",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Dowe,
            Self::TypeScript,
            Self::JavaScript,
            Self::Go,
            Self::Rust,
            Self::Python,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisibilityCondition {
    Static(ResponsiveValue<bool>),
    Signal(String),
    NumberComparison {
        path: String,
        comparison: ReactiveNumberComparison,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgPathFill {
    None,
    CurrentColor,
    Color(ColorToken),
    RawFill {
        color: &'static str,
        opacity: u8,
        even_odd: bool,
    },
    Fill {
        color: Option<ColorToken>,
        opacity: u8,
        even_odd: bool,
    },
    RawStroke {
        color: &'static str,
        opacity: u8,
        width: u16,
        line_cap: SvgLineCap,
        line_join: SvgLineJoin,
    },
    LiteralFill {
        red: u8,
        green: u8,
        blue: u8,
        opacity: u8,
        even_odd: bool,
    },
    LiteralStroke {
        red: u8,
        green: u8,
        blue: u8,
        opacity: u8,
        width: u16,
        line_cap: SvgLineCap,
        line_join: SvgLineJoin,
    },
    Stroke {
        color: Option<ColorToken>,
        opacity: u8,
        width: u16,
        line_cap: SvgLineCap,
        line_join: SvgLineJoin,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgLineCap {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgLineJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Success,
    Error,
    Info,
    Warning,
}

impl AlertKind {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "success" => Some(Self::Success),
            "error" => Some(Self::Error),
            "info" => Some(Self::Info),
            "warning" => Some(Self::Warning),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Error => "error",
            Self::Info => "info",
            Self::Warning => "warning",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Success, Self::Error, Self::Info, Self::Warning]
    }

    pub fn color(self) -> ColorFamily {
        match self {
            Self::Success => ColorFamily::Success,
            Self::Error => ColorFamily::Danger,
            Self::Info => ColorFamily::Info,
            Self::Warning => ColorFamily::Warning,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontConfig {
    pub default_family: FontFamily,
    pub install: Vec<FontFamily>,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            default_family: FontFamily::Inter,
            install: Vec::new(),
        }
    }
}

impl FontConfig {
    pub fn effective_families(&self, used: &BTreeSet<FontFamily>) -> BTreeSet<FontFamily> {
        let mut fonts = BTreeSet::new();
        fonts.insert(self.default_family);
        fonts.extend(self.install.iter().copied());
        fonts.extend(used.iter().copied());
        fonts
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpacingProps {
    pub p: Option<ResponsiveValue<ScaleValue>>,
    pub px: Option<ResponsiveValue<ScaleValue>>,
    pub py: Option<ResponsiveValue<ScaleValue>>,
    pub pl: Option<ResponsiveValue<ScaleValue>>,
    pub pr: Option<ResponsiveValue<ScaleValue>>,
    pub pt: Option<ResponsiveValue<ScaleValue>>,
    pub pb: Option<ResponsiveValue<ScaleValue>>,
}

impl SpacingProps {
    pub fn with_horizontal_padding_default(&self, value: ResponsiveValue<ScaleValue>) -> Self {
        if self.p.is_some() {
            return self.clone();
        }

        let mut spacing = self.clone();
        if spacing.px.is_none() {
            match (spacing.pl.is_some(), spacing.pr.is_some()) {
                (false, false) => spacing.px = Some(value.clone()),
                (false, true) => spacing.pl = Some(value),
                (true, false) => spacing.pr = Some(value),
                (true, true) => {}
            }
        }
        spacing
    }

    pub fn with_padding_default(&self, value: ResponsiveValue<ScaleValue>) -> Self {
        if self.p.is_some() {
            return self.clone();
        }
        if self.px.is_none()
            && self.py.is_none()
            && self.pl.is_none()
            && self.pr.is_none()
            && self.pt.is_none()
            && self.pb.is_none()
        {
            return Self {
                p: Some(value),
                ..Default::default()
            };
        }

        let mut spacing = self.clone();
        if spacing.px.is_none() {
            match (spacing.pl.is_some(), spacing.pr.is_some()) {
                (false, false) => spacing.px = Some(value.clone()),
                (false, true) => spacing.pl = Some(value.clone()),
                (true, false) => spacing.pr = Some(value.clone()),
                (true, true) => {}
            }
        }
        if spacing.py.is_none() {
            match (spacing.pt.is_some(), spacing.pb.is_some()) {
                (false, false) => spacing.py = Some(value),
                (false, true) => spacing.pt = Some(value),
                (true, false) => spacing.pb = Some(value),
                (true, true) => {}
            }
        }
        spacing
    }

    pub fn with_padding_axis_defaults(
        &self,
        horizontal: ResponsiveValue<ScaleValue>,
        vertical: ResponsiveValue<ScaleValue>,
    ) -> Self {
        if self.p.is_some() {
            return self.clone();
        }

        let mut spacing = self.clone();
        if spacing.px.is_none() {
            match (spacing.pl.is_some(), spacing.pr.is_some()) {
                (false, false) => spacing.px = Some(horizontal.clone()),
                (false, true) => spacing.pl = Some(horizontal.clone()),
                (true, false) => spacing.pr = Some(horizontal),
                (true, true) => {}
            }
        }
        if spacing.py.is_none() {
            match (spacing.pt.is_some(), spacing.pb.is_some()) {
                (false, false) => spacing.py = Some(vertical.clone()),
                (false, true) => spacing.pt = Some(vertical.clone()),
                (true, false) => spacing.pb = Some(vertical),
                (true, true) => {}
            }
        }
        spacing
    }
}

pub fn section_content_spacing(spacing: &SpacingProps) -> SpacingProps {
    spacing.with_padding_axis_defaults(
        ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: ScaleValue::from_half_steps(8),
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: ScaleValue::from_half_steps(12),
            },
        ]),
        ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: ScaleValue::from_half_steps(20),
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: ScaleValue::from_half_steps(32),
            },
        ]),
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SizingProps {
    pub w: Option<ResponsiveValue<SizeValue>>,
    pub h: Option<ResponsiveValue<SizeValue>>,
    pub min_w: Option<ResponsiveValue<SizeValue>>,
    pub min_h: Option<ResponsiveValue<SizeValue>>,
    pub max_w: Option<ResponsiveValue<SizeValue>>,
    pub max_h: Option<ResponsiveValue<SizeValue>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GridItemProps {
    pub col_span: Option<ResponsiveValue<GridSpan>>,
    pub row_span: Option<ResponsiveValue<GridSpan>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponsiveValue<T> {
    pub entries: Vec<ResponsiveEntry<T>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponsiveEntry<T> {
    pub breakpoint: Breakpoint,
    pub value: T,
}

impl<T> ResponsiveValue<T> {
    pub fn scalar(value: T) -> Self {
        Self {
            entries: vec![ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value,
            }],
        }
    }
}

impl<T> ResponsiveValue<T> {
    pub fn ordered(mut entries: Vec<ResponsiveEntry<T>>) -> Self {
        entries.sort_by_key(|entry| entry.breakpoint.order());
        let mut unique = Vec::new();

        for entry in entries {
            if let Some(index) = unique
                .iter()
                .position(|existing: &ResponsiveEntry<T>| existing.breakpoint == entry.breakpoint)
            {
                unique[index] = entry;
            } else {
                unique.push(entry);
            }
        }

        Self { entries: unique }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentProp {
    pub name: String,
    pub value: PropValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropValue {
    String(String),
    Number(String),
    Boolean(bool),
    Responsive(Vec<ResponsivePropEntry>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponsivePropEntry {
    pub breakpoint: String,
    pub value: PropScalar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropScalar {
    String(String),
    Number(String),
    Boolean(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Breakpoint {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

impl Breakpoint {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "xs" => Some(Self::Xs),
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            "xl" => Some(Self::Xl),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }

    pub fn min_width(self) -> u16 {
        match self {
            Self::Xs => 0,
            Self::Sm => 640,
            Self::Md => 768,
            Self::Lg => 1024,
            Self::Xl => 1280,
        }
    }

    fn order(self) -> u8 {
        match self {
            Self::Xs => 0,
            Self::Sm => 1,
            Self::Md => 2,
            Self::Lg => 3,
            Self::Xl => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinComponent {
    Box,
    Section,
    Flex,
    Grid,
    Input,
    Select,
    Option,
    Code,
    Video,
    Iframe,
    Device,
    Canvas,
    Candlestick,
    ArcChart,
    AreaChart,
    BarChart,
    LineChart,
    PieChart,
    Table,
    Divider,
    Button,
    Brand,
    Banner,
    IconButton,
    ToggleTheme,
    SelectTheme,
    Fab,
    FabAction,
    Slider,
    Dropzone,
    ComboBox,
    ComboOption,
    CsvField,
    CsvColumn,
    DragDrop,
    DragGroup,
    DragItem,
    Editor,
    ImageCropper,
    PasswordField,
    PhoneField,
    PinField,
    Textarea,
    Alert,
    Icon,
    Svg,
    Path,
    AppBar,
    Footer,
    BottomBar,
    NavMenu,
    SideNav,
    RailNav,
    Sidebar,
    Scaffold,
    Splash,
    Drawer,
    Avatar,
    Badge,
    Chip,
    Skeleton,
    Modal,
    AlertDialog,
    Tooltip,
    Toast,
    Dropdown,
    Command,
    AvatarGroup,
    ChatBox,
    Empty,
    Marquee,
    TypeWriter,
    RichText,
    Record,
    ToggleGroup,
    Collapsible,
    Countdown,
    Map,
    Audio,
    Image,
    Accordion,
    Carousel,
    Checkbox,
    Color,
    Date,
    DateRange,
    RadioGroup,
    Toggle,
    Card,
    Tabs,
    Tab,
    Title,
    Text,
}

impl BuiltinComponent {
    pub const ALL: &'static [Self] = &[
        Self::Box,
        Self::Section,
        Self::Flex,
        Self::Grid,
        Self::Input,
        Self::Select,
        Self::Option,
        Self::Code,
        Self::Video,
        Self::Iframe,
        Self::Device,
        Self::Canvas,
        Self::Candlestick,
        Self::ArcChart,
        Self::AreaChart,
        Self::BarChart,
        Self::LineChart,
        Self::PieChart,
        Self::Table,
        Self::Divider,
        Self::Button,
        Self::Brand,
        Self::Banner,
        Self::IconButton,
        Self::ToggleTheme,
        Self::SelectTheme,
        Self::Fab,
        Self::FabAction,
        Self::Slider,
        Self::Dropzone,
        Self::ComboBox,
        Self::ComboOption,
        Self::CsvField,
        Self::CsvColumn,
        Self::DragDrop,
        Self::DragGroup,
        Self::DragItem,
        Self::Editor,
        Self::ImageCropper,
        Self::PasswordField,
        Self::PhoneField,
        Self::PinField,
        Self::Textarea,
        Self::Alert,
        Self::Icon,
        Self::Svg,
        Self::Path,
        Self::AppBar,
        Self::Footer,
        Self::BottomBar,
        Self::NavMenu,
        Self::SideNav,
        Self::RailNav,
        Self::Sidebar,
        Self::Scaffold,
        Self::Splash,
        Self::Drawer,
        Self::Avatar,
        Self::Badge,
        Self::Chip,
        Self::Skeleton,
        Self::Modal,
        Self::AlertDialog,
        Self::Tooltip,
        Self::Toast,
        Self::Dropdown,
        Self::Command,
        Self::AvatarGroup,
        Self::ChatBox,
        Self::Empty,
        Self::Marquee,
        Self::TypeWriter,
        Self::RichText,
        Self::Record,
        Self::ToggleGroup,
        Self::Collapsible,
        Self::Countdown,
        Self::Map,
        Self::Audio,
        Self::Image,
        Self::Accordion,
        Self::Carousel,
        Self::Checkbox,
        Self::Color,
        Self::Date,
        Self::DateRange,
        Self::RadioGroup,
        Self::Toggle,
        Self::Card,
        Self::Tabs,
        Self::Tab,
        Self::Title,
        Self::Text,
    ];

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Box" => Some(Self::Box),
            "Section" => Some(Self::Section),
            "Flex" => Some(Self::Flex),
            "Grid" => Some(Self::Grid),
            "Input" => Some(Self::Input),
            "Select" => Some(Self::Select),
            "Option" => Some(Self::Option),
            "Code" => Some(Self::Code),
            "Video" => Some(Self::Video),
            "Iframe" => Some(Self::Iframe),
            "Device" => Some(Self::Device),
            "Canvas" => Some(Self::Canvas),
            "Candlestick" => Some(Self::Candlestick),
            "ArcChart" => Some(Self::ArcChart),
            "AreaChart" => Some(Self::AreaChart),
            "BarChart" => Some(Self::BarChart),
            "LineChart" => Some(Self::LineChart),
            "PieChart" => Some(Self::PieChart),
            "Table" => Some(Self::Table),
            "Divider" => Some(Self::Divider),
            "Button" => Some(Self::Button),
            "Brand" => Some(Self::Brand),
            "Banner" => Some(Self::Banner),
            "IconButton" => Some(Self::IconButton),
            "ToggleTheme" => Some(Self::ToggleTheme),
            "SelectTheme" => Some(Self::SelectTheme),
            "Fab" => Some(Self::Fab),
            "fabAction" => Some(Self::FabAction),
            "Slider" => Some(Self::Slider),
            "Dropzone" => Some(Self::Dropzone),
            "ComboBox" => Some(Self::ComboBox),
            "comboOption" => Some(Self::ComboOption),
            "CsvField" => Some(Self::CsvField),
            "csvColumn" => Some(Self::CsvColumn),
            "DragDrop" => Some(Self::DragDrop),
            "dragGroup" => Some(Self::DragGroup),
            "dragItem" => Some(Self::DragItem),
            "Editor" => Some(Self::Editor),
            "ImageCropper" => Some(Self::ImageCropper),
            "PasswordField" => Some(Self::PasswordField),
            "PhoneField" => Some(Self::PhoneField),
            "PinField" => Some(Self::PinField),
            "Textarea" => Some(Self::Textarea),
            "Alert" => Some(Self::Alert),
            "Icon" => Some(Self::Icon),
            "Svg" => Some(Self::Svg),
            "Path" => Some(Self::Path),
            "AppBar" => Some(Self::AppBar),
            "Footer" => Some(Self::Footer),
            "BottomBar" => Some(Self::BottomBar),
            "NavMenu" => Some(Self::NavMenu),
            "SideNav" => Some(Self::SideNav),
            "RailNav" => Some(Self::RailNav),
            "Sidebar" => Some(Self::Sidebar),
            "Scaffold" => Some(Self::Scaffold),
            "Splash" => Some(Self::Splash),
            "Drawer" => Some(Self::Drawer),
            "Avatar" => Some(Self::Avatar),
            "Badge" => Some(Self::Badge),
            "Chip" => Some(Self::Chip),
            "Skeleton" => Some(Self::Skeleton),
            "Modal" => Some(Self::Modal),
            "AlertDialog" => Some(Self::AlertDialog),
            "Tooltip" => Some(Self::Tooltip),
            "Toast" => Some(Self::Toast),
            "Dropdown" => Some(Self::Dropdown),
            "Command" => Some(Self::Command),
            "AvatarGroup" => Some(Self::AvatarGroup),
            "ChatBox" => Some(Self::ChatBox),
            "Empty" => Some(Self::Empty),
            "Marquee" => Some(Self::Marquee),
            "TypeWriter" => Some(Self::TypeWriter),
            "RichText" => Some(Self::RichText),
            "Record" => Some(Self::Record),
            "ToggleGroup" => Some(Self::ToggleGroup),
            "Collapsible" => Some(Self::Collapsible),
            "Countdown" => Some(Self::Countdown),
            "Map" => Some(Self::Map),
            "Audio" => Some(Self::Audio),
            "Image" => Some(Self::Image),
            "Accordion" => Some(Self::Accordion),
            "Carousel" => Some(Self::Carousel),
            "Checkbox" => Some(Self::Checkbox),
            "Color" => Some(Self::Color),
            "Date" => Some(Self::Date),
            "DateRange" => Some(Self::DateRange),
            "RadioGroup" => Some(Self::RadioGroup),
            "Toggle" => Some(Self::Toggle),
            "Card" => Some(Self::Card),
            "Tabs" => Some(Self::Tabs),
            "tab" => Some(Self::Tab),
            "Title" => Some(Self::Title),
            "Text" => Some(Self::Text),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Box => "Box",
            Self::Section => "Section",
            Self::Flex => "Flex",
            Self::Grid => "Grid",
            Self::Input => "Input",
            Self::Select => "Select",
            Self::Option => "Option",
            Self::Code => "Code",
            Self::Video => "Video",
            Self::Iframe => "Iframe",
            Self::Device => "Device",
            Self::Canvas => "Canvas",
            Self::Candlestick => "Candlestick",
            Self::ArcChart => "ArcChart",
            Self::AreaChart => "AreaChart",
            Self::BarChart => "BarChart",
            Self::LineChart => "LineChart",
            Self::PieChart => "PieChart",
            Self::Table => "Table",
            Self::Divider => "Divider",
            Self::Button => "Button",
            Self::Brand => "Brand",
            Self::Banner => "Banner",
            Self::IconButton => "IconButton",
            Self::ToggleTheme => "ToggleTheme",
            Self::SelectTheme => "SelectTheme",
            Self::Fab => "Fab",
            Self::FabAction => "fabAction",
            Self::Slider => "Slider",
            Self::Dropzone => "Dropzone",
            Self::ComboBox => "ComboBox",
            Self::ComboOption => "comboOption",
            Self::CsvField => "CsvField",
            Self::CsvColumn => "csvColumn",
            Self::DragDrop => "DragDrop",
            Self::DragGroup => "dragGroup",
            Self::DragItem => "dragItem",
            Self::Editor => "Editor",
            Self::ImageCropper => "ImageCropper",
            Self::PasswordField => "PasswordField",
            Self::PhoneField => "PhoneField",
            Self::PinField => "PinField",
            Self::Textarea => "Textarea",
            Self::Alert => "Alert",
            Self::Icon => "Icon",
            Self::Svg => "Svg",
            Self::Path => "Path",
            Self::AppBar => "AppBar",
            Self::Footer => "Footer",
            Self::BottomBar => "BottomBar",
            Self::NavMenu => "NavMenu",
            Self::SideNav => "SideNav",
            Self::RailNav => "RailNav",
            Self::Sidebar => "Sidebar",
            Self::Scaffold => "Scaffold",
            Self::Splash => "Splash",
            Self::Drawer => "Drawer",
            Self::Avatar => "Avatar",
            Self::Badge => "Badge",
            Self::Chip => "Chip",
            Self::Skeleton => "Skeleton",
            Self::Modal => "Modal",
            Self::AlertDialog => "AlertDialog",
            Self::Tooltip => "Tooltip",
            Self::Toast => "Toast",
            Self::Dropdown => "Dropdown",
            Self::Command => "Command",
            Self::AvatarGroup => "AvatarGroup",
            Self::ChatBox => "ChatBox",
            Self::Empty => "Empty",
            Self::Marquee => "Marquee",
            Self::TypeWriter => "TypeWriter",
            Self::RichText => "RichText",
            Self::Record => "Record",
            Self::ToggleGroup => "ToggleGroup",
            Self::Collapsible => "Collapsible",
            Self::Countdown => "Countdown",
            Self::Map => "Map",
            Self::Audio => "Audio",
            Self::Image => "Image",
            Self::Accordion => "Accordion",
            Self::Carousel => "Carousel",
            Self::Checkbox => "Checkbox",
            Self::Color => "Color",
            Self::Date => "Date",
            Self::DateRange => "DateRange",
            Self::RadioGroup => "RadioGroup",
            Self::Toggle => "Toggle",
            Self::Card => "Card",
            Self::Tabs => "Tabs",
            Self::Tab => "tab",
            Self::Title => "Title",
            Self::Text => "Text",
        }
    }
}
