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
    Password {
        props: PasswordProps,
    },
    Phone {
        props: PhoneProps,
    },
    Pin {
        props: PinProps,
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
