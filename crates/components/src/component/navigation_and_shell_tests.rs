use crate::AvatarSize;

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

