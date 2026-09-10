fn register_compose_structure_consumed_props(node: &ViewNode, context: &ComposeReactiveContext) {
    match node {
        ViewNode::Box { props, .. } | ViewNode::Section { props, .. } => {
            if props.bg.is_some() || props.bg_binding.is_some() {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::Box,
                    "bg",
                    "StyleProps.bg",
                );
            }
            if props.text.is_some() || props.text_binding.is_some() {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::Box,
                    "color",
                    "StyleProps.text",
                );
            }
            if props.rounded.is_some() || props.rounded_binding.is_some() {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::Box,
                    "rounded",
                    "StyleProps.rounded",
                );
            }
            if props.spacing.p.is_some() || props.spacing.p_binding.is_some() {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::Box,
                    "p",
                    "SpacingProps.p",
                );
            }
        }
        ViewNode::Candlestick { .. } => register_compose_chart_consumed_props(
            dowe_components::BuiltinComponent::Candlestick,
            context,
        ),
        ViewNode::ArcChart { .. } => register_compose_chart_consumed_props(
            dowe_components::BuiltinComponent::ArcChart,
            context,
        ),
        ViewNode::AreaChart { .. } => register_compose_chart_consumed_props(
            dowe_components::BuiltinComponent::AreaChart,
            context,
        ),
        ViewNode::BarChart { .. } => register_compose_chart_consumed_props(
            dowe_components::BuiltinComponent::BarChart,
            context,
        ),
        ViewNode::LineChart { .. } => register_compose_chart_consumed_props(
            dowe_components::BuiltinComponent::LineChart,
            context,
        ),
        ViewNode::PieChart { .. } => register_compose_chart_consumed_props(
            dowe_components::BuiltinComponent::PieChart,
            context,
        ),
        ViewNode::Tabs { props, .. } => {
            let component = if props.variant == dowe_components::TabsVariant::Stepper {
                dowe_components::BuiltinComponent::Stepper
            } else {
                dowe_components::BuiltinComponent::Tabs
            };
            register_compose_structural_consumed_props(
                component,
                context,
                &[("position", "TabsProps.position")],
            );
            if props.style.element.bind.is_some() {
                context.register_consumed_prop(component, "bind", "ElementProps.bind");
            }
            register_compose_structural_item_consumed_props(
                component,
                dowe_components::ViewItemKind::Tab,
                context,
                &[
                    ("id", "TabItem.id"),
                    ("label", "TabItem.label"),
                    ("i18n", "TabItem.i18n"),
                ],
            );
        }
        ViewNode::Accordion { .. } => register_compose_structural_item_consumed_props(
            dowe_components::BuiltinComponent::Accordion,
            dowe_components::ViewItemKind::Accordion,
            context,
            &[
                ("id", "AccordionItem.id"),
                ("label", "AccordionItem.label"),
                ("disabled", "AccordionItem.disabled"),
                ("defaultOpen", "AccordionItem.default_open"),
            ],
        ),
        ViewNode::Carousel { .. } => register_compose_structural_consumed_props(
            dowe_components::BuiltinComponent::Carousel,
            context,
            &[
                ("slidesPerView", "CarouselProps.slides_per_view"),
                ("autoplay", "CarouselProps.autoplay"),
                ("orientation", "CarouselProps.orientation"),
            ],
        ),
        ViewNode::Table { .. } => register_compose_structural_item_consumed_props(
            dowe_components::BuiltinComponent::Table,
            dowe_components::ViewItemKind::TableColumn,
            context,
            &[
                ("field", "TableColumn.field"),
                ("label", "TableColumn.label"),
                ("align", "TableColumn.align"),
            ],
        ),
        ViewNode::Tree { .. } => register_compose_structural_consumed_props(
            dowe_components::BuiltinComponent::Tree,
            context,
            &[
                ("data", "TreeProps.data"),
                ("bind", "TreeProps.bind"),
                ("defaultOpen", "TreeProps.default_open"),
                ("emptyLabel", "TreeProps.empty_label"),
                ("ariaLabel", "TreeProps.aria_label"),
                ("onSelect", "TreeProps.on_select"),
                ("variant", "VariantProps.variant"),
                ("scheme", "VariantProps.color"),
            ],
        ),
        ViewNode::NavMenu { .. } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::NavMenu,
                "variant",
                "NavMenuProps.style.variant",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::NavMenu,
                "scheme",
                "NavMenuProps.style.color",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::NavMenu,
                "size",
                "NavMenuProps.size",
            );
            register_compose_structural_item_consumed_props(
                dowe_components::BuiltinComponent::NavMenu,
                dowe_components::ViewItemKind::NavMenu,
                context,
                &[
                    ("label", "NavMenuItemProps.label"),
                    ("i18n", "NavMenuItemProps.i18n"),
                    ("description", "NavMenuItemProps.description"),
                    ("href", "NavMenuItemProps.navigation"),
                ],
            );
        }
        ViewNode::SideNav { .. } => {
            for (prop, field) in [
                ("variant", "SideNavProps.style.variant"),
                ("scheme", "SideNavProps.style.color"),
                ("size", "SideNavProps.size"),
                ("wide", "SideNavProps.wide"),
            ] {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::SideNav,
                    prop,
                    field,
                );
            }
            register_compose_structural_item_consumed_props(
                dowe_components::BuiltinComponent::SideNav,
                dowe_components::ViewItemKind::SideNav,
                context,
                &[
                    ("label", "SideNavItemProps.label"),
                    ("i18n", "SideNavItemProps.i18n"),
                    ("description", "SideNavItemProps.description"),
                    ("href", "SideNavItemProps.navigation"),
                ],
            );
        }
        ViewNode::RailNav { .. } => {
            for (prop, field) in [
                ("variant", "RailNavProps.style.variant"),
                ("scheme", "RailNavProps.style.color"),
                ("size", "RailNavProps.size"),
            ] {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::RailNav,
                    prop,
                    field,
                );
            }
            register_compose_structural_item_consumed_props(
                dowe_components::BuiltinComponent::RailNav,
                dowe_components::ViewItemKind::RailNav,
                context,
                &[
                    ("label", "RailNavItemProps.label"),
                    ("i18n", "RailNavItemProps.i18n"),
                    ("href", "RailNavItemProps.navigation"),
                ],
            );
        }
        ViewNode::BottomBar { .. } => {
            for (prop, field) in [
                ("floating", "BarProps.floating"),
                ("bordered", "BarProps.bordered"),
                ("blurred", "BarProps.blurred"),
                ("boxed", "BarProps.boxed"),
            ] {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::BottomBar,
                    prop,
                    field,
                );
            }
            register_compose_structural_item_consumed_props(
                dowe_components::BuiltinComponent::BottomBar,
                dowe_components::ViewItemKind::BottomBar,
                context,
                &[
                    ("label", "BottomBarTab.label"),
                    ("href", "BottomBarTab.navigation"),
                ],
            );
        }
        ViewNode::Svg { .. } => register_compose_structural_consumed_props(
            dowe_components::BuiltinComponent::Svg,
            context,
            &[("viewBox", "SvgProps.view_box"), ("data", "SvgProps.data")],
        ),
        ViewNode::Canvas { props } => {
            let consumed = if props.is_draw {
                &[
                    ("scene", "CanvasProps.scene"),
                    ("bind", "CanvasProps.layer_bind"),
                    ("selected", "CanvasProps.selected_layer"),
                    ("draw", "CanvasProps.draw"),
                    ("drawMode", "CanvasProps.draw_mode"),
                    ("onPointer", "CanvasProps.on_pointer"),
                    ("onKey", "CanvasProps.on_key"),
                    ("onMotion", "CanvasProps.on_motion"),
                    ("onLayerAdd", "CanvasProps.on_layer_add"),
                    ("onLayerChange", "CanvasProps.on_layer_change"),
                    ("onLayerRemove", "CanvasProps.on_layer_remove"),
                    ("onLayerSelect", "CanvasProps.on_layer_select"),
                ][..]
            } else {
                &[
                    ("scene", "CanvasProps.scene"),
                    ("draw", "CanvasProps.draw"),
                    ("drawMode", "CanvasProps.draw_mode"),
                    ("onPointer", "CanvasProps.on_pointer"),
                    ("onKey", "CanvasProps.on_key"),
                    ("onMotion", "CanvasProps.on_motion"),
                ][..]
            };
            register_compose_structural_consumed_props(
                if props.is_draw { dowe_components::BuiltinComponent::Draw } else { dowe_components::BuiltinComponent::Canvas },
                context,
                consumed,
            )
        }
        ViewNode::Modal { .. } => register_compose_structural_consumed_props(
            dowe_components::BuiltinComponent::Modal,
            context,
            &[("bind", "ModalProps.open")],
        ),
        ViewNode::AlertDialog { .. } => register_compose_structural_consumed_props(
            dowe_components::BuiltinComponent::AlertDialog,
            context,
            &[("bind", "AlertDialogProps.open")],
        ),
        ViewNode::Command { .. } => register_compose_structural_consumed_props(
            dowe_components::BuiltinComponent::Command,
            context,
            &[("bind", "CommandProps.open")],
        ),
        ViewNode::Toast { .. } => register_compose_structural_consumed_props(
            dowe_components::BuiltinComponent::Toast,
            context,
            &[("source", "ToastProps.source")],
        ),
        ViewNode::AppBar { .. } => register_compose_structural_consumed_props(
            dowe_components::BuiltinComponent::AppBar,
            context,
            &[
                ("position", "BarProps.position"),
                ("floating", "BarProps.floating"),
                ("bordered", "BarProps.bordered"),
                ("blurred", "BarProps.blurred"),
                ("hideOnScroll", "BarProps.hide_on_scroll"),
                ("dockOnScroll", "BarProps.dock_on_scroll"),
            ],
        ),
        ViewNode::Footer { .. } => register_compose_structural_consumed_props(
            dowe_components::BuiltinComponent::Footer,
            context,
            &[
                ("bordered", "BarProps.bordered"),
                ("blurred", "BarProps.blurred"),
                ("boxed", "BarProps.boxed"),
            ],
        ),
        ViewNode::Sidebar { .. } => register_compose_structural_consumed_props(
            dowe_components::BuiltinComponent::Sidebar,
            context,
            &[
                ("variant", "SidebarProps.style.variant"),
                ("scheme", "SidebarProps.style.color"),
                ("size", "SidebarProps.style.size"),
            ],
        ),
        ViewNode::Scaffold { .. } => register_compose_structural_consumed_props(
            dowe_components::BuiltinComponent::Scaffold,
            context,
            &[
                ("safeAreaTop", "ScaffoldProps.safe_area_top"),
                ("safeAreaBottom", "ScaffoldProps.safe_area_bottom"),
            ],
        ),
        ViewNode::Drawer { .. } => register_compose_structural_consumed_props(
            dowe_components::BuiltinComponent::Drawer,
            context,
            &[
                ("bind", "DrawerProps.open"),
                ("position", "DrawerProps.position"),
                ("disableOverlayClose", "DrawerProps.disable_overlay_close"),
                ("hideCloseButton", "DrawerProps.hide_close_button"),
            ],
        ),
        _ => {}
    }
}
