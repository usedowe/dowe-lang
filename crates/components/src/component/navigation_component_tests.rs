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
            string_prop("bind", "drawerOpen"),
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
        vec![string_prop("bind", "drawerOpen")],
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

