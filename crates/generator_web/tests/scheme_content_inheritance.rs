use dowe_components::*;
use dowe_generator_web::build_page_chunk;
use std::path::Path;

fn text(value: &str) -> ViewNode {
    ViewNode::Text {
        props: Default::default(),
        value: value.into(),
    }
}

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
                    color_explicit: true,
                    variant_explicit: true,
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
                                color_explicit: true,
                                variant_explicit: true,
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
        color_explicit: true,
        variant_explicit: true,
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
        let route = scheme_inheritance_route(family);
        let chunk = build_page_chunk(
            Path::new("/project"),
            Path::new("/project/page.dowe"),
            "scheme roles",
            &route.page_tree,
        );
        for base in ["card", "appbar", "modal", "drawer", "accordion"] {
            let selector = format!(".{base}.is-solid.is-{}{{", family.as_str());
            let rule = chunk
                .css_content
                .split(&selector)
                .nth(1)
                .unwrap_or_else(|| panic!("missing {selector}"))
                .split('}')
                .next()
                .unwrap();
            assert!(
                rule.contains(&format!(
                    "--dowe-content-text:var(--dowe-{})",
                    family.text_token().as_str()
                )),
                "{selector}: {rule}"
            );
            assert!(
                rule.contains(&format!(
                    "--dowe-content-title:var(--dowe-{})",
                    family.title_token().as_str()
                )),
                "{selector}: {rule}"
            );
            assert!(
                rule.contains(&format!("background-color:var(--dowe-{})", family.as_str())),
                "{selector}: {rule}"
            );
        }
        assert!(chunk.css_content.contains(".card.is-solid.is-surface{"));
    }
}
