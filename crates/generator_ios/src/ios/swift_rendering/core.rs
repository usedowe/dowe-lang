fn render_swift_node_in_flow(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    register_swift_consumed_props(node, context);
    if let Some(expression) = context.node_expression(node) {
        let pad = " ".repeat(indent);
        output.push_str(&format!("{pad}{expression}\n"));
        return;
    }
    if let Some(show) = node_element_props(node).and_then(|props| props.show.as_ref()) {
        let pad = " ".repeat(indent);
        output.push_str(&format!(
            "{pad}if {} {{\n",
            swift_show_condition(show, context)
        ));
        render_swift_node_expression(
            node,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}}}\n"));
    } else {
        render_swift_node_expression(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
    }
}

pub fn render_report_for_dev_routes(routes: &[dowe_components::ViewRoute]) -> dowe_components::RenderReport {
    render_report_for_target(routes, dowe_components::RenderTarget::IosDev)
}

pub fn render_report_for_routes(routes: &[dowe_components::ViewRoute]) -> dowe_components::RenderReport {
    render_report_for_target(routes, dowe_components::RenderTarget::Ios)
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
        dowe_components::RenderTarget::Ios,
        consumed_props_for_tree(tree),
    );
    debug_assert!(report.validate().is_ok());
    report
}

pub fn consumed_props_for_tree(tree: &ViewNode) -> Vec<dowe_components::ConsumedProp> {
    let registry = SwiftReactiveContext::default();
    fn collect(node: &ViewNode, context: &SwiftReactiveContext) {
        register_swift_consumed_props(node, context);
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
    let context = SwiftReactiveContext::default();
    register_swift_consumed_props(node, &context);
    context.consumed_props.borrow().entries().to_vec()
}

fn register_swift_consumed_props(node: &ViewNode, context: &SwiftReactiveContext) {
    if let Some(component) = swift_form_component(node)
        && dowe_components::node_element_props(node)
            .and_then(|props| props.bind.as_deref())
            .is_some()
    {
        context.register_consumed_prop(component, "bind", "ElementProps.bind");
    }
    match node {
        ViewNode::Box { props, .. } | ViewNode::Section { props, .. } => {
            if props.bg.is_some() || props.bg_binding.is_some() {
                context.register_consumed_prop(dowe_components::BuiltinComponent::Box, "bg", "StyleProps.bg");
            }
            if props.text.is_some() || props.text_binding.is_some() {
                context.register_consumed_prop(dowe_components::BuiltinComponent::Box, "color", "StyleProps.text");
            }
            if props.rounded.is_some() || props.rounded_binding.is_some() {
                context.register_consumed_prop(dowe_components::BuiltinComponent::Box, "rounded", "StyleProps.rounded");
            }
            if props.spacing.p.is_some() || props.spacing.p_binding.is_some() {
                context.register_consumed_prop(dowe_components::BuiltinComponent::Box, "p", "SpacingProps.p");
            }
        }
        ViewNode::Candlestick { .. } => register_swift_chart_consumed_props(dowe_components::BuiltinComponent::Candlestick, context),
        ViewNode::ArcChart { .. } => register_swift_chart_consumed_props(dowe_components::BuiltinComponent::ArcChart, context),
        ViewNode::AreaChart { .. } => register_swift_chart_consumed_props(dowe_components::BuiltinComponent::AreaChart, context),
        ViewNode::BarChart { .. } => register_swift_chart_consumed_props(dowe_components::BuiltinComponent::BarChart, context),
        ViewNode::LineChart { .. } => register_swift_chart_consumed_props(dowe_components::BuiltinComponent::LineChart, context),
        ViewNode::PieChart { .. } => register_swift_chart_consumed_props(dowe_components::BuiltinComponent::PieChart, context),
        ViewNode::Tabs { .. } => {
            register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Tabs, context, &[("position", "TabsProps.position")]);
            register_swift_structural_item_consumed_props(dowe_components::BuiltinComponent::Tabs, dowe_components::ViewItemKind::Tab, context, &[("id", "TabItem.id"), ("label", "TabItem.label"), ("i18n", "TabItem.i18n")]);
        }
        ViewNode::Accordion { .. } => register_swift_structural_item_consumed_props(dowe_components::BuiltinComponent::Accordion, dowe_components::ViewItemKind::Accordion, context, &[("id", "AccordionItem.id"), ("label", "AccordionItem.label"), ("disabled", "AccordionItem.disabled"), ("defaultOpen", "AccordionItem.default_open")]),
        ViewNode::Carousel { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Carousel, context, &[("slidesPerView", "CarouselProps.slides_per_view"), ("autoplay", "CarouselProps.autoplay"), ("orientation", "CarouselProps.orientation")]),
        ViewNode::Table { .. } => register_swift_structural_item_consumed_props(dowe_components::BuiltinComponent::Table, dowe_components::ViewItemKind::TableColumn, context, &[("field", "TableColumn.field"), ("label", "TableColumn.label"), ("align", "TableColumn.align")]),
        ViewNode::NavMenu { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::NavMenu, "variant", "NavMenuProps.style.variant");
            context.register_consumed_prop(dowe_components::BuiltinComponent::NavMenu, "scheme", "NavMenuProps.style.color");
            context.register_consumed_prop(dowe_components::BuiltinComponent::NavMenu, "size", "NavMenuProps.size");
            register_swift_structural_item_consumed_props(dowe_components::BuiltinComponent::NavMenu, dowe_components::ViewItemKind::NavMenu, context, &[("label", "NavMenuItemProps.label"), ("i18n", "NavMenuItemProps.i18n"), ("description", "NavMenuItemProps.description"), ("href", "NavMenuItemProps.navigation")]);
        }
        ViewNode::SideNav { .. } => {
            for (prop, field) in [("variant", "SideNavProps.style.variant"), ("scheme", "SideNavProps.style.color"), ("size", "SideNavProps.size"), ("wide", "SideNavProps.wide")] { context.register_consumed_prop(dowe_components::BuiltinComponent::SideNav, prop, field); }
            register_swift_structural_item_consumed_props(dowe_components::BuiltinComponent::SideNav, dowe_components::ViewItemKind::SideNav, context, &[("label", "SideNavItemProps.label"), ("i18n", "SideNavItemProps.i18n"), ("description", "SideNavItemProps.description"), ("href", "SideNavItemProps.navigation")]);
        }
        ViewNode::RailNav { .. } => {
            for (prop, field) in [("variant", "RailNavProps.style.variant"), ("scheme", "RailNavProps.style.color"), ("size", "RailNavProps.size")] { context.register_consumed_prop(dowe_components::BuiltinComponent::RailNav, prop, field); }
            register_swift_structural_item_consumed_props(dowe_components::BuiltinComponent::RailNav, dowe_components::ViewItemKind::RailNav, context, &[("label", "RailNavItemProps.label"), ("i18n", "RailNavItemProps.i18n"), ("href", "RailNavItemProps.navigation")]);
        }
        ViewNode::BottomBar { .. } => {
            for (prop, field) in [("floating", "BarProps.floating"), ("bordered", "BarProps.bordered"), ("blurred", "BarProps.blurred"), ("boxed", "BarProps.boxed")] { context.register_consumed_prop(dowe_components::BuiltinComponent::BottomBar, prop, field); }
            register_swift_structural_item_consumed_props(dowe_components::BuiltinComponent::BottomBar, dowe_components::ViewItemKind::BottomBar, context, &[("label", "BottomBarTab.label"), ("href", "BottomBarTab.navigation")]);
        }
        ViewNode::Svg { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Svg, context, &[("viewBox", "SvgProps.view_box"), ("data", "SvgProps.data")]),
        ViewNode::Modal { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Modal, context, &[("bind", "ModalProps.open")]),
        ViewNode::AlertDialog { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::AlertDialog, context, &[("bind", "AlertDialogProps.open")]),
        ViewNode::Command { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Command, context, &[("bind", "CommandProps.open")]),
        ViewNode::Toast { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Toast, context, &[("source", "ToastProps.source")]),
        ViewNode::AppBar { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::AppBar, context, &[("position", "BarProps.position"), ("floating", "BarProps.floating"), ("bordered", "BarProps.bordered"), ("blurred", "BarProps.blurred"), ("hideOnScroll", "BarProps.hide_on_scroll"), ("dockOnScroll", "BarProps.dock_on_scroll")]),
        ViewNode::Footer { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Footer, context, &[("bordered", "BarProps.bordered"), ("blurred", "BarProps.blurred"), ("boxed", "BarProps.boxed")]),
        ViewNode::Sidebar { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Sidebar, context, &[("variant", "SidebarProps.style.variant"), ("scheme", "SidebarProps.style.color"), ("size", "SidebarProps.style.size")]),
        ViewNode::Drawer { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Drawer, context, &[("bind", "DrawerProps.open"), ("position", "DrawerProps.position"), ("disableOverlayClose", "DrawerProps.disable_overlay_close"), ("hideCloseButton", "DrawerProps.hide_close_button")]),
        ViewNode::Audio { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::Audio, "src", "AudioProps.src");
        }
        ViewNode::Video { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::Video, "src", "VideoProps.src");
            context.register_consumed_prop(dowe_components::BuiltinComponent::Video, "poster", "VideoProps.poster");
            context.register_consumed_prop(dowe_components::BuiltinComponent::Video, "autoplay", "VideoProps.autoplay");
        }
        ViewNode::Iframe { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::Iframe, "src", "IframeProps.src");
            context.register_consumed_prop(dowe_components::BuiltinComponent::Iframe, "sandbox", "IframeProps.sandbox");
            context.register_consumed_prop(dowe_components::BuiltinComponent::Iframe, "allow", "IframeProps.allow");
        }
        ViewNode::Device { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::Device, "device", "DeviceProps.device");
            context.register_consumed_prop(dowe_components::BuiltinComponent::Device, "zoom", "DeviceProps.options");
            context.register_consumed_prop(dowe_components::BuiltinComponent::Device, "fit", "DeviceProps.options");
        }
        ViewNode::Camera { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::Camera, "facing", "CameraProps.facing");
            context.register_consumed_prop(dowe_components::BuiltinComponent::Camera, "onCapture", "CameraProps.on_capture");
            context.register_consumed_prop(dowe_components::BuiltinComponent::Camera, "onError", "CameraProps.on_error");
        }
        ViewNode::Microphone { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::Microphone, "onError", "MicrophoneProps.on_error");
        }
        ViewNode::Date { .. } => {
        }
        ViewNode::DateRange { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::DateRange, "start", "DateRangeProps.start");
            context.register_consumed_prop(dowe_components::BuiltinComponent::DateRange, "end", "DateRangeProps.end");
        }
        ViewNode::Password { .. } => {
        }
        ViewNode::Phone { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::Phone, "country", "PhoneProps.country");
        }
        ViewNode::Pin { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::Pin, "length", "PinProps.length");
        }
        ViewNode::Textarea { .. } => {
        }
        ViewNode::Color { .. } => {
        }
        ViewNode::Dropzone { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::Dropzone, "multiple", "DropzoneProps.multiple");
            context.register_consumed_prop(dowe_components::BuiltinComponent::Dropzone, "accept", "DropzoneProps.accept");
        }
        ViewNode::Checkbox { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::Checkbox, "checked", "CheckboxProps.checked");
        }
        ViewNode::Toggle { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::Toggle, "checked", "ToggleProps.checked");
        }
        ViewNode::RadioGroup { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::RadioGroup, "orientation", "RadioGroupProps.orientation");
        }
        ViewNode::Slider { .. } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::Slider, "min", "SliderProps.min");
            context.register_consumed_prop(dowe_components::BuiltinComponent::Slider, "max", "SliderProps.max");
            context.register_consumed_prop(dowe_components::BuiltinComponent::Slider, "step", "SliderProps.step");
        }
        ViewNode::Avatar { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Avatar, context, &[("name", "AvatarProps.name"), ("alt", "AvatarProps.alt"), ("icon", "SideNavIcon.props")]),
        ViewNode::AvatarGroup { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::AvatarGroup, context, &[("items", "AvatarGroupProps.items"), ("size", "AvatarGroupProps.size")]),
        ViewNode::Badge { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Badge, context, &[("variant", "VariantProps.variant"), ("scheme", "VariantProps.color")]),
        ViewNode::Chip { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Chip, context, &[("variant", "VariantProps.variant"), ("scheme", "VariantProps.color"), ("size", "VariantProps.size"), ("rounded", "VariantProps.style.rounded")]),
        ViewNode::ChatBox { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::ChatBox, context, &[("messages", "ChatBoxProps.messages"), ("loading", "ChatBoxProps.loading"), ("sending", "ChatBoxProps.sending"), ("streaming", "ChatBoxProps.streaming"), ("hasMore", "ChatBoxProps.has_more")]),
        ViewNode::Empty { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Empty, context, &[("title", "EmptyProps.title"), ("description", "EmptyProps.description"), ("actionLabel", "EmptyProps.action_label")]),
        ViewNode::Marquee { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Marquee, context, &[("speed", "MarqueeProps.speed"), ("pauseOnHover", "MarqueeProps.pause_on_hover"), ("reverse", "MarqueeProps.reverse"), ("orientation", "MarqueeProps.orientation"), ("fade", "MarqueeProps.fade"), ("fadeColor", "MarqueeProps.fade_color"), ("gap", "MarqueeProps.gap")]),
        ViewNode::TypeWriter { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::TypeWriter, context, &[("typeSpeed", "TypeWriterProps.type_speed"), ("deleteSpeed", "TypeWriterProps.delete_speed"), ("afterTyped", "TypeWriterProps.after_typed"), ("afterDeleted", "TypeWriterProps.after_deleted"), ("repeat", "TypeWriterProps.repeat")]),
        ViewNode::RichText { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::RichText, context, &[("text", "RichTextMark.text"), ("style", "RichTextMark.style"), ("color", "RichTextMark.color")]),
        ViewNode::Record { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Record, context, &[("name", "RecordProps.name"), ("url", "RecordProps.url"), ("disabled", "RecordProps.disabled"), ("maxDuration", "RecordProps.max_duration")]),
        ViewNode::ToggleGroup { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::ToggleGroup, context, &[("value", "ToggleGroupProps.value"), ("selected", "ToggleGroupProps.selected"), ("multiple", "ToggleGroupProps.multiple"), ("wide", "ToggleGroupProps.wide"), ("vertical", "ToggleGroupProps.vertical"), ("disabled", "ToggleGroupProps.disabled"), ("ariaLabel", "ToggleGroupProps.aria_label")]),
        ViewNode::Collapsible { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Collapsible, context, &[("label", "CollapsibleProps.label"), ("defaultOpen", "CollapsibleProps.default_open"), ("disabled", "CollapsibleProps.disabled")]),
        ViewNode::Countdown { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Countdown, context, &[("target", "CountdownProps.target"), ("showDays", "CountdownProps.show_days"), ("showHours", "CountdownProps.show_hours"), ("showMinutes", "CountdownProps.show_minutes"), ("showSeconds", "CountdownProps.show_seconds"), ("size", "CountdownProps.size"), ("onComplete", "CountdownProps.on_complete")]),
        ViewNode::Map { .. } => register_swift_structural_consumed_props(dowe_components::BuiltinComponent::Map, context, &[("centerLat", "MapProps.center_lat"), ("centerLng", "MapProps.center_lng"), ("zoom", "MapProps.zoom"), ("height", "MapProps.height"), ("width", "MapProps.width"), ("showControls", "MapProps.show_controls"), ("showScale", "MapProps.show_scale"), ("interactive", "MapProps.interactive"), ("onLocation", "MapProps.on_location"), ("onLocationError", "MapProps.on_location_error"), ("onRoute", "MapProps.on_route")]),
        ViewNode::Image { props: _ } => {
            context.register_consumed_prop(dowe_components::BuiltinComponent::Image, "src", "ImageProps.src");
            context.register_consumed_prop(dowe_components::BuiltinComponent::Image, "alt", "ImageProps.alt");
            context.register_consumed_prop(dowe_components::BuiltinComponent::Image, "objectFit", "ImageProps.object_fit");
            context.register_consumed_prop(dowe_components::BuiltinComponent::Image, "loading", "ImageProps.loading");
        }
        ViewNode::Text { props, .. } | ViewNode::Title { props, .. } => {
            if props.size.is_some() || props.size_binding.is_some() {
                context.register_consumed_prop(dowe_components::BuiltinComponent::Text, "size", "TextProps.size");
            }
            if props.weight.is_some() || props.weight_binding.is_some() {
                context.register_consumed_prop(dowe_components::BuiltinComponent::Text, "weight", "TextProps.weight");
            }
            if props.letter_spacing.is_some() || props.letter_spacing_binding.is_some() {
                context.register_consumed_prop(dowe_components::BuiltinComponent::Text, "spacing", "TextProps.letter_spacing");
            }
        }
        ViewNode::Input { props } | ViewNode::Select { props, .. } => {
            let component = if matches!(node, ViewNode::Input { .. }) { dowe_components::BuiltinComponent::Input } else { dowe_components::BuiltinComponent::Select };
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
                context.register_consumed_prop(dowe_components::BuiltinComponent::Button, "scheme", "VariantProps.color");
            }
            if props.variant.is_some() || props.variant_binding.is_some() {
                context.register_consumed_prop(dowe_components::BuiltinComponent::Button, "variant", "VariantProps.variant");
            }
            if props.size.is_some() || props.size_binding.is_some() {
                context.register_consumed_prop(dowe_components::BuiltinComponent::Button, "size", "VariantProps.size");
            }
            if props.style.rounded.is_some() || props.style.rounded_binding.is_some() {
                context.register_consumed_prop(dowe_components::BuiltinComponent::Button, "rounded", "VariantProps.style.rounded");
            }
            if props.reactive.loading.is_some() {
                context.register_consumed_prop(dowe_components::BuiltinComponent::Button, "loading", "VariantProps.reactive.loading");
            }
            if props.reactive.disabled.is_some() {
                context.register_consumed_prop(dowe_components::BuiltinComponent::Button, "disabled", "VariantProps.reactive.disabled");
            }
        }
        _ => {}
    }
}

fn register_swift_structural_item_consumed_props(
    component: dowe_components::BuiltinComponent,
    item: dowe_components::ViewItemKind,
    context: &SwiftReactiveContext,
    props: &[(&'static str, &'static str)],
) {
    for (prop, field) in props {
        let mut registry = context.consumed_props.borrow_mut();
        dowe_components::register_consumed_item(&mut registry, component, item, *prop, *field);
    }
}

fn register_swift_structural_consumed_props(
    component: dowe_components::BuiltinComponent,
    context: &SwiftReactiveContext,
    props: &[(&'static str, &'static str)],
) {
    for (prop, field) in props {
        context.register_consumed_prop(component, prop, field);
    }
}

fn register_swift_chart_consumed_props(component: dowe_components::BuiltinComponent, context: &SwiftReactiveContext) {
    for (prop, field) in [("data", "ChartCommonProps.data"), ("series", "ChartCommonProps.series"), ("size", "ChartCommonProps.size"), ("palette", "ChartCommonProps.palette"), ("loading", "ChartCommonProps.loading")] {
        context.register_consumed_prop(component, prop, field);
    }
    let fields = match component {
        dowe_components::BuiltinComponent::Candlestick => vec![("stream", "CandlestickProps.stream"), ("upColor", "CandlestickProps.up_color"), ("downColor", "CandlestickProps.down_color"), ("maxPoints", "CandlestickProps.max_points")],
        dowe_components::BuiltinComponent::ArcChart => vec![("centerText", "ArcChartProps.center_text"), ("centerValue", "ArcChartProps.center_value"), ("thickness", "ArcChartProps.thickness"), ("gap", "ArcChartProps.gap"), ("startAngle", "ArcChartProps.start_angle"), ("endAngle", "ArcChartProps.end_angle"), ("showGlow", "ArcChartProps.show_glow")],
        dowe_components::BuiltinComponent::AreaChart => vec![("curve", "AreaChartProps.curve"), ("strokeWidth", "AreaChartProps.stroke_width"), ("fillOpacity", "AreaChartProps.fill_opacity"), ("stacked", "AreaChartProps.stacked"), ("showPoints", "AreaChartProps.show_points"), ("showGlow", "AreaChartProps.show_glow")],
        dowe_components::BuiltinComponent::BarChart => vec![("grouped", "BarChartProps.grouped"), ("stacked", "BarChartProps.stacked"), ("showValues", "BarChartProps.show_values"), ("barRadius", "BarChartProps.bar_radius"), ("showGlow", "BarChartProps.show_glow")],
        dowe_components::BuiltinComponent::LineChart => vec![("curve", "LineChartProps.curve"), ("strokeWidth", "LineChartProps.stroke_width"), ("pointRadius", "LineChartProps.point_radius"), ("showGradientFill", "LineChartProps.show_gradient_fill"), ("showGlow", "LineChartProps.show_glow")],
        dowe_components::BuiltinComponent::PieChart => vec![("donut", "PieChartProps.donut"), ("donutWidth", "PieChartProps.donut_width"), ("centerLabel", "PieChartProps.center_label"), ("centerValue", "PieChartProps.center_value"), ("startAngle", "PieChartProps.start_angle"), ("padAngle", "PieChartProps.pad_angle"), ("hideLabels", "PieChartProps.hide_labels"), ("hideValues", "PieChartProps.hide_values"), ("hidePercentages", "PieChartProps.hide_percentages"), ("showGlow", "PieChartProps.show_glow")],
        _ => Vec::new(),
    };
    for (prop, field) in fields {
        context.register_consumed_prop(component, prop, field);
    }
}

fn swift_form_component(node: &ViewNode) -> Option<dowe_components::BuiltinComponent> {
    match node {
        ViewNode::Input { .. } => Some(dowe_components::BuiltinComponent::Input),
        ViewNode::Select { .. } => Some(dowe_components::BuiltinComponent::Select),
        ViewNode::Checkbox { .. } => Some(dowe_components::BuiltinComponent::Checkbox),
        ViewNode::Toggle { .. } => Some(dowe_components::BuiltinComponent::Toggle),
        ViewNode::RadioGroup { .. } => Some(dowe_components::BuiltinComponent::RadioGroup),
        ViewNode::Slider { .. } => Some(dowe_components::BuiltinComponent::Slider),
        ViewNode::Date { .. } => Some(dowe_components::BuiltinComponent::Date),
        ViewNode::DateRange { .. } => Some(dowe_components::BuiltinComponent::DateRange),
        ViewNode::Password { .. } => Some(dowe_components::BuiltinComponent::Password),
        ViewNode::Phone { .. } => Some(dowe_components::BuiltinComponent::Phone),
        ViewNode::Pin { .. } => Some(dowe_components::BuiltinComponent::Pin),
        ViewNode::Textarea { .. } => Some(dowe_components::BuiltinComponent::Textarea),
        ViewNode::Color { .. } => Some(dowe_components::BuiltinComponent::Color),
        ViewNode::Dropzone { .. } => Some(dowe_components::BuiltinComponent::Dropzone),
        _ => None,
    }
}

fn render_swift_node_expression(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    if should_type_erase_swift_node(node) {
        let pad = " ".repeat(indent);
        output.push_str(&format!("{pad}AnyView(\n"));
        render_swift_node_body(
            node,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad})\n"));
    } else {
        render_swift_node_body(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
    }
}

fn should_type_erase_swift_node(node: &ViewNode) -> bool {
    match node {
        ViewNode::Scope { .. } | ViewNode::Splash { .. } | ViewNode::Children => false,
        ViewNode::Alert { props } if props.visible.is_some() => false,
        _ => true,
    }
}

fn render_swift_node_body(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    match node {
        ViewNode::Scope { .. }
        | ViewNode::Splash { .. }
        | ViewNode::Each { .. }
        | ViewNode::Box { .. }
        | ViewNode::Section { .. }
        | ViewNode::Flex { .. }
        | ViewNode::Grid { .. }
        | ViewNode::Card { .. }
        | ViewNode::Brand { .. }
        | ViewNode::Banner { .. }
        | ViewNode::Children => render_swift_structure_node(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Button { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::SelectTheme { .. }
        | ViewNode::Fab { .. }
        | ViewNode::Input { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Select { .. }
        | ViewNode::ComboBox { .. }
        | ViewNode::CsvField { .. }
        | ViewNode::DragDrop { .. }
        | ViewNode::Editor { .. }
        | ViewNode::ImageCropper { .. }
        | ViewNode::Password { .. }
        | ViewNode::Phone { .. }
        | ViewNode::Pin { .. }
        | ViewNode::Textarea { .. }
        | ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Toggle { .. } => render_swift_form_node(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Audio { .. }
        | ViewNode::Image { .. }
        | ViewNode::Camera { .. }
        | ViewNode::Microphone { .. }
        | ViewNode::Accordion { .. }
        | ViewNode::Carousel { .. }
        | ViewNode::Code { .. }
        | ViewNode::Video { .. }
        | ViewNode::Iframe { .. }
        | ViewNode::Device { .. }
        | ViewNode::Canvas { .. }
        | ViewNode::Diagram { .. }
        | ViewNode::Candlestick { .. }
        | ViewNode::ArcChart { .. }
        | ViewNode::AreaChart { .. }
        | ViewNode::BarChart { .. }
        | ViewNode::LineChart { .. }
        | ViewNode::PieChart { .. }
        | ViewNode::Table { .. } => render_swift_media_data_node(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::AvatarGroup { .. }
        | ViewNode::ChatBox { .. }
        | ViewNode::Empty { .. }
        | ViewNode::Marquee { .. }
        | ViewNode::TypeWriter { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Record { .. }
        | ViewNode::ToggleGroup { .. }
        | ViewNode::Collapsible { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Map { .. } => render_swift_rich_display_node(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Divider { .. }
        | ViewNode::Title { .. }
        | ViewNode::Text { .. }
        | ViewNode::Alert { .. }
        | ViewNode::Svg { .. } => render_swift_text_svg_alert_node(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::AppBar { .. }
        | ViewNode::Footer { .. }
        | ViewNode::BottomBar { .. }
        | ViewNode::SideNav { .. }
        | ViewNode::RailNav { .. }
        | ViewNode::Sidebar { .. }
        | ViewNode::NavMenu { .. }
        | ViewNode::Scaffold { .. }
        | ViewNode::Tabs { .. }
        | ViewNode::Drawer { .. } => render_swift_navigation_shell_node(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
        ViewNode::Avatar { .. }
        | ViewNode::Badge { .. }
        | ViewNode::Chip { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::Modal { .. }
        | ViewNode::AlertDialog { .. }
        | ViewNode::Tooltip { .. }
        | ViewNode::Toast { .. }
        | ViewNode::Dropdown { .. }
        | ViewNode::Command { .. } => render_swift_overlay_node(
            node,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        ),
    }
}
