use dowe_components::BuiltinComponent;

fn text_line_css(value: TextSize) -> &'static str {
    text_typography(false, value).line_height
}

fn title_text_size_css(value: TextSize) -> String {
    fluid_text_size_css(text_typography(true, value).font_size)
}

fn title_text_line_css(value: TextSize) -> &'static str {
    text_typography(true, value).line_height
}

fn title_text_weight_css(value: TextSize) -> &'static str {
    text_weight_number(text_typography(true, value).weight)
}

fn title_text_spacing_css(value: TextSize) -> String {
    format!("{}em", text_typography(true, value).letter_spacing_em)
}

fn text_weight_css(value: TextWeight) -> &'static str {
    text_weight_number(value)
}

fn text_spacing_css(value: TextSpacing) -> String {
    format!("{}em", text_spacing_em(value))
}

fn text_token(family: ColorFamily) -> &'static str {
    family.text_token().as_str()
}

fn title_token(family: ColorFamily) -> &'static str {
    family.title_token().as_str()
}

fn family_color_token(family: ColorFamily) -> &'static str {
    family.color_token().as_str()
}

fn family_text_token(family: ColorFamily) -> &'static str {
    family.text_token().as_str()
}

fn append_responsive_rule(css: &mut String, breakpoint: Breakpoint, class_name: &str, body: &str) {
    css.push_str(&format!(
        ".{}\\:{}{{{body}}}",
        breakpoint.as_str(),
        css_class_name(class_name)
    ));
}

fn append_rule(css: &mut String, class_name: &str, body: &str) {
    css.push_str(&format!(".{}{{{body}}}", css_class_name(class_name)));
}

fn css_class_name(value: &str) -> String {
    value.replace(':', "\\:").replace('.', "\\.")
}

fn page_file_name(page: &ViewPage) -> String {
    let file_name = page.route_path.trim_matches('/').replace('/', "-");
    if file_name.is_empty() {
        "index".to_string()
    } else {
        file_name
    }
}

#[derive(Clone, Default)]
struct ReactiveRenderContext {
    constants: Vec<(String, String)>,
    signals: Vec<(String, String)>,
    actions: Vec<(String, String)>,
    consumed_props: std::rc::Rc<std::cell::RefCell<dowe_components::PropConsumptionRegistry>>,
}

impl ReactiveRenderContext {
    fn register_consumed_prop(
        &self,
        component: BuiltinComponent,
        prop: &'static str,
        ir_field: &'static str,
    ) {
        dowe_components::register_consumed_prop(
            &mut self.consumed_props.borrow_mut(),
            component,
            prop,
            ir_field,
        );
    }

}

impl ReactiveRenderContext {
    fn with_scope(
        &self,
        constants: &[ViewConstant],
        signals: &[ViewSignal],
        actions: &[ViewAction],
    ) -> Self {
        let mut context = self.clone();
        context.constants.extend(
            constants
                .iter()
                .map(|constant| (constant.name.clone(), constant.id.clone())),
        );
        context.signals.extend(
            signals
                .iter()
                .map(|signal| (signal.name.clone(), signal.id.clone())),
        );
        context.actions.extend(
            actions
                .iter()
                .map(|action| (action.name.clone(), action.id.clone())),
        );
        context
    }

    fn signal_path(&self, value: &str) -> String {
        if let Some(value) = value.strip_prefix('!') {
            return format!("!{}", self.signal_path(value));
        }
        let (root, suffix) = value
            .split_once('.')
            .map(|(root, suffix)| (root, Some(suffix)))
            .unwrap_or((value, None));
        let resolved = self
            .signals
            .iter()
            .rev()
            .chain(self.constants.iter().rev())
            .find(|(name, _)| name == root);
        let Some((_, id)) = resolved else {
            return value.to_string();
        };
        suffix
            .map(|suffix| format!("{id}.{suffix}"))
            .unwrap_or_else(|| id.clone())
    }

    fn action_id(&self, value: &str) -> String {
        self.actions
            .iter()
            .rev()
            .find(|(name, _)| name == value)
            .map(|(_, id)| id.clone())
            .unwrap_or_else(|| value.to_string())
    }
}

fn render_html_with_inspector(
    node: &ViewNode,
    children_html: Option<&str>,
    inspector: Option<&ViewInspectorMap>,
) -> String {
    if inspector.is_none() {
        return render_html_with_context(node, children_html, &ReactiveRenderContext::default());
    }
    with_view_inspector(inspector, || {
        render_html_with_context(node, children_html, &ReactiveRenderContext::default())
    })
}

fn render_html_with_context(
    node: &ViewNode,
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    register_rendered_node_props(node, context);
    with_view_inspector_node(|| render_html_node_with_context(node, children_html, context))
}

pub fn render_report_for_desktop_routes(routes: &[dowe_components::ViewRoute]) -> dowe_components::RenderReport {
    render_report_for_target(routes, dowe_components::RenderTarget::Desktop)
}

pub fn render_report_for_routes(routes: &[dowe_components::ViewRoute]) -> dowe_components::RenderReport {
    render_report_for_target(routes, dowe_components::RenderTarget::Web)
}

fn render_report_for_target(
    routes: &[dowe_components::ViewRoute],
    target: dowe_components::RenderTarget,
) -> dowe_components::RenderReport {
    let report = dowe_components::RenderReport::from_routes(
        target,
        routes
            .iter()
            .map(|route| dowe_components::RouteRenderReport {
                route_path: route.route_path.clone(),
                accepted: Vec::new(),
                lowered: Vec::new(),
                present: Vec::new(),
                consumed: {
                    let mut entries = consumed_props_for_tree(&route.layout_tree);
                    entries.extend(consumed_props_for_tree(&route.page_tree));
                    entries
                },
                emitted: Vec::new(),
            })
            .collect(),
    );
    debug_assert!(report.validate().is_ok());
    report
}

pub fn render_report_for_tree(tree: &ViewNode) -> dowe_components::RenderReport {
    let report = dowe_components::RenderReport::new(
        dowe_components::RenderTarget::Web,
        consumed_props_for_tree(tree),
    );
    debug_assert!(report.validate().is_ok());
    report
}

pub fn consumed_props_for_tree(tree: &ViewNode) -> Vec<dowe_components::ConsumedProp> {
    let registry = ReactiveRenderContext::default();
    fn collect(node: &ViewNode, context: &ReactiveRenderContext) {
        register_rendered_node_props(node, context);
        for children in dowe_components::node_child_groups(node) {
            for child in children {
                collect(child, context);
            }
        }
    }
    collect(tree, &registry);
    registry.consumed_props.borrow().entries().to_vec()
}

pub fn consumed_props_for_node(node: &ViewNode) -> Vec<dowe_components::ConsumedProp> {
    let context = ReactiveRenderContext::default();
    register_rendered_node_props(node, &context);
    context.consumed_props.borrow().entries().to_vec()
}

fn register_rendered_node_props(node: &ViewNode, context: &ReactiveRenderContext) {
    if let Some(component) = form_component(node)
        && dowe_components::node_element_props(node).and_then(|props| props.bind.as_deref()).is_some()
    {
        context.register_consumed_prop(component, "bind", "ElementProps.bind");
    }
    match node {
        ViewNode::Box { props, .. } | ViewNode::Section { props, .. } => {
            if props.bg.is_some() || props.bg_binding.is_some() {
                context.register_consumed_prop(BuiltinComponent::Box, "bg", "StyleProps.bg");
            }
            if props.text.is_some() || props.text_binding.is_some() {
                context.register_consumed_prop(BuiltinComponent::Box, "color", "StyleProps.text");
            }
            if props.rounded.is_some() || props.rounded_binding.is_some() {
                context.register_consumed_prop(BuiltinComponent::Box, "rounded", "StyleProps.rounded");
            }
            if props.spacing.p.is_some() || props.spacing.p_binding.is_some() {
                context.register_consumed_prop(BuiltinComponent::Box, "p", "SpacingProps.p");
            }
        }
        ViewNode::Candlestick { .. } => register_chart_consumed_props(BuiltinComponent::Candlestick, context),
        ViewNode::ArcChart { .. } => register_chart_consumed_props(BuiltinComponent::ArcChart, context),
        ViewNode::AreaChart { .. } => register_chart_consumed_props(BuiltinComponent::AreaChart, context),
        ViewNode::BarChart { .. } => register_chart_consumed_props(BuiltinComponent::BarChart, context),
        ViewNode::LineChart { .. } => register_chart_consumed_props(BuiltinComponent::LineChart, context),
        ViewNode::PieChart { .. } => register_chart_consumed_props(BuiltinComponent::PieChart, context),
        ViewNode::Tabs { .. } => {
            register_structural_consumed_props(BuiltinComponent::Tabs, context, &[("position", "TabsProps.position")]);
            register_structural_item_consumed_props(BuiltinComponent::Tabs, dowe_components::ViewItemKind::Tab, context, &[("id", "TabItem.id"), ("label", "TabItem.label"), ("i18n", "TabItem.i18n")]);
        }
        ViewNode::Accordion { .. } => {
            register_structural_item_consumed_props(BuiltinComponent::Accordion, dowe_components::ViewItemKind::Accordion, context, &[("id", "AccordionItem.id"), ("label", "AccordionItem.label"), ("disabled", "AccordionItem.disabled"), ("defaultOpen", "AccordionItem.default_open")]);
        }
        ViewNode::Carousel { .. } => register_structural_consumed_props(BuiltinComponent::Carousel, context, &[("slidesPerView", "CarouselProps.slides_per_view"), ("autoplay", "CarouselProps.autoplay"), ("orientation", "CarouselProps.orientation")]),
        ViewNode::Table { .. } => register_structural_item_consumed_props(BuiltinComponent::Table, dowe_components::ViewItemKind::TableColumn, context, &[("field", "TableColumn.field"), ("label", "TableColumn.label"), ("align", "TableColumn.align")]),
        ViewNode::NavMenu { .. } => {
            context.register_consumed_prop(BuiltinComponent::NavMenu, "variant", "NavMenuProps.style.variant");
            context.register_consumed_prop(BuiltinComponent::NavMenu, "scheme", "NavMenuProps.style.color");
            context.register_consumed_prop(BuiltinComponent::NavMenu, "size", "NavMenuProps.size");
            register_structural_item_consumed_props(BuiltinComponent::NavMenu, dowe_components::ViewItemKind::NavMenu, context, &[("label", "NavMenuItemProps.label"), ("i18n", "NavMenuItemProps.i18n"), ("description", "NavMenuItemProps.description"), ("href", "NavMenuItemProps.navigation")]);
        }
        ViewNode::SideNav { .. } => {
            for (prop, field) in [("variant", "SideNavProps.style.variant"), ("scheme", "SideNavProps.style.color"), ("size", "SideNavProps.size"), ("wide", "SideNavProps.wide")] { context.register_consumed_prop(BuiltinComponent::SideNav, prop, field); }
            register_structural_item_consumed_props(BuiltinComponent::SideNav, dowe_components::ViewItemKind::SideNav, context, &[("label", "SideNavItemProps.label"), ("i18n", "SideNavItemProps.i18n"), ("description", "SideNavItemProps.description"), ("href", "SideNavItemProps.navigation")]);
        }
        ViewNode::RailNav { .. } => {
            for (prop, field) in [("variant", "RailNavProps.style.variant"), ("scheme", "RailNavProps.style.color"), ("size", "RailNavProps.size")] { context.register_consumed_prop(BuiltinComponent::RailNav, prop, field); }
            register_structural_item_consumed_props(BuiltinComponent::RailNav, dowe_components::ViewItemKind::RailNav, context, &[("label", "RailNavItemProps.label"), ("i18n", "RailNavItemProps.i18n"), ("href", "RailNavItemProps.navigation")]);
        }
        ViewNode::BottomBar { .. } => {
            for (prop, field) in [("floating", "BarProps.floating"), ("bordered", "BarProps.bordered"), ("blurred", "BarProps.blurred"), ("boxed", "BarProps.boxed")] { context.register_consumed_prop(BuiltinComponent::BottomBar, prop, field); }
            register_structural_item_consumed_props(BuiltinComponent::BottomBar, dowe_components::ViewItemKind::BottomBar, context, &[("label", "BottomBarTab.label"), ("href", "BottomBarTab.navigation")]);
        }
        ViewNode::Svg { .. } => register_structural_consumed_props(BuiltinComponent::Svg, context, &[("viewBox", "SvgProps.view_box"), ("data", "SvgProps.data")]),
        ViewNode::Modal { .. } => register_structural_consumed_props(BuiltinComponent::Modal, context, &[("bind", "ModalProps.open")]),
        ViewNode::AlertDialog { .. } => register_structural_consumed_props(BuiltinComponent::AlertDialog, context, &[("bind", "AlertDialogProps.open")]),
        ViewNode::Command { .. } => register_structural_consumed_props(BuiltinComponent::Command, context, &[("bind", "CommandProps.open")]),
        ViewNode::Toast { .. } => register_structural_consumed_props(BuiltinComponent::Toast, context, &[("source", "ToastProps.source")]),
        ViewNode::AppBar { .. } => register_structural_consumed_props(BuiltinComponent::AppBar, context, &[("position", "BarProps.position"), ("floating", "BarProps.floating"), ("bordered", "BarProps.bordered"), ("blurred", "BarProps.blurred"), ("hideOnScroll", "BarProps.hide_on_scroll"), ("dockOnScroll", "BarProps.dock_on_scroll")]),
        ViewNode::Footer { .. } => register_structural_consumed_props(BuiltinComponent::Footer, context, &[("bordered", "BarProps.bordered"), ("blurred", "BarProps.blurred"), ("boxed", "BarProps.boxed")]),
        ViewNode::Sidebar { .. } => register_structural_consumed_props(BuiltinComponent::Sidebar, context, &[("variant", "SidebarProps.style.variant"), ("scheme", "SidebarProps.style.color"), ("size", "SidebarProps.style.size")]),
        ViewNode::Drawer { .. } => register_structural_consumed_props(BuiltinComponent::Drawer, context, &[("bind", "DrawerProps.open"), ("position", "DrawerProps.position"), ("disableOverlayClose", "DrawerProps.disable_overlay_close"), ("hideCloseButton", "DrawerProps.hide_close_button")]),
        ViewNode::Audio { .. } => {
            context.register_consumed_prop(BuiltinComponent::Audio, "src", "AudioProps.src");
        }
        ViewNode::Video { .. } => {
            context.register_consumed_prop(BuiltinComponent::Video, "src", "VideoProps.src");
            context.register_consumed_prop(BuiltinComponent::Video, "poster", "VideoProps.poster");
            context.register_consumed_prop(BuiltinComponent::Video, "autoplay", "VideoProps.autoplay");
        }
        ViewNode::Iframe { .. } => {
            context.register_consumed_prop(BuiltinComponent::Iframe, "src", "IframeProps.src");
            context.register_consumed_prop(BuiltinComponent::Iframe, "sandbox", "IframeProps.sandbox");
            context.register_consumed_prop(BuiltinComponent::Iframe, "allow", "IframeProps.allow");
        }
        ViewNode::Device { .. } => {
            context.register_consumed_prop(BuiltinComponent::Device, "device", "DeviceProps.device");
            context.register_consumed_prop(BuiltinComponent::Device, "zoom", "DeviceProps.options");
            context.register_consumed_prop(BuiltinComponent::Device, "fit", "DeviceProps.options");
        }
        ViewNode::Camera { .. } => {
            context.register_consumed_prop(BuiltinComponent::Camera, "facing", "CameraProps.facing");
            context.register_consumed_prop(BuiltinComponent::Camera, "onCapture", "CameraProps.on_capture");
            context.register_consumed_prop(BuiltinComponent::Camera, "onError", "CameraProps.on_error");
        }
        ViewNode::Microphone { .. } => {
            context.register_consumed_prop(BuiltinComponent::Microphone, "onError", "MicrophoneProps.on_error");
        }
        ViewNode::Date { .. } => {
        }
        ViewNode::DateRange { .. } => {
            context.register_consumed_prop(BuiltinComponent::DateRange, "start", "DateRangeProps.start");
            context.register_consumed_prop(BuiltinComponent::DateRange, "end", "DateRangeProps.end");
        }
        ViewNode::Password { .. } => {
        }
        ViewNode::Phone { .. } => {
            context.register_consumed_prop(BuiltinComponent::Phone, "country", "PhoneProps.country");
        }
        ViewNode::Pin { .. } => {
            context.register_consumed_prop(BuiltinComponent::Pin, "length", "PinProps.length");
        }
        ViewNode::Textarea { .. } => {
        }
        ViewNode::Color { .. } => {
        }
        ViewNode::Dropzone { .. } => {
            context.register_consumed_prop(BuiltinComponent::Dropzone, "multiple", "DropzoneProps.multiple");
            context.register_consumed_prop(BuiltinComponent::Dropzone, "accept", "DropzoneProps.accept");
        }
        ViewNode::Checkbox { .. } => {
            context.register_consumed_prop(BuiltinComponent::Checkbox, "checked", "CheckboxProps.checked");
        }
        ViewNode::Toggle { .. } => {
            context.register_consumed_prop(BuiltinComponent::Toggle, "checked", "ToggleProps.checked");
        }
        ViewNode::RadioGroup { .. } => {
            context.register_consumed_prop(BuiltinComponent::RadioGroup, "orientation", "RadioGroupProps.orientation");
        }
        ViewNode::Slider { .. } => {
            context.register_consumed_prop(BuiltinComponent::Slider, "min", "SliderProps.min");
            context.register_consumed_prop(BuiltinComponent::Slider, "max", "SliderProps.max");
            context.register_consumed_prop(BuiltinComponent::Slider, "step", "SliderProps.step");
        }
        ViewNode::Avatar { .. } => register_structural_consumed_props(BuiltinComponent::Avatar, context, &[("name", "AvatarProps.name"), ("alt", "AvatarProps.alt"), ("icon", "SideNavIcon.props")]),
        ViewNode::AvatarGroup { .. } => register_structural_consumed_props(BuiltinComponent::AvatarGroup, context, &[("items", "AvatarGroupProps.items"), ("size", "AvatarGroupProps.size")]),
        ViewNode::Badge { .. } => register_structural_consumed_props(BuiltinComponent::Badge, context, &[("variant", "VariantProps.variant"), ("scheme", "VariantProps.color")]),
        ViewNode::Chip { .. } => register_structural_consumed_props(BuiltinComponent::Chip, context, &[("variant", "VariantProps.variant"), ("scheme", "VariantProps.color"), ("size", "VariantProps.size"), ("rounded", "VariantProps.style.rounded")]),
        ViewNode::ChatBox { .. } => register_structural_consumed_props(BuiltinComponent::ChatBox, context, &[("messages", "ChatBoxProps.messages"), ("loading", "ChatBoxProps.loading"), ("sending", "ChatBoxProps.sending"), ("streaming", "ChatBoxProps.streaming"), ("hasMore", "ChatBoxProps.has_more")]),
        ViewNode::Empty { .. } => register_structural_consumed_props(BuiltinComponent::Empty, context, &[("title", "EmptyProps.title"), ("description", "EmptyProps.description"), ("actionLabel", "EmptyProps.action_label")]),
        ViewNode::Marquee { .. } => register_structural_consumed_props(BuiltinComponent::Marquee, context, &[("speed", "MarqueeProps.speed"), ("pauseOnHover", "MarqueeProps.pause_on_hover"), ("reverse", "MarqueeProps.reverse"), ("orientation", "MarqueeProps.orientation"), ("fade", "MarqueeProps.fade"), ("fadeColor", "MarqueeProps.fade_color"), ("gap", "MarqueeProps.gap")]),
        ViewNode::TypeWriter { .. } => register_structural_consumed_props(BuiltinComponent::TypeWriter, context, &[("typeSpeed", "TypeWriterProps.type_speed"), ("deleteSpeed", "TypeWriterProps.delete_speed"), ("afterTyped", "TypeWriterProps.after_typed"), ("afterDeleted", "TypeWriterProps.after_deleted"), ("repeat", "TypeWriterProps.repeat")]),
        ViewNode::RichText { .. } => register_structural_consumed_props(BuiltinComponent::RichText, context, &[("text", "RichTextMark.text"), ("style", "RichTextMark.style"), ("color", "RichTextMark.color")]),
        ViewNode::Record { .. } => register_structural_consumed_props(BuiltinComponent::Record, context, &[("name", "RecordProps.name"), ("url", "RecordProps.url"), ("disabled", "RecordProps.disabled"), ("maxDuration", "RecordProps.max_duration")]),
        ViewNode::ToggleGroup { .. } => register_structural_consumed_props(BuiltinComponent::ToggleGroup, context, &[("value", "ToggleGroupProps.value"), ("selected", "ToggleGroupProps.selected"), ("multiple", "ToggleGroupProps.multiple"), ("wide", "ToggleGroupProps.wide"), ("vertical", "ToggleGroupProps.vertical"), ("disabled", "ToggleGroupProps.disabled"), ("ariaLabel", "ToggleGroupProps.aria_label")]),
        ViewNode::Collapsible { .. } => register_structural_consumed_props(BuiltinComponent::Collapsible, context, &[("label", "CollapsibleProps.label"), ("defaultOpen", "CollapsibleProps.default_open"), ("disabled", "CollapsibleProps.disabled")]),
        ViewNode::Countdown { .. } => register_structural_consumed_props(BuiltinComponent::Countdown, context, &[("target", "CountdownProps.target"), ("showDays", "CountdownProps.show_days"), ("showHours", "CountdownProps.show_hours"), ("showMinutes", "CountdownProps.show_minutes"), ("showSeconds", "CountdownProps.show_seconds"), ("size", "CountdownProps.size"), ("onComplete", "CountdownProps.on_complete")]),
        ViewNode::Map { .. } => register_structural_consumed_props(BuiltinComponent::Map, context, &[("centerLat", "MapProps.center_lat"), ("centerLng", "MapProps.center_lng"), ("zoom", "MapProps.zoom"), ("height", "MapProps.height"), ("width", "MapProps.width"), ("showControls", "MapProps.show_controls"), ("showScale", "MapProps.show_scale"), ("interactive", "MapProps.interactive"), ("onLocation", "MapProps.on_location"), ("onLocationError", "MapProps.on_location_error"), ("onRoute", "MapProps.on_route")]),
        ViewNode::Image { props: _ } => {
            context.register_consumed_prop(BuiltinComponent::Image, "src", "ImageProps.src");
            context.register_consumed_prop(BuiltinComponent::Image, "alt", "ImageProps.alt");
            context.register_consumed_prop(BuiltinComponent::Image, "objectFit", "ImageProps.object_fit");
            context.register_consumed_prop(BuiltinComponent::Image, "loading", "ImageProps.loading");
        }
        ViewNode::Text { props, .. } | ViewNode::Title { props, .. } => {
            if props.size.is_some() || props.size_binding.is_some() {
                context.register_consumed_prop(BuiltinComponent::Text, "size", "TextProps.size");
            }
            if props.weight.is_some() || props.weight_binding.is_some() {
                context.register_consumed_prop(BuiltinComponent::Text, "weight", "TextProps.weight");
            }
            if props.letter_spacing.is_some() || props.letter_spacing_binding.is_some() {
                context.register_consumed_prop(BuiltinComponent::Text, "spacing", "TextProps.letter_spacing");
            }
        }
        ViewNode::Input { props } | ViewNode::Select { props, .. } => {
            let component = if matches!(node, ViewNode::Input { .. }) { BuiltinComponent::Input } else { BuiltinComponent::Select };
            if props.color.is_some() || props.color_binding.is_some() {
                context.register_consumed_prop(component, "scheme", "VariantProps.color");
            }
            if props.variant.is_some() || props.variant_binding.is_some() {
                context.register_consumed_prop(component, "variant", "VariantProps.variant");
            }
            if props.size.is_some() || props.size_binding.is_some() {
                context.register_consumed_prop(component, "size", "VariantProps.size");
            }
            if props.element.bind.is_some() {
                context.register_consumed_prop(component, "bind", "ElementProps.bind");
            }
            if props.label.is_some() {
                context.register_consumed_prop(component, "label", "VariantProps.label");
            }
            if props.placeholder.is_some() {
                context.register_consumed_prop(component, "placeholder", "VariantProps.placeholder");
            }
            if props.i18n.is_some() {
                context.register_consumed_prop(component, "i18n", "VariantProps.i18n");
            }
        }
        ViewNode::Button { props, .. } => {
            if props.color.is_some() || props.color_binding.is_some() {
                context.register_consumed_prop(BuiltinComponent::Button, "scheme", "VariantProps.color");
            }
            if props.variant.is_some() || props.variant_binding.is_some() {
                context.register_consumed_prop(BuiltinComponent::Button, "variant", "VariantProps.variant");
            }
            if props.size.is_some() || props.size_binding.is_some() {
                context.register_consumed_prop(BuiltinComponent::Button, "size", "VariantProps.size");
            }
            if props.style.rounded.is_some() || props.style.rounded_binding.is_some() {
                context.register_consumed_prop(BuiltinComponent::Button, "rounded", "VariantProps.style.rounded");
            }
            if props.reactive.loading.is_some() {
                context.register_consumed_prop(BuiltinComponent::Button, "loading", "VariantProps.reactive.loading");
            }
            if props.reactive.disabled.is_some() {
                context.register_consumed_prop(BuiltinComponent::Button, "disabled", "VariantProps.reactive.disabled");
            }
        }
        _ => {}
    }
}

fn register_structural_item_consumed_props(
    component: BuiltinComponent,
    item: dowe_components::ViewItemKind,
    context: &ReactiveRenderContext,
    props: &[(&'static str, &'static str)],
) {
    for (prop, field) in props {
        let mut registry = context.consumed_props.borrow_mut();
        dowe_components::register_consumed_item(&mut registry, component, item, *prop, *field);
    }
}

fn register_structural_consumed_props(
    component: BuiltinComponent,
    context: &ReactiveRenderContext,
    props: &[(&'static str, &'static str)],
) {
    for (prop, field) in props {
        context.register_consumed_prop(component, prop, field);
    }
}

fn register_chart_consumed_props(component: BuiltinComponent, context: &ReactiveRenderContext) {
    for (prop, field) in [("data", "ChartCommonProps.data"), ("series", "ChartCommonProps.series"), ("size", "ChartCommonProps.size"), ("palette", "ChartCommonProps.palette"), ("loading", "ChartCommonProps.loading")] {
        context.register_consumed_prop(component, prop, field);
    }
    let fields = match component {
        BuiltinComponent::Candlestick => vec![("stream", "CandlestickProps.stream"), ("upColor", "CandlestickProps.up_color"), ("downColor", "CandlestickProps.down_color"), ("maxPoints", "CandlestickProps.max_points")],
        BuiltinComponent::ArcChart => vec![("centerText", "ArcChartProps.center_text"), ("centerValue", "ArcChartProps.center_value"), ("thickness", "ArcChartProps.thickness"), ("gap", "ArcChartProps.gap"), ("startAngle", "ArcChartProps.start_angle"), ("endAngle", "ArcChartProps.end_angle"), ("showGlow", "ArcChartProps.show_glow")],
        BuiltinComponent::AreaChart => vec![("curve", "AreaChartProps.curve"), ("strokeWidth", "AreaChartProps.stroke_width"), ("fillOpacity", "AreaChartProps.fill_opacity"), ("stacked", "AreaChartProps.stacked"), ("showPoints", "AreaChartProps.show_points"), ("showGlow", "AreaChartProps.show_glow")],
        BuiltinComponent::BarChart => vec![("grouped", "BarChartProps.grouped"), ("stacked", "BarChartProps.stacked"), ("showValues", "BarChartProps.show_values"), ("barRadius", "BarChartProps.bar_radius"), ("showGlow", "BarChartProps.show_glow")],
        BuiltinComponent::LineChart => vec![("curve", "LineChartProps.curve"), ("strokeWidth", "LineChartProps.stroke_width"), ("pointRadius", "LineChartProps.point_radius"), ("showGradientFill", "LineChartProps.show_gradient_fill"), ("showGlow", "LineChartProps.show_glow")],
        BuiltinComponent::PieChart => vec![("donut", "PieChartProps.donut"), ("donutWidth", "PieChartProps.donut_width"), ("centerLabel", "PieChartProps.center_label"), ("centerValue", "PieChartProps.center_value"), ("startAngle", "PieChartProps.start_angle"), ("padAngle", "PieChartProps.pad_angle"), ("hideLabels", "PieChartProps.hide_labels"), ("hideValues", "PieChartProps.hide_values"), ("hidePercentages", "PieChartProps.hide_percentages"), ("showGlow", "PieChartProps.show_glow")],
        _ => Vec::new(),
    };
    for (prop, field) in fields {
        context.register_consumed_prop(component, prop, field);
    }
}

fn form_component(node: &ViewNode) -> Option<BuiltinComponent> {
    match node {
        ViewNode::Input { .. } => Some(BuiltinComponent::Input),
        ViewNode::Select { .. } => Some(BuiltinComponent::Select),
        ViewNode::Checkbox { .. } => Some(BuiltinComponent::Checkbox),
        ViewNode::Toggle { .. } => Some(BuiltinComponent::Toggle),
        ViewNode::RadioGroup { .. } => Some(BuiltinComponent::RadioGroup),
        ViewNode::Slider { .. } => Some(BuiltinComponent::Slider),
        ViewNode::Date { .. } => Some(BuiltinComponent::Date),
        ViewNode::DateRange { .. } => Some(BuiltinComponent::DateRange),
        ViewNode::Password { .. } => Some(BuiltinComponent::Password),
        ViewNode::Phone { .. } => Some(BuiltinComponent::Phone),
        ViewNode::Pin { .. } => Some(BuiltinComponent::Pin),
        ViewNode::Textarea { .. } => Some(BuiltinComponent::Textarea),
        ViewNode::Color { .. } => Some(BuiltinComponent::Color),
        ViewNode::Dropzone { .. } => Some(BuiltinComponent::Dropzone),
        _ => None,
    }
}

fn render_html_node_with_context(
    node: &ViewNode,
    children_html: Option<&str>,
    context: &ReactiveRenderContext,
) -> String {
    match node {
        ViewNode::Scope {
            constants,
            signals,
            actions,
            children,
        } => {
            let context = context.with_scope(constants, signals, actions);
            children
                .iter()
                .map(|child| render_html_with_context(child, children_html, &context))
                .collect::<String>()
        }
        ViewNode::Splash {
            binding,
            initial,
            content,
            children,
        } => {
            let binding = escape_attr(&context.signal_path(binding));
            let mut html = format!(r#"<div data-dowe-splash="{binding}">"#);
            if *initial {
                html.push_str("<div data-dowe-splash-main hidden>");
            } else {
                html.push_str("<div data-dowe-splash-main>");
            }
            for child in content {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str("</div>");
            if *initial {
                html.push_str("<div data-dowe-splash-content>");
            } else {
                html.push_str("<div data-dowe-splash-content hidden>");
            }
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str("</div></div>");
            html
        }
        ViewNode::Box { props, children } => {
            let mut html = format!(
                "<div{}>",
                attrs(box_classes(props), Some(&props.element), None, context)
            );
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str("</div>");
            html
        }
        ViewNode::Section { props, children } => {
            let mut html = format!(
                "<section{}><div{}>",
                attrs(section_classes(props), Some(&props.element), None, context),
                attrs(section_body_classes(props), None, None, context)
            );
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str("</div></section>");
            html
        }
        ViewNode::Flex { props, children } => {
            let mut html = format!(
                "<div{}>",
                attrs(
                    layout_classes("flex", props),
                    Some(&props.style.element),
                    None,
                    context
                )
            );
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str("</div>");
            html
        }
        ViewNode::Grid { props, children } => {
            let mut html = format!(
                "<div{}>",
                attrs(
                    grid_classes(props),
                    Some(&props.style.element),
                    None,
                    context
                )
            );
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str("</div>");
            html
        }
        ViewNode::Card { props, children } => {
            let mut card_attrs = attrs(
                variant_classes("card", props),
                Some(&props.element),
                None,
                context,
            );
            if props.reactive.variant.is_some() || props.reactive.scheme.is_some() {
                card_attrs.push_str(r#" data-dowe-variant-binding="true""#);
                if let Some(path) = props.reactive.variant.as_deref() {
                    card_attrs.push_str(&format!(
                        r#" data-dowe-variant="{}""#,
                        escape_attr(&context.signal_path(path))
                    ));
                }
                if let Some(path) = props.reactive.scheme.as_deref() {
                    card_attrs.push_str(&format!(
                        r#" data-dowe-scheme="{}""#,
                        escape_attr(&context.signal_path(path))
                    ));
                }
            }
            let mut html = format!("<article{}>", card_attrs);
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str("</article>");
            html
        }
        ViewNode::Tabs { props, tabs } => render_tabs_html(props, tabs, children_html, context),
        ViewNode::NavMenu { props, items } => {
            render_nav_menu_html(props, items, children_html, context)
        }
        ViewNode::Button { props, children } => {
            let (open, close) = button_tags(props, context);
            let mut html = open;
            if let Some(icon) = props.loading_icon.as_ref() {
                html.push_str(
                    r#"<span class="button-loading" data-dowe-button-loading hidden aria-hidden="true">"#,
                );
                html.push_str(&render_svg_html(&icon.props, &icon.paths, context));
                html.push_str("</span>");
                html.push_str("<span data-dowe-button-content>");
            }
            if let Some(icon) = props.icon_start.as_ref() {
                html.push_str("<span data-dowe-button-icon-start data-dowe-swap-on>");
                html.push_str(&render_svg_html(&icon.props, &icon.paths, context));
                html.push_str("</span>");
            }
            if let Some(icon) = props.swap_icon_off.as_ref() {
                html.push_str("<span data-dowe-button-icon-start data-dowe-swap-off hidden>");
                html.push_str(&render_svg_html(&icon.props, &icon.paths, context));
                html.push_str("</span>");
            }
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            if let Some(icon) = props.icon_end.as_ref() {
                html.push_str("<span data-dowe-button-icon-end>");
                html.push_str(&render_svg_html(&icon.props, &icon.paths, context));
                html.push_str("</span>");
            }
            if props.loading_icon.is_some() {
                html.push_str("</span>");
            }
            html.push_str(close);
            html
        }
        ViewNode::Brand { props, children } => {
            let (open, close) = brand_tags(props, context);
            let mut html = open;
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str(close);
            html
        }
        ViewNode::Banner { props, children } => {
            let (open, close) = banner_tags(props, context);
            let mut html = open;
            for child in children {
                html.push_str(&render_html_with_context(child, children_html, context));
            }
            html.push_str(close);
            html
        }
        ViewNode::ToggleTheme { props } => render_theme_toggle_html(props, context),
        ViewNode::SelectTheme { props } => render_theme_select_html(props, context),
        ViewNode::Fab { props, actions } => render_fab_html(props, actions, context),
        ViewNode::Input { props } => render_input_html(props, context),
        ViewNode::Slider { props } => render_slider_html(props, context),
        ViewNode::Dropzone { props } => render_dropzone_html(props, context),
        ViewNode::Select {
            props,
            options,
            option_each,
        } => render_select_html(props, options, option_each.as_ref(), context),
        ViewNode::ComboBox { props, options } => render_combo_box_html(props, options, context),
        ViewNode::CsvField { props, columns } => render_csv_field_html(props, columns, context),
        ViewNode::DragDrop {
            props,
            items,
            groups,
        } => render_drag_drop_html(props, items, groups, context),
        ViewNode::Editor { props } => render_editor_html(props, context),
        ViewNode::ImageCropper { props } => render_image_cropper_html(props, context),
        ViewNode::Password { props } => render_password_html(props, context),
        ViewNode::Phone { props } => render_phone_html(props, context),
        ViewNode::Pin { props } => render_pin_html(props, context),
        ViewNode::Textarea { props } => render_textarea_html(props, context),
        ViewNode::Audio { props } => render_audio_html(props, context),
        ViewNode::Camera { props } => render_camera_html(props, context),
        ViewNode::Microphone { props } => render_microphone_html(props, context),
        ViewNode::Image { props } => render_image_html(props, context),
        ViewNode::Code { props } => render_code_html(props, context),
        ViewNode::Video { props } => render_video_html(props, context),
        ViewNode::Iframe { props } => render_iframe_html(props, context),
        ViewNode::Device { props, iframe } => render_device_html(props, iframe, context),
        ViewNode::Canvas { props } => render_canvas_html(props, context),
        ViewNode::Candlestick { props } => render_candlestick_html(props, context),
        ViewNode::Diagram { props } => render_diagram_html(props, context),
        ViewNode::ArcChart { props } => render_arc_chart_html(props, context),
        ViewNode::AreaChart { props } => render_area_chart_html(props, context),
        ViewNode::BarChart { props } => render_bar_chart_html(props, context),
        ViewNode::LineChart { props } => render_line_chart_html(props, context),
        ViewNode::PieChart { props } => render_pie_chart_html(props, context),
        ViewNode::Table { props } => render_table_html(props, context),
        ViewNode::Divider { props } => render_divider_html(props, context),
        ViewNode::Title { props, value } => render_text_html(
            "title",
            text_classes("title", props),
            Some(&props.style.element),
            value,
            props.i18n.as_deref(),
            props.as_tag.as_deref(),
            context,
        ),
        ViewNode::Text { props, value } => render_text_html(
            "text",
            text_classes("text", props),
            Some(&props.style.element),
            value,
            props.i18n.as_deref(),
            None,
            context,
        ),
        ViewNode::Alert { props } => {
            let message = dynamic_text_attr(&props.message, context);
            let content = if message.is_empty() {
                escape_html(&props.message)
            } else {
                String::new()
            };
            let close = props
                .on_close
                .as_ref()
                .map(|action| {
                    format!(
                        r#"<button class="alert-close" type="button" data-dowe-click="{}">&times;</button>"#,
                        escape_attr(&context.action_id(action))
                    )
                })
                .unwrap_or_default();
            format!(
                r#"<div{}><span data-dowe-alert-message{}>{}</span>{}</div>"#,
                attrs(
                    variant_classes("alert", &props.style),
                    Some(&props.style.element),
                    Some(&alert_attrs(props, context)),
                    context
                ),
                message,
                content,
                close
            )
        }
        ViewNode::Avatar { props, icon } => render_avatar_html(props, icon.as_ref(), context),
        ViewNode::AvatarGroup { props, items } => render_avatar_group_html(props, items, context),
        ViewNode::ChatBox { props } => render_chat_box_html(props, context),
        ViewNode::Empty { props } => render_empty_html(props, context),
        ViewNode::Marquee { props, children } => {
            render_marquee_html(props, children, children_html, context)
        }
        ViewNode::TypeWriter { props, items } => render_type_writer_html(props, items, context),
        ViewNode::RichText { props, marks } => render_rich_text_html(props, marks, context),
        ViewNode::Record { props } => render_record_html(props, context),
        ViewNode::ToggleGroup { props, items } => render_toggle_group_html(props, items, context),
        ViewNode::Collapsible { props, children } => {
            render_collapsible_html(props, children, children_html, context)
        }
        ViewNode::Countdown { props } => render_countdown_html(props, context),
        ViewNode::Map {
            props,
            markers,
            waypoints,
        } => render_map_html(props, markers, waypoints, context),
        ViewNode::Badge { props, children } => {
            render_badge_html(props, children, children_html, context)
        }
        ViewNode::Chip {
            props,
            value,
            start,
            end,
        } => render_chip_html(props, value, start.as_ref(), end.as_ref(), context),
        ViewNode::Skeleton { props } => render_skeleton_html(props, context),
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => render_modal_html(props, header, body, footer, children_html, context),
        ViewNode::AlertDialog { props } => render_alert_dialog_html(props, context),
        ViewNode::Tooltip { props, children } => {
            render_tooltip_html(props, children, children_html, context)
        }
        ViewNode::Toast { props } => render_toast_html(props, context),
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            entries,
            footer,
        } => render_dropdown_html(
            props,
            trigger,
            header,
            entries,
            footer,
            children_html,
            context,
        ),
        ViewNode::Command { props, entries } => render_command_html(props, entries, context),
        ViewNode::Accordion { props, items } => {
            render_accordion_html(props, items, children_html, context)
        }
        ViewNode::Carousel { props, slides } => {
            render_carousel_html(props, slides, children_html, context)
        }
        ViewNode::Checkbox { props } => render_checkbox_html(props, context),
        ViewNode::Color { props } => render_color_html(props, context),
        ViewNode::Date { props } => render_date_html(props, context),
        ViewNode::DateRange { props } => render_date_range_html(props, context),
        ViewNode::RadioGroup { props, options } => render_radio_group_html(props, options, context),
        ViewNode::Toggle { props } => render_toggle_html(props, context),
        ViewNode::Svg { props, paths } => render_svg_html(props, paths, context),
        ViewNode::AppBar {
            props,
            top,
            start,
            center,
            end,
            bottom,
            mobile_menu,
        } => render_bar_html(
            "header",
            "appbar",
            props,
            top,
            start,
            center,
            end,
            bottom,
            mobile_menu.as_ref(),
            children_html,
            context,
        ),
        ViewNode::Footer {
            props,
            top,
            start,
            center,
            end,
            bottom,
        } => render_bar_html(
            "footer",
            "footer",
            props,
            top,
            start,
            center,
            end,
            bottom,
            None,
            children_html,
            context,
        ),
        ViewNode::BottomBar {
            props,
            tabs,
            ..
        } => render_bottom_bar_html(props, tabs, context),
        ViewNode::SideNav { props, items } => {
            render_side_nav_html("sidenav", props, items, context)
        }
        ViewNode::RailNav { props, items } => render_rail_nav_html(props, items, context),
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => render_sidebar_html(props, header, body, footer, children_html, context),
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
        } => render_scaffold_html(
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
            children_html,
            context,
        ),
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => render_drawer_html(props, header, body, footer, children_html, context),
        ViewNode::Each {
            item,
            collection,
            key,
            children,
        } => {
            let children = children
                .iter()
                .map(|child| render_html_with_context(child, children_html, context))
                .collect::<String>();
            format!(
                r#"<div data-dowe-each="{}" data-dowe-item="{}" data-dowe-key="{}"><template>{}</template></div>"#,
                escape_attr(&context.signal_path(collection)),
                escape_attr(item),
                escape_attr(key),
                children
            )
        }
        ViewNode::Children => children_html.unwrap_or_default().to_string(),
    }
}
