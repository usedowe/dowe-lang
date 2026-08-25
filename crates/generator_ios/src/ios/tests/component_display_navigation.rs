#[test]
fn generates_fragment_aware_native_history_and_deep_links() {
    let output = generate_ios(
        &[index_route_with_navigation(), signup_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    let routing = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweRouting.swift"))
        .expect("routing");

    assert!(views.contains("struct DoweRouteEntry: Hashable"));
    assert!(views.contains("@State private var rootEntry: DoweRouteEntry"));
    assert!(views.contains("_rootEntry = State(initialValue: DoweRouteEntry"));
    assert!(views.contains("@State private var navigationPath: [DoweRouteEntry] = []"));
    assert!(views.contains("@State private var routeRevision = 0"));
    assert!(views.contains(
        "routeContent(currentEntry, viewportWidth: doweSafeAreaWidth(geometry, safeAreaInsets), viewportHeight: doweSafeAreaHeight(geometry, safeAreaInsets))"
    ));
    assert!(views.contains(".simultaneousGesture(backSwipeGesture)"));
    assert!(!views.contains("NavigationStack(path: $navigationPath)"));
    assert!(!views.contains(".navigationDestination(for: DoweRouteEntry.self)"));
    assert!(!views.contains(".toolbar(.hidden, for: .navigationBar)"));
    assert!(!views.contains(".navigationBarHidden(true)"));
    assert!(views.contains("private var backSwipeGesture: some Gesture"));
    assert!(views.contains("navigationPath.append(destination)"));
    assert!(views.contains("navigationPath.removeLast()"));
    assert!(views.contains(
        "private func navigate(_ operation: String, _ target: String, _ fragment: String?)"
    ));
    assert!(views.contains("if destination == currentEntry"));
    assert!(views.contains("routeRevision += 1"));
    assert!(views.contains(".id(routeRevision)"));
    assert!(views.contains(r#"{ navigate("push", "/signup", "join") }"#));
    assert!(views.contains(r#"{ navigate("replace", "", "hero") }"#));
    assert!(views.contains("{ goBack() }"));
    assert!(views.contains(r#"navigate("replace", path, url.fragment)"#));
    assert!(views.contains("ScrollViewReader { proxy in"));
    assert!(views.contains("doweScroll(proxy, activeFragment)"));
    assert!(views.contains("proxy.scrollTo(\"__dowe_page_top\", anchor: .top)"));
    assert!(views.contains("routeSection0()\n                    .id(\"__dowe_page_top\")"));
    assert!(
        views.contains(".onChange(of: activeFragment) { _, value in doweScroll(proxy, value) }")
    );
    assert!(!views.contains(".onChange(of: activeFragment) { value in doweScroll(proxy, value) }"));
    assert!(views.contains(".id(\"hero\")"));
    assert!(
        routing
            .content
            .contains("static let sections: [String: [String]]")
    );
    assert!(routing.content.contains(r#""/signup": ["join"]"#));
}

fn advanced_form_route() -> ViewRoute {
    ViewRoute {
        id: "advanced".to_string(),
        route_path: "/advanced".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: advanced_form_tree(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn advanced_form_tree() -> ViewNode {
    ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::ComboBox {
                props: ComboBoxProps {
                    style: bound_style_with_size(
                        "profile.role",
                        "Role",
                        "Choose role",
                        ButtonSize::Sm,
                    ),
                    value: Some("editor".to_string()),
                    search_placeholder: "Search roles".to_string(),
                    empty_text: "No roles".to_string(),
                    loading_text: "Loading".to_string(),
                    loading_more_text: "Loading more".to_string(),
                    clearable: true,
                    disabled: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
                options: vec![ComboOption {
                    value: "admin".to_string(),
                    label: "Admin".to_string(),
                    description: Some("Full access".to_string()),
                    src: None,
                    icon: None,
                    disabled: false,
                }],
            },
            ViewNode::CsvField {
                props: CsvFieldProps {
                    style: advanced_style("Import", None, ComponentVariant::Outlined),
                    button_text: "Upload CSV".to_string(),
                    modal_title: "Review import".to_string(),
                    instructions: "Columns are checked".to_string(),
                    cancel_text: "Cancel".to_string(),
                    confirm_text: "Import".to_string(),
                    clear_text: "Clear".to_string(),
                    preview_title: "Preview".to_string(),
                    multiple: false,
                    show_preview: true,
                    preview_rows: 3,
                    preview_page_size: 10,
                    error_text: None,
                },
                columns: vec![CsvColumn {
                    name: "email".to_string(),
                    label: Some("Email".to_string()),
                }],
            },
            ViewNode::DragDrop {
                props: DragDropProps {
                    style: advanced_style("Tasks", None, ComponentVariant::Solid),
                    empty_text: "No tasks".to_string(),
                    direction: DragDropDirection::Horizontal,
                    allow_group_transfer: true,
                    disabled: false,
                    size: ButtonSize::Md,
                },
                items: Vec::new(),
                groups: vec![DragGroup {
                    id: "todo".to_string(),
                    title: Some("Todo".to_string()),
                    items: vec![DragItem {
                        id: "draft".to_string(),
                        label: Some("Draft".to_string()),
                        description: Some("Prepare".to_string()),
                        disabled: false,
                    }],
                }],
            },
            ViewNode::Editor {
                props: EditorProps {
                    style: bound_style("profile.notes", "Notes", "Write notes"),
                    value: None,
                    min_height: 180,
                    hide_toolbar: false,
                    disabled: false,
                    readonly: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::ImageCropper {
                props: ImageCropperProps {
                    style: bound_style("profile.avatar", "Avatar", "Upload avatar"),
                    src: None,
                    alt: "Avatar".to_string(),
                    accept: "image/*".to_string(),
                    aspect_ratio: None,
                    min_width: 128,
                    min_height: 128,
                    max_width: None,
                    max_height: None,
                    shape: ImageCropperShape::Circle,
                    disabled: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::Password {
                props: PasswordProps {
                    style: bound_style_with_size(
                        "profile.password",
                        "Password",
                        "Create password",
                        ButtonSize::Md,
                    ),
                    value: None,
                    hide_strength: false,
                    weak_label: "Weak".to_string(),
                    medium_label: "Medium".to_string(),
                    strong_label: "Strong".to_string(),
                    disabled: false,
                    readonly: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::Phone {
                props: PhoneProps {
                    style: bound_style_with_size(
                        "profile.phone",
                        "Phone",
                        "Phone number",
                        ButtonSize::Lg,
                    ),
                    value: None,
                    country: Some("US".to_string()),
                    dial_code_name: "dialCode".to_string(),
                    search_placeholder: "Search countries".to_string(),
                    empty_text: "No countries".to_string(),
                    loading_text: "Loading".to_string(),
                    priority_countries: vec!["US".to_string()],
                    disabled: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::Pin {
                props: PinProps {
                    style: bound_style("profile.pin", "Code", ""),
                    value: None,
                    length: 6,
                    kind: PinKind::Number,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::Textarea {
                props: TextareaProps {
                    style: bound_style("profile.bio", "Bio", "Short bio"),
                    value: None,
                    rows: 4,
                    cols: None,
                    max_length: Some(160),
                    resize: true,
                    disabled: false,
                    readonly: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
        ],
    }
}

fn bound_style(bind: &str, label: &str, placeholder: &str) -> VariantProps {
    let mut style = advanced_style(label, Some(placeholder), ComponentVariant::Outlined);
    style.element.bind = Some(bind.to_string());
    style.label_floating = true;
    style
}

fn bound_style_with_size(
    bind: &str,
    label: &str,
    placeholder: &str,
    size: ButtonSize,
) -> VariantProps {
    let mut style = bound_style(bind, label, placeholder);
    style.size = Some(size);
    style
}

fn advanced_style(
    label: &str,
    placeholder: Option<&str>,
    variant: ComponentVariant,
) -> VariantProps {
    VariantProps {
        label: Some(label.to_string()),
        placeholder: placeholder.map(str::to_string),
        variant: Some(variant),
        color: Some(ColorFamily::Primary),
        ..Default::default()
    }
}

