pub fn apply_design_defaults_to_tree(tree: &mut ViewNode, defaults: &DesignDefaults) {
    match tree {
        ViewNode::Splash {
            content, children, ..
        } => {
            for child in content.iter_mut().chain(children) {
                apply_design_defaults_to_tree(child, defaults);
            }
        }
        ViewNode::Scope {
            actions, children, ..
        } => {
            apply_design_defaults_to_actions(actions, defaults);
            for child in children {
                apply_design_defaults_to_tree(child, defaults);
            }
        }
        ViewNode::Each { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Brand { children, .. }
        | ViewNode::Banner { children, .. }
        | ViewNode::Marquee { children, .. }
        | ViewNode::Collapsible { children, .. }
        | ViewNode::Badge { children, .. } => {
            for child in children {
                apply_design_defaults_to_tree(child, defaults);
            }
        }
        ViewNode::Section { props, children } => {
            apply_section_defaults(props, defaults);
            for child in children {
                apply_design_defaults_to_tree(child, defaults);
            }
        }
        ViewNode::Tooltip { props, children } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Tooltip);
            for child in children {
                apply_design_defaults_to_tree(child, defaults);
            }
        }
        ViewNode::Button { props, children } => {
            let slot = if props.icon_only {
                DesignComponentSlot::IconButton
            } else {
                DesignComponentSlot::Button
            };
            apply_variant_defaults(props, defaults, slot);
            if props.icon_only {
                normalize_icon_button_visual_props(props);
            } else {
                normalize_button_visual_props(props);
            }
            apply_action_control_press_feedback(props);
            for child in children {
                apply_design_defaults_to_tree(child, defaults);
            }
        }
        ViewNode::Card { props, children } => {
            apply_variant_defaults(props, defaults, DesignComponentSlot::Card);
            for child in children {
                apply_design_defaults_to_tree(child, defaults);
            }
        }
        ViewNode::Chip { props, .. } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Chip);
        }
        ViewNode::Avatar { props, .. } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Avatar);
        }
        ViewNode::AvatarGroup { props, .. } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Avatar);
        }
        ViewNode::Input { props } => {
            apply_variant_defaults(props, defaults, DesignComponentSlot::Input);
            apply_label_floating_default(props, defaults, DesignComponentSlot::Input);
        }
        ViewNode::Select { props, .. } => {
            apply_variant_defaults(props, defaults, DesignComponentSlot::Select);
            apply_label_floating_default(props, defaults, DesignComponentSlot::Select);
        }
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Drawer);
            for child in header.iter_mut().chain(body).chain(footer) {
                apply_design_defaults_to_tree(child, defaults);
            }
        }
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Modal);
            for child in header.iter_mut().chain(body).chain(footer) {
                apply_design_defaults_to_tree(child, defaults);
            }
        }
        ViewNode::Dropdown {
            props,
            trigger,
            header,
            footer,
            ..
        } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Dropdown);
            for child in trigger.iter_mut().chain(header).chain(footer) {
                apply_design_defaults_to_tree(child, defaults);
            }
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
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::AppBar);
            for child in top
                .iter_mut()
                .chain(start)
                .chain(center)
                .chain(end)
                .chain(bottom)
            {
                apply_design_defaults_to_tree(child, defaults);
            }
        }
        ViewNode::Footer {
            props,
            top,
            start,
            center,
            end,
            bottom,
        } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Footer);
            for child in top
                .iter_mut()
                .chain(start)
                .chain(center)
                .chain(end)
                .chain(bottom)
            {
                apply_design_defaults_to_tree(child, defaults);
            }
        }
        ViewNode::BottomBar { props, .. } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui);
        }
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Sidebar);
            for child in header.iter_mut().chain(body).chain(footer) {
                apply_design_defaults_to_tree(child, defaults);
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
                .iter_mut()
                .chain(start)
                .chain(main)
                .chain(end)
                .chain(bottom_bar)
                .chain(overlays)
            {
                apply_design_defaults_to_tree(child, defaults);
            }
        }
        ViewNode::Tabs { props, tabs } => {
            if props.variant != TabsVariant::Stepper {
                apply_tabs_defaults(props, defaults);
            }
            for tab in tabs {
                for child in &mut tab.children {
                    apply_design_defaults_to_tree(child, defaults);
                }
            }
        }
        ViewNode::Accordion { props, items } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Accordion);
            for item in items {
                for child in &mut item.children {
                    apply_design_defaults_to_tree(child, defaults);
                }
            }
        }
        ViewNode::Carousel { props, slides } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui);
            for slide in slides {
                for child in &mut slide.children {
                    apply_design_defaults_to_tree(child, defaults);
                }
            }
        }
        ViewNode::ComboBox { props, .. } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui);
        }
        ViewNode::Color { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui);
            apply_label_floating_default(&mut props.style, defaults, DesignComponentSlot::Color);
        }
        ViewNode::Date { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Date);
            apply_label_floating_default(&mut props.style, defaults, DesignComponentSlot::Date);
        }
        ViewNode::DateRange { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui);
            apply_label_floating_default(&mut props.style, defaults, DesignComponentSlot::DateRange);
        }
        ViewNode::RadioGroup { props, .. } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::Toggle { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::ToggleTheme { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::SelectTheme { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::Slider { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::Dropzone { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::Checkbox { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Checkbox)
        }
        ViewNode::Fab { props, .. } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Button);
            apply_action_control_press_feedback(&mut props.style);
        }
        ViewNode::AlertDialog { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui);
        }
        ViewNode::Toast { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Toast);
        }
        ViewNode::Command { props, .. } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui);
        }
        ViewNode::Audio { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::Camera { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::Microphone { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::Image { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::Record { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::ToggleGroup { props, .. } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Button)
        }
        ViewNode::ChatBox { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Card);
        }
        ViewNode::Empty { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Card);
        }
        ViewNode::CsvField { props, .. } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui);
        }
        ViewNode::DragDrop { props, .. } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui);
        }
        ViewNode::Editor { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui);
        }
        ViewNode::ImageCropper { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui);
        }
        ViewNode::Password { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Password);
            apply_label_floating_default(&mut props.style, defaults, DesignComponentSlot::Password);
        }
        ViewNode::Phone { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui);
        }
        ViewNode::Pin { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Pin);
        }
        ViewNode::Textarea { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Textarea);
            apply_label_floating_default(
                &mut props.style,
                defaults,
                DesignComponentSlot::Textarea,
            );
        }
        ViewNode::Code { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::Video { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::Candlestick { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::Diagram { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::Table { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::Tree { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::Alert { props } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::ArcChart { props } => {
            apply_variant_defaults(&mut props.common.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::AreaChart { props } => {
            apply_variant_defaults(&mut props.common.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::BarChart { props } => {
            apply_variant_defaults(&mut props.common.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::LineChart { props } => {
            apply_variant_defaults(&mut props.common.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::PieChart { props } => {
            apply_variant_defaults(&mut props.common.style, defaults, DesignComponentSlot::Ui)
        }
        ViewNode::NavMenu { props, .. } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::NavMenu);
        }
        ViewNode::SideNav { props, .. } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::SideNav);
        }
        ViewNode::RailNav { props, .. } => {
            apply_variant_defaults(&mut props.style, defaults, DesignComponentSlot::Ui);
        }
        ViewNode::Title { props, .. } => {
            apply_text_defaults(props, defaults, DesignComponentSlot::Title);
        }
        ViewNode::Text { props, .. } => {
            apply_text_defaults(props, defaults, DesignComponentSlot::Text);
        }
        ViewNode::Divider { .. }
        | ViewNode::Iframe { .. }
        | ViewNode::Device { .. }
        | ViewNode::Canvas { .. }
        | ViewNode::Svg { .. }
        | ViewNode::TypeWriter { .. }
        | ViewNode::RichText { .. }
        | ViewNode::Map { .. }
        | ViewNode::Countdown { .. }
        | ViewNode::Skeleton { .. }
        | ViewNode::Children => {}
    }
}

