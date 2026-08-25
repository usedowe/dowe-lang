fn stateful_scaffold_drawer_layout_route(boxed: bool) -> ViewRoute {
    ViewRoute {
        id: "login".to_string(),
        route_path: "/login".to_string(),
        layout_tree: ViewNode::Scope {
            constants: Vec::new(),
            signals: vec![
                ViewSignal {
                    id: "layout.drawer.open".to_string(),
                    name: "drawerOpen".to_string(),
                    storage_key: "drawerOpen".to_string(),
                    scope: dowe_components::ViewSignalScope::Page,
                    storage: dowe_components::ViewSignalStorage::None,
                    initial: ViewSignalValue::Bool(false),
                    schema: None,
                },
                ViewSignal {
                    id: "layout.drawer.visible".to_string(),
                    name: "drawerVisible".to_string(),
                    storage_key: "drawerVisible".to_string(),
                    scope: dowe_components::ViewSignalScope::Page,
                    storage: dowe_components::ViewSignalStorage::None,
                    initial: ViewSignalValue::Bool(true),
                    schema: None,
                },
            ],
            actions: vec![ViewAction {
                id: "layout.drawer.open.action".to_string(),
                name: "openDrawer".to_string(),
                params: Vec::new(),
                return_type: None,
                kind: ViewActionKind::Assign(ViewAssignAction {
                    target: "drawerOpen".to_string(),
                    source: "drawerVisible".to_string(),
                    literal: None,
                    call: None,
                }),
            }],
            children: vec![ViewNode::Scaffold {
                props: ScaffoldProps {
                    boxed,
                    ..Default::default()
                },
                app_bar: vec![ViewNode::AppBar {
                    props: BarProps {
                        position: BarPosition::Fixed,
                        ..bar_props(false)
                    },
                    top: Vec::new(),
                    start: vec![ViewNode::Button {
                        props: VariantProps {
                            element: ElementProps {
                                on_click: Some("openDrawer".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        children: vec![text("Menu")],
                    }],
                    center: vec![text("Docs")],
                    end: Vec::new(),
                    bottom: Vec::new(),
                mobile_menu: None,
                }],
                start: vec![ViewNode::Sidebar {
                    props: SidebarProps {
                        style: VariantProps::default(),
                    },
                    header: Vec::new(),
                    body: vec![ViewNode::SideNav {
                        props: SideNavProps {
                            style: VariantProps::default(),
                            size: SideNavSize::Sm,
                            wide: true,
                            reactive_wide: None,
                        },
                        items: vec![SideNavItem::Item(SideNavItemProps {
                            label: "Overview".to_string(),
                            i18n: None,
                            description: None,
                            description_i18n: None,
                            status: None,
                            status_i18n: None,
                            icon: None,
                            on_click: None,
                            navigation: None,
                        })],
                    }],
                    footer: Vec::new(),
                }],
                main: vec![
                    ViewNode::Drawer {
                        props: DrawerProps {
                            style: VariantProps::default(),
                            open: "drawerOpen".to_string(),
                            position: DrawerPosition::Start,
                            disable_overlay_close: false,
                            hide_close_button: false,
                        },
                        header: Vec::new(),
                        body: vec![ViewNode::SideNav {
                            props: SideNavProps {
                                style: VariantProps::default(),
                                size: SideNavSize::Sm,
                                wide: true,
                                reactive_wide: None,
                            },
                            items: vec![SideNavItem::Item(SideNavItemProps {
                                label: "Overview".to_string(),
                                i18n: None,
                                description: None,
                                description_i18n: None,
                                status: None,
                                status_i18n: None,
                                icon: None,
                                on_click: None,
                                navigation: None,
                            })],
                        }],
                        footer: Vec::new(),
                    },
                    ViewNode::Children,
                ],
                end: Vec::new(),
                bottom_bar: Vec::new(),
                overlays: Vec::new(),
            }],
        },
        page_tree: ViewNode::RichText {
            props: TextProps::default(),
            marks: vec![RichTextMark {
                text: "drawerOpen openDrawer".to_string(),
                style: RichTextMarkStyle::Mark,
                color: ColorFamily::Primary,
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

#[test]
fn generates_plain_brand_navigation_with_explicit_size() {
    let mut brand_route = route();
    brand_route.layout_tree = ViewNode::Children;
    brand_route.page_tree = ViewNode::Brand {
        props: BrandProps {
            style: StyleProps {
                sizing: SizingProps {
                    w: Some(ResponsiveValue::scalar(SizeValue::Scale(
                        ScaleValue::from_half_steps(64),
                    ))),
                    h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                        ScaleValue::from_half_steps(16),
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            },
            navigation: Some(NavigationAction::Internal {
                path: "/".to_string(),
                fragment: None,
                operation: NavigationOperation::Push,
            }),
            label: Some("Dowe home".to_string()),
        },
        children: vec![text("Dowe")],
    };
    let output = generate_ios(
        &[brand_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = swift_content(&output);

    assert!(generated.contains("Button(action: { navigate(\"push\", \"/\", nil) })"));
    assert!(generated.contains("HStack(spacing: 0)"));
    assert!(generated.contains("DoweSize.fixed(CGFloat(128))"));
    assert!(generated.contains("DoweSize.fixed(CGFloat(32))"));
    assert!(generated.contains(".contentShape(Rectangle())"));
    assert!(generated.contains(".buttonStyle(.plain)"));
    assert!(generated.contains(".accessibilityLabel(Text(\"Dowe home\"))"));
}

#[test]
fn generates_external_banner_without_button_chrome() {
    let mut banner_route = route();
    banner_route.layout_tree = ViewNode::Children;
    banner_route.page_tree = ViewNode::Banner {
        props: BannerProps {
            style: StyleProps {
                spacing: SpacingProps {
                    p: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(12))),
                    ..Default::default()
                },
                ..Default::default()
            },
            navigation: NavigationAction::External {
                url: "https://dowe.dev/cloud".to_string(),
                web_target: dowe_components::WebTarget::Blank,
                native_external_mode: dowe_components::NativeExternalMode::System,
            },
            label: Some("Explore Dowe Cloud".to_string()),
        },
        children: vec![text("Build beyond code")],
    };
    let output = generate_ios(
        &[banner_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = swift_content(&output);

    assert!(
        generated
            .contains("Button(action: { openExternal(\"system\", \"https://dowe.dev/cloud\") })")
    );
    assert!(generated.contains("VStack(alignment: .leading, spacing: 0)"));
    assert!(generated.contains(".contentShape(Rectangle())"));
    assert!(generated.contains(".buttonStyle(.plain)"));
    assert!(generated.contains(".accessibilityLabel(Text(\"Explore Dowe Cloud\"))"));
}
