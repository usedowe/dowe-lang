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
        let output = generate_ios(
            &[scheme_inheritance_route(family)],
            &FontConfig::default(),
            &design,
            &[],
        );
        let source = swift_content(&output);
        assert!(source.contains(&format!(".background(DoweDesign.{})", family.as_str())));
        assert!(source.contains(&format!(
            ".foregroundStyle(DoweDesign.{})",
            family.text_token().as_str()
        )));
        assert!(
            source.contains(&format!(
                ".environment(\\.doweTitleColor, DoweDesign.{})",
                family.title_token().as_str()
            )),
            "{} title",
            family.as_str()
        );
        assert!(
            source
                .matches(&format!(
                    "titleColor: DoweDesign.{}",
                    family.title_token().as_str()
                ))
                .count()
                >= 3
        );
        assert!(source.contains(".background(DoweDesign.surface)"));
        assert!(source.contains("rowSurface.foregroundStyle(contentColor)"));
        let menu = source
            .split("struct DoweNavMenuItem<")
            .nth(1)
            .unwrap()
            .split("struct DoweSideNavArrow")
            .next()
            .unwrap();
        assert!(!menu.contains("DoweDesign.backgroundText"));
        assert!(source.contains(".environment(\\.doweTitleColor, DoweDesign.backgroundTitle)"));
    }
}

#[test]
#[cfg(target_os = "macos")]
fn scheme_nav_menu_foreground_typechecks_in_swiftui() {
    let output = generate_ios(
        &[scheme_inheritance_route(ColorFamily::Primary)],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let source = swift_content(&output);
    let item = source
        .split("struct DoweNavMenuItem<")
        .nth(1)
        .unwrap()
        .split("struct DoweSideNavArrow")
        .next()
        .unwrap();
    let source = format!(
        "import SwiftUI\nenum DoweDesign {{ static let radius: CGFloat = 8 }}\nstruct DoweNavMenuItem<{item}"
    );
    let directory = std::env::temp_dir().join(format!("dowe-scheme-swift-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let file = directory.join("Scheme.swift");
    std::fs::write(&file, source).unwrap();
    let result = std::process::Command::new("xcrun")
        .args(["swiftc", "-typecheck"])
        .arg(&file)
        .output()
        .expect("Swift compiler");
    std::fs::remove_dir_all(directory).unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
