#[test]
fn validates_tabs_props_entries_and_defaults() {
    let overview = tabs_tab_component(
        vec![
            string_prop("id", "overview"),
            string_prop("label", "Overview"),
        ],
        vec![text_node("Overview content").expect("text")],
    )
    .expect("overview tab");
    let details = tabs_tab_component(
        vec![
            string_prop("id", "details"),
            string_prop("label", "Details"),
        ],
        vec![text_node("Details content").expect("text")],
    )
    .expect("details tab");
    let node = tabs_component_node(
        vec![
            string_prop("variant", "line"),
            string_prop("scheme", "primary"),
            string_prop("position", "end"),
        ],
        vec![overview, details],
    )
    .expect("tabs");

    match node {
        ViewNode::Tabs { props, tabs } => {
            assert_eq!(props.variant, TabsVariant::Line);
            assert_eq!(props.color, ColorFamily::Primary);
            assert_eq!(props.position, TabsPosition::End);
            assert_eq!(tabs.len(), 2);
            assert_eq!(tabs[0].id, "overview");
            assert_eq!(tabs[0].label, "Overview");
            assert_eq!(
                first_text(&tabs[1].children[0]),
                Some("Details content".to_string())
            );
        }
        _ => panic!("tabs"),
    }

    let default_node = tabs_component_node(
        Vec::new(),
        vec![
            tabs_tab_component(
                vec![string_prop("id", "one"), string_prop("label", "One")],
                vec![text_node("One").expect("text")],
            )
            .expect("tab"),
        ],
    )
    .expect("default tabs");
    match default_node {
        ViewNode::Tabs { props, .. } => {
            assert_eq!(props.variant, TabsVariant::Pills);
            assert_eq!(props.color, ColorFamily::Primary);
            assert_eq!(props.position, TabsPosition::Top);
            assert!(!props.variant_explicit);
            assert!(!props.color_explicit);
        }
        _ => panic!("tabs"),
    }
}

#[test]
fn resolves_tabs_defaults_with_theme_and_usage_precedence() {
    let make_tabs = |props| {
        tabs_component_node(
            props,
            vec![
                tabs_tab_component(
                    vec![string_prop("id", "one"), string_prop("label", "One")],
                    vec![text_node("One").expect("text")],
                )
                .expect("tab"),
            ],
        )
        .expect("tabs")
    };

    let mut built_in = make_tabs(Vec::new());
    super::apply_design_defaults_to_tree(
        &mut built_in,
        &super::DesignDefaults::with_builtin_defaults(),
    );
    let ViewNode::Tabs { props, .. } = built_in else {
        panic!("tabs");
    };
    assert_eq!(props.variant, TabsVariant::Pills);
    assert_eq!(props.color, ColorFamily::Primary);
    assert!(props.style.border.is_none());
    assert!(props.style.shadow.is_none());

    let mut theme_defaults = super::DesignDefaults::with_builtin_defaults();
    theme_defaults
        .tabs_variant
        .insert(super::DesignComponentSlot::Tabs, TabsVariant::Line);
    theme_defaults
        .scheme
        .insert(super::DesignComponentSlot::Tabs, ColorFamily::Muted);

    let mut themed = make_tabs(Vec::new());
    super::apply_design_defaults_to_tree(&mut themed, &theme_defaults);
    let ViewNode::Tabs { props, .. } = themed else {
        panic!("tabs");
    };
    assert_eq!(props.variant, TabsVariant::Line);
    assert_eq!(props.color, ColorFamily::Muted);

    let mut explicit = make_tabs(vec![
        string_prop("variant", "ghost"),
        string_prop("scheme", "success"),
    ]);
    super::apply_design_defaults_to_tree(&mut explicit, &theme_defaults);
    let ViewNode::Tabs { props, .. } = explicit else {
        panic!("tabs");
    };
    assert_eq!(props.variant, TabsVariant::Ghost);
    assert_eq!(props.color, ColorFamily::Success);
}

#[test]
fn rejects_invalid_tabs_contracts() {
    assert_eq!(
        tabs_component_node(Vec::new(), Vec::new()).expect_err("empty tabs"),
        ComponentError::invalid_prop_combination("Tabs requires at least one tab")
    );
    assert_eq!(
        tabs_component_node(
            vec![string_prop("color", "primary")],
            vec![
                tabs_tab_component(
                    vec![string_prop("id", "one"), string_prop("label", "One")],
                    vec![text_node("One").expect("text")],
                )
                .expect("tab"),
            ],
        )
        .expect_err("color"),
        ComponentError::new("unknown prop `color` on `Tabs`; use `scheme` for visual family")
    );
    assert_eq!(
        tabs_component_node(
            Vec::new(),
            vec![
                tabs_tab_component(
                    vec![string_prop("id", "one"), string_prop("label", "One")],
                    vec![text_node("One").expect("text")],
                )
                .expect("tab"),
                tabs_tab_component(
                    vec![string_prop("id", "one"), string_prop("label", "Duplicate")],
                    vec![text_node("Duplicate").expect("text")],
                )
                .expect("duplicate tab"),
            ],
        )
        .expect_err("duplicate"),
        ComponentError::invalid_prop_combination("duplicate Tabs tab id `one`")
    );
    assert_eq!(
        tabs_tab_component(
            vec![string_prop("id", "one"), string_prop("label", "One")],
            Vec::new(),
        )
        .expect_err("children"),
        ComponentError::invalid_prop_combination("Tabs tab `one` requires at least one child")
    );
    assert_eq!(
        container_component_node(
            BuiltinComponent::Tab,
            vec![string_prop("id", "one"), string_prop("label", "One")],
            vec![text_node("One").expect("text")],
            false,
        )
        .expect_err("tab outside tabs"),
        ComponentError::invalid_prop_combination("tab can only be used inside Tabs")
    );
}

#[test]
fn validates_stepper_entries_orientation_and_errors() {
    let account = stepper_step_component(
        vec![
            string_prop("id", "account"),
            string_prop("label", "Account"),
        ],
        vec![text_node("Account content").expect("text")],
    )
    .expect("account step");
    let profile = stepper_step_component(
        vec![
            string_prop("id", "profile"),
            string_prop("label", "Profile"),
        ],
        vec![text_node("Profile content").expect("text")],
    )
    .expect("profile step");
    let node = stepper_component_node(
        vec![
            string_prop("scheme", "success"),
            string_prop("orientation", "vertical"),
        ],
        vec![account, profile],
    )
    .expect("stepper");

    let ViewNode::Tabs { props, tabs } = node else {
        panic!("stepper");
    };
    assert_eq!(props.variant, TabsVariant::Stepper);
    assert_eq!(props.color, ColorFamily::Success);
    assert_eq!(props.position, TabsPosition::Start);
    assert_eq!(tabs.len(), 2);

    assert_eq!(
        stepper_component_node(Vec::new(), Vec::new()).expect_err("empty"),
        ComponentError::invalid_prop_combination("Stepper requires at least one step")
    );
    assert_eq!(
        stepper_component_node(
            vec![string_prop("orientation", "diagonal")],
            vec![
                stepper_step_component(
                    vec![string_prop("id", "one"), string_prop("label", "One")],
                    vec![text_node("One").expect("text")],
                )
                .expect("step")
            ],
        )
        .expect_err("orientation"),
        ComponentError::invalid_prop("orientation", "horizontal or vertical")
    );
}

#[test]
fn validates_divider_props_and_defaults() {
    let default_node = divider_node(Vec::new()).expect("divider");
    match default_node {
        ViewNode::Divider { props } => {
            assert_eq!(props.orientation, DividerOrientation::Horizontal);
            assert_eq!(props.color, ColorFamily::Muted);
            assert!(props.style.element.id.is_none());
        }
        _ => panic!("divider"),
    }

    let node = divider_node(vec![
        string_prop("orientation", "vertical"),
        string_prop("scheme", "primary"),
        string_prop("id", "main-divider"),
        number_prop("h", 24),
    ])
    .expect("divider");

    match node {
        ViewNode::Divider { props } => {
            assert_eq!(props.orientation, DividerOrientation::Vertical);
            assert_eq!(props.color, ColorFamily::Primary);
            assert_eq!(props.style.element.id.as_deref(), Some("main-divider"));
            assert!(props.style.sizing.h.is_some());
        }
        _ => panic!("divider"),
    }

    assert_eq!(
        divider_node(vec![string_prop("orientation", "diagonal")]).expect_err("orientation"),
        ComponentError::invalid_prop("orientation", "horizontal or vertical")
    );
}

#[test]
fn validates_radio_group_orientation_props_and_defaults() {
    let option = radio_option_component(vec![
        string_prop("value", "basic"),
        string_prop("label", "Basic"),
    ])
    .expect("option");
    let default_node =
        radio_group_component_node(Vec::new(), vec![option.clone()]).expect("radio group");
    match default_node {
        ViewNode::RadioGroup { props, .. } => {
            assert_eq!(props.orientation, RadioGroupOrientation::Vertical);
            assert_eq!(props.size, ButtonSize::Md);
        }
        _ => panic!("radio group"),
    }

    let horizontal_node = radio_group_component_node(
        vec![string_prop("orientation", "horizontal")],
        vec![option.clone()],
    )
    .expect("horizontal radio group");
    match horizontal_node {
        ViewNode::RadioGroup { props, .. } => {
            assert_eq!(props.orientation, RadioGroupOrientation::Horizontal);
        }
        _ => panic!("radio group"),
    }

    assert_eq!(
        radio_group_component_node(vec![string_prop("orientation", "grid")], vec![option])
            .expect_err("orientation"),
        ComponentError::invalid_prop("orientation", "vertical or horizontal")
    );
}

#[test]
fn validates_carousel_variants_and_defaults() {
    let slide = carousel_slide_component(
        vec![string_prop("id", "one")],
        vec![text_node("Slide").expect("text")],
    )
    .expect("slide");
    let default_node = carousel_component_node(Vec::new(), vec![slide.clone()]).expect("carousel");
    match default_node {
        ViewNode::Carousel { props, .. } => assert_eq!(props.variant, CarouselVariant::Simple),
        _ => panic!("carousel"),
    }

    for variant in CarouselVariant::all() {
        let node = carousel_component_node(
            vec![string_prop("variant", variant.as_str())],
            vec![slide.clone()],
        )
        .expect("carousel variant");
        match node {
            ViewNode::Carousel { props, .. } => assert_eq!(props.variant, *variant),
            _ => panic!("carousel"),
        }
    }

    assert!(carousel_component_node(vec![string_prop("variant", "wheel")], vec![slide],).is_err());
}

#[test]
fn validates_select_options() {
    let option = select_option_component(vec![
        string_prop("value", "admin"),
        string_prop("label", "Admin"),
        string_prop("description", "Full access"),
    ])
    .expect("option");
    assert_eq!(option.value, "admin");
    assert_eq!(option.label, "Admin");
    assert_eq!(option.description.as_deref(), Some("Full access"));

    let node = select_node(
        vec![
            string_prop("bind", "profile.role"),
            string_prop("label", "Role"),
            string_prop("placeholder", "Choose role"),
            boolean_prop("labelFloating", true),
            string_prop("variant", "outlined"),
            string_prop("scheme", "secondary"),
        ],
        vec![
            option,
            select_option_component(vec![
                string_prop("value", "viewer"),
                string_prop("label", "Viewer"),
            ])
            .expect("viewer"),
        ],
    )
    .expect("select");

    match node {
        ViewNode::Select { props, options, .. } => {
            assert_eq!(props.element.bind.as_deref(), Some("profile.role"));
            assert_eq!(props.label.as_deref(), Some("Role"));
            assert_eq!(props.placeholder.as_deref(), Some("Choose role"));
            assert!(props.label_floating);
            assert_eq!(props.variant, Some(ComponentVariant::Outlined));
            assert_eq!(props.color, Some(ColorFamily::Secondary));
            assert_eq!(options.len(), 2);
        }
        _ => panic!("select"),
    }

    let duplicate = select_node(
        Vec::new(),
        vec![
            select_option_component(vec![
                string_prop("value", "admin"),
                string_prop("label", "Admin"),
            ])
            .expect("admin"),
            select_option_component(vec![
                string_prop("value", "admin"),
                string_prop("label", "Duplicate"),
            ])
            .expect("duplicate"),
        ],
    )
    .expect_err("duplicate");
    assert_eq!(
        duplicate,
        ComponentError::invalid_prop_combination("duplicate Select option value `admin`")
    );
}
