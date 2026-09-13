fn scheme_inheritance_route(family: ColorFamily) -> ViewRoute {
    let label = NavMenuItemProps {
        label: "Inherited menu".into(),
        i18n: None,
        description: None,
        description_i18n: None,
        icon: None,
        on_click: None,
        navigation: None,
    };
    let mut route = ViewRoute {
        id: "scheme-inheritance".into(),
        route_path: "/scheme-inheritance".into(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::AppBar {
            props: BarProps {
                style: VariantProps {
                    color: Some(family),
                    variant: Some(ComponentVariant::Solid),
                    ..Default::default()
                },
                ..Default::default()
            },
            top: vec![],
            start: vec![],
            end: vec![],
            bottom: vec![],
            mobile_menu: None,
            center: vec![ViewNode::Box {
                props: Default::default(),
                children: vec![
                    text("Inherited text"),
                    ViewNode::Title {
                        props: Default::default(),
                        value: "Inherited title".into(),
                    },
                    ViewNode::NavMenu {
                        props: NavMenuProps {
                            style: VariantProps {
                                variant: Some(ComponentVariant::Ghost),
                                color: Some(ColorFamily::Muted),
                                ..Default::default()
                            },
                            size: SideNavSize::Md,
                        },
                        items: vec![NavMenuItem::Submenu {
                            props: label,
                            items: vec![],
                        }],
                    },
                ],
            }],
        },
        sections: vec![],
        navigation_actions: vec![],
    };
    let style = VariantProps {
        color: Some(family),
        variant: Some(ComponentVariant::Solid),
        ..Default::default()
    };
    route.page_tree = ViewNode::Scope {
        constants: vec![],
        signals: vec![ViewSignal {
            id: "schemeOpen".into(),
            name: "open".into(),
            storage_key: "open".into(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::Bool(true),
            schema: None,
        }],
        actions: vec![],
        children: vec![
            route.page_tree,
            ViewNode::Card {
                props: style.clone(),
                children: vec![
                    ViewNode::Card {
                        props: Default::default(),
                        children: vec![text("Default card")],
                    },
                    ViewNode::Title {
                        props: Default::default(),
                        value: "Card sibling title".into(),
                    },
                ],
            },
            ViewNode::Drawer {
                props: DrawerProps {
                    style: style.clone(),
                    open: "open".into(),
                    position: DrawerPosition::End,
                    disable_overlay_close: false,
                    hide_close_button: false,
                },
                header: vec![],
                body: scheme_role_children("Drawer title"),
                footer: vec![],
            },
            ViewNode::Modal {
                props: ModalProps {
                    style: style.clone(),
                    open: "open".into(),
                    on_close: None,
                    disable_overlay_close: false,
                    hide_close_button: false,
                },
                header: vec![],
                body: scheme_role_children("Modal title"),
                footer: vec![],
            },
            ViewNode::Accordion {
                props: AccordionProps {
                    style,
                    multiple: false,
                },
                items: vec![AccordionItem {
                    id: "item".into(),
                    label: "Accordion".into(),
                    disabled: false,
                    default_open: true,
                    children: scheme_role_children("Accordion title"),
                }],
            },
        ],
    };
    dowe_components::apply_design_defaults_to_tree(
        &mut route.page_tree,
        &DesignConfig::default().defaults,
    );
    route
}

fn scheme_role_children(title: &str) -> Vec<ViewNode> {
    vec![ViewNode::Box {
        props: Default::default(),
        children: vec![
            text("Surface text"),
            ViewNode::Title {
                props: Default::default(),
                value: title.into(),
            },
        ],
    }]
}

#[test]
fn scheme_content_inheritance_for_all_families() {
    for family in ColorFamily::all()
        .iter()
        .copied()
        .chain([ColorFamily::from_name("brand").unwrap()])
    {
        let mut design = DesignConfig::default();
        for theme in &mut design.themes {
            theme.colors.insert(family.color_token(), "#1F3A5F".into());
            theme.colors.insert(family.text_token(), "#EBF2FA".into());
            theme.colors.insert(family.title_token(), "#FFFFFF".into());
        }
        let output = generate_android(
            &[scheme_inheritance_route(family)],
            &FontConfig::default(),
            &design,
            &[],
        );
        let source = output
            .files
            .iter()
            .map(|file| file.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(source.contains(&format!("LocalContentColor provides DoweDesign.{}, LocalDoweTitleColor provides DoweDesign.{}", family.text_token().as_str(), family.title_token().as_str())), "{} roles", family.as_str());
        let dev = dev_java_source(&output);
        let title = super::java_color(family.title_token());
        let text = super::java_color(family.text_token());
        assert!(
            dev.content
                .contains(&format!("doweText(\"Inherited title\", {title},")),
            "{} title",
            family.as_str()
        );
        assert!(
            dev.content.contains(&format!(" : {text}")),
            "{} menu",
            family.as_str()
        );
        for label in [
            "Drawer title",
            "Modal title",
            "Accordion title",
            "Card sibling title",
        ] {
            assert!(
                dev.content
                    .contains(&format!("doweText(\"{label}\", {title},")),
                "{} {label}",
                family.as_str()
            );
        }
        assert!(
            source
                .matches(&format!(
                    "titleColor = DoweDesign.{}",
                    family.title_token().as_str()
                ))
                .count()
                >= 3
        );
        assert!(dev.content.contains("Label.getCurrentTextColor()"));
        assert!(source.contains("LocalDoweTitleColor provides DoweDesign.backgroundTitle"));
    }
}
