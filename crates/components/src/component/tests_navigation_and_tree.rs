#[test]
fn validates_responsive_text_typography_props() {
    let node = text_component_node(
        BuiltinComponent::Text,
        vec![
            responsive_string_prop("size", &[("xs", "sm"), ("md", "lg")]),
            responsive_string_prop("align", &[("xs", "start"), ("md", "justify")]),
            responsive_string_prop(
                "weight",
                &[("xs", "thin"), ("md", "extralight"), ("lg", "black")],
            ),
            responsive_string_prop("spacing", &[("xs", "normal"), ("md", "wide")]),
            responsive_string_prop("bg", &[("md", "info")]),
            responsive_string_prop(
                "font",
                &[("xs", "inter"), ("md", "manrope"), ("lg", "lora")],
            ),
        ],
        "Login",
    )
    .expect("text");

    match node {
        ViewNode::Text { props, .. } => {
            assert_eq!(props.size.expect("size").entries.len(), 2);
            assert_eq!(
                props.align.expect("align").entries[1].value,
                TextAlign::Justify
            );
            assert_eq!(
                props.weight.expect("weight").entries[2].value,
                TextWeight::Black
            );
            assert_eq!(
                props.letter_spacing.expect("spacing").entries[1].value,
                TextSpacing::Wide
            );
            assert!(props.style.bg.is_some());
            let font = props.style.font.expect("font");
            assert_eq!(font.entries.len(), 3);
            assert_eq!(font.entries[1].value, FontFamily::Manrope);
            assert_eq!(font.entries[2].value, FontFamily::Lora);
        }
        _ => panic!("text"),
    }
}

#[test]
fn validates_show_visibility_props() {
    let node = container_component_node(
        BuiltinComponent::Box,
        vec![responsive_boolean_prop(
            "show",
            &[("xs", false), ("md", true)],
        )],
        vec![text_node("Ready").expect("text")],
        false,
    )
    .expect("box");

    match node {
        ViewNode::Box { props, .. } => match props.element.show.expect("show") {
            VisibilityCondition::Static(value) => {
                assert_eq!(value.entries.len(), 2);
                assert_eq!(value.entries[0].breakpoint, Breakpoint::Xs);
                assert!(!value.entries[0].value);
                assert_eq!(value.entries[1].breakpoint, Breakpoint::Md);
                assert!(value.entries[1].value);
            }
            VisibilityCondition::Signal(_) => panic!("static show"),
            VisibilityCondition::NumberComparison { .. } => panic!("static show"),
            VisibilityCondition::StringEquality { .. } => panic!("static show"),
        },
        _ => panic!("box"),
    }

    let node = text_component_node(
        BuiltinComponent::Text,
        vec![string_prop("show", "isReady")],
        "Ready",
    )
    .expect("text");

    match node {
        ViewNode::Text { props, .. } => {
            assert_eq!(
                props.style.element.show,
                Some(VisibilityCondition::Signal("isReady".to_string()))
            );
        }
        _ => panic!("text"),
    }

    let error = text_component_node(
        BuiltinComponent::Text,
        vec![responsive_string_prop("show", &[("xs", "false")])],
        "Ready",
    )
    .expect_err("invalid show");
    assert_eq!(error, ComponentError::invalid_prop("show", "boolean"));
}

#[test]
fn validates_side_nav_props_entries_and_icons() {
    let icon = super::side_nav_icon_component(
        svg_component_node(
            vec![string_prop("viewBox", "0 0 24 24")],
            vec![svg_path_component(vec![string_prop("d", "M3 11l9-8 9 8v10H3z")]).expect("path")],
        )
        .expect("svg"),
    )
    .expect("icon");
    let item = super::side_nav_item_component(
        vec![
            string_prop("label", "Home"),
            string_prop("description", "Overview"),
            string_prop("href", "/"),
        ],
        Some(icon),
    )
    .expect("item");
    let submenu = super::side_nav_submenu_component(
        vec![string_prop("label", "Content")],
        None,
        true,
        false,
        vec![super::SideNavItemProps {
            label: "Blogs".to_string(),
            i18n: None,
            description: None,
            description_i18n: None,
            status: None,
            status_i18n: None,
            icon: None,
            on_click: None,
            navigation: None,
        }],
    )
    .expect("submenu");
    let node = super::side_nav_component_node(
        vec![
            string_prop("variant", "ghost"),
            string_prop("scheme", "primary"),
            string_prop("size", "lg"),
            boolean_prop("wide", true),
        ],
        vec![item, submenu],
    )
    .expect("side nav");

    match node {
        ViewNode::SideNav { props, items } => {
            assert_eq!(props.style.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.style.color, Some(ColorFamily::Primary));
            assert_eq!(props.size, super::SideNavSize::Lg);
            assert!(props.wide);
            assert!(matches!(&items[0], super::SideNavItem::Item(props) if props.icon.is_some()));
            assert!(
                matches!(&items[1], super::SideNavItem::Submenu { open: true, bordered: false, items, .. } if items.len() == 1)
            );
        }
        _ => panic!("side nav"),
    }
    let structural_scheme = super::side_nav_component_node(
        vec![string_prop("scheme", "primary")],
        vec![super::SideNavItem::Divider],
    )
    .expect("primary scheme");
    let ViewNode::SideNav { props, .. } = structural_scheme else {
        panic!("side nav primary scheme");
    };
    assert_eq!(props.style.color, Some(ColorFamily::Primary));
}

#[test]
fn side_nav_memory_keys_use_ids_and_normalized_structure() {
    let items = vec![super::SideNavItem::Item(super::SideNavItemProps {
        label: "Home".to_string(),
        i18n: None,
        description: None,
        description_i18n: None,
        status: None,
        status_i18n: None,
        icon: None,
        on_click: None,
        navigation: None,
    })];
    let node = super::side_nav_component_node(Vec::new(), items.clone()).expect("side nav");
    let ViewNode::SideNav { mut props, .. } = node else {
        panic!("side nav");
    };
    assert_eq!(props.style.variant, None);
    assert_eq!(props.style.color, None);
    let structural = super::side_nav_memory_key(&props, &items);
    assert!(structural.starts_with("structure:"));
    assert_eq!(structural, super::side_nav_memory_key(&props, &items));
    props.style.element.id = Some("primary-navigation".to_string());
    assert_eq!(
        super::side_nav_memory_key(&props, &items),
        "id:primary-navigation"
    );
    props.style.element.id = Some("secondary-navigation".to_string());
    assert_eq!(
        super::side_nav_memory_key(&props, &items),
        "id:secondary-navigation"
    );
}

#[test]
fn validates_rail_nav_props_items_and_required_icons() {
    let item = super::rail_nav_item_component(vec![
        string_prop("label", "Home"),
        string_prop("icon", "home"),
        string_prop("href", "/"),
    ])
    .expect("item");
    let node = super::rail_nav_component_node(
        vec![
            string_prop("variant", "ghost"),
            string_prop("scheme", "primary"),
            string_prop("size", "lg"),
            boolean_prop("showLabels", true),
        ],
        vec![item, super::RailNavItem::Divider],
    )
    .expect("rail nav");

    match node {
        ViewNode::RailNav { props, items } => {
            assert_eq!(props.style.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.style.color, Some(ColorFamily::Primary));
            assert_eq!(props.size, super::SideNavSize::Lg);
            assert!(props.show_labels);
            assert!(
                matches!(&items[0], super::RailNavItem::Item(props) if props.navigation.is_some())
            );
            assert!(matches!(&items[1], super::RailNavItem::Divider));
        }
        _ => panic!("rail nav"),
    }

    let missing_icon = super::rail_nav_item_component(vec![string_prop("label", "Home")])
        .expect_err("missing icon");
    assert!(missing_icon
        .to_string()
        .contains("invalid value for prop `icon`"));

    let conflicting_action = super::rail_nav_item_component(vec![
        string_prop("label", "Home"),
        string_prop("icon", "home"),
        string_prop("href", "/"),
        string_prop("onClick", "openHome"),
    ])
    .expect_err("conflicting action");
    assert!(conflicting_action
        .to_string()
        .contains("`href` and `onClick` cannot be used on the same RailNav item"));
}

#[test]
fn validates_navigation_shell_components() {
    let nav_item = super::nav_menu_item_component(
        vec![string_prop("label", "Home"), string_prop("href", "/")],
        None,
    )
    .expect("nav item");
    let submenu = super::nav_menu_submenu_component(
        vec![string_prop("label", "Docs")],
        None,
        vec![super::NavMenuItemProps {
            label: "Guide".to_string(),
            i18n: Some("navigation.guide".to_string()),
            description: Some("Start here".to_string()),
            description_i18n: None,
            icon: None,
            on_click: None,
            navigation: None,
        }],
    )
    .expect("submenu");
    let megamenu = super::nav_menu_megamenu_component(
        vec![string_prop("label", "Resources")],
        None,
        vec![text_node("Resource hub").expect("text")],
        true,
    )
    .expect("megamenu");
    let nav_menu = super::nav_menu_component_node(
        vec![
            string_prop("variant", "ghost"),
            string_prop("scheme", "primary"),
            string_prop("size", "lg"),
        ],
        vec![nav_item, submenu, megamenu],
    )
    .expect("nav menu");

    match &nav_menu {
        ViewNode::NavMenu { props, items } => {
            assert_eq!(props.style.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.style.color, Some(ColorFamily::Primary));
            assert_eq!(props.size, super::SideNavSize::Lg);
            assert_eq!(items.len(), 3);
            assert!(
                matches!(&items[0], super::NavMenuItem::Item(props) if props.navigation.is_some())
            );
            assert!(
                matches!(&items[1], super::NavMenuItem::Submenu { items, .. } if items.len() == 1)
            );
            assert!(
                matches!(&items[2], super::NavMenuItem::Megamenu { content, .. } if content.len() == 1)
            );
        }
        _ => panic!("nav menu"),
    }

    let sidebar_item = super::side_nav_item_component(
        vec![string_prop("label", "Side Home"), string_prop("href", "/")],
        None,
    )
    .expect("sidebar item");
    let sidebar_nav = super::side_nav_component_node(
        vec![string_prop("size", "sm"), boolean_prop("wide", true)],
        vec![sidebar_item],
    )
    .expect("sidebar nav");
    let sidebar = super::sidebar_component_node(
        vec![
            string_prop("variant", "solid"),
            string_prop("scheme", "primary"),
        ],
        vec![text_node("Header").expect("header")],
        vec![sidebar_nav],
        vec![text_node("Footer").expect("footer")],
        false,
    )
    .expect("sidebar");

    match &sidebar {
        ViewNode::Sidebar {
            props,
            header,
            body,
            footer,
        } => {
            assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
            assert_eq!(props.style.color, Some(ColorFamily::Primary));
            assert_eq!(header.len(), 1);
            assert_eq!(body.len(), 1);
            assert_eq!(footer.len(), 1);
        }
        _ => panic!("sidebar"),
    }

    let scaffold = super::scaffold_component_node(
        vec![boolean_prop("boxed", true)],
        vec![nav_menu],
        vec![sidebar],
        vec![text_node("Main").expect("main")],
        Vec::new(),
        vec![text_node("Bottom").expect("bottom")],
        vec![text_node("Overlay").expect("overlay")],
        true,
    )
    .expect("scaffold");
    match scaffold {
        ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
        } => {
            assert!(props.boxed);
            assert_eq!(app_bar.len(), 1);
            assert_eq!(start.len(), 1);
            assert_eq!(main.len(), 1);
            assert!(end.is_empty());
            assert_eq!(bottom_bar.len(), 1);
            assert_eq!(overlays.len(), 1);
        }
        _ => panic!("scaffold"),
    }

    let error = super::nav_menu_component_node(
        vec![string_prop("color", "primary")],
        vec![
            super::nav_menu_item_component(vec![string_prop("label", "Home")], None).expect("item"),
        ],
    )
    .expect_err("color error");
    assert_eq!(
        error,
        ComponentError::new("unknown prop `color` on `NavMenu`; use `scheme` for visual family")
    );

    let error = super::sidebar_component_node(
        vec![string_prop("color", "primary")],
        Vec::new(),
        vec![text_node("Body").expect("body")],
        Vec::new(),
        false,
    )
    .expect_err("color error");
    assert_eq!(
        error,
        ComponentError::new("unknown prop `color` on `Sidebar`; use `scheme` for visual family")
    );
}

#[test]
fn validates_drawer_props_and_children() {
    let node = super::drawer_component_node(
        vec![
            string_prop("open", "drawerOpen"),
            string_prop("position", "end"),
            string_prop("variant", "ghost"),
            string_prop("scheme", "primary"),
            boolean_prop("disableOverlayClose", true),
            boolean_prop("hideCloseButton", true),
            responsive_boolean_prop("show", &[("xs", true), ("md", false)]),
        ],
        vec![text_node("Menu").expect("header")],
        vec![text_node("Navigation").expect("text")],
        vec![text_node("Footer").expect("footer")],
        false,
    )
    .expect("drawer");

    match node {
        ViewNode::Drawer {
            props,
            header,
            body,
            footer,
        } => {
            assert_eq!(props.open, "drawerOpen");
            assert_eq!(props.position, super::DrawerPosition::End);
            assert_eq!(props.style.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.style.color, Some(ColorFamily::Primary));
            assert!(props.disable_overlay_close);
            assert!(props.hide_close_button);
            assert!(props.style.element.show.is_some());
            assert_eq!(header.len(), 1);
            assert_eq!(body.len(), 1);
            assert_eq!(footer.len(), 1);
        }
        _ => panic!("drawer"),
    }

    let error = super::drawer_component_node(
        vec![string_prop("open", "drawerOpen")],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
    )
    .expect_err("children");
    assert_eq!(
        error,
        ComponentError::invalid_prop_combination("Drawer requires body children")
    );
}

#[test]
fn validates_display_and_overlay_component_props() {
    let avatar = super::avatar_component_node(
        vec![
            string_prop("name", "Ada"),
            string_prop("scheme", "success"),
            string_prop("variant", "solid"),
            string_prop("size", "lg"),
            string_prop("status", "online"),
            boolean_prop("bordered", true),
        ],
        None,
    )
    .expect("avatar");
    match avatar {
        ViewNode::Avatar { props, .. } => {
            assert_eq!(props.style.color, Some(ColorFamily::Success));
            assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
            assert_eq!(props.size, ButtonSize::Lg);
            assert_eq!(props.status, Some(super::AvatarStatus::Online));
            assert!(props.bordered);
        }
        _ => panic!("avatar"),
    }

    let badge = super::badge_component_node(
        vec![
            string_prop("text", "3"),
            string_prop("position", "bottom-right"),
        ],
        vec![text_node("Inbox").expect("text")],
        false,
    )
    .expect("badge");
    assert!(matches!(
        badge,
        ViewNode::Badge {
            props: super::BadgeProps {
                position: super::OverlayCornerPosition::BottomRight,
                ..
            },
            ..
        }
    ));

    let chip = super::chip_component_node(
        vec![string_prop("onClose", "close"), string_prop("size", "sm")],
        "Filter",
        None,
        None,
    )
    .expect("chip");
    assert!(matches!(
        chip,
        ViewNode::Chip {
            props: super::ChipProps {
                on_close: Some(_),
                ..
            },
            ..
        }
    ));

    let skeleton = super::skeleton_component_node(vec![
        string_prop("variant", "rounded"),
        string_prop("animation", "pulse"),
    ])
    .expect("skeleton");
    assert!(matches!(
        skeleton,
        ViewNode::Skeleton {
            props: super::SkeletonProps {
                variant: super::SkeletonVariant::Rounded,
                animation: super::SkeletonAnimation::Pulse,
                ..
            }
        }
    ));

    let modal = super::modal_component_node(
        vec![
            string_prop("open", "open"),
            string_prop("scheme", "surface"),
        ],
        vec![text_node("Header").expect("text")],
        vec![text_node("Body").expect("text")],
        vec![text_node("Footer").expect("text")],
        false,
    )
    .expect("modal");
    assert!(matches!(
        modal,
        ViewNode::Modal {
            props: super::ModalProps { open, .. },
            header,
            body,
            footer,
        } if open == "open" && header.len() == 1 && body.len() == 1 && footer.len() == 1
    ));

    let dialog = super::alert_dialog_component_node(vec![
        string_prop("open", "open"),
        string_prop("title", "Delete?"),
        string_prop("description", "Cannot undo."),
        string_prop("onConfirm", "confirm"),
    ])
    .expect("dialog");
    assert!(matches!(
        dialog,
        ViewNode::AlertDialog {
            props: super::AlertDialogProps {
                open,
                on_confirm: Some(_),
                ..
            },
        } if open == "open"
    ));

    let tooltip = super::tooltip_component_node(
        vec![
            string_prop("label", "More"),
            string_prop("position", "end"),
            string_prop("scheme", "muted"),
        ],
        vec![text_node("Trigger").expect("text")],
        false,
    )
    .expect("tooltip");
    assert!(matches!(
        tooltip,
        ViewNode::Tooltip {
            props: super::TooltipProps {
                position: super::OverlayPosition::End,
                ..
            },
            ..
        }
    ));

    let toast = super::toast_component_node(vec![
        string_prop("type", "success"),
        string_prop("description", "Saved"),
        string_prop("position", "top-right"),
        string_prop("variant", "outlined"),
        string_prop("scheme", "surface"),
        boolean_prop("showIcon", true),
    ])
    .expect("toast");
    assert!(matches!(
        toast,
        ViewNode::Toast {
            props: super::ToastProps {
                kind: super::ToastKind::Success,
                position: super::OverlayCornerPosition::TopRight,
                show_icon: true,
                style: super::VariantProps {
                    variant: Some(super::ComponentVariant::Outlined),
                    color: Some(super::ColorFamily::Surface),
                    ..
                },
                ..
            },
        }
    ));

    let dropdown = super::dropdown_component_node(
        vec![string_prop("scheme", "surface")],
        vec![text_node("Menu").expect("text")],
        Vec::new(),
        vec![super::OverlayEntry::Item(
            super::overlay_item_component(
                BuiltinComponent::Dropdown,
                vec![string_prop("label", "Profile")],
                None,
            )
            .expect("item"),
        )],
        Vec::new(),
        false,
    )
    .expect("dropdown");
    assert!(matches!(dropdown, ViewNode::Dropdown { entries, .. } if entries.len() == 1));

    let command = super::command_component_node(
        vec![string_prop("open", "open"), string_prop("shortcut", "p")],
        vec![super::CommandEntry::Item(
            super::overlay_item_component(
                BuiltinComponent::Command,
                vec![string_prop("label", "Home")],
                None,
            )
            .expect("item"),
        )],
    )
    .expect("command");
    assert!(matches!(
        command,
        ViewNode::Command {
            props: super::CommandProps {
                open: Some(open),
                shortcut,
                ..
            },
            ..
        } if open == "open" && shortcut == "p"
    ));

    let error = super::avatar_component_node(vec![string_prop("color", "primary")], None)
        .expect_err("color");
    assert_eq!(
        error,
        ComponentError::new("unknown prop `color` on `Avatar`; use `scheme` for visual family")
    );
}

#[test]
fn rejects_invalid_design_props() {
    let error = container_component_node(
        BuiltinComponent::Flex,
        vec![number_prop("py", 13)],
        vec![text_node("Hello").expect("text")],
        false,
    )
    .expect_err("error");

    assert_eq!(
        error,
        ComponentError::invalid_prop("py", "Dowe scale value from 0 to 96")
    );

    let error = input_node(vec![string_prop("scheme", "primaryText")]).expect_err("error");
    assert_eq!(
        error,
        ComponentError::invalid_prop(
            "scheme",
            "primary, secondary, accent, muted, success, info, warning or danger"
        )
    );

    let error = input_node(vec![string_prop("color", "primary")]).expect_err("error");
    assert_eq!(
        error,
        ComponentError::unknown_prop(BuiltinComponent::Input, "color")
    );

    let error = container_component_node(
        BuiltinComponent::Card,
        vec![string_prop("color", "primary")],
        Vec::new(),
        false,
    )
    .expect_err("error");
    assert_eq!(
        error,
        ComponentError::unknown_prop(BuiltinComponent::Card, "color")
    );

    let error = container_component_node(
        BuiltinComponent::Alert,
        vec![
            string_prop("type", "success"),
            string_prop("message", "Saved"),
            string_prop("color", "primary"),
        ],
        Vec::new(),
        false,
    )
    .expect_err("error");
    assert_eq!(
        error,
        ComponentError::unknown_prop(BuiltinComponent::Alert, "color")
    );

    let error = text_component_node(
        BuiltinComponent::Text,
        vec![string_prop("font", "Inter")],
        "Hello",
    )
    .expect_err("error");
    assert_eq!(
        error,
        ComponentError::invalid_prop(
            "font",
            "system, inter, roboto, montserrat, lato, poppins, manrope, quicksand, lora, syne, jost or puritan"
        )
    );
}
