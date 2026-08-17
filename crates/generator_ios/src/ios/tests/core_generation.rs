#[test]
fn generates_persistent_view_store_for_swiftui() {
    let mut persistent = route();
    persistent.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "session01".to_string(),
            name: "session".to_string(),
            storage_key: "views/store/session:session".to_string(),
            scope: dowe_components::ViewSignalScope::Global,
            storage: dowe_components::ViewSignalStorage::Local,
            initial: ViewSignalValue::Object(vec![(
                "token".to_string(),
                ViewSignalValue::String(String::new()),
            )]),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![text("Session")],
    };
    let output = generate_ios(
        &[persistent],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = swift_content(&output);

    assert!(generated.contains(
        "DoweSignalMetadata(name: \"views/store/session:session\", scope: \"global\", storage: \"local\")"
    ));
    assert!(
        generated.contains("UserDefaults.standard.data(forKey: Self.storageKey(metadata.name))")
    );
    assert!(
        generated
            .contains("UserDefaults.standard.set(data, forKey: Self.storageKey(metadata.name))")
    );
    assert!(generated.contains("Self.compatibleSignalValue(stored, fallback)"));
}

#[test]
fn generates_dowe_global_toast_presenter_for_swiftui() {
    let mut toast_route = route();
    toast_route.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: Vec::new(),
        actions: vec![ViewAction {
            id: "notify01".to_string(),
            name: "notify".to_string(),
            params: Vec::new(),
            return_type: None,
            kind: ViewActionKind::Sequence(vec![ViewFunctionStatement::Toast(ViewToastAction {
                kind: "success".to_string(),
                title: "Saved".to_string(),
                message: "Changes published".to_string(),
                duration: Some(3000),
                scheme: Some("surface".to_string()),
                variant: Some("outlined".to_string()),
                position: Some("top-right".to_string()),
            })]),
        }],
        children: vec![text("Notify")],
    };
    let generated = swift_content(&generate_ios(
        &[toast_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(generated.contains("DoweGlobalToast(toast: state.toast, close: state.closeToast)"));
    assert!(generated.contains("doweCardContainer(toast.variant, toast.scheme)"));
    assert!(generated.contains("DoweOverlayCloseIcon(color: DoweDesign.softMutedText)"));
    assert!(generated.contains(".accessibilityLabel(\"Close toast\")"));
    assert!(!generated.contains("UIAlertController"));
}

#[test]
fn fills_request_path_placeholders_from_signal_names() {
    let output = generate_ios(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = swift_content(&output);

    assert!(generated.contains("NSRegularExpression(pattern: \":[A-Za-z_][A-Za-z0-9_]*\")"));
    assert!(generated.contains("signals.reversed().first(where: { $0.value.name == name })"));
    assert!(!generated.contains("signals.last(where:"));
}

#[test]
fn generates_fixed_fab_as_route_overlay_with_dowe_icons() {
    let mut fab_route = route();
    fab_route.page_tree = fixed_fab_page();
    let output = generate_ios(
        &[fab_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = swift_content(&output);

    assert!(generated.contains("ZStack(alignment: .topLeading)"));
    assert!(generated.contains("private func fixedFab0() -> some View"));
    assert!(generated.contains("if doweFixedFabOpen0"));
    assert!(generated.contains("doweFixedFabOpen0.toggle()"));
    assert!(generated.contains("VStack(alignment: .trailing, spacing: CGFloat(12))"));
    assert!(generated.contains("HStack(spacing: CGFloat(12))"));
    assert!(generated.contains(".clipShape(Capsule())"));
    assert!(generated.contains(".contentShape(Capsule())"));
    assert!(generated.contains("ZStack {"));
    assert!(generated.contains(
        ".frame(maxWidth: .infinity, maxHeight: .infinity)\n                .contentShape(Circle())"
    ));
    assert!(generated.contains(".rotationEffect(.degrees(doweFixedFabOpen0 ? 45 : 0))"));
    let rotation = generated
        .find(".rotationEffect(.degrees(doweFixedFabOpen0 ? 45 : 0))")
        .expect("Fab rotation");
    let gesture = generated
        .find(".modifier(DoweGestureModifier(preset: .press, transition: .smooth))")
        .expect("Fab press gesture");
    assert!(rotation < gesture);
    assert!(generated.contains("DoweSvgView(viewBox:"));
    assert!(generated.contains("maxHeight: .infinity, alignment: .bottomTrailing"));
    assert!(!generated.contains("Image(systemName: \"plus\")"));
}

#[test]
fn generates_relative_absolute_and_fixed_boxes_as_swiftui_overlays() {
    let mut positioned_route = route();
    positioned_route.page_tree = positioned_box_page();
    let generated = swift_content(&generate_ios(
        &[positioned_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(generated.contains("ZStack(alignment: .topLeading)"));
    assert!(generated.contains("alignment: .topTrailing"));
    assert!(generated.contains(".padding(.top, doweResponsive(viewportWidth, xs: CGFloat(16)))"));
    assert!(
        generated.contains(".padding(.trailing, doweResponsive(viewportWidth, xs: CGFloat(24)))")
    );
    assert!(generated.contains("alignment: .bottomTrailing"));
    assert!(generated.contains("private func fixedBox0() -> some View"));
}

fn positioned_box_page() -> ViewNode {
    ViewNode::Box {
        props: StyleProps {
            extras: Some(Box::new(dowe_components::StyleExtras {
                position: dowe_components::PositionProps {
                    mode: BoxPosition::Relative,
                    ..Default::default()
                },
                ..Default::default()
            })),
            sizing: SizingProps {
                min_h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                    ScaleValue::from_half_steps(64),
                ))),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![
            text("Flow content"),
            ViewNode::Box {
                props: StyleProps {
                    extras: Some(Box::new(dowe_components::StyleExtras {
                        position: dowe_components::PositionProps {
                            mode: BoxPosition::Absolute,
                            top: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                            right: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(12))),
                            ..Default::default()
                        },
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                children: vec![text("Proof")],
            },
            ViewNode::Box {
                props: StyleProps {
                    extras: Some(Box::new(dowe_components::StyleExtras {
                        position: dowe_components::PositionProps {
                            mode: BoxPosition::Fixed,
                            right: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                            bottom: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                            ..Default::default()
                        },
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                children: vec![text("Persistent")],
            },
        ],
    }
}

fn fixed_fab_page() -> ViewNode {
    let mut style = VariantProps {
        color: Some(ColorFamily::Primary),
        variant: Some(ComponentVariant::Solid),
        size: Some(ButtonSize::Lg),
        ..Default::default()
    };
    style.style.motion_mut().gesture = Some(ViewGesture::Press);
    ViewNode::Scope {
        constants: Vec::new(),
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![
            text("Scrollable content"),
            ViewNode::Fab {
                props: FabProps {
                    style,
                    position: OverlayCornerPosition::BottomRight,
                    fixed: true,
                    offset_x: ScaleValue::from_half_steps(8),
                    offset_y: ScaleValue::from_half_steps(8),
                    icon: ViewIcon::Plus,
                    label: "Open actions".to_string(),
                },
                actions: vec![FabAction {
                    label: "Edit".to_string(),
                    icon: ViewIcon::Edit,
                    color: ColorFamily::Info,
                    on_click: None,
                    navigation: Some(NavigationAction::Internal {
                        path: "/edit".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                }],
            },
        ],
    }
}

#[test]
fn generates_immutable_swift_constants() {
    let mut constant_route = route();
    constant_route.page_tree = ViewNode::Scope {
        constants: vec![dowe_components::ViewConstant {
            id: "plans01".to_string(),
            name: "plans".to_string(),
            value: ViewSignalValue::Array(vec![ViewSignalValue::String("Starter".to_string())]),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![text("Plans")],
    };
    let output = generate_ios(
        &[constant_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = swift_content(&output);
    assert!(generated.contains("constants: [\"plans01\": [\"Starter\"]]"));
    assert!(generated.contains("private let constants: [String: Any]"));
}

#[test]
fn generates_init_sequence_and_reactive_splash_for_swiftui() {
    let mut splash_route = route();
    splash_route.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "loading01".to_string(),
            name: "isLoading".to_string(),
            storage_key: "isLoading".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::Bool(true),
            schema: None,
        }],
        actions: vec![ViewAction::init(
            "init01".to_string(),
            vec![
                ViewFunctionStatement::Request {
                    result: "result".to_string(),
                    action: ViewRequestAction {
                        method: ViewRequestMethod::Get,
                        path: "/api/users".to_string(),
                        base_env: None,
                        headers: Vec::new(),
                        body: None,
                        update: None,
                        reset: None,
                        success_alert: None,
                        success_message: None,
                        error_alert: None,
                        error_message: None,
                        autoload: false,
                    },
                },
                ViewFunctionStatement::If {
                    result: "result".to_string(),
                    success: vec![
                        ViewFunctionStatement::Assign(ViewAssignAction {
                            target: "isLoading".to_string(),
                            source: "$dowe:bool:false".to_string(),
                            literal: None,
                            call: None,
                        }),
                        ViewFunctionStatement::Toast(ViewToastAction {
                            kind: "success".to_string(),
                            title: "Loaded".to_string(),
                            message: "Users loaded".to_string(),
                            duration: Some(1500),
                            scheme: Some("success".to_string()),
                            variant: Some("soft".to_string()),
                            position: Some("top-right".to_string()),
                        }),
                    ],
                    error: vec![
                        ViewFunctionStatement::Reset(dowe_components::ViewResetAction {
                            target: "isLoading".to_string(),
                        }),
                        ViewFunctionStatement::Toast(ViewToastAction {
                            kind: "error".to_string(),
                            title: "Error".to_string(),
                            message: "Users failed".to_string(),
                            duration: None,
                            scheme: None,
                            variant: None,
                            position: None,
                        }),
                    ],
                },
            ],
        )],
        children: vec![ViewNode::Splash {
            binding: "isLoading".to_string(),
            initial: true,
            content: vec![text("Users"), fixed_fab_page()],
            children: vec![text("Loading users")],
        }],
    };
    let generated = swift_content(&generate_ios(
        &[splash_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(generated.contains(".task { state.load([\"init01\"]) }"));
    assert!(generated.contains(".sequence([.request(\"result\""));
    assert!(generated.contains(".branch(\"result\""));
    assert!(generated.contains(".assign(\"loading01\""));
    assert!(generated.contains(".reset(\"loading01\")"));
    assert!(generated.contains(".toast(\"success\", \"Loaded\", \"Users loaded\", 1500"));
    assert!(generated.contains("if state.bool(\"loading01\")"));
    assert!(generated.contains("if !state.bool(\"loading01\")"));
}

#[test]
fn generates_terminal_replace_redirect_for_swiftui() {
    let mut redirect_route = route();
    redirect_route.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: Vec::new(),
        actions: vec![ViewAction::init(
            "init01".to_string(),
            vec![ViewFunctionStatement::Redirect {
                path: "/login".to_string(),
            }],
        )],
        children: vec![text("Home")],
    };
    let generated = swift_content(&generate_ios(
        &[redirect_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(generated.contains(".redirect(\"/login\")"));
    assert!(generated.contains("redirectPath = path"));
    assert!(generated.contains("navigate(\"replace\", path, nil)"));
    assert!(generated.contains("if await runSteps"));
}

#[test]
fn resolves_button_values_from_each_item_scope() {
    let mut scoped_route = route();
    scoped_route.page_tree = ViewNode::Scope {
        constants: vec![dowe_components::ViewConstant {
            id: "buttons01".to_string(),
            name: "buttons".to_string(),
            value: ViewSignalValue::Array(vec![ViewSignalValue::Object(vec![
                (
                    "id".to_string(),
                    ViewSignalValue::String("success".to_string()),
                ),
                (
                    "label".to_string(),
                    ViewSignalValue::String("Success".to_string()),
                ),
                (
                    "variant".to_string(),
                    ViewSignalValue::String("solid".to_string()),
                ),
                (
                    "scheme".to_string(),
                    ViewSignalValue::String("success".to_string()),
                ),
                (
                    "size".to_string(),
                    ViewSignalValue::String("lg".to_string()),
                ),
            ])]),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![ViewNode::Each {
            item: "button".to_string(),
            collection: "buttons".to_string(),
            key: "button.id".to_string(),
            children: vec![ViewNode::Button {
                props: VariantProps {
                    reactive: ReactiveVariantProps {
                        variant: Some("button.variant".to_string()),
                        scheme: Some("button.scheme".to_string()),
                        size: Some("button.size".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("{button.label}")],
            }],
        }],
    };
    let output = generate_ios(
        &[scoped_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<String>();
    assert!(generated.contains("state.text(\"item.variant\", item: row.value)"));
    assert!(generated.contains("state.text(\"item.scheme\", item: row.value)"));
    assert!(
        generated
            .contains("doweButtonHorizontalPadding(state.text(\"item.size\", item: row.value))")
    );
    assert!(generated.contains("doweButtonMinHeight(state.text(\"item.size\", item: row.value))"));
    assert!(generated.contains("state.text(\"item.label\", item: row.value)"));
}

#[test]
fn generates_swift_select_options_from_constant_each() {
    let mut constant_route = route();
    constant_route.page_tree = ViewNode::Scope {
        constants: vec![dowe_components::ViewConstant {
            id: "options01".to_string(),
            name: "options".to_string(),
            value: ViewSignalValue::Array(Vec::new()),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![ViewNode::Select {
            props: Default::default(),
            options: Vec::new(),
            option_each: Some(SelectOptionEach {
                item: "option".to_string(),
                collection: "options".to_string(),
                key: "option.id".to_string(),
                value: "option.value".to_string(),
                label: "option.label".to_string(),
                description: None,
            }),
        }],
    };
    let output = generate_ios(
        &[constant_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = swift_content(&output);
    assert!(generated.contains("state.rows(\"options01\").map { row in DoweSelectOption"));
    assert!(generated.contains("state.text(\"item.value\", item: row.value)"));
    assert!(generated.contains("state.text(\"item.label\", item: row.value)"));
}

fn swift_content(output: &IosOutput) -> String {
    output
        .files
        .iter()
        .filter(|file| {
            file.relative_path
                .extension()
                .and_then(|value| value.to_str())
                == Some("swift")
        })
        .map(|file| file.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn generates_swiftui_box_and_text() {
    let output = generate_ios(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    assert!(!output.files.iter().any(|file| {
        file.relative_path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.starts_with("DowePhoneCatalog"))
    }));
    let views = swift_content(&output);

    assert!(views.contains("VStack(alignment: .leading, spacing: 0)"));
    assert!(views.contains("AnyView("));
    assert!(!views.contains("() -> AnyView in"));
    assert!(views.contains("routeSection0()"));
    assert!(views.contains("private func routeSection0() -> some View"));
    assert!(views.contains("private let activePath = \"/login\""));
    assert!(!views.contains("        let activePath ="));
    assert!(!views.contains("VStack(alignment: .leading) {"));
    assert!(views.contains(".frame(maxWidth: .infinity, alignment: .leading)"));
    assert!(views.contains(".background(DoweDesign.primary)"));
    assert!(views.contains("Text(verbatim: \"Layout\")"));
    assert!(views.contains("Text(verbatim: \"Login\")"));

    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");
    assert!(plist.content.contains("CFBundleExecutable"));
    assert!(plist.content.contains("DoweIosApp"));
    assert!(plist.content.contains("CFBundleURLSchemes"));
    assert!(plist.content.contains("dowe-dev"));
    assert!(plist.content.contains("UILaunchScreen"));
    assert!(plist.content.contains("NSAllowsLocalNetworking"));
    assert!(plist.content.contains("UIAppFonts"));
    assert!(plist.content.contains("Fonts/inter-regular.ttf"));

    let host = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweIosDevHost.swift"))
        .expect("dev host");
    let module = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweIosViewModule.swift"))
        .expect("dev module");
    assert!(host.content.contains("dlopen(file.path"));
    assert!(host.content.contains("/_dowe/dev/modules/manifest.json"));
    assert!(host.content.contains("moduleEndpoint = resolveEndpoint()"));
    assert!(host.content.contains("showWaitingState(in: controller)"));
    assert!(host.content.contains("Preparing Dowe app"));
    assert!(
        host.content
            .contains("The first iOS build can take a few minutes.")
    );
    assert!(host.content.contains("waitingView?.removeFromSuperview()"));
    assert!(
        host.content
            .contains("UserDefaults.standard.set(value, forKey: endpointKey)")
    );
    assert!(
        host.content
            .contains("UserDefaults.standard.string(forKey: endpointKey)")
    );
    assert!(host.content.contains(
        "moduleEndpoint = resolveEndpoint()\n        restoreCachedModule()\n        poll()"
    ));
    assert!(host.content.contains("applicationSupportDirectory"));
    assert!(!host.content.contains("temporaryDirectory"));
    assert!(
        host.content
            .contains("UserDefaults.standard.string(forKey: activeVersionKey)")
    );
    assert!(
        host.content
            .contains("UserDefaults.standard.set(version, forKey: activeVersionKey)")
    );
    assert!(host.content.contains("private var activeRoute = \"/\""));
    assert!(!host.content.contains("dowe.hmr.route"));
    assert!(
        !host
            .content
            .contains("UserDefaults.standard.string(forKey: activeRouteKey)")
    );
    assert!(
        !host
            .content
            .contains("UserDefaults.standard.set(path, forKey: activeRouteKey)")
    );
    assert!(host.content.contains("persistCurrentPath()"));
    assert!(
        module
            .content
            .contains("@_cdecl(\"dowe_create_root_view_controller\")")
    );
    assert!(
        module
            .content
            .contains("@objc(DoweIosDevModuleController___DOWE_IOS_SOURCE_REVISION__)")
    );
    let explicit_objc_names = output
        .files
        .iter()
        .flat_map(|file| file.content.lines())
        .filter(|line| line.trim_start().starts_with("@objc("))
        .collect::<Vec<_>>();
    assert!(!explicit_objc_names.is_empty());
    assert!(
        explicit_objc_names
            .iter()
            .all(|line| line.contains("__DOWE_IOS_SOURCE_REVISION__"))
    );
    let pages = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.swift"))
        .expect("pages");
    assert!(!pages.content.contains("__DOWE_IOS_SOURCE_REVISION__"));
    assert!(views.contains("init(initialPath: String = DoweRoutes.initialPath"));
    assert!(views.contains("routeChanged(path)"));
}

#[test]
fn generates_swiftui_text_alignment() {
    let mut aligned = route();
    aligned.page_tree = ViewNode::Title {
        props: TextProps {
            align: Some(ResponsiveValue::scalar(TextAlign::End)),
            ..Default::default()
        },
        value: "Aligned".to_string(),
    };
    let output = generate_ios(
        &[aligned],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains("doweText(\"Aligned\", alignment: doweResponsive"));
    assert!(views.contains(".frame(maxWidth: .infinity, alignment: doweResponsive"));
    assert!(views.contains("Alignment.trailing"));
    assert!(views.contains("DoweTextAlignment.end"));
}

#[test]
fn generates_swiftui_justified_text() {
    let mut aligned = route();
    aligned.page_tree = ViewNode::Text {
        props: TextProps {
            align: Some(ResponsiveValue::scalar(TextAlign::Justify)),
            ..Default::default()
        },
        value: "Justified".to_string(),
    };
    let output = generate_ios(
        &[aligned],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains("doweText(\"Justified\", alignment: doweResponsive"));
    assert!(views.contains("DoweTextAlignment.justify"));
    assert!(views.contains("doweJustifiedAttributedText"));
}

#[test]
fn generates_static_text_as_verbatim_swiftui_content() {
    let mut literal_route = route();
    literal_route.page_tree = text("info@dowe.dev");
    let views = swift_content(&generate_ios(
        &[literal_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(views.contains("Text(verbatim: \"info@dowe.dev\")"));
    assert!(!views.contains("Text(\"info@dowe.dev\")"));
}

#[test]
fn preserves_multiline_text_in_swiftui_content() {
    let mut multiline = route();
    multiline.page_tree = ViewNode::Title {
        props: TextProps::default(),
        value: "Full-stack development,\nfrom one codebase".to_string(),
    };
    let views = swift_content(&generate_ios(
        &[multiline],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(views.contains("Text(verbatim: \"Full-stack development,\\nfrom one codebase\")"));
}

#[test]
fn inherits_container_foreground_and_preserves_text_overrides() {
    let mut color_route = route();
    color_route.layout_tree = ViewNode::Children;
    color_route.page_tree = container_foreground_tree();
    let views = swift_content(&generate_ios(
        &[color_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    let box_inherited = views
        .find("Text(verbatim: \"Box inherited\")")
        .expect("Box text");
    let box_override = views
        .find("Text(verbatim: \"Box override\")")
        .expect("Box override");
    assert!(!views[box_inherited..box_override].contains(".foregroundStyle("));
    assert!(views[box_override..].contains(
        ".foregroundStyle(doweResponsive(viewportWidth, xs: DoweDesign.danger) ?? DoweDesign.backgroundText)"
    ));
    assert!(views[box_override..].contains(
        ".foregroundStyle(doweResponsive(viewportWidth, xs: DoweDesign.primaryText) ?? DoweDesign.backgroundText)"
    ));

    let card_inherited = views
        .find("Text(verbatim: \"Card inherited\")")
        .expect("Card text");
    assert!(views.contains("Text(verbatim: \"Card title inherited\")"));
    assert!(views.contains(".modifier(DoweTitleColorModifier(explicitColor: nil))"));
    assert!(views.contains("static let defaultValue: Color? = nil"));
    assert!(!views.contains("static let defaultValue: Color = DoweDesign.backgroundTitle"));
    assert!(views.contains(
        "content.foregroundStyle(explicitColor ?? inheritedColor ?? DoweDesign.backgroundTitle)"
    ));
    assert!(views.contains(".environment(\\.doweTitleColor, DoweDesign.softMutedTitle)"));
    let card_override = views
        .find("Text(verbatim: \"Card override\")")
        .expect("Card override");
    assert!(!views[card_inherited..card_override].contains(".foregroundStyle("));
    let card_tail = &views[card_override..];
    let override_color = card_tail
        .find(".modifier(DoweTitleColorModifier(explicitColor: doweResponsive(viewportWidth, xs: DoweDesign.warning)))")
        .expect("Card override color");
    let inherited_color = card_tail
        .find(".foregroundStyle(DoweDesign.softMutedText)")
        .expect("Card content color");
    assert!(override_color < inherited_color);
}

#[test]
fn keeps_fixed_width_box_content_leading_aligned() {
    let mut fixed_width = route();
    fixed_width.layout_tree = ViewNode::Children;
    fixed_width.page_tree = ViewNode::Box {
        props: StyleProps {
            bg: Some(ResponsiveValue::scalar(ColorToken::SoftPrimary)),
            text: Some(ResponsiveValue::scalar(ColorToken::SoftPrimaryText)),
            spacing: dowe_components::SpacingProps {
                p: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(6))),
                ..Default::default()
            },
            sizing: dowe_components::SizingProps {
                w: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                    ScaleValue::from_half_steps(24),
                ))),
                ..Default::default()
            },
            rounded: Some(ResponsiveValue::scalar(RoundedSize::Md)),
            ..Default::default()
        },
        children: vec![text("H")],
    };

    let output = generate_ios(
        &[fixed_width],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("VStack(alignment: .leading, spacing: 0)"));
    assert!(views.contains(
        ".frame(width: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(48)))), alignment: .leading)"
    ));
    assert!(views.contains(
        ".frame(maxWidth: doweMaxSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(48)))), alignment: .leading)"
    ));
    assert!(!views.contains(
        ".frame(width: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(48)))))"
    ));
}

#[test]
fn generates_space_between_flex_with_adaptive_spacers() {
    let mut flex_route = route();
    flex_route.layout_tree = ViewNode::Children;
    flex_route.page_tree = ViewNode::Flex {
        props: dowe_components::LayoutProps {
            justify: Some(ResponsiveValue::scalar(dowe_components::Justify::Between)),
            align: Some(ResponsiveValue::scalar(dowe_components::Align::Center)),
            gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                ScaleValue::from_half_steps(6),
            )))),
            ..Default::default()
        },
        children: vec![text("Palette"), text("Live")],
    };

    let output = generate_ios(
        &[flex_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains(
        "HStack(alignment: doweVerticalAlignment(doweResponsive(viewportWidth, xs: DoweAlign.center)), spacing: doweFlexStackSpacing(doweResponsive(viewportWidth, xs: DoweJustify.between), gap: doweResponsive(viewportWidth, xs: CGFloat(12))))"
    ));
    assert!(views.contains(
        "if let spacerGap = doweFlexBetweenSpacer(doweResponsive(viewportWidth, xs: DoweJustify.between), gap: doweResponsive(viewportWidth, xs: CGFloat(12)))"
    ));
    assert!(views.contains("Spacer(minLength: spacerGap)"));
    assert!(views.contains("enum DoweJustify: Equatable"));
    assert!(views.contains("justify == .between ? CGFloat(0) : gap ?? CGFloat(0)"));
}

#[test]
fn generates_wrapped_flex_flow_layout() {
    let mut flex_route = route();
    flex_route.layout_tree = ViewNode::Children;
    flex_route.page_tree = ViewNode::Flex {
        props: dowe_components::LayoutProps {
            wrap: true,
            gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                ScaleValue::from_half_steps(6),
            )))),
            ..Default::default()
        },
        children: vec![text("First"), text("Second")],
    };
    let output = generate_ios(
        &[flex_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains("DoweFlowLayout(justify: nil, align: nil, gap:"));
    assert!(views.contains("struct DoweFlowLayout: Layout"));
    assert!(views.contains("var contentWidth: CGFloat = 0"));
    assert!(!views.contains("rows.map { row in row.map"));
}

#[test]
fn keeps_swiftui_box_background_and_foreground_across_nested_boxes() {
    let mut nested = route();
    nested.layout_tree = ViewNode::Children;
    nested.page_tree = ViewNode::Box {
        props: StyleProps {
            bg: Some(ResponsiveValue::scalar(ColorToken::Surface)),
            text: Some(ResponsiveValue::scalar(ColorToken::SurfaceText)),
            spacing: dowe_components::SpacingProps {
                p: Some(responsive_scale(&[
                    (Breakpoint::Xs, 5),
                    (Breakpoint::Md, 7),
                ])),
                ..Default::default()
            },
            sizing: dowe_components::SizingProps {
                min_h: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                    ScaleValue::from_half_steps(72),
                ))),
                ..Default::default()
            },
            rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
            border: Some(ResponsiveValue::scalar(dowe_components::BorderWidth(1))),
            ..Default::default()
        },
        children: vec![ViewNode::Grid {
            props: GridProps {
                columns: Some(ResponsiveValue::ordered(vec![
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Xs,
                        value: GridTracks::Count(1),
                    },
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: GridTracks::Count(3),
                    },
                ])),
                gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                    ScaleValue::from_half_steps(10),
                )))),
                ..Default::default()
            },
            children: vec![
                ViewNode::Box {
                    props: Default::default(),
                    children: vec![ViewNode::Title {
                        props: Default::default(),
                        value: "Dowe Source Format".to_string(),
                    }],
                },
                ViewNode::Box {
                    props: Default::default(),
                    children: vec![ViewNode::Title {
                        props: Default::default(),
                        value: "Compiler-owned output".to_string(),
                    }],
                },
            ],
        }],
    };

    let output = generate_ios(
        &[nested],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    let grid_start = views
        .find("DoweGridLayout(columns: doweResponsive(viewportWidth, xs: 1, md: 3) ?? 1")
        .expect("nested grid");
    let surface_section = &views[grid_start..];
    let padding = surface_section
        .find(".padding(EdgeInsets(top: doweResponsive(viewportWidth, xs: CGFloat(20), md: CGFloat(28)) ?? CGFloat(0)")
        .expect("box padding");
    let min_height = surface_section
        .find(".frame(minHeight: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(144))), viewportHeight: viewportHeight))")
        .expect("box min height");
    let background = surface_section
        .find(".background(doweResponsive(viewportWidth, xs: DoweDesign.surface) ?? Color.clear)")
        .expect("box background");
    let foreground = surface_section
        .find(".foregroundStyle(doweResponsive(viewportWidth, xs: DoweDesign.surfaceText) ?? DoweDesign.backgroundText)")
        .expect("box foreground");
    let border = surface_section
        .find(".overlay(RoundedRectangle(cornerRadius: doweResponsive(viewportWidth, xs: CGFloat(12)) ?? DoweDesign.radius).stroke(DoweDesign.backgroundText, lineWidth: doweResponsive(viewportWidth, xs: CGFloat(1)) ?? CGFloat(0)))")
        .expect("box border");

    assert!(padding < background);
    assert!(min_height < background);
    assert!(background < foreground);
    assert!(foreground < border);
    assert!(views.contains("Text(verbatim: \"Dowe Source Format\")"));
    assert!(views.contains("Text(verbatim: \"Compiler-owned output\")"));
}

#[test]
fn keeps_card_shadow_after_swiftui_card_shape() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Card {
        props: VariantProps {
            style: StyleProps {
                shadow: Some(ResponsiveValue::scalar(ShadowSize::Lg)),
                shadow_color: Some(ColorFamily::Primary),
                rounded: Some(ResponsiveValue::scalar(RoundedSize::Md)),
                extras: Some(Box::new(StyleExtras {
                    motion: ViewMotionStyle {
                        animation: Some(ViewAnimation::FadeIn),
                        rotate: Some(ResponsiveValue::scalar(ViewRotation(-7))),
                        scale: Some(ResponsiveValue::scalar(ViewScale(105))),
                        translate_x: Some(ResponsiveValue::scalar(ViewTranslation(-3))),
                        transition: Some(ViewTransition::Spring),
                        gesture: Some(ViewGesture::Lift),
                        ..Default::default()
                    },
                    ..Default::default()
                })),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Raised")],
    };

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    let card_start = views
        .find("VStack(alignment: .leading, spacing: 0)")
        .expect("card");
    let card_output = &views[card_start..];
    let clip = card_output
        .find(".clipShape(RoundedRectangle(cornerRadius: doweResponsive(viewportWidth, xs: CGFloat(8)) ?? DoweDesign.radius))")
        .expect("card clip");
    let shadow = card_output
        .find(".background(DoweShadowSurface(shadow: DoweShadowSpec(color: DoweDesign.primary.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(44)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(18)) ?? CGFloat(0)), cornerRadius: doweResponsive(viewportWidth, xs: CGFloat(8)) ?? DoweDesign.radius))")
        .expect("card shadow");
    let animation = card_output
        .find(".modifier(DoweAnimationModifier(preset: .fadeIn))")
        .expect("card animation");

    assert!(shadow > clip);
    assert!(animation > shadow);
    assert!(card_output.contains(
        ".rotationEffect(.degrees(doweResponsive(viewportWidth, xs: Double(-7)) ?? Double(0)))"
    ));
    assert!(card_output.contains(
        ".scaleEffect(CGFloat(doweResponsive(viewportWidth, xs: Double(1.05)) ?? Double(1)))"
    ));
    assert!(card_output.contains(".offset(x: CGFloat(doweResponsive(viewportWidth, xs: Double(-6)) ?? Double(0)), y: CGFloat(0))"));
    assert!(
        card_output.contains(".modifier(DoweGestureModifier(preset: .lift, transition: .spring))")
    );
    assert!(views.contains("@Environment(\\.accessibilityReduceMotion) private var reduceMotion"));
}

#[test]
fn generates_touch_driven_swiftui_gestures() {
    let output = generate_ios(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("@GestureState private var pressed = false"));
    assert!(views.contains("DragGesture(minimumDistance: 0)"));
    assert!(views.contains(".simultaneousGesture(pressGesture)"));
    assert!(views.contains("preset == .grow && (activeHover || activePress)"));
    assert!(views.contains("preset == .tilt && (activeHover || activePress)"));
    assert!(views.contains("return CGFloat(0.94)"));
    assert!(!views.contains(".onLongPressGesture(minimumDuration: 0"));
}

#[test]
fn applies_button_press_feedback_after_the_complete_swiftui_surface() {
    let mut button = VariantProps::default();
    button.style.motion_mut().gesture = Some(ViewGesture::Press);
    let mut button_route = route();
    button_route.layout_tree = ViewNode::Children;
    button_route.page_tree = ViewNode::Button {
        props: button,
        children: vec![text("Press")],
    };
    let output = generate_ios(
        &[button_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("button page");
    let background = page
        .content
        .find(".background(")
        .expect("button background");
    let button_style = page
        .content
        .find(".buttonStyle(.plain)")
        .expect("plain button style");
    let gesture = page
        .content
        .find(".modifier(DoweGestureModifier(preset: .press, transition: .smooth))")
        .expect("press gesture");

    assert!(background < button_style);
    assert!(button_style < gesture);
    assert_eq!(
        page.content
            .matches(".modifier(DoweGestureModifier(preset: .press, transition: .smooth))")
            .count(),
        1
    );
}

#[test]
fn generates_diffuse_semantic_shadows_for_portable_components() {
    let shadow_style = |size, color| StyleProps {
        shadow: Some(ResponsiveValue::scalar(size)),
        shadow_color: Some(color),
        ..Default::default()
    };
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::Card {
                props: VariantProps {
                    style: StyleProps {
                        rounded: Some(ResponsiveValue::scalar(RoundedSize::Md)),
                        ..shadow_style(ShadowSize::Md, ColorFamily::Primary)
                    },
                    ..Default::default()
                },
                children: vec![text("Card")],
            },
            ViewNode::Button {
                props: VariantProps {
                    style: shadow_style(ShadowSize::Sm, ColorFamily::Secondary),
                    ..Default::default()
                },
                children: vec![text("Button")],
            },
            ViewNode::Avatar {
                props: AvatarProps {
                    style: VariantProps {
                        style: StyleProps {
                            spacing: SpacingProps {
                                p: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                                ..Default::default()
                            },
                            border: Some(ResponsiveValue::ordered(vec![ResponsiveEntry {
                                breakpoint: Breakpoint::Md,
                                value: BorderWidth(2),
                            }])),
                            border_color: Some(ColorFamily::Warning),
                            ..shadow_style(ShadowSize::Lg, ColorFamily::Tertiary)
                        },
                        ..Default::default()
                    },
                    src: None,
                    name: Some("Dowe".to_string()),
                    alt: "Dowe".to_string(),
                    size: ButtonSize::Md,
                    status: None,
                    bordered: true,
                },
                icon: None,
            },
            ViewNode::Chip {
                props: ChipProps {
                    style: VariantProps {
                        style: StyleProps {
                            spacing: SpacingProps {
                                p: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                                ..Default::default()
                            },
                            rounded: Some(ResponsiveValue::scalar(RoundedSize::Full)),
                            border: Some(ResponsiveValue::ordered(vec![ResponsiveEntry {
                                breakpoint: Breakpoint::Md,
                                value: BorderWidth(2),
                            }])),
                            border_color: Some(ColorFamily::Danger),
                            ..shadow_style(ShadowSize::Xs, ColorFamily::Success)
                        },
                        variant: Some(ComponentVariant::Outlined),
                        ..Default::default()
                    },
                    on_close: None,
                },
                value: "Chip".to_string(),
                start: None,
                end: None,
            },
            ViewNode::Input {
                props: VariantProps {
                    style: StyleProps {
                        border: Some(ResponsiveValue::ordered(vec![ResponsiveEntry {
                            breakpoint: Breakpoint::Md,
                            value: BorderWidth(2),
                        }])),
                        border_color: Some(ColorFamily::Danger),
                        rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
                        ..shadow_style(ShadowSize::Md, ColorFamily::Info)
                    },
                    variant: Some(ComponentVariant::Outlined),
                    color: Some(ColorFamily::Info),
                    label: Some("Workspace".to_string()),
                    placeholder: Some("dowe-app".to_string()),
                    ..Default::default()
                },
            },
        ],
    };

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    for expected in [
        "DoweShadowSpec(color: DoweDesign.primary.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(24)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(10)) ?? CGFloat(0)), cornerRadius: doweResponsive(viewportWidth, xs: CGFloat(8)) ?? DoweDesign.radius",
        "DoweShadowSpec(color: DoweDesign.secondary.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(12)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(4)) ?? CGFloat(0)), cornerRadius: DoweDesign.radius",
        "shadow: Optional(DoweShadowSpec(color: DoweDesign.tertiary.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(44)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(18)) ?? CGFloat(0)))",
        "shadow: Optional(DoweShadowSpec(color: DoweDesign.success.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(2)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(1)) ?? CGFloat(0)))",
        "shadow: Optional(DoweShadowSpec(color: DoweDesign.info.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(24)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(10)) ?? CGFloat(0)))",
    ] {
        assert!(views.contains(expected), "missing {expected}");
    }
    assert!(views.contains("struct DoweShadowSurface: View"));
    assert!(views.contains("options: .shadowOnly"));
    assert!(views.contains("context.blendMode = .destinationOut"));
    assert!(views.contains("DoweShadowSurface(shadow: shadow, cornerRadius: CGFloat(9999))"));
    assert!(views.contains("DoweShadowSurface(shadow: shadow, cornerRadius: radius)"));
    assert!(
        views.contains(
            ".overlay(Circle().stroke(borderColor ?? Color.clear, lineWidth: borderWidth))"
        )
    );
    assert!(views.contains(".overlay(RoundedRectangle(cornerRadius: radius).stroke(borderColor ?? Color.clear, lineWidth: borderWidth))"));
    assert!(
        views.contains(
            "radius: doweResponsive(viewportWidth, xs: CGFloat(999)) ?? DoweDesign.radius"
        )
    );
    assert!(views.contains(".clipShape(RoundedRectangle(cornerRadius: radius))"));
    assert_eq!(views.matches("DoweDesign.info.opacity(0.28)").count(), 1);

    let input_runtime_start = views
        .find("struct DoweInputField: View")
        .expect("input runtime");
    let input_runtime = &views[input_runtime_start..];
    let input_border = input_runtime
        .find(".stroke(borderColor ?? Color.clear")
        .expect("input border");
    let input_shadow = input_runtime
        .find("DoweShadowSurface(shadow: shadow, cornerRadius: radius)")
        .expect("input field shadow");
    assert!(input_border < input_shadow);

    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("page");
    let button_start = page
        .content
        .find("Text(verbatim: \"Button\")")
        .expect("button");
    let button_output = &page.content[button_start..];
    let button_background = button_output
        .find(".background(DoweDesign.primary)")
        .expect("button background");
    let button_clip = button_output
        .find(".clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))")
        .expect("button clip");
    let button_shadow = button_output
        .find("DoweShadowSpec(color: DoweDesign.secondary.opacity(0.28)")
        .expect("button shadow");
    assert!(button_background < button_clip);
    assert!(button_clip < button_shadow);

    let avatar_start = page.content.find("DoweAvatar(").expect("avatar");
    let chip_start = page.content.find("DoweChip(").expect("chip");
    let input_start = page.content.find("DoweInputField(").expect("input field");
    let avatar_output = &page.content[avatar_start..chip_start];
    let chip_output = &page.content[chip_start..input_start];
    assert!(
        avatar_output
            .find("shadow: Optional(DoweShadowSpec")
            .expect("avatar shadow")
            < avatar_output
                .find(".padding(")
                .expect("avatar outer padding")
    );
    assert!(
        chip_output
            .find("shadow: Optional(DoweShadowSpec")
            .expect("chip shadow")
            < chip_output.find(".padding(").expect("chip outer padding")
    );
    assert!(!avatar_output.contains(".clipShape("));
    assert!(!chip_output.contains(".clipShape("));
    assert!(avatar_output.contains("borderColor: (doweResponsive(viewportWidth, md: CGFloat(2))) == nil ? Optional(DoweDesign.primaryText) : Optional(DoweDesign.warning)"));
    assert!(
        avatar_output
            .contains("borderWidth: doweResponsive(viewportWidth, md: CGFloat(2)) ?? CGFloat(3)")
    );
    assert!(chip_output.contains("borderColor: (doweResponsive(viewportWidth, md: CGFloat(2))) == nil ? Optional(DoweDesign.primary) : Optional(DoweDesign.danger)"));
    assert!(
        chip_output
            .contains("borderWidth: doweResponsive(viewportWidth, md: CGFloat(2)) ?? CGFloat(1)")
    );

    let input = page
        .content
        .lines()
        .find(|line| line.contains("DoweInputField") && line.contains("label: \"Workspace\""))
        .expect("input");
    assert!(input.contains("shadow: Optional(DoweShadowSpec"));
    assert!(input.contains("borderColor: (doweResponsive(viewportWidth, md: CGFloat(2))) == nil ? Optional(DoweDesign.muted) : Optional(DoweDesign.danger)"));
    assert!(
        input.contains("borderWidth: doweResponsive(viewportWidth, md: CGFloat(2)) ?? CGFloat(1)")
    );
}

#[test]
fn keeps_button_shadow_after_reactive_effective_radius() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Button {
        props: VariantProps {
            style: StyleProps {
                rounded: Some(ResponsiveValue::scalar(RoundedSize::Md)),
                shadow: Some(ResponsiveValue::scalar(ShadowSize::Sm)),
                shadow_color: Some(ColorFamily::Secondary),
                extras: Some(Box::new(StyleExtras {
                    motion: ViewMotionStyle {
                        animation: Some(ViewAnimation::SlideUp),
                        ..Default::default()
                    },
                    ..Default::default()
                })),
                ..Default::default()
            },
            reactive: ReactiveVariantProps {
                rounded: Some("radius".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Reactive")],
    };

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("page");
    let radius = "doweButtonRadius(state.text(\"radius\", fallback: \"md\"))";
    let clip = page
        .content
        .find(&format!(
            ".clipShape(RoundedRectangle(cornerRadius: {radius}))"
        ))
        .expect("effective button clip");
    let shadow = page
        .content
        .find(&format!(
            "DoweShadowSpec(color: DoweDesign.secondary.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(12)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(4)) ?? CGFloat(0)), cornerRadius: {radius}"
        ))
        .expect("effective button shadow");
    let animation = page
        .content
        .find(".modifier(DoweAnimationModifier(preset: .slideUp))")
        .expect("button animation");
    assert!(clip < shadow);
    assert!(shadow < animation);
}

#[test]
fn generates_progressive_neutral_swiftui_shadow_strength() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Box {
        props: StyleProps {
            shadow: Some(ResponsiveValue::ordered(vec![
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xs,
                    value: ShadowSize::Xs,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Sm,
                    value: ShadowSize::Sm,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Md,
                    value: ShadowSize::Md,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Lg,
                    value: ShadowSize::Lg,
                },
                ResponsiveEntry {
                    breakpoint: Breakpoint::Xl,
                    value: ShadowSize::Xl,
                },
            ])),
            ..Default::default()
        },
        children: vec![text("Neutral")],
    };

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains(".shadow(color: Color.black.opacity(doweResponsive(viewportWidth, xs: Double(0.12), sm: Double(0.14), md: Double(0.16), lg: Double(0.18), xl: Double(0.22)) ?? Double(0)), radius: doweResponsive(viewportWidth, xs: CGFloat(2), sm: CGFloat(12), md: CGFloat(24), lg: CGFloat(44), xl: CGFloat(70)) ?? CGFloat(0), x: CGFloat(0), y: doweResponsive(viewportWidth, xs: CGFloat(1), sm: CGFloat(4), md: CGFloat(10), lg: CGFloat(18), xl: CGFloat(28)) ?? CGFloat(0))"));
}

#[test]
fn generates_shared_swiftui_layout_once_for_multiple_routes() {
    let mut first = route();
    first.layout_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::Box {
                props: Default::default(),
                children: vec![text("Layout")],
            },
            ViewNode::Children,
        ],
    };
    let mut second = first.clone();
    second.route_path = "/signup".to_string();
    second.page_tree = ViewNode::Text {
        props: Default::default(),
        value: "Signup".to_string(),
    };

    let output = generate_ios(
        &[first, second],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let layouts_index = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweLayouts.swift"))
        .expect("layouts index");
    let layout = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweLayout0.swift"))
        .expect("layout");
    let login = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("login");
    let signup = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageSignupView.swift"))
        .expect("signup");

    assert_eq!(
        layout
            .content
            .matches("struct DoweLayout0<Content: View>")
            .count(),
        1
    );
    assert!(!layouts_index.content.contains("struct DoweLayout"));
    assert_eq!(
        layout.content.matches("Text(verbatim: \"Layout\")").count(),
        1
    );
    assert!(layout.content.contains("layoutSection0()"));
    assert!(
        layout
            .content
            .contains("private func layoutSection0() -> some View")
    );
    assert!(login.content.contains("DoweLayout0("));
    assert!(signup.content.contains("DoweLayout0("));
    assert!(!login.content.contains("Text(verbatim: \"Layout\")"));
    assert!(!signup.content.contains("Text(verbatim: \"Layout\")"));
    assert!(login.content.contains("Text(verbatim: \"Login\")"));
    assert!(signup.content.contains("Text(verbatim: \"Signup\")"));
}

#[test]
fn generates_reusable_swiftui_layouts_as_independent_files() {
    let mut first = route();
    first.layout_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![text("First layout"), ViewNode::Children],
    };
    let mut second = route();
    second.route_path = "/signup".to_string();
    second.layout_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![text("Second layout"), ViewNode::Children],
    };

    let output = generate_ios(
        &[first, second],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let first_layout = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweLayout0.swift"))
        .expect("first layout");
    let second_layout = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweLayout1.swift"))
        .expect("second layout");

    assert!(
        first_layout
            .content
            .contains("Text(verbatim: \"First layout\")")
    );
    assert!(
        !first_layout
            .content
            .contains("Text(verbatim: \"Second layout\")")
    );
    assert!(
        second_layout
            .content
            .contains("Text(verbatim: \"Second layout\")")
    );
    assert!(
        !second_layout
            .content
            .contains("Text(verbatim: \"First layout\")")
    );
}

#[test]
fn keeps_layouts_composed_when_page_reads_layout_state() {
    let mut contextual = route();
    contextual.layout_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "layout.message".to_string(),
            name: "message".to_string(),
            storage_key: "message".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::String("Layout message".to_string()),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![ViewNode::Box {
            props: Default::default(),
            children: vec![ViewNode::Children],
        }],
    };
    contextual.page_tree = ViewNode::Text {
        props: Default::default(),
        value: "{message}".to_string(),
    };

    let output = generate_ios(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let layouts = swift_content(&output);
    let login = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("login");

    assert!(!layouts.contains("struct DoweLayout0<"));
    assert!(!login.content.contains("DoweLayout0("));
    assert!(login.content.contains("state.text(\"layout.message\")"));
}

#[test]
fn reuses_stateful_swiftui_layout_when_page_does_not_read_layout_state() {
    let mut contextual = route();
    contextual.layout_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "layout.open".to_string(),
            name: "open".to_string(),
            storage_key: "open".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::Bool(false),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![ViewNode::Box {
            props: Default::default(),
            children: vec![ViewNode::Children],
        }],
    };
    contextual.page_tree = ViewNode::Text {
        props: Default::default(),
        value: "Login".to_string(),
    };

    let output = generate_ios(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let layouts = swift_content(&output);
    let login = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("login");

    assert!(layouts.contains("struct DoweLayout0<"));
    assert!(login.content.contains("DoweLayout0("));
    assert!(login.content.contains("\"layout.open\": false"));
}

#[test]
fn reuses_stateful_scaffold_drawer_layout_when_page_mentions_binding_literals() {
    let contextual = stateful_scaffold_drawer_layout_route(false);

    let output = generate_ios(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let layouts = swift_content(&output);
    let login = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("login");

    assert!(layouts.contains("struct DoweLayout0<"));
    assert!(layouts.contains("DoweDrawer(open: state.bool(\"layout.drawer.open\")"));
    assert_eq!(layouts.matches("private func layoutSection").count(), 3);
    assert!(layouts.contains("layoutSection0()"));
    assert!(layouts.contains("layoutSection1()"));
    assert!(layouts.contains("layoutSection2()"));
    assert!(login.content.contains("DoweLayout0("));
    assert!(login.content.contains("\"layout.drawer.open\": false"));
    assert!(login.content.contains("\"layout.drawer.visible\": true"));
    assert!(login.content.contains(
        "\"layout.drawer.open.action\": .assign(\"layout.drawer.open\", \"layout.drawer.visible\", nil, DoweActionMetadata(params: [:], returnType: nil))"
    ));
    assert!(
        !login
            .content
            .contains("DoweDrawer(open: state.bool(\"layout.drawer.open\")")
    );
    let generated = output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<String>();
    assert!(
        !login
            .content
            .contains("ScrollView {\n                DoweLayout0(")
    );
    assert!(generated.contains("ScrollView {"));
    assert!(layouts.contains(
        "content\n                        }\n                        .frame(maxWidth: .infinity, alignment: .topLeading)"
    ));
    assert!(!layouts.contains(
        "content\n                        }\n                        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)"
    ));
    assert!(layouts.contains(
        "                    }\n                    .frame(maxWidth: .infinity, alignment: .topLeading)\n                    }\n                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)"
    ));

    let boxed = stateful_scaffold_drawer_layout_route(true);
    let boxed_output = generate_ios(
        &[boxed],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let boxed_layouts = swift_content(&boxed_output);
    assert!(boxed_layouts.contains(".frame(maxWidth: CGFloat(1536), alignment: .topLeading)"));
    assert!(boxed_layouts.contains(".frame(maxWidth: .infinity, alignment: .top)"));
}

#[test]
fn generates_ios_app_metadata() {
    let output = generate_ios_with_app_and_translations(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
        &TranslationCatalog::default(),
        "Clinic Desk",
        "com.example.clinic",
    );
    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");

    assert!(plist.content.contains("<string>Clinic Desk</string>"));
    assert!(
        plist
            .content
            .contains("<string>com.example.clinic</string>")
    );
    assert!(plist.content.contains("<key>CFBundleName</key>"));
}

#[test]
fn generates_ios_app_icon_metadata_for_phone_and_tablet() {
    let output = generate_ios_with_app_translations_and_icons(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
        &TranslationCatalog::default(),
        "Clinic Desk",
        "com.example.clinic",
        true,
    );
    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");

    assert!(plist.content.contains("<key>CFBundleIcons</key>"));
    assert!(plist.content.contains("<key>CFBundleIcons~ipad</key>"));
    assert!(plist.content.contains("<string>AppIcon60x60</string>"));
    assert!(plist.content.contains("<string>AppIcon76x76</string>"));
    assert!(plist.content.contains("<key>CFBundleIconName</key>"));
    assert!(plist.content.contains("<string>AppIcon</string>"));
}

#[test]
fn generates_swiftui_section_backgrounds() {
    let output = generate_ios(
        &[section_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("enum DoweSectionBackground"));
    assert!(views.contains("DoweSectionBackgroundView(background: background)"));
    assert!(views.contains(".padding(EdgeInsets(top: doweResponsive(viewportWidth, xs: CGFloat(40), md: CGFloat(64)) ?? CGFloat(0), leading: doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(24)) ?? CGFloat(0), bottom: doweResponsive(viewportWidth, xs: CGFloat(40), md: CGFloat(64)) ?? CGFloat(0), trailing: doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(24)) ?? CGFloat(0)))"));
    let section = &views[views
        .find("doweResponsive(viewportWidth, xs: DoweSectionBackground.aurora")
        .expect("section")..];
    let padding = section
        .find(".padding(EdgeInsets")
        .expect("section padding");
    let max_width = section
        .find(".frame(maxWidth: CGFloat(1536), alignment: .leading)")
        .expect("boxed max width");
    let centered = section
        .find(".frame(maxWidth: .infinity, alignment: .center)")
        .expect("boxed centering");
    assert!(padding < max_width);
    assert!(max_width < centered);
    assert!(views.contains("doweResponsive(viewportWidth, xs: DoweSectionBackground.aurora, md: DoweSectionBackground.ocean)"));
    assert!(views.contains("LinearGradient(colors: [DoweDesign.softPrimary, DoweDesign.softSecondary, DoweDesign.softTertiary]"));
    assert!(views.contains("DoweCoverImage(source:"));
    assert!(views.contains("https://example.com/hero.jpg"));
    assert!(views.contains("DoweOverlay.color(Color.black.opacity(0.35))"));
    assert!(views.contains("DoweOverlayView(overlay: overlay)"));
}

#[test]
fn generates_responsive_section_centering_for_swiftui() {
    let mut route = section_route();
    let ViewNode::Box { children, .. } = &mut route.page_tree else {
        panic!("section route root");
    };
    let ViewNode::Section { props, .. } = &mut children[0] else {
        panic!("section route child");
    };
    props.center = Some(ResponsiveValue::ordered(vec![
        ResponsiveEntry {
            breakpoint: Breakpoint::Xs,
            value: false,
        },
        ResponsiveEntry {
            breakpoint: Breakpoint::Md,
            value: true,
        },
    ]));

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains(
        "VStack(alignment: (doweResponsive(viewportWidth, xs: false, md: true) ?? false) ? .center : .leading, spacing: 0)"
    ));
    assert!(views.contains(".frame(maxWidth: .infinity, alignment: .leading)"));
}

#[test]
fn generates_responsive_section_gap_for_swiftui() {
    let mut route = section_route();
    let ViewNode::Box { children, .. } = &mut route.page_tree else {
        panic!("section route root");
    };
    let ViewNode::Section { props, .. } = &mut children[0] else {
        panic!("section route child");
    };
    props.gap = Some(ResponsiveValue::ordered(vec![
        ResponsiveEntry {
            breakpoint: Breakpoint::Xs,
            value: GapValue::Single(GapSize::Scale(ScaleValue(4))),
        },
        ResponsiveEntry {
            breakpoint: Breakpoint::Md,
            value: GapValue::Single(GapSize::Scale(ScaleValue(8))),
        },
    ]));

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains(
        "VStack(alignment: .leading, spacing: doweResponsive(viewportWidth, xs: CGFloat(8), md: CGFloat(16)) ?? CGFloat(0))"
    ));
}

#[test]
fn generates_native_ios_translation_resources() {
    let mut localized_route = route();
    localized_route.page_tree = ViewNode::Title {
        props: TextProps {
            i18n: Some("home.hero.title".to_string()),
            ..Default::default()
        },
        value: "Dowe builds systems.".to_string(),
    };
    let output = generate_ios_with_translations(
        &[localized_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
        &translations(),
    );
    let views = swift_content(&output);
    assert!(views.contains(r#"String(localized: "home.hero.title")"#));
    let english = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("en.lproj/Localizable.strings"))
        .expect("english");
    assert!(english.content.contains("Dowe builds systems."));
    let spanish = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("es.lproj/Localizable.strings"))
        .expect("spanish");
    assert!(spanish.content.contains("Dowe construye sistemas."));
    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");
    assert!(plist.content.contains("CFBundleDevelopmentRegion"));
    assert!(plist.content.contains("<string>en</string>"));
}

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
