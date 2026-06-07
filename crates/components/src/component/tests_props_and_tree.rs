    #[test]
    fn validates_button_visual_props() {
        let node = container_component_node(
            BuiltinComponent::Button,
            vec![
                string_prop("variant", "soft"),
                string_prop("scheme", "warning"),
                string_prop("size", "xs"),
                string_prop("rounded", "full"),
            ],
            vec![text_node("Save").expect("text")],
            false,
        )
        .expect("button");

        match node {
            ViewNode::Button { props, .. } => {
                assert_eq!(props.variant, Some(ComponentVariant::Soft));
                assert_eq!(props.color, Some(ColorFamily::Warning));
                assert_eq!(props.size, Some(ButtonSize::Xs));
                assert!(props.style.rounded.is_some());
                assert_eq!(
                    props.style.spacing.px.expect("px").entries[0].value,
                    ScaleValue::from_half_steps(5)
                );
                assert_eq!(
                    props.style.spacing.py.expect("py").entries[0].value,
                    ScaleValue::from_half_steps(3)
                );
            }
            _ => panic!("button"),
        }

        let error = container_component_node(
            BuiltinComponent::Button,
            vec![string_prop("size", "2xl")],
            vec![text_node("Save").expect("text")],
            false,
        )
        .expect_err("size error");
        assert_eq!(
            error,
            ComponentError::invalid_prop("size", "xs, sm, md, lg or xl")
        );

        let error = container_component_node(
            BuiltinComponent::Button,
            vec![string_prop("rounded", "circle")],
            vec![text_node("Save").expect("text")],
            false,
        )
        .expect_err("rounded error");
        assert_eq!(
            error,
            ComponentError::invalid_prop("rounded", "xs, sm, md, lg, xl or full")
        );
    }

    #[test]
    fn validates_box_and_card_animation_props() {
        let box_node = container_component_node(
            BuiltinComponent::Box,
            vec![string_prop("animation", "fadeIn")],
            vec![text_node("Hero").expect("text")],
            false,
        )
        .expect("box");
        match box_node {
            ViewNode::Box { props, .. } => {
                assert_eq!(props.animation, Some(ViewAnimation::FadeIn));
            }
            _ => panic!("box"),
        }

        let card_node = container_component_node(
            BuiltinComponent::Card,
            vec![string_prop("animation", "slideUp")],
            vec![text_node("Card").expect("text")],
            false,
        )
        .expect("card");
        match card_node {
            ViewNode::Card { props, .. } => {
                assert_eq!(props.style.animation, Some(ViewAnimation::SlideUp));
            }
            _ => panic!("card"),
        }

        let error = container_component_node(
            BuiltinComponent::Box,
            vec![string_prop("animation", "bounce")],
            Vec::new(),
            false,
        )
        .expect_err("animation");
        assert_eq!(
            error,
            ComponentError::invalid_prop(
                "animation",
                "none, fadeIn, slideUp, slideDown, slideLeft, slideRight or scaleIn"
            )
        );

        let error = container_component_node(
            BuiltinComponent::Flex,
            vec![string_prop("animation", "fadeIn")],
            Vec::new(),
            false,
        )
        .expect_err("flex");
        assert_eq!(error, ComponentError::unknown_prop(BuiltinComponent::Flex, "animation"));
    }

    #[test]
    fn validates_text_title_typography_props() {
        let node = text_component_node(
            BuiltinComponent::Title,
            vec![
                string_prop("size", "4xl"),
                string_prop("color", "primary"),
                string_prop("bg", "softPrimary"),
                string_prop("weight", "black"),
                string_prop("spacing", "tight"),
                string_prop("i18n", "home.hero.title"),
                string_prop("font", "poppins"),
                number_prop("p", 4),
                string_prop("rounded", "md"),
            ],
            "Welcome",
        )
        .expect("title");

        match node {
            ViewNode::Title { props, value } => {
                assert_eq!(value, "Welcome");
                assert!(props.size.is_some());
                assert!(props.style.text.is_some());
                assert!(props.style.bg.is_some());
                assert_eq!(
                    props.style.font.expect("font").entries[0].value,
                    FontFamily::Poppins
                );
                assert_eq!(
                    props.weight.expect("weight").entries[0].value,
                    TextWeight::Black
                );
                assert_eq!(
                    props.letter_spacing.expect("spacing").entries[0].value,
                    TextSpacing::Tight
                );
                assert_eq!(props.i18n.as_deref(), Some("home.hero.title"));
                assert!(props.style.spacing.p.is_some());
                assert!(props.style.rounded.is_some());
            }
            _ => panic!("title"),
        }

        let error = text_component_node(
            BuiltinComponent::Text,
            vec![string_prop("i18n", "home..title")],
            "Fallback",
        )
        .expect_err("i18n");
        assert_eq!(
            error,
            ComponentError::invalid_prop("i18n", "i18n key segments separated by dots")
        );
    }

    #[test]
    fn validates_svg_component_props_and_paths() {
        let path = svg_path_component(vec![
            string_prop("d", "M0 0h24v24H0z"),
            string_prop("fill", "currentColor"),
        ])
        .expect("path");
        assert_eq!(path.fill, SvgPathFill::CurrentColor);

        let node = svg_component_node(
            vec![
                string_prop("viewBox", "0 0 24 24"),
                string_prop("color", "tertiary"),
                number_prop("w", 8),
                number_prop("h", 8),
            ],
            vec![path],
        )
        .expect("svg");

        match node {
            ViewNode::Svg { props, paths } => {
                assert_eq!(props.view_box.as_str(), "0 0 24 24");
                assert!(props.style.text.is_some());
                assert!(props.style.sizing.w.is_some());
                assert_eq!(paths.len(), 1);
            }
            _ => panic!("svg"),
        }

        let fill = svg_path_component(vec![
            string_prop("d", "M0 0L1 1"),
            string_prop("fill", "primary"),
        ])
        .expect("fill");
        assert_eq!(fill.fill, SvgPathFill::Color(super::ColorToken::Primary));
    }

    #[test]
    fn rejects_invalid_svg_component_usage() {
        let error = svg_component_node(vec![string_prop("viewBox", "0 0 24 24")], Vec::new())
            .expect_err("empty svg");
        assert_eq!(
            error,
            ComponentError::invalid_prop_combination("Svg requires at least one Path child")
        );

        let error = svg_component_node(
            vec![string_prop("viewBox", "0 0 0 24")],
            vec![svg_path_component(vec![string_prop("d", "M0 0")]).expect("path")],
        )
        .expect_err("viewbox");
        assert_eq!(
            error,
            ComponentError::invalid_prop("viewBox", "four numbers with positive width and height")
        );

        let error =
            svg_path_component(vec![string_prop("d", "M0 0 <script")]).expect_err("path data");
        assert_eq!(
            error,
            ComponentError::invalid_prop("d", "portable SVG path data")
        );

        let error = svg_path_component(vec![
            string_prop("d", "M0 0"),
            string_prop("fill", "url(#gradient)"),
        ])
        .expect_err("fill");
        assert_eq!(
            error,
            ComponentError::invalid_prop("fill", "currentColor, none or color token")
        );
    }

    #[test]
    fn validates_responsive_text_typography_props() {
        let node = text_component_node(
            BuiltinComponent::Text,
            vec![
                responsive_string_prop("size", &[("xs", "sm"), ("md", "lg")]),
                responsive_string_prop("weight", &[("xs", "thin"), ("md", "extralight"), ("lg", "black")]),
                responsive_string_prop("spacing", &[("xs", "normal"), ("md", "wide")]),
                responsive_string_prop("bg", &[("md", "softInfo")]),
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
                vec![svg_path_component(vec![string_prop("d", "M3 11l9-8 9 8v10H3z")])
                    .expect("path")],
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
        let node = super::side_nav_component_node(
            vec![
                string_prop("variant", "soft"),
                string_prop("scheme", "surface"),
                string_prop("size", "lg"),
                boolean_prop("wide", true),
            ],
            vec![item],
        )
        .expect("side nav");

        match node {
            ViewNode::SideNav { props, items } => {
                assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
                assert_eq!(props.style.color, Some(ColorFamily::Surface));
                assert_eq!(props.size, super::SideNavSize::Lg);
                assert!(props.wide);
                assert!(matches!(&items[0], super::SideNavItem::Item(props) if props.icon.is_some()));
            }
            _ => panic!("side nav"),
        }
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
                description: Some("Start here".to_string()),
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
                string_prop("variant", "soft"),
                string_prop("scheme", "surface"),
                string_prop("size", "lg"),
            ],
            vec![nav_item, submenu, megamenu],
        )
        .expect("nav menu");

        match &nav_menu {
            ViewNode::NavMenu { props, items } => {
                assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
                assert_eq!(props.style.color, Some(ColorFamily::Surface));
                assert_eq!(props.size, super::SideNavSize::Lg);
                assert_eq!(items.len(), 3);
                assert!(matches!(&items[0], super::NavMenuItem::Item(props) if props.navigation.is_some()));
                assert!(matches!(&items[1], super::NavMenuItem::Submenu { items, .. } if items.len() == 1));
                assert!(matches!(&items[2], super::NavMenuItem::Megamenu { content, .. } if content.len() == 1));
            }
            _ => panic!("nav menu"),
        }

        let sidebar_item = super::side_nav_item_component(
            vec![string_prop("label", "Side Home"), string_prop("href", "/")],
            None,
        )
        .expect("sidebar item");
        let sidebar = super::sidebar_component_node(
            vec![
                string_prop("variant", "solid"),
                string_prop("scheme", "primary"),
                string_prop("size", "sm"),
                boolean_prop("wide", true),
            ],
            vec![sidebar_item],
        )
        .expect("sidebar");

        match &sidebar {
            ViewNode::Sidebar { props, items } => {
                assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
                assert_eq!(props.style.color, Some(ColorFamily::Primary));
                assert_eq!(props.size, super::SideNavSize::Sm);
                assert!(props.wide);
                assert_eq!(items.len(), 1);
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
            } => {
                assert!(props.boxed);
                assert_eq!(app_bar.len(), 1);
                assert_eq!(start.len(), 1);
                assert_eq!(main.len(), 1);
                assert!(end.is_empty());
                assert_eq!(bottom_bar.len(), 1);
            }
            _ => panic!("scaffold"),
        }

        let error = super::nav_menu_component_node(
            vec![string_prop("color", "primary")],
            vec![super::nav_menu_item_component(vec![string_prop("label", "Home")], None)
                .expect("item")],
        )
        .expect_err("color error");
        assert_eq!(
            error,
            ComponentError::new("unknown prop `color` on `NavMenu`; use `scheme` for visual family")
        );

        let error = super::sidebar_component_node(
            vec![string_prop("color", "primary")],
            vec![super::side_nav_item_component(vec![string_prop("label", "Home")], None)
                .expect("item")],
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
                string_prop("variant", "soft"),
                string_prop("scheme", "surface"),
                boolean_prop("disableOverlayClose", true),
                boolean_prop("hideCloseButton", true),
                responsive_boolean_prop("show", &[("xs", true), ("md", false)]),
            ],
            vec![text_node("Navigation").expect("text")],
            false,
        )
        .expect("drawer");

        match node {
            ViewNode::Drawer { props, children } => {
                assert_eq!(props.open, "drawerOpen");
                assert_eq!(props.position, super::DrawerPosition::End);
                assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
                assert_eq!(props.style.color, Some(ColorFamily::Surface));
                assert!(props.disable_overlay_close);
                assert!(props.hide_close_button);
                assert!(props.style.element.show.is_some());
                assert_eq!(children.len(), 1);
            }
            _ => panic!("drawer"),
        }

        let error = super::drawer_component_node(
            vec![string_prop("open", "drawerOpen")],
            Vec::new(),
            false,
        )
        .expect_err("children");
        assert_eq!(
            error,
            ComponentError::invalid_prop_combination("Drawer requires at least one child")
        );
    }

    #[test]
    fn validates_display_and_overlay_component_props() {
        let avatar = super::avatar_component_node(
            vec![
                string_prop("name", "Ada"),
                string_prop("scheme", "success"),
                string_prop("variant", "soft"),
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
                assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
                assert_eq!(props.size, ButtonSize::Lg);
                assert_eq!(props.status, Some(super::AvatarStatus::Online));
                assert!(props.bordered);
            }
            _ => panic!("avatar"),
        }

        let badge = super::badge_component_node(
            vec![string_prop("text", "3"), string_prop("position", "bottom-right")],
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
            vec![string_prop("open", "open"), string_prop("scheme", "surface")],
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

        let error = input_node(vec![string_prop("scheme", "onPrimary")]).expect_err("error");
        assert_eq!(
            error,
            ComponentError::invalid_prop(
                "scheme",
                "primary, secondary, tertiary, muted, success, info, warning or danger"
            )
        );

        let error = input_node(vec![string_prop("color", "primary")]).expect_err("error");
        assert_eq!(error, ComponentError::unknown_prop(BuiltinComponent::Input, "color"));

        let error = container_component_node(
            BuiltinComponent::Card,
            vec![string_prop("color", "primary")],
            Vec::new(),
            false,
        )
        .expect_err("error");
        assert_eq!(error, ComponentError::unknown_prop(BuiltinComponent::Card, "color"));

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
        assert_eq!(error, ComponentError::unknown_prop(BuiltinComponent::Alert, "color"));

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
                "system, inter, roboto, montserrat, lato, poppins, manrope, quicksand or lora"
            )
        );
    }

    #[test]
    fn composes_children_with_page_tree() {
        let layout = ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Text {
                    props: Default::default(),
                    value: "Before".to_string(),
                },
                ViewNode::Children,
                ViewNode::Text {
                    props: Default::default(),
                    value: "After".to_string(),
                },
            ],
        };
        let page = ViewNode::Box {
            props: Default::default(),
            children: vec![ViewNode::Text {
                props: Default::default(),
                value: "Login".to_string(),
            }],
        };

        assert_eq!(
            compose_tree(&layout, &page),
            ViewNode::Box {
                props: Default::default(),
                children: vec![
                    ViewNode::Text {
                        props: Default::default(),
                        value: "Before".to_string()
                    },
                    page,
                    ViewNode::Text {
                        props: Default::default(),
                        value: "After".to_string()
                    }
                ]
            }
        );
    }

    #[test]
    fn finds_first_text() {
        let tree = ViewNode::Box {
            props: Default::default(),
            children: vec![ViewNode::Box {
                props: Default::default(),
                children: vec![ViewNode::Text {
                    props: Default::default(),
                    value: "Login".to_string(),
                }],
            }],
        };

        assert_eq!(first_text(&tree), Some("Login".to_string()));
    }

    fn string_prop(name: &str, value: &str) -> ComponentProp {
        ComponentProp {
            name: name.to_string(),
            value: PropValue::String(value.to_string()),
        }
    }

    fn number_prop(name: &str, value: i32) -> ComponentProp {
        number_string_prop(name, &value.to_string())
    }

    fn number_string_prop(name: &str, value: &str) -> ComponentProp {
        ComponentProp {
            name: name.to_string(),
            value: PropValue::Number(value.to_string()),
        }
    }

    fn boolean_prop(name: &str, value: bool) -> ComponentProp {
        ComponentProp {
            name: name.to_string(),
            value: PropValue::Boolean(value),
        }
    }

    fn responsive_number_prop(name: &str, entries: &[(&str, i32)]) -> ComponentProp {
        ComponentProp {
            name: name.to_string(),
            value: PropValue::Responsive(
                entries
                    .iter()
                    .map(|(breakpoint, value)| ResponsivePropEntry {
                        breakpoint: (*breakpoint).to_string(),
                        value: super::PropScalar::Number(value.to_string()),
                    })
                    .collect(),
            ),
        }
    }

    fn responsive_boolean_prop(name: &str, entries: &[(&str, bool)]) -> ComponentProp {
        ComponentProp {
            name: name.to_string(),
            value: PropValue::Responsive(
                entries
                    .iter()
                    .map(|(breakpoint, value)| ResponsivePropEntry {
                        breakpoint: (*breakpoint).to_string(),
                        value: super::PropScalar::Boolean(*value),
                    })
                    .collect(),
            ),
        }
    }

    fn responsive_string_prop(name: &str, entries: &[(&str, &str)]) -> ComponentProp {
        ComponentProp {
            name: name.to_string(),
            value: PropValue::Responsive(
                entries
                    .iter()
                    .map(|(breakpoint, value)| ResponsivePropEntry {
                        breakpoint: (*breakpoint).to_string(),
                        value: super::PropScalar::String((*value).to_string()),
                    })
                    .collect(),
            ),
        }
    }
