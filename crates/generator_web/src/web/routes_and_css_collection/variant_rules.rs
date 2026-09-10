fn collect_variant_rules<'a>(
    node: &'a ViewNode,
    variants: &mut Vec<(&'static str, ColorFamily, ComponentVariant)>,
) {
    match node {
        ViewNode::Splash {
            content, children, ..
        } => {
            for child in content.iter().chain(children) {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Scope {
            actions, children, ..
        } => {
            for action in actions {
                collect_action_toast_variant_rules(action, variants);
            }
            for child in children {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Each { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Brand { children, .. }
        | ViewNode::Banner { children, .. } => {
            for child in children {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Card { props, children } => {
            if props.reactive.scheme.is_some() || props.reactive.variant.is_some() {
                for variant in [
                    ComponentVariant::Solid,
                    ComponentVariant::Solid,
                    ComponentVariant::Outlined,
                    ComponentVariant::Ghost,
                ] {
                    for color in [
                        ColorFamily::Primary,
                        ColorFamily::Secondary,
                        ColorFamily::Accent,
                        ColorFamily::Muted,
                        ColorFamily::Success,
                        ColorFamily::Info,
                        ColorFamily::Warning,
                        ColorFamily::Danger,
                        ColorFamily::Background,
                        ColorFamily::Surface,
                    ] {
                        let mut reactive_props = props.clone();
                        reactive_props.variant = Some(variant);
                        reactive_props.color = Some(color);
                        push_variant_rule(variants, "card", &reactive_props);
                    }
                }
            } else {
                push_variant_rule(variants, "card", props);
            }
            for child in children {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => {
            push_variant_rule(variants, "drawer", &props.style);
            for child in header.iter().chain(body).chain(footer) {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Avatar { props, .. } => push_variant_rule(variants, "avatar", &props.style),
        ViewNode::AvatarGroup { props, .. } => {
            push_variant_rule(variants, "avatar", &props.style);
            let counter = (
                "avatar-group-counter",
                props.style.color.unwrap_or(ColorFamily::Primary),
                props.style.variant.unwrap_or(ComponentVariant::Solid),
            );
            if !variants.contains(&counter) {
                variants.push(counter);
            }
        }
        ViewNode::ChatBox { props } => push_variant_rule(variants, "chat-box", &props.style),
        ViewNode::Empty { props } => {
            push_variant_rule(variants, "empty", &props.style);
            push_variant_rule(variants, "button", &props.style);
        }
        ViewNode::Marquee { children, .. } => {
            for child in children {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::TypeWriter { .. } => {}
        ViewNode::RichText { .. } => {}
        ViewNode::Record { props } => push_variant_rule(variants, "media", &props.style),
        ViewNode::ToggleGroup { props, .. } => {
            push_variant_rule(variants, "toggle-group", &props.style);
            push_variant_rule(variants, "toggle-group-item", &props.style);
        }
        ViewNode::Collapsible { props, children } => {
            push_variant_rule(variants, "collapsible", &props.style);
            for child in children {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Countdown { props } => {
            push_variant_rule(variants, "countdown-box", &props.style);
        }
        ViewNode::Map { props, .. } => push_variant_rule(variants, "map", &props.style),
        ViewNode::Badge { props, children } => {
            push_variant_rule(variants, "badge-content", &props.style);
            for child in children {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Chip { props, .. } => push_variant_rule(variants, "chip", &props.style),
        ViewNode::Skeleton { .. } => {}
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            push_variant_rule(variants, "modal", &props.style);
            for child in header.iter().chain(body).chain(footer) {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::AlertDialog { props } => {
            let modal = alert_dialog_modal_props(props);
            push_variant_rule(variants, "modal", &modal.style);
            let cancel = ("button", ColorFamily::Muted, ComponentVariant::Outlined);
            if !variants.contains(&cancel) {
                variants.push(cancel);
            }
            let confirm = (
                "button",
                props.style.color.unwrap_or(ColorFamily::Danger),
                ComponentVariant::Solid,
            );
            if !variants.contains(&confirm) {
                variants.push(confirm);
            }
        }
        ViewNode::Tooltip { props, children } => {
            push_variant_rule(variants, "tooltip-popover", &props.style);
            for child in children {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Toast { props } => push_variant_rule(variants, "toast", &props.style),
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            footer,
            ..
        } => {
            push_variant_rule(variants, "dropdown-popover", &props.style);
            for child in trigger.iter().chain(header).chain(footer) {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Command { props, .. } => push_variant_rule(variants, "command", &props.style),
        ViewNode::Audio { props } => push_variant_rule(variants, "media", &props.style),
        ViewNode::Image { props } => push_variant_rule(variants, "image", &props.style),
        ViewNode::Camera { props } => push_variant_rule(variants, "camera", &props.style),
        ViewNode::Microphone { props } => {
            push_variant_rule(variants, "microphone", &props.style)
        }
        ViewNode::Accordion { props, items } => {
            push_variant_rule(variants, "accordion", &props.style);
            for item in items {
                for child in &item.children {
                    collect_variant_rules(child, variants);
                }
            }
        }
        ViewNode::Carousel { props, slides } => {
            push_variant_rule(variants, "carousel", &props.style);
            for slide in slides {
                for child in &slide.children {
                    collect_variant_rules(child, variants);
                }
            }
        }
        ViewNode::Checkbox { props } => {
            push_form_variant_rules(variants, "checkbox", &props.style)
        }
        ViewNode::Color { props } => push_form_variant_rules(variants, "control", &props.style),
        ViewNode::Date { props } => push_form_variant_rules(variants, "control", &props.style),
        ViewNode::DateRange { props } => {
            push_form_variant_rules(variants, "control", &props.style)
        }
        ViewNode::RadioGroup { props, .. } => {
            let base = if matches!(props.presentation, RadioGroupPresentation::Card) {
                "radio-card-group"
            } else {
                "radio-group"
            };
            push_form_variant_rules(variants, base, &props.style)
        }
        ViewNode::Toggle { props } => {
            push_form_variant_rules(variants, "toggle", &props.style)
        }
        ViewNode::Button { props, children } => {
            if props.reactive.variant.is_some() || props.reactive.scheme.is_some() {
                for variant in [
                    ComponentVariant::Solid,
                    ComponentVariant::Solid,
                    ComponentVariant::Outlined,
                    ComponentVariant::Ghost,
                ] {
                    for color in [
                        ColorFamily::Primary,
                        ColorFamily::Secondary,
                        ColorFamily::Accent,
                        ColorFamily::Muted,
                        ColorFamily::Success,
                        ColorFamily::Info,
                        ColorFamily::Warning,
                        ColorFamily::Danger,
                    ] {
                        let mut reactive_props = props.clone();
                        reactive_props.variant = Some(variant);
                        reactive_props.color = Some(color);
                        push_variant_rule(variants, "button", &reactive_props);
                    }
                }
            } else {
                push_variant_rule(variants, "button", props);
            }
            for child in children {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::ToggleTheme { props } => {
            push_variant_rule(variants, "theme-toggle", &props.style)
        }
        ViewNode::SelectTheme { props } => push_variant_rule(variants, "control", &props.style),
        ViewNode::Fab { props, actions } => {
            push_variant_rule(variants, "fab-trigger", &props.style);
            for action in actions {
                push_variant_rule(
                    variants,
                    "fab-action-button",
                    &VariantProps {
                        color: Some(action.color),
                        variant: props.style.variant,
                        ..VariantProps::default()
                    },
                );
            }
        }
        ViewNode::Input { props } => {
            push_form_variant_rules(variants, "control", props);
        }
        ViewNode::Select { props, .. } => {
            push_form_variant_rules(variants, "control", props);
        }
        ViewNode::ComboBox { props, .. } => {
            push_form_variant_rules(variants, "control", &props.style)
        }
        ViewNode::Password { props } => {
            push_form_variant_rules(variants, "control", &props.style)
        }
        ViewNode::Phone { props } => {
            push_form_variant_rules(variants, "control", &props.style)
        }
        ViewNode::Textarea { props } => {
            push_form_variant_rules(variants, "control", &props.style)
        }
        ViewNode::CsvField { props, .. } => {
            push_form_variant_rules(variants, "button", &props.style);
        }
        ViewNode::DragDrop { props, .. } => {
            push_form_variant_rules(variants, "drag-drop", &props.style);
        }
        ViewNode::Editor { props } => {
            push_form_variant_rules(variants, "editor", &props.style);
        }
        ViewNode::ImageCropper { props } => {
            push_form_variant_rules(variants, "image-cropper", &props.style);
        }
        ViewNode::Pin { props } => {
            push_form_variant_rules(variants, "control", &props.style);
        }
        ViewNode::Slider { props } => {
            push_form_variant_rules(variants, "slider", &props.style)
        }
        ViewNode::Dropzone { props } => {
            push_form_variant_rules(variants, "dropzone-input", &props.style)
        }
        ViewNode::Code { props } => {
            push_variant_rule(variants, "code-block", &props.style);
        }
        ViewNode::Video { props } => {
            push_variant_rule(variants, "video", &props.style);
        }
        ViewNode::Iframe { .. } => {}
        ViewNode::Device { .. } => {}
        ViewNode::Canvas { .. } => {}
        ViewNode::Candlestick { props } => {
            push_variant_rule(variants, "candlestick", &props.style);
        }
        ViewNode::Diagram { props } => {
            push_variant_rule(variants, "diagram", &props.style);
        }
        ViewNode::ArcChart { props } => {
            push_variant_rule(variants, "arc-chart-container", &props.common.style);
        }
        ViewNode::AreaChart { props } => {
            push_variant_rule(variants, "area-chart-container", &props.common.style);
        }
        ViewNode::BarChart { props } => {
            push_variant_rule(variants, "bar-chart-container", &props.common.style);
        }
        ViewNode::LineChart { props } => {
            push_variant_rule(variants, "line-chart-container", &props.common.style);
        }
        ViewNode::PieChart { props } => {
            push_variant_rule(variants, "pie-chart-container", &props.common.style);
        }
        ViewNode::Table { props } => {
            push_variant_rule(variants, "table", &props.style);
        }
        ViewNode::Tree { props } => {
            push_variant_rule(variants, "tree", &props.style);
        }
        ViewNode::Divider { .. } => {}
        ViewNode::Alert { props } => {
            push_variant_rule(variants, "alert", &props.style);
        }
        ViewNode::AppBar {
            props,
            top,
            start,
            center,
            end,
            bottom,
            ..
        } => {
            push_variant_rule(variants, "appbar", &props.style);
            for child in top
                .iter()
                .chain(start)
                .chain(center)
                .chain(end)
                .chain(bottom)
            {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Footer {
            props,
            top,
            start,
            center,
            end,
            bottom,
            ..
        } => {
            push_variant_rule(variants, "footer", &props.style);
            for child in top
                .iter()
                .chain(start)
                .chain(center)
                .chain(end)
                .chain(bottom)
            {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::BottomBar { props, .. } => {
            push_variant_rule(variants, "bottombar", &props.style);
        }
        ViewNode::SideNav { props, .. } => {
            if props.style.reactive.variant.is_some() || props.style.reactive.scheme.is_some() {
                for variant in [
                    ComponentVariant::Solid,
                    ComponentVariant::Solid,
                    ComponentVariant::Outlined,
                    ComponentVariant::Ghost,
                ] {
                    for color in [
                        ColorFamily::Primary,
                        ColorFamily::Secondary,
                        ColorFamily::Accent,
                        ColorFamily::Success,
                        ColorFamily::Info,
                        ColorFamily::Warning,
                        ColorFamily::Danger,
                    ] {
                        let mut reactive_props = props.style.clone();
                        reactive_props.variant = Some(variant);
                        reactive_props.color = Some(color);
                        push_variant_rule(variants, "sidenav", &reactive_props);
                    }
                }
            } else {
                push_variant_rule(variants, "sidenav", &props.style);
            }
        }
        ViewNode::RailNav { props, .. } => {
            push_variant_rule(variants, "railnav", &props.style);
            if !props.show_labels {
                let tooltip = (
                    "tooltip-popover",
                    ColorFamily::Muted,
                    ComponentVariant::Solid,
                );
                if !variants.contains(&tooltip) {
                    variants.push(tooltip);
                }
            }
        }
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => {
            push_variant_rule(variants, "sidebar", &props.style);
            for child in header.iter().chain(body).chain(footer) {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::NavMenu { props, items } => {
            push_variant_rule(variants, "navmenu", &props.style);
            for item in items {
                collect_nav_menu_variant_rules(item, variants);
            }
        }
        ViewNode::Scaffold {
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
            ..
        } => {
            for child in app_bar
                .iter()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
                .chain(overlays)
            {
                collect_variant_rules(child, variants);
            }
        }
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                for child in &tab.children {
                    collect_variant_rules(child, variants);
                }
            }
        }
        ViewNode::Svg { .. } => {}
        ViewNode::Title { .. } | ViewNode::Text { .. } | ViewNode::Children => {}
    }
}
