pub fn apply_theme_catalog_to_tree(tree: &mut ViewNode, design: &DesignConfig) {
    if let ViewNode::SelectTheme { props } = tree {
        props.themes = design
            .themes
            .iter()
            .map(|theme| theme.name.clone())
            .collect();
        props.default_theme = design.default_theme.clone();
    }
    match tree {
        ViewNode::Splash {
            content, children, ..
        } => {
            for child in content.iter_mut().chain(children) {
                apply_theme_catalog_to_tree(child, design);
            }
        }
        ViewNode::Scope { children, .. }
        | ViewNode::Box { children, .. }
        | ViewNode::Section { children, .. }
        | ViewNode::Flex { children, .. }
        | ViewNode::Grid { children, .. }
        | ViewNode::Card { children, .. }
        | ViewNode::Button { children, .. }
        | ViewNode::Brand { children, .. }
        | ViewNode::Banner { children, .. }
        | ViewNode::Badge { children, .. }
        | ViewNode::Tooltip { children, .. }
        | ViewNode::Marquee { children, .. }
        | ViewNode::Collapsible { children, .. }
        | ViewNode::Each { children, .. } => {
            for child in children {
                apply_theme_catalog_to_tree(child, design);
            }
        }
        ViewNode::Tabs { tabs, .. } => {
            for tab in tabs {
                for child in &mut tab.children {
                    apply_theme_catalog_to_tree(child, design);
                }
            }
        }
        ViewNode::Accordion { items, .. } => {
            for item in items {
                for child in &mut item.children {
                    apply_theme_catalog_to_tree(child, design);
                }
            }
        }
        ViewNode::Carousel { slides, .. } => {
            for slide in slides {
                for child in &mut slide.children {
                    apply_theme_catalog_to_tree(child, design);
                }
            }
        }
        ViewNode::Drawer {
            header,
            body,
            footer,
            ..
        }
        | ViewNode::Modal {
            header,
            body,
            footer,
            ..
        }
        | ViewNode::Sidebar {
            header,
            body,
            footer,
            ..
        } => {
            for child in header
                .iter_mut()
                .chain(body.iter_mut())
                .chain(footer.iter_mut())
            {
                apply_theme_catalog_to_tree(child, design);
            }
        }
        ViewNode::AppBar {
            top,
            start,
            center,
            end,
            bottom,
            ..
        }
        | ViewNode::Footer {
            top,
            start,
            center,
            end,
            bottom,
            ..
        } => {
            for child in top
                .iter_mut()
                .chain(start.iter_mut())
                .chain(center.iter_mut())
                .chain(end.iter_mut())
                .chain(bottom.iter_mut())
            {
                apply_theme_catalog_to_tree(child, design);
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
                .chain(start.iter_mut())
                .chain(main.iter_mut())
                .chain(end.iter_mut())
                .chain(bottom_bar.iter_mut())
                .chain(overlays.iter_mut())
            {
                apply_theme_catalog_to_tree(child, design);
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            footer,
            ..
        } => {
            for child in trigger
                .iter_mut()
                .chain(header.iter_mut())
                .chain(footer.iter_mut())
            {
                apply_theme_catalog_to_tree(child, design);
            }
        }
        ViewNode::NavMenu { items, .. } => {
            for item in items {
                if let NavMenuItem::Megamenu { content, .. } = item {
                    for child in content {
                        apply_theme_catalog_to_tree(child, design);
                    }
                }
            }
        }
        _ => {}
    }
}

fn apply_variant_defaults(
    props: &mut VariantProps,
    defaults: &DesignDefaults,
    slot: DesignComponentSlot,
) {
    if props.size.is_none() {
        props.size = defaults
            .size
            .get(&slot)
            .or_else(|| defaults.size.get(&DesignComponentSlot::Ui))
            .copied();
    }
    if props.variant.is_none() {
        props.variant = defaults
            .variant
            .get(&slot)
            .or_else(|| defaults.variant.get(&DesignComponentSlot::Ui))
            .copied();
    }
    if props.color.is_none() {
        props.color = defaults
            .scheme
            .get(&slot)
            .or_else(|| defaults.scheme.get(&DesignComponentSlot::Ui))
            .copied();
    }
    apply_style_defaults(&mut props.style, defaults, slot);
}

fn apply_tabs_defaults(props: &mut TabsProps, defaults: &DesignDefaults) {
    if !props.variant_explicit {
        props.variant = defaults
            .tabs_variant
            .get(&DesignComponentSlot::Tabs)
            .or_else(|| defaults.tabs_variant.get(&DesignComponentSlot::Ui))
            .copied()
            .unwrap_or(TabsVariant::Pills);
    }
    if !props.color_explicit {
        props.color = defaults
            .scheme
            .get(&DesignComponentSlot::Tabs)
            .or_else(|| defaults.scheme.get(&DesignComponentSlot::Ui))
            .copied()
            .unwrap_or(ColorFamily::Primary);
    }
    apply_style_defaults(&mut props.style, defaults, DesignComponentSlot::Tabs);
}

fn apply_label_floating_default(
    props: &mut VariantProps,
    defaults: &DesignDefaults,
    slot: DesignComponentSlot,
) {
    if !props.label_floating {
        props.label_floating = defaults
            .label_floating
            .get(&slot)
            .copied()
            .unwrap_or(false);
    }
}

fn apply_style_defaults(
    props: &mut StyleProps,
    defaults: &DesignDefaults,
    slot: DesignComponentSlot,
) {
    if props.rounded.is_none()
        && let Some(value) = defaults
            .radius
            .get(&slot)
            .or_else(|| defaults.radius.get(&DesignComponentSlot::Ui))
    {
        props.rounded = Some(ResponsiveValue::scalar(*value));
    }
    if props.border.is_none()
        && let Some(value) = defaults
            .border
            .get(&slot)
            .or_else(|| defaults.border.get(&DesignComponentSlot::Ui))
    {
        props.border = Some(ResponsiveValue::scalar(*value));
    }
    if props.border_color.is_none() {
        props.border_color = defaults
            .border_color
            .get(&slot)
            .or_else(|| defaults.border_color.get(&DesignComponentSlot::Ui))
            .copied();
    }
    if props.shadow.is_none()
        && let Some(value) = defaults
            .shadow
            .get(&slot)
            .or_else(|| defaults.shadow.get(&DesignComponentSlot::Ui))
    {
        props.shadow = Some(ResponsiveValue::scalar(*value));
    }
    if props.shadow_color.is_none() {
        props.shadow_color = defaults
            .shadow_color
            .get(&slot)
            .or_else(|| defaults.shadow_color.get(&DesignComponentSlot::Ui))
            .copied();
    }
}
