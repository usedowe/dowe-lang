#[test]
fn generates_persistent_view_store_for_compose_and_dev_shell() {
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
    let output = generate_android(
        &[persistent],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        generated
            .contains("DoweSignalMetadata(\"views/store/session:session\", \"global\", \"local\")")
    );
    assert!(generated.contains(
        "dowePutSignalMetadata(\"session01\", \"views/store/session:session\", \"global\", \"local\")"
    ));
    assert!(generated.contains("getSharedPreferences(\"dowe_view_state\""));
    assert!(generated.contains("compatibleSignalValue(stored, initial[id])"));
}

#[test]
fn generates_flex_item_behavior_for_flex_parents_but_not_grid_children() {
    let mut flex_route = route();
    flex_route.layout_tree = ViewNode::Children;
    flex_route.page_tree = ViewNode::Section {
        props: StyleProps {
            sizing: SizingProps {
                h: Some(ResponsiveValue::scalar(SizeValue::ViewportMinus(
                    ScaleValue::from_half_steps(0),
                ))),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![ViewNode::Grid {
            props: GridProps {
                style: StyleProps {
                    flex: Some(ResponsiveValue::ordered(vec![
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Xs,
                            value: FlexItem::Fill,
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Md,
                            value: FlexItem::None,
                        },
                    ])),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![ViewNode::Grid {
                props: GridProps {
                    style: StyleProps {
                        flex: Some(ResponsiveValue::scalar(FlexItem::Fill)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("Grid item")],
            }],
        }],
    };
    let output = generate_android(
        &[flex_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let compose = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("compose pages");
    let dev = dev_java_source(&output);

    assert_eq!(
        compose
            .content
            .matches("Modifier.weight(1f, fill = true)")
            .count(),
        1
    );
    assert!(
        compose
            .content
            .contains("xs = Modifier.weight(1f, fill = true), md = Modifier")
    );
    assert!(compose.content.contains(".fillMaxHeight()"));
    assert!(dev.content.contains("DOWE_FLEX_FILL"));
    assert!(dev.content.contains("DOWE_FLEX_NONE"));
    assert!(dev.content.contains("doweApplyFlexItem("));
    assert!(dev.content.contains("doweMeasureColumn"));
    assert!(all_android_source(&output).contains("int[] rowHeights = new int[0];"));
}

#[test]
fn preserves_multiline_text_in_compose_and_dev_shell() {
    let mut multiline = route();
    multiline.page_tree = ViewNode::Title {
        props: TextProps::default(),
        value: "Full-stack development,\nfrom one codebase".to_string(),
    };
    let output = generate_android(
        &[multiline],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let compose = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("Compose pages");
    assert!(
        compose
            .content
            .contains("Full-stack development,\\nfrom one codebase")
    );

    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("Full-stack development,\\nfrom one codebase")
    );
}

#[test]
fn inherits_container_foreground_and_preserves_text_overrides() {
    let mut color_route = route();
    color_route.layout_tree = ViewNode::Children;
    color_route.page_tree = container_foreground_tree();
    let output = generate_android(
        &[color_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let compose = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("Compose pages");

    assert!(compose.content.contains(
        "CompositionLocalProvider(LocalContentColor provides (doweResponsive(viewportWidth, xs = DoweDesign.primaryText) ?: LocalContentColor.current))"
    ));
    assert!(
        compose
            .content
            .contains("Text(\"Box inherited\", modifier = Modifier, color = Color.Unspecified")
    );
    assert!(compose.content.contains(
        "Text(\"Box override\", modifier = Modifier, color = doweResponsive(viewportWidth, xs = DoweDesign.danger) ?: LocalContentColor.current"
    ));
    assert!(compose.content.contains(
        "CardDefaults.cardColors(containerColor = DoweDesign.muted, contentColor = DoweDesign.mutedText)"
    ));
    assert!(
        compose
            .content
            .contains("Text(\"Card inherited\", modifier = Modifier, color = Color.Unspecified")
    );
    assert!(compose.content.contains(
        "CompositionLocalProvider(LocalDoweTitleColor provides DoweDesign.mutedTitle)"
    ));
    assert!(compose.content.contains(
        "Text(\"Card title inherited\", modifier = Modifier, color = LocalDoweTitleColor.current"
    ));
    assert!(compose.content.contains(
        "Text(\"Card override\", modifier = Modifier, color = doweResponsive(viewportWidth, xs = DoweDesign.warning) ?: LocalDoweTitleColor.current"
    ));
    for (label, token) in [
        ("Section inherited", "secondaryText"),
        ("Flex inherited", "tertiaryText"),
        ("Grid inherited", "mutedText"),
        ("Brand inherited", "surfaceText"),
        ("Banner inherited", "infoText"),
        ("Marquee inherited", "warningText"),
        ("Scaffold inherited", "dangerText"),
    ] {
        assert!(compose.content.contains(&format!(
            "CompositionLocalProvider(LocalContentColor provides (doweResponsive(viewportWidth, xs = DoweDesign.{token}) ?: LocalContentColor.current))"
        )));
        assert!(compose.content.contains(&format!(
            "Text(\"{label}\", modifier = Modifier, color = Color.Unspecified"
        )));
    }
    assert!(
        compose
            .content
            .contains("CompositionLocalProvider(LocalContentColor provides contentColor)")
    );
    assert!(compose.content.contains(
        "Text(\"Collapsible inherited\", modifier = Modifier, color = Color.Unspecified"
    ));
    assert!(compose.content.contains(
        "DoweTypeWriter(texts = listOf(\"TypeWriter inherited\"), typeSpeed = 10, deleteSpeed = 5, afterTyped = 20, afterDeleted = 10, repeat = false, contentColor = LocalContentColor.current"
    ));

    let dev = dev_java_source(&output);
    let box_inherited = dev
        .content
        .find("doweText(\"Box inherited\"")
        .expect("Box inherited text");
    assert!(dev.content[box_inherited..box_inherited + 320].contains("DOWE_PRIMARY_TEXT"));
    let box_override = dev
        .content
        .find("doweText(\"Box override\"")
        .expect("Box override text");
    assert!(dev.content[box_override..box_override + 320].contains("DOWE_DANGER"));
    assert!(
        dev.content
            .contains("doweText(\"Card inherited\", DOWE_MUTED_TEXT")
    );
    assert!(
        dev.content
            .contains("doweText(\"Card title inherited\", DOWE_MUTED_TITLE")
    );
    let card_override = dev
        .content
        .find("doweText(\"Card override\"")
        .expect("Card override text");
    assert!(dev.content[card_override..card_override + 320].contains("DOWE_WARNING"));
    for (label, token) in [
        ("Section inherited", "DOWE_SECONDARY_TEXT"),
        ("Flex inherited", "DOWE_TERTIARY_TEXT"),
        ("Grid inherited", "DOWE_MUTED_TEXT"),
        ("Brand inherited", "DOWE_SURFACE_TEXT"),
        ("Banner inherited", "DOWE_INFO_TEXT"),
        ("Marquee inherited", "DOWE_WARNING_TEXT"),
        ("Scaffold inherited", "DOWE_DANGER_TEXT"),
    ] {
        let start = dev
            .content
            .find(&format!("doweText(\"{label}\""))
            .unwrap_or_else(|| panic!("{label} text"));
        assert!(
            dev.content[start..start + 320].contains(token),
            "{label} should inherit {token}"
        );
    }
}

#[test]
fn generates_fixed_fab_as_native_overlay_with_dowe_icons() {
    let mut fab_route = route();
    fab_route.page_tree = fixed_fab_page();
    let output = generate_android(
        &[fab_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);

    assert!(generated.contains("var doweFixedFabOpen0 by remember"));
    assert!(generated.contains("Modifier.fillMaxSize().padding(horizontal"));
    assert!(generated.contains("if (doweFixedFabOpen0)"));
    assert!(generated.contains(
        "verticalArrangement = Arrangement.spacedBy(12.dp, alignment = Alignment.Bottom)"
    ));
    assert!(generated.contains(".rotate(if (doweFixedFabOpen0) 45f else 0f)"));
    assert!(
        generated.contains(".doweGesture(DoweGesturePreset.Press, DoweTransitionPreset.Smooth)")
    );
    assert!(generated.contains("DoweSvg(viewBox ="));
    assert!(generated.contains("setTag(\"dowe-fixed-fab\")"));
    assert!(generated.contains("Gravity.BOTTOM | Gravity.END"));
    assert!(generated.contains(".setRotation(open ? 45f : 0f)"));
    assert!(generated.contains("doweWrapContentWidth("));
    assert!(!generated.contains("setMinimumWidth(doweDp(180))"));
    assert!(!generated.contains("Text(\"+\")"));

    let mut top_fab_route = route();
    top_fab_route.page_tree = fixed_fab_page_at(OverlayCornerPosition::TopRight);
    let top_generated = all_android_source(&generate_android(
        &[top_fab_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));
    let mut top_lines = top_generated.lines();
    let trigger_line = top_lines
        .find(|line| line.contains(".setContentDescription(\"Open actions\")"))
        .expect("top Fab trigger");
    let trigger = trigger_line
        .trim()
        .split('.')
        .next()
        .expect("top Fab trigger variable");
    assert!(top_generated.contains(&format!("doweGesture({trigger}, \"press\", \"smooth\");")));
    let first_child_addition = top_lines
        .find(|line| line.contains("doweAdd("))
        .expect("top Fab first child addition");
    assert!(first_child_addition.contains(&format!(", {trigger},")));
}

#[test]
fn generates_relative_absolute_and_fixed_boxes_as_native_overlays() {
    let mut positioned_route = route();
    positioned_route.page_tree = positioned_box_page();
    let output = generate_android(
        &[positioned_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);

    assert!(generated.contains("Alignment.TopEnd"));
    assert!(generated.contains(".padding(top = doweResponsive(viewportWidth, xs = 16.dp) ?: 0.dp, end = doweResponsive(viewportWidth, xs = 24.dp) ?: 0.dp)"));
    assert!(generated.contains("Alignment.BottomEnd"));
    assert!(generated.contains(".padding(end = doweResponsive(viewportWidth, xs = 16.dp) ?: 0.dp, bottom = doweResponsive(viewportWidth, xs = 16.dp) ?: 0.dp)"));
    assert!(generated.contains("FrameLayout"));
    assert!(generated.contains("Gravity.TOP | Gravity.END"));
    assert!(generated.contains("dowe-fixed-box"));
    let dev = dev_java_source(&output).content;
    let proof = dev
        .find("doweText(\"Proof\"")
        .expect("positioned box content");
    let visibility = dev[..proof]
        .rfind("if (doweShow(doweResponsiveBool(viewportWidth, false, null, null, true, null))) {")
        .expect("positioned box visibility");
    assert!(proof - visibility < 2_000);
    let relative_width = dev[..proof]
        .rfind("setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));")
        .expect("relative box width");
    assert!(proof - relative_width < 3_000);
}

#[test]
fn generates_relative_box_cover_from_project_assets_for_compose_and_dev_launcher() {
    let mut cover_route = route();
    cover_route.page_tree = relative_box_cover_page();
    let output = generate_android(
        &[cover_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);
    assert!(generated.contains("DoweCoverBox("));
    assert!(generated.contains("/assets/img/guarias-login.webp"));
    let cover_runtime = generated
        .split("private fun DoweCoverBox")
        .nth(1)
        .expect("cover runtime")
        .split("private fun DoweGrid")
        .next()
        .expect("cover runtime boundary");
    assert!(cover_runtime.contains("doweLoadImageBitmap(context, source)"));
    assert!(!cover_runtime.contains("setImageURI(Uri.parse(source))"));

    let dev = dev_java_source(&output).content;
    assert!(dev.contains("/assets/img/guarias-login.webp"));
    assert!(dev.contains("CoverImage.setScaleType(ImageView.ScaleType.CENTER_CROP)"));
    assert!(dev.contains("doweLoadImageBitmap(view"));
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
                    element: ElementProps {
                        show: Some(VisibilityCondition::Static(responsive_bool(&[
                            (Breakpoint::Xs, false),
                            (Breakpoint::Lg, true),
                        ]))),
                        ..Default::default()
                    },
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

fn relative_box_cover_page() -> ViewNode {
    let mut page = positioned_box_page();
    let ViewNode::Box { props, .. } = &mut page else {
        panic!("relative box page");
    };
    props.cover = Some(ResponsiveValue::scalar(CoverSource(
        "/assets/img/guarias-login.webp".to_string(),
    )));
    page
}

fn fixed_fab_page() -> ViewNode {
    fixed_fab_page_at(OverlayCornerPosition::BottomRight)
}

fn fixed_fab_page_at(position: OverlayCornerPosition) -> ViewNode {
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
                    position,
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
fn generates_global_toasts_for_sequential_request_functions() {
    let mut sequential = route();
    sequential.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "session01".to_string(),
            name: "session".to_string(),
            storage_key: "session".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::Object(Vec::new()),
            schema: None,
        }],
        actions: vec![ViewAction {
            id: "login01".to_string(),
            name: "login".to_string(),
            params: Vec::new(),
            return_type: None,
            kind: ViewActionKind::Sequence(vec![
                ViewFunctionStatement::Request {
                    result: "res".to_string(),
                    action: ViewRequestAction {
                        method: ViewRequestMethod::Post,
                        path: "/api/auth/login".to_string(),
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
                    result: "res".to_string(),
                    success: vec![
                        ViewFunctionStatement::Assign(ViewAssignAction {
                            target: "session".to_string(),
                            source: "$dowe:literal".to_string(),
                            literal: Some(ViewSignalValue::Object(Vec::new())),
                            call: None,
                        }),
                        ViewFunctionStatement::Toast(ViewToastAction {
                            kind: "success".to_string(),
                            title: "Success".to_string(),
                            message: "Signed in".to_string(),
                            duration: Some(3000),
                            scheme: Some("success".to_string()),
                            variant: Some("soft".to_string()),
                            position: Some("top-right".to_string()),
                        }),
                    ],
                    error: vec![ViewFunctionStatement::Toast(ViewToastAction {
                        kind: "error".to_string(),
                        title: "Error".to_string(),
                        message: "Login failed".to_string(),
                        duration: None,
                        scheme: None,
                        variant: None,
                        position: None,
                    })],
                },
            ]),
        }],
        children: vec![text("Login")],
    };
    let output = generate_android(
        &[sequential],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);

    assert!(generated.contains("DoweAction.Sequence(listOf(DoweStep.Request"));
    assert!(generated.contains("DoweStep.Toast(\"success\", \"Success\", \"Signed in\", 3000"));
    assert!(generated.contains("DoweAction.sequence(new DoweStep[] {DoweStep.request"));
    assert!(generated.contains(
        "return new DoweStep(\"assign\", null, null, null, null, target, source, literal, hasLiteral, call, null, null, null, null, null, null);"
    ));
    assert!(generated.contains("DoweStep.toast(\"success\", \"Success\", \"Signed in\", 3000, \"success\", \"soft\", \"top-right\")"));
    assert!(generated.contains("DoweGlobalToast(toast = state.toast, close = state::closeToast, viewportWidth = viewportWidth)"));
    assert!(generated.contains("doweCardContainer(toast.variant, toast.scheme)"));
    assert!(generated.contains("doweShowToast(step);"));
    assert!(generated.contains("setContentDescription(\"Close toast\")"));
    assert!(generated.contains("contentDescription = \"Close toast\""));
    assert!(!generated.contains("android.widget.Toast.makeText"));
    assert!(generated.contains("signals.entries.lastOrNull { it.value.name == name }"));
    assert!(generated.contains("doweRequestPath(action.path, body, item)"));
}

#[test]
fn generates_terminal_replace_redirect_for_compose_and_dev_shell() {
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
    let output = generate_android(
        &[redirect_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);

    assert!(generated.contains("DoweStep.Redirect(\"/login\")"));
    assert!(generated.contains("redirectPath = step.path"));
    assert!(generated.contains("navigate(\"replace\", path, null)"));
    assert!(generated.contains("DoweStep.redirect(\"/login\")"));
    assert!(generated.contains("doweNavigate(\"replace\", step.target, null);"));
}

#[test]
fn generates_init_and_reactive_splash_for_compose_and_dev_shell() {
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
            vec![ViewFunctionStatement::Assign(ViewAssignAction {
                target: "isLoading".to_string(),
                source: "$dowe:bool:false".to_string(),
                literal: None,
                call: None,
            })],
        )],
        children: vec![ViewNode::Splash {
            binding: "isLoading".to_string(),
            initial: true,
            content: vec![text("Users"), fixed_fab_page()],
            children: vec![text("Loading users")],
        }],
    };
    let generated = all_android_source(&generate_android(
        &[splash_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(generated.contains("LaunchedEffect(Unit) { state.load(listOf(\"init01\")) }"));
    assert!(generated.contains("if (state.bool(\"loading01\"))"));
    assert!(generated.contains("if (!state.bool(\"loading01\"))"));
    assert!(generated.contains("if (doweBool(\"loading01\"))"));
    assert!(generated.contains("doweRunStartup(new String[] {\"init01\"})"));
}

#[test]
fn generates_immutable_compose_constants() {
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
    let output = generate_android(
        &[constant_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);
    assert!(
        generated
            .contains("constants = mapOf<String, Any?>(\"plans01\" to listOf<Any?>(\"Starter\"))")
    );
    assert!(generated.contains("private val constants: Map<String, Any?>"));
}

#[test]
fn generates_dynamic_image_source_for_compose_and_dev_shell() {
    let mut image_route = route();
    image_route.page_tree = ViewNode::Scope {
        constants: vec![dowe_components::ViewConstant {
            id: "features01".to_string(),
            name: "features".to_string(),
            value: ViewSignalValue::Array(vec![ViewSignalValue::Object(vec![
                (
                    "id".to_string(),
                    ViewSignalValue::String("feature".to_string()),
                ),
                (
                    "cover".to_string(),
                    ViewSignalValue::String("/assets/feature.webp".to_string()),
                ),
            ])]),
        }],
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![ViewNode::Each {
            item: "feature".to_string(),
            collection: "features".to_string(),
            key: "feature.id".to_string(),
            children: vec![ViewNode::Image {
                props: ImageProps {
                    style: VariantProps::default(),
                    src: String::new(),
                    reactive_src: Some("feature.cover".to_string()),
                    alt: "Feature".to_string(),
                    aspect: ImageAspect::Auto,
                    object_fit: ImageObjectFit::Cover,
                    loading: ImageLoading::Lazy,
                    hide_controls: true,
                },
            }],
        }],
    };
    let output = generate_android(
        &[image_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);
    assert!(generated.contains("DoweImage(source = state.text(\"item.cover\", row.value)"));
    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("doweImage(doweTextValue(\"item.cover\", row0)")
    );
}

#[test]
fn generates_compose_select_options_from_constant_each() {
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
    let output = generate_android(
        &[constant_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);
    assert!(generated.contains("state.rows(\"options01\").map { row -> DoweSelectOption"));
    assert!(generated.contains("state.text(\"item.value\", row.value)"));
    assert!(generated.contains("state.text(\"item.label\", row.value)"));
    assert!(generated.contains("doweRowTextValues(\"options01\", \"item.label\")"));
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
                (
                    "rounded".to_string(),
                    ViewSignalValue::String("full".to_string()),
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
                        rounded: Some("button.rounded".to_string()),
                        ..Default::default()
                    },
                    style: StyleProps {
                        shadow: Some(ResponsiveValue::scalar(ShadowSize::Sm)),
                        shadow_color: Some(ColorFamily::Secondary),
                        rounded: Some(ResponsiveValue::scalar(RoundedSize::Md)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("{button.label}")],
            }],
        }],
    };
    let output = generate_android(
        &[scoped_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);
    assert!(generated.contains("state.text(\"item.variant\", row.value)"));
    assert!(generated.contains("state.text(\"item.scheme\", row.value)"));
    assert!(
        generated.contains("doweButtonHorizontalPadding(state.text(\"item.size\", row.value))")
    );
    assert!(generated.contains("doweButtonMinHeight(state.text(\"item.size\", row.value))"));
    assert!(generated.contains("(\"outlined\".equals(doweTextValue(\"item.variant\", row"));
    assert!(generated.contains("state.text(\"item.label\", row.value)"));
    assert!(generated.contains("doweTextValue(\"item.scheme\", row"));
    assert!(generated.contains("setText(doweTextValue(\"item.label\", row"));
    assert!(generated.contains(
        "shape = RoundedCornerShape(doweButtonRadius(state.text(\"item.rounded\", row.value)))"
    ));
    assert!(
        generated.contains("DOWE_SECONDARY, doweButtonRadius(doweTextValue(\"item.rounded\", row")
    );
    assert!(!generated.contains(".doweRounded(doweResponsive(viewportWidth, xs = 8.dp))"));
    assert!(!generated.contains("doweResponsiveFloat(viewportWidth, 8f, null, null, null, null)"));
}

#[test]
fn generates_compose_box_and_text() {
    let output = generate_android(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(
        views
            .content
            .contains("Column(modifier = Modifier.fillMaxWidth()) {")
    );
    assert!(
        views
            .content
            .contains("Card(modifier = Modifier.fillMaxWidth()")
    );
    assert!(
        views
            .content
            .contains("Box(modifier = Modifier.fillMaxSize().background(DoweDesign.background))")
    );
    assert!(
        views
            .content
            .contains("Box(modifier = Modifier.fillMaxSize().verticalScroll(scrollState))")
    );
    assert!(
        !views
            .content
            .contains("import androidx.compose.foundation.layout.matchParentSize")
    );
    assert_eq!(
        views
            .content
            .matches("private fun doweFontFamily(value: DoweFont?): FontFamily")
            .count(),
        1
    );
    assert!(
        views
            .content
            .contains("Text(\"Layout\", modifier = Modifier, color = Color.Unspecified")
    );
    assert!(
        views
            .content
            .contains("Text(\"Login\", modifier = Modifier, color = Color.Unspecified")
    );
    assert!(
        views
            .content
            .contains("Font(R.font.inter_light, FontWeight.Thin)")
    );
    assert!(
        views
            .content
            .contains("Font(R.font.inter_regular, FontWeight.Normal)")
    );
    assert!(
        views
            .content
            .contains("Font(R.font.inter_extrabold, FontWeight.Black)")
    );
    assert!(views.content.contains("DoweFont.Inter -> DoweFonts.inter"));

    let root_gradle = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("build.gradle.kts"))
        .expect("root gradle");
    assert!(
        root_gradle
            .content
            .contains("org.jetbrains.kotlin.plugin.compose")
    );
    let gradle_properties = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("gradle.properties"))
        .expect("gradle properties");
    assert!(
        gradle_properties
            .content
            .contains("android.useAndroidX=true")
    );
    assert!(
        gradle_properties
            .content
            .contains("org.gradle.jvmargs=-Xmx2048m")
    );
    assert!(
        gradle_properties
            .content
            .contains("kotlin.daemon.jvmargs=-Xmx8192m")
    );
    let app_gradle = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("app/build.gradle.kts"))
        .expect("app gradle");
    assert!(app_gradle.content.contains("JvmTarget.JVM_17"));
    assert!(
        app_gradle
            .content
            .contains("androidx.compose:compose-bom:2026.06.01")
    );
    assert!(app_gradle.content.contains("androidx.compose.ui:ui\")"));

    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("root.setGravity(Gravity.TOP | Gravity.START)")
    );
    assert!(
        dev.content
            .contains("private static int DOWE_BACKGROUND = Color.rgb(255, 255, 255);")
    );
    assert!(
        dev.content
            .contains("background.setBackgroundColor(DOWE_BACKGROUND)")
    );
    assert!(
        dev.content
            .contains("root.setBackgroundColor(DOWE_BACKGROUND)")
    );
    assert!(
        dev.content
            .contains("getWindow().setStatusBarColor(Color.TRANSPARENT)")
    );
    assert!(
        dev.content
            .contains("getWindow().setNavigationBarColor(Color.TRANSPARENT)")
    );
    assert!(
        dev.content
            .contains("getWindow().setDecorFitsSystemWindows(false)")
    );
    assert!(
        dev.content
            .contains("boolean useDarkIcons = Color.luminance(DOWE_BACKGROUND) > 0.179f")
    );
    assert!(dev.content.contains(
        "getWindow().getInsetsController().setSystemBarsAppearance(useDarkIcons ? mask : 0, mask)"
    ));
    assert!(
        dev.content
            .contains("doweApplyTheme(name);\n        doweApplySystemBarAppearance();")
    );
    assert!(dev.content.contains("view.setOnApplyWindowInsetsListener"));
    assert!(dev.content.contains("scrollView.setClipToPadding(true);"));
    assert!(dev.content.contains(
        "scrollView.setOnScrollChangeListener((view, scrollX, scrollY, oldScrollX, oldScrollY) -> doweUpdatePinnedAppBarDock(scrollY > doweDp(100), true));"
    ));
    assert!(dev.content.contains("scrollView.addView(root"));
    assert!(dev.content.contains(
            "new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT)"
        ));
    assert!(dev.content.contains("doweCard(DOWE_PRIMARY, null)"));
    assert!(dev.content.contains(
        "private GradientDrawable doweInputBackground(int color, Integer strokeColor, float radius)"
    ));
    assert!(dev.content.contains("if (strokeColor != null)"));
    assert!(dev.content.contains("doweText(\"Layout\""));
    assert!(dev.content.contains("doweText(\"Login\""));
    assert!(dev.content.contains("final class DoweDevRouteLogin"));
    assert!(
        dev.content
            .contains("DoweDevLayout0.render(this, root, pageRoot -> renderPage(this, pageRoot));")
    );
    assert!(dev.content.contains("final class DoweDevLayout0"));
    assert!(dev.content.contains("page.accept(view0);"));
    assert!(dev.content.contains("private static void renderPage("));
    assert!(dev.content.contains("doweFontName(null)"));
    assert!(
        dev.content
            .contains("return value == null ? \"Inter\" : value;")
    );
    assert!(
        dev.content
            .contains("Typeface bundled = getResources().getFont(resource);")
    );
    assert!(dev.content.contains("return R.font.inter_light;"));
    assert!(dev.content.contains("return R.font.inter_regular;"));
    assert!(
        dev.content
            .contains("view.setTypeface(doweTypeface(font, weight));")
    );
    assert!(
        !dev.content
            .contains("Typeface baseTypeface = Typeface.create(font, Typeface.NORMAL);")
    );
    assert!(
        output
            .files
            .iter()
            .any(|file| file.relative_path.ends_with("dev/AndroidManifest.xml"))
    );
    let manifest = output
        .files
        .iter()
        .find(|file| {
            file.relative_path
                .ends_with("app/src/main/AndroidManifest.xml")
        })
        .expect("manifest");
    assert!(manifest.content.contains(r#"android:scheme="dowe-dev""#));
    assert!(
        manifest
            .content
            .contains(r#"android:windowSoftInputMode="adjustResize""#)
    );
    let main_activity = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("MainActivity.kt"))
        .expect("main activity");
    assert!(
        main_activity
            .content
            .contains("import androidx.activity.enableEdgeToEdge")
    );
    assert!(main_activity.content.contains("enableEdgeToEdge()"));
    assert!(
        main_activity
            .content
            .contains("val useDarkSystemBarIcons = DoweDesign.background.luminance() > 0.179f")
    );
    assert!(
        main_activity
            .content
            .contains("isAppearanceLightStatusBars = useDarkSystemBarIcons")
    );
    assert!(
        main_activity
            .content
            .contains("isAppearanceLightNavigationBars = useDarkSystemBarIcons")
    );
    assert!(
        main_activity
            .content
            .contains("restoreThemePreference()\n        applyIntentRoute(intent)")
    );
    assert!(main_activity.content.contains(
        "getSharedPreferences(\"dowe\", MODE_PRIVATE)\n            .getString(\"theme-preference\", DoweThemeModule.defaultTheme)"
    ));
    assert!(
        main_activity
            .content
            .contains("DoweDesign.applyTheme(storedTheme)")
    );
    assert!(
        !views
            .content
            .contains("val storedTheme = context.getSharedPreferences")
    );

    let hot_host = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevHostActivity.java"))
        .expect("hot host");
    assert!(hot_host.content.contains("DexClassLoader"));
    assert!(
        hot_host
            .content
            .contains("/_dowe/dev/modules/manifest.json")
    );
    assert!(hot_host.content.contains("getConstructor(Activity.class)"));
    assert!(hot_host.content.contains("resolveEndpoint(getIntent())"));
    assert!(
        hot_host
            .content
            .contains("getSharedPreferences(HMR_PREFERENCES, MODE_PRIVATE)")
    );
    assert!(
        hot_host
            .content
            .contains("putString(HMR_ENDPOINT, value).apply()")
    );
    assert!(hot_host.content.contains("getString(HMR_ENDPOINT, \"\")"));
    assert!(
        hot_host
            .content
            .contains("setContentView(loading);\n        restoreCachedModule();\n        poll();")
    );
    assert!(
        hot_host
            .content
            .contains("new File(getFilesDir(), \"dowe-modules\")")
    );
    assert!(hot_host.content.contains("getString(HMR_VERSION, \"\")"));
    assert!(hot_host.content.contains("putString(HMR_VERSION, version)"));
    assert!(dev.content.contains("extends ContextThemeWrapper"));
    assert!(
        dev.content
            .contains("public void mount(String preferredPath, Intent launchIntent)")
    );
}

#[test]
fn generates_compose_text_alignment() {
    let mut aligned = route();
    aligned.page_tree = ViewNode::Text {
        props: TextProps {
            align: Some(ResponsiveValue::scalar(TextAlign::Center)),
            ..Default::default()
        },
        value: "Aligned".to_string(),
    };
    let output = generate_android(
        &[aligned],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains(
        "Text(\"Aligned\", modifier = Modifier.fillMaxWidth(), color = Color.Unspecified,"
    ));
    assert!(views.content.contains("textAlign = TextAlign.Center"));
    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("setGravity(doweResponsiveInt(viewportWidth")
    );
    assert!(dev.content.contains("Gravity.CENTER_HORIZONTAL"));
}

#[test]
fn generates_text_and_title_background_for_compose_and_dev_launcher() {
    let mut styled = route();
    styled.layout_tree = ViewNode::Children;
    styled.page_tree = ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::Text {
                props: TextProps {
                    size: Some(ResponsiveValue::scalar(TextSize::Sm)),
                    style: StyleProps {
                        text: Some(ResponsiveValue::scalar(ColorToken::TertiaryText)),
                        bg: Some(ResponsiveValue::scalar(ColorToken::Tertiary)),
                        rounded: Some(ResponsiveValue::scalar(RoundedSize::Full)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                value: "Text badge".to_string(),
            },
            ViewNode::Title {
                props: TextProps {
                    size: Some(ResponsiveValue::scalar(TextSize::Sm)),
                    style: StyleProps {
                        text: Some(ResponsiveValue::scalar(ColorToken::TertiaryText)),
                        bg: Some(ResponsiveValue::scalar(ColorToken::Tertiary)),
                        rounded: Some(ResponsiveValue::scalar(RoundedSize::Full)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                value: "Title badge".to_string(),
            },
        ],
    };
    let output = generate_android(
        &[styled],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    let text_modifier = ".doweRounded(doweResponsive(viewportWidth, xs = 999.dp)).doweBackground(doweResponsive(viewportWidth, xs = DoweDesign.tertiary))";
    assert_eq!(views.content.matches(text_modifier).count(), 2);
    assert!(views.content.contains("DoweDesign.tertiaryText"));

    let dev = dev_java_source(&output);
    assert_eq!(
        dev.content
            .matches("Background = doweResponsiveInt(viewportWidth, DOWE_TERTIARY")
            .count(),
        2
    );
}

#[test]
fn resets_android_dev_route_after_process_relaunch() {
    let output = generate_android(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let hot_host = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevHostActivity.java"))
        .expect("hot host");
    let dev = dev_java_source(&output);

    assert!(!hot_host.content.contains("HMR_ROUTE"));
    assert!(!hot_host.content.contains("persistCurrentPath()"));
    assert!(hot_host.content.contains(
        "boolean initialMount = activeModule == null;\n            String path = initialMount ? null : activeModulePath();"
    ));
    assert!(
        hot_host
            .content
            .contains("mount.invoke(module, path, initialMount ? getIntent() : null)")
    );
    assert!(hot_host.content.contains(
        "private String activeModulePath() {\n        if (activeModule != null && activePath != null)"
    ));
    assert!(
        hot_host
            .content
            .contains("return null;\n    }\n\n    private void poll()")
    );
    assert!(dev.content.contains(
        "if (doweCanRoute(preferredPath)) {\n            currentPath = preferredPath;\n        }\n        doweApplyIntentRoute();"
    ));
    assert!(dev.content.contains(
        "if (data == null) {\n            return;\n        }\n        String path = data.getPath();"
    ));
}

#[test]
fn preserves_fixed_height_for_empty_grid_items_on_android() {
    let mut fixed_height_route = route();
    fixed_height_route.page_tree = ViewNode::Grid {
        props: GridProps {
            columns: Some(ResponsiveValue::scalar(GridTracks::Count(3))),
            gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                ScaleValue::from_half_steps(4),
            )))),
            style: StyleProps {
                shadow: Some(ResponsiveValue::scalar(ShadowSize::Lg)),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![ViewNode::Box {
            props: StyleProps {
                bg: Some(ResponsiveValue::scalar(ColorToken::Primary)),
                border: Some(ResponsiveValue::scalar(BorderWidth(1))),
                rounded: Some(ResponsiveValue::scalar(RoundedSize::Sm)),
                sizing: dowe_components::SizingProps {
                    h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                        ScaleValue::from_half_steps(16),
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: Vec::new(),
        }],
    };

    let output = generate_android(
        &[fixed_height_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let compose = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("compose pages");
    let dev = dev_java_source(&output);

    assert!(compose.content.contains(
        ".doweBackground(doweResponsive(viewportWidth, xs = DoweDesign.primary)).doweHeight(doweResponsive(viewportWidth, xs = DoweSize.Fixed(32.dp)))"
    ));
    assert!(
        compose
            .content
            .contains("shape = RoundedCornerShape(0.dp), color = Color.Black")
    );
    assert!(
        dev.content
            .contains("Height = doweResponsiveInt(viewportWidth")
    );
    assert!(dev.content.contains("doweShadow(view0"));
    assert!(dev.content.contains(", 0f, null);"));
    assert!(dev.content.contains("int childHeight = childParams == null ? ViewGroup.LayoutParams.WRAP_CONTENT : childParams.height;"));
    assert!(
        dev.content
            .contains("int childHeightSpec = getChildMeasureSpec(")
    );
    assert!(
        dev.content
            .contains("child.measure(childWidthSpec, childHeightSpec);")
    );
}

#[test]
fn preserves_fixed_box_width_inside_android_grids() {
    let mut fixed_width_route = route();
    fixed_width_route.page_tree = ViewNode::Grid {
        props: GridProps {
            columns: Some(ResponsiveValue::scalar(GridTracks::Count(1))),
            ..Default::default()
        },
        children: vec![
            ViewNode::Box {
                props: StyleProps {
                    sizing: dowe_components::SizingProps {
                        w: Some(ResponsiveValue::scalar(SizeValue::Scale(
                            ScaleValue::from_half_steps(24),
                        ))),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("H")],
            },
            ViewNode::Box {
                props: StyleProps::default(),
                children: vec![text("Full width")],
            },
        ],
    };

    let output = generate_android(
        &[fixed_width_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let compose = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("compose pages");
    let dev = dev_java_source(&output);

    assert!(
        compose
            .content
            .contains(".doweWidth(doweResponsive(viewportWidth, xs = DoweSize.Fixed(48.dp)))")
    );
    assert!(
        dev.content
            .contains("Width = doweResponsiveInt(viewportWidth, 48,")
    );
    assert!(dev.content.contains(
        "int childWidth = childParams == null ? ViewGroup.LayoutParams.WRAP_CONTENT : childParams.width;"
    ));
    assert!(dev.content.contains(
        "MeasureSpec.makeMeasureSpec(Math.min(childWidth, cellWidth), MeasureSpec.EXACTLY)"
    ));
    assert!(dev.content.contains(
        "child.layout(childLeft, rowTop, childLeft + child.getMeasuredWidth(), rowTop + child.getMeasuredHeight());"
    ));
}

#[test]
fn generates_non_throwing_dev_json_stringify_runtime() {
    let output = generate_android(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);

    assert!(dev.content.contains(
        "if (\"json.stringify\".equals(name)) return doweJsonString(args.get(\"value\"), Boolean.TRUE.equals(args.get(\"pretty\")));"
    ));
    assert!(
        dev.content
            .contains("private String doweJsonString(Object value, boolean pretty) {")
    );
    assert!(!dev.content.contains(
        "if (\"json.stringify\".equals(name)) return doweJson(args.get(\"value\")).toString();"
    ));
}

#[test]
fn generates_android_box_border_for_compose_and_dev_launcher() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Box {
        props: StyleProps {
            border: Some(ResponsiveValue::scalar(BorderWidth(2))),
            rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
            ..Default::default()
        },
        children: vec![text("Bordered")],
    };

    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains(
        ".border(doweResponsive(viewportWidth, xs = 2.dp) ?: 0.dp, DoweDesign.backgroundText, RoundedCornerShape(doweResponsive(viewportWidth, xs = 12.dp) ?: DoweDesign.radius))"
    ));

    let dev = dev_java_source(&output);

    assert!(dev.content.contains(
        "private GradientDrawable doweStyledBackground(int color, Integer strokeColor, Integer strokeWidth, float radius)"
    ));
    assert!(dev.content.contains(
        "view0.setBackground(doweStyledBackground(Color.TRANSPARENT, DOWE_BACKGROUND_TEXT, doweResponsiveInt(viewportWidth, 2, null, null, null, null), doweFloat(doweResponsiveFloat(viewportWidth, 12f, null, null, null, null), DOWE_RADIUS)))"
    ));
    assert!(
        dev.content
            .contains("background.setStroke(doweDp(strokeWidth), strokeColor)")
    );
}

#[test]
fn preserves_explicit_rounded_values_across_android_renderers() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Box {
        props: StyleProps {
            bg: Some(ResponsiveValue::scalar(ColorToken::Primary)),
            rounded: Some(ResponsiveValue::scalar(RoundedSize::Full)),
            ..Default::default()
        },
        children: vec![ViewNode::Button {
            props: VariantProps {
                style: StyleProps {
                    rounded: Some(ResponsiveValue::scalar(RoundedSize::Full)),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Continue")],
        }],
    };

    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    let dev = dev_java_source(&output);

    assert!(views.content.contains(".doweRounded(doweResponsive(viewportWidth, xs = 999.dp)).doweBackground(doweResponsive(viewportWidth, xs = DoweDesign.primary))"));
    assert!(views.content.contains(
        "RoundedCornerShape(doweResponsive(viewportWidth, xs = 999.dp) ?: DoweDesign.radius)"
    ));
    assert!(dev.content.contains(
        "doweRound(view0, doweResponsiveFloat(viewportWidth, 999f, null, null, null, null))"
    ));
    assert!(dev.content.contains(
        "doweRound(view1, doweResponsiveFloat(viewportWidth, 999f, null, null, null, null))"
    ));
    assert!(dev.content.contains("view.setClipToOutline(true)"));
}

#[test]
fn generates_tinted_card_shadow_for_compose() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Card {
        props: VariantProps {
            style: StyleProps {
                shadow: Some(ResponsiveValue::scalar(ShadowSize::Lg)),
                shadow_color: Some(ColorFamily::Primary),
                rounded: Some(ResponsiveValue::scalar(RoundedSize::Md)),
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Raised")],
    };

    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains(
        ".doweShadow(radius = doweResponsive(viewportWidth, xs = 44.dp) ?: 0.dp, shape = RoundedCornerShape(doweResponsive(viewportWidth, xs = 8.dp) ?: DoweDesign.radius), color = DoweDesign.primary, alpha = 0.28f)"
    ));
    assert!(views.content.contains(
        ".doweShadow(radius = doweResponsive(viewportWidth, xs = 44.dp) ?: 0.dp, shape = RoundedCornerShape(doweResponsive(viewportWidth, xs = 8.dp) ?: DoweDesign.radius), color = DoweDesign.primary, alpha = 0.28f).doweRounded(doweResponsive(viewportWidth, xs = 8.dp))"
    ));
    assert!(views.content.contains("private fun Modifier.doweShadow("));
    assert!(views.content.contains("dropShadow("));
    assert!(views.content.contains("DoweDropShadow("));
    assert!(
        views
            .content
            .contains("elevation = CardDefaults.cardElevation(defaultElevation = 0.dp)")
    );
    assert!(
        !views
            .content
            .contains("CardDefaults.cardElevation(defaultElevation = doweResponsive")
    );
}

#[test]
fn generates_centered_icon_button_without_empty_android_label() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::Button {
                props: VariantProps {
                    style: StyleProps {
                        sizing: SizingProps {
                            w: Some(ResponsiveValue::scalar(SizeValue::Scale(
                                ScaleValue::from_half_steps(20),
                            ))),
                            h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                                ScaleValue::from_half_steps(20),
                            ))),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    icon_start: Some(solar_control_icon("settings").expect("settings icon")),
                    icon_only: true,
                    label: Some("Open settings".to_string()),
                    navigation: Some(NavigationAction::Internal {
                        path: "/settings".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                    ..Default::default()
                },
                children: Vec::new(),
            },
            ViewNode::Button {
                props: VariantProps {
                    style: StyleProps {
                        spacing: SpacingProps {
                            px: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                            py: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(5))),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    navigation: Some(NavigationAction::Internal {
                        path: "/save".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                    ..Default::default()
                },
                children: vec![text("Save")],
            },
        ],
    };
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let compose = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("Compose pages");
    let dev = dev_java_source(&output);
    let route_shards = output
        .files
        .iter()
        .filter(|file| {
            file.relative_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("DoweDevRoute"))
        })
        .map(|file| file.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        compose
            .content
            .contains(".semantics { contentDescription = \"Open settings\" }.defaultMinSize")
    );
    assert!(
        compose
            .content
            .contains("onClick = { navigate(\"push\", \"/settings\", null) }")
    );
    assert!(
        compose
            .content
            .contains("contentPadding = PaddingValues(start = doweResponsive(viewportWidth")
    );
    assert!(
        compose
            .content
            .contains("onClick = { navigate(\"push\", \"/save\", null) }")
    );
    assert!(dev.content.contains("setGravity(Gravity.CENTER)"));
    assert!(
        dev.content
            .contains("setContentDescription(\"Open settings\")")
    );
    assert!(
        dev.content
            .contains("setOnClickListener(v -> doweNavigate(\"push\", \"/settings\", null))")
    );
    assert!(
        dev.content
            .contains("setOnClickListener(v -> doweNavigate(\"push\", \"/save\", null))")
    );
    assert!(!route_shards.contains(r#"doweText("")"#));
}

#[test]
fn preserves_static_button_variants_with_reactive_scheme_on_android() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "scheme01".to_string(),
            name: "schemeChoice".to_string(),
            storage_key: "schemeChoice".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::String("primary".to_string()),
            schema: None,
        }],
        actions: Vec::new(),
        children: [
            ComponentVariant::Solid,
            ComponentVariant::Soft,
            ComponentVariant::Outlined,
            ComponentVariant::Ghost,
        ]
        .into_iter()
        .map(|variant| ViewNode::Button {
            props: VariantProps {
                variant: Some(variant),
                reactive: ReactiveVariantProps {
                    scheme: Some("schemeChoice".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Action")],
        })
        .collect(),
    };
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = all_android_source(&output);

    assert!(generated.contains("doweButtonContainer(\"solid\", state.text("));
    assert!(generated.contains("doweButtonContainer(\"soft\", state.text("));
    assert!(generated.contains("doweButtonContainer(\"outlined\", state.text("));
    assert!(generated.contains("doweButtonContainer(\"ghost\", state.text("));
    assert!(generated.contains("doweButtonContainer(\"soft\", doweTextValue("));
    assert!(generated.contains("doweButtonContainer(\"outlined\", doweTextValue("));
    assert!(generated.contains("if (\"outlined\" == \"outlined\") BorderStroke"));
    assert!(generated.contains("\"outlined\".equals(\"outlined\")"));
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
                        style: shadow_style(ShadowSize::Lg, ColorFamily::Tertiary),
                        ..Default::default()
                    },
                    src: None,
                    name: Some("Dowe".to_string()),
                    alt: "Dowe".to_string(),
                    size: ButtonSize::Md,
                    status: None,
                    bordered: false,
                },
                icon: None,
            },
            ViewNode::Chip {
                props: ChipProps {
                    style: VariantProps {
                        style: shadow_style(ShadowSize::Xs, ColorFamily::Success),
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
                        sizing: SizingProps {
                            w: Some(ResponsiveValue::scalar(SizeValue::Full)),
                            ..Default::default()
                        },
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

    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    for expected in [
        "radius = doweResponsive(viewportWidth, xs = 24.dp) ?: 0.dp, shape = RoundedCornerShape(doweResponsive(viewportWidth, xs = 8.dp) ?: DoweDesign.radius), color = DoweDesign.primary, alpha = 0.28f",
        "radius = doweResponsive(viewportWidth, xs = 12.dp) ?: 0.dp, shape = RoundedCornerShape(DoweDesign.radius), color = DoweDesign.secondary, alpha = 0.28f",
        "radius = doweResponsive(viewportWidth, xs = 44.dp) ?: 0.dp, shape = RoundedCornerShape(999.dp), color = DoweDesign.tertiary, alpha = 0.28f",
        "radius = doweResponsive(viewportWidth, xs = 2.dp) ?: 0.dp, shape = RoundedCornerShape(null ?: DoweDesign.radius), color = DoweDesign.success, alpha = 0.28f",
        "radius = doweResponsive(viewportWidth, xs = 24.dp) ?: 0.dp, shape = RoundedCornerShape(doweResponsive(viewportWidth, xs = 12.dp) ?: DoweDesign.radius), color = DoweDesign.info, alpha = 0.28f",
    ] {
        assert!(views.content.contains(expected), "missing {expected}");
    }
    assert!(views.content.contains("radius <= 2.dp -> 0.12f"));
    assert!(views.content.contains("radius <= 12.dp -> 0.14f"));
    assert!(views.content.contains("radius <= 24.dp -> 0.16f"));
    assert!(views.content.contains("radius <= 44.dp -> 0.18f"));
    assert!(views.content.contains("else -> 0.22f"));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("BlurMaskFilter.Blur.NORMAL"));
    assert!(dev.content.contains("canvas.clipOutPath(surface)"));
    assert!(dev.content.contains("doweDrawChildShadows(this, canvas)"));
    assert!(dev.content.contains("view.setStateListAnimator(null);"));
    assert!(!dev.content.contains("setOutlineAmbientShadowColor"));
    assert!(!dev.content.contains("setOutlineSpotShadowColor"));
    for expected in [
        "doweResponsiveInt(viewportWidth, 24, null, null, null, null), DOWE_PRIMARY, doweFloat(doweResponsiveFloat(viewportWidth, 8f, null, null, null, null), DOWE_RADIUS), 0.28f",
        "doweResponsiveInt(viewportWidth, 12, null, null, null, null), DOWE_SECONDARY, DOWE_RADIUS, 0.28f",
        "doweResponsiveInt(viewportWidth, 44, null, null, null, null), DOWE_TERTIARY, 999f, 0.28f",
        "doweResponsiveInt(viewportWidth, 2, null, null, null, null), DOWE_SUCCESS, DOWE_RADIUS, 0.28f",
    ] {
        assert!(dev.content.contains(expected), "missing {expected}");
    }
    let field_line = dev
        .content
        .lines()
        .find(|line| line.contains(".setHint(\"dowe-app\")"))
        .expect("input field");
    let field = field_line
        .trim_start()
        .split('.')
        .next()
        .expect("field variable");
    assert!(dev.content.contains(&format!(
        "doweShadow({field}, doweResponsiveInt(viewportWidth, 24, null, null, null, null), DOWE_INFO, doweFloat(doweResponsiveFloat(viewportWidth, 12f, null, null, null, null), DOWE_RADIUS), 0.28f);"
    )));
    assert!(dev.content.contains(&format!(
        "doweRound({field}, doweResponsiveFloat(viewportWidth, 12f, null, null, null, null));"
    )));
}

#[test]
fn reuses_identical_dev_layout_methods_across_routes() {
    let first = route();
    let mut second = route();
    second.id = "signup".to_string();
    second.route_path = "/signup".to_string();
    second.page_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![text("Signup")],
    };

    let output = generate_android(
        &[first, second],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);

    assert_eq!(dev.content.matches("final class DoweDevLayout0").count(), 1);
    assert_eq!(
        dev.content
            .matches("DoweDevLayout0.render(this, root, pageRoot -> renderPage(this, pageRoot));")
            .count(),
        2
    );
}

#[test]
fn omits_absent_dev_padding_branches() {
    let props = StyleProps {
        spacing: SpacingProps {
            px: Some(responsive_scale(&[(Breakpoint::Xs, 8)])),
            ..Default::default()
        },
        ..Default::default()
    };
    let mut output = String::new();

    super::apply_dev_android_style(&props, "view0", true, &mut output);

    assert!(output.contains("Integer view0PaddingX = doweResponsiveInt"));
    assert!(!output.contains("Integer view0Padding ="));
    assert!(!output.contains("Integer view0PaddingY ="));
    assert!(!output.contains("Integer view0PaddingLeft ="));
    assert!(!output.contains("Integer view0PaddingRight ="));
    assert!(!output.contains("Integer view0PaddingTop ="));
    assert!(!output.contains("Integer view0PaddingBottom ="));
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

    let output = generate_android(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);

    assert!(!dev.content.contains("final class DoweDevLayout0"));
    assert!(!dev.content.contains("private static void renderPage("));
    assert!(
        dev.content
            .contains("doweTextValue(\"layout.message\", null)")
    );
}

#[test]
fn reuses_stateful_dev_layout_when_page_does_not_read_layout_state() {
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

    let output = generate_android(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);

    assert!(dev.content.contains("final class DoweDevLayout0"));
    assert!(dev.content.contains("private static void renderPage("));
    assert!(
        dev.content
            .contains("DoweDevLayout0.render(this, root, pageRoot -> renderPage(this, pageRoot));")
    );
    assert!(
        dev.content
            .contains("dowePutInitial(\"layout.open\", false);")
    );
}

#[test]
fn reuses_stateful_scaffold_drawer_layout_when_page_mentions_binding_literals() {
    let contextual = stateful_scaffold_drawer_layout_route(false);

    let output = generate_android(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);

    assert!(dev.content.contains("final class DoweDevLayout0"));
    assert!(dev.content.contains("private static void renderPage("));
    assert!(
        dev.content
            .contains("DoweDevLayout0.render(this, root, pageRoot -> renderPage(this, pageRoot));")
    );
    assert!(
        dev.content
            .contains("dowePutInitial(\"layout.drawer.open\", false);")
    );
    assert!(
        dev.content
            .contains("dowePutInitial(\"layout.drawer.visible\", true);")
    );
    assert!(
        dev.content
            .contains("doweActions.put(\"layout.drawer.open.action\", DoweAction.assign(\"layout.drawer.open\", \"layout.drawer.visible\"));")
    );
    let generated = output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<String>();
    assert!(generated.contains("Box(modifier = Modifier.fillMaxSize())"));
    assert!(generated.contains(
        "Row(modifier = Modifier.fillMaxWidth().weight(1f).verticalScroll(scrollState))"
    ));

    let boxed = stateful_scaffold_drawer_layout_route(true);
    let boxed_output = generate_android(
        &[boxed],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let boxed_generated = boxed_output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<String>();
    assert!(boxed_generated.contains(
        "Box(modifier = Modifier.fillMaxWidth().weight(1f), contentAlignment = Alignment.TopCenter)"
    ));
    assert!(boxed_generated.contains(
        "Row(modifier = Modifier.widthIn(max = 1536.dp).fillMaxSize().verticalScroll(scrollState))"
    ));
}

#[test]
fn generates_compose_and_dev_section_backgrounds() {
    let output = generate_android(
        &[section_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(
        views
            .content
            .contains("private enum class DoweSectionBackground")
    );
    assert!(views.content.contains("DoweSectionBackgroundBox("));
    assert!(views.content.contains(
        "Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.TopCenter)"
    ));
    assert!(
        views.content.contains(
            "Column(modifier = Modifier.widthIn(max = 1536.dp).fillMaxWidth().dowePadding"
        )
    );
    assert!(views.content.contains("Column(modifier = Modifier.dowePadding(all = null, horizontal = doweResponsive(viewportWidth, xs = 16.dp, md = 24.dp), vertical = doweResponsive(viewportWidth, xs = 40.dp, md = 64.dp)"));
    assert!(views.content.contains("background = doweResponsive(viewportWidth, xs = DoweSectionBackground.Aurora, md = DoweSectionBackground.Ocean)"));
    assert!(views.content.contains("Brush.linearGradient(listOf(DoweDesign.primary, DoweDesign.secondary, DoweDesign.tertiary))"));
    assert!(views.content.contains("DoweCoverBox("));
    assert!(views.content.contains("https://example.com/hero.jpg"));
    assert!(
        views
            .content
            .contains("DoweOverlay.Solid(Color.Black.copy(alpha = 0.35f))")
    );

    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("private GradientDrawable doweSectionBackground(String value)")
    );
    assert!(
        dev.content
            .contains("String view1SectionBackground = doweResponsiveString(viewportWidth, \"aurora\", null, \"ocean\", null, null)")
    );
    assert!(
        dev.content
            .contains("view1.setBackground(doweSectionBackground(view1SectionBackground));")
    );
    assert!(dev.content.contains("doweBoxedContainer(1536)"));
    assert!(
        dev.content
            .contains("PaddingX = doweResponsiveInt(viewportWidth, 16, null, 24, null, null)")
    );
    assert!(
        dev.content
            .contains("PaddingY = doweResponsiveInt(viewportWidth, 40, null, 64, null, null)")
    );
}

#[test]
fn generates_responsive_section_centering_for_compose_and_dev_android() {
    let mut route = section_route();
    let ViewNode::Box { children, .. } = &mut route.page_tree else {
        panic!("section route root");
    };
    let ViewNode::Section { props, .. } = &mut children[0] else {
        panic!("section route child");
    };
    props.center_x = Some(ResponsiveValue::ordered(vec![
        ResponsiveEntry {
            breakpoint: Breakpoint::Xs,
            value: false,
        },
        ResponsiveEntry {
            breakpoint: Breakpoint::Md,
            value: true,
        },
    ]));

    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains(
        "horizontalAlignment = (doweResponsive(viewportWidth, xs = false, md = true) ?: false) ? Alignment.CenterHorizontally : Alignment.Start"
    ));
    assert!(
        views
            .content
            .contains("Column(modifier = Modifier.fillMaxWidth()")
    );

    let dev = dev_java_source(&output);
    assert!(dev.content.contains(
        "setGravity((Boolean.TRUE.equals(false) ? Gravity.CENTER_VERTICAL : Gravity.TOP) | (Boolean.TRUE.equals(doweResponsiveBool(viewportWidth, false, null, true, null, null)) ? Gravity.CENTER_HORIZONTAL : Gravity.START));"
    ));
}

#[test]
fn fills_height_bounded_section_body_for_compose_and_dev_android() {
    let mut route = section_route();
    let ViewNode::Box { children, .. } = &mut route.page_tree else {
        panic!("section route root");
    };
    let ViewNode::Section {
        props,
        children: section_children,
    } = &mut children[0]
    else {
        panic!("section route child");
    };
    props.sizing.min_h = Some(ResponsiveValue::scalar(SizeValue::ViewportMinus(
        ScaleValue::from_half_steps(0),
    )));
    *section_children = vec![ViewNode::Grid {
        props: GridProps {
            columns: Some(ResponsiveValue::scalar(GridTracks::Count(1))),
            style: StyleProps {
                sizing: SizingProps {
                    min_h: Some(ResponsiveValue::scalar(SizeValue::Full)),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        children: vec![text("Grid")],
    }];

    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains(
        "Column(modifier = Modifier.widthIn(max = 1536.dp).fillMaxWidth().fillMaxHeight().dowePadding"
    ));
    assert!(
        views
            .content
            .contains("doweMinHeight(doweResponsive(viewportWidth, xs = DoweSize.Full))")
    );

    let dev = dev_java_source(&output);
    assert!(dev.content.contains("MinHeight = doweResponsiveInt"));
    assert!(
        dev.content
            .contains("== ViewGroup.LayoutParams.MATCH_PARENT")
    );
    assert!(
        dev.content
            .contains("Params.height = ViewGroup.LayoutParams.MATCH_PARENT")
    );
}

#[test]
fn generates_responsive_auto_and_full_height_constraints_for_containers() {
    let sizing = SizingProps {
        h: Some(ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: SizeValue::Auto,
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: SizeValue::Full,
            },
        ])),
        min_h: Some(ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: SizeValue::Auto,
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: SizeValue::Full,
            },
        ])),
        max_h: Some(ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: SizeValue::Auto,
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: SizeValue::Full,
            },
        ])),
        ..Default::default()
    };
    let mut height_route = route();
    height_route.layout_tree = ViewNode::Children;
    height_route.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: Vec::new(),
        actions: Vec::new(),
        children: vec![
            ViewNode::Box {
                props: StyleProps {
                    sizing: sizing.clone(),
                    ..Default::default()
                },
                children: Vec::new(),
            },
            ViewNode::Section {
                props: StyleProps {
                    sizing: sizing.clone(),
                    ..Default::default()
                },
                children: Vec::new(),
            },
            ViewNode::Grid {
                props: GridProps {
                    style: StyleProps {
                        sizing,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: Vec::new(),
            },
        ],
    };
    let output = generate_android(
        &[height_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    let dev = dev_java_source(&output);

    assert!(
        views
            .content
            .contains("doweResponsive(viewportWidth, xs = DoweSize.Auto, md = DoweSize.Full)")
    );
    assert!(
        views
            .content
            .contains("DoweSize.Full -> doweMaxParentHeight()")
    );
    assert!(dev.content.contains(
        "if (view0Width != null) { view0SizeParams.width = doweDimension(view0Width); }"
    ));
    assert!(dev.content.contains(
        "view0SizeParams = new ViewGroup.LayoutParams(\n                view0Width != null ? doweDimension(view0Width) : ViewGroup.LayoutParams.WRAP_CONTENT,"
    ));
    assert!(
        dev.content
            .contains("ViewGroup.LayoutParams view0MinHeightParams")
    );
    assert!(
        dev.content
            .contains("ViewGroup.LayoutParams view1SizeParams")
    );
    assert!(
        dev.content
            .contains("LinearLayout.LayoutParams view1Params")
    );
    assert!(dev.content.contains(
        "if (value == ViewGroup.LayoutParams.MATCH_PARENT || value == ViewGroup.LayoutParams.WRAP_CONTENT)"
    ));
    assert!(
        dev.content
            .contains("value == ViewGroup.LayoutParams.WRAP_CONTENT")
    );
}

#[test]
fn generates_responsive_cover_box_height_for_android_dev_and_compose() {
    let mut cover_route = route();
    cover_route.layout_tree = ViewNode::Children;
    cover_route.page_tree = ViewNode::Box {
        props: StyleProps {
            cover: Some(ResponsiveValue::scalar(CoverSource(
                "/assets/img/guarias-login.webp".to_string(),
            ))),
            extras: Some(Box::new(StyleExtras {
                position: dowe_components::PositionProps {
                    mode: BoxPosition::Relative,
                    ..Default::default()
                },
                ..Default::default()
            })),
            sizing: SizingProps {
                h: Some(ResponsiveValue::ordered(vec![
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Xs,
                        value: SizeValue::Scale(ScaleValue::from_half_steps(112)),
                    },
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: SizeValue::Full,
                    },
                ])),
                min_h: Some(ResponsiveValue::ordered(vec![
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Xs,
                        value: SizeValue::Auto,
                    },
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: SizeValue::Full,
                    },
                ])),
                max_h: Some(ResponsiveValue::ordered(vec![
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Xs,
                        value: SizeValue::Auto,
                    },
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: SizeValue::Full,
                    },
                ])),
                w: Some(ResponsiveValue::scalar(SizeValue::Full)),
                ..Default::default()
            },
            ..Default::default()
        },
        children: Vec::new(),
    };

    let output = generate_android(
        &[cover_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(
        views
            .content
            .contains("DoweSize.Fixed(224.dp), md = DoweSize.Full")
    );
    assert!(views.content.contains("DoweCoverBox("));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains(
        "Integer view0Height = doweResponsiveInt(viewportWidth, 224, null, ViewGroup.LayoutParams.MATCH_PARENT, null, null);"
    ));
    assert!(dev.content.contains(
        "view0SizeParams = new ViewGroup.LayoutParams(\n                view0Width != null ? doweDimension(view0Width) : ViewGroup.LayoutParams.WRAP_CONTENT,\n                view0Height != null ? doweDimension(view0Height) : ViewGroup.LayoutParams.WRAP_CONTENT"
    ));
    assert!(dev.content.contains(
        "view0MinHeightParams = new ViewGroup.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.MATCH_PARENT);"
    ));
    assert!(dev.content.contains("doweConstrain(view0,"));
    assert!(
        dev.content
            .contains("CoverImage.setScaleType(ImageView.ScaleType.CENTER_CROP)")
    );
}

#[test]
fn generates_responsive_section_gap_for_compose_and_dev_android() {
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

    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    assert!(views.content.contains(
        "verticalArrangement = Arrangement.spacedBy(doweResponsive(viewportWidth, xs = 8.dp, md = 16.dp) ?: 0.dp)"
    ));

    let dev = dev_java_source(&output);
    assert!(dev.content.contains(
        "doweAdd(view2, view3, doweResponsiveInt(viewportWidth, 8, null, 16, null, null), false);"
    ));
}

#[test]
fn generates_android_app_metadata() {
    let output = generate_android_with_app_and_translations(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
        &TranslationCatalog::default(),
        "Clinic Desk",
        "com.example.clinic",
    );
    let gradle = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("app/build.gradle.kts"))
        .expect("gradle");
    let app_manifest = output
        .files
        .iter()
        .find(|file| {
            file.relative_path
                .ends_with("app/src/main/AndroidManifest.xml")
        })
        .expect("app manifest");
    let dev_manifest = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("dev/AndroidManifest.xml"))
        .expect("dev manifest");
    let dev = dev_java_source(&output);

    assert!(
        gradle
            .content
            .contains(r#"applicationId = "com.example.clinic""#)
    );
    assert!(
        app_manifest
            .content
            .contains(r#"android:label="Clinic Desk""#)
    );
    assert!(
        dev_manifest
            .content
            .contains(r#"package="com.example.clinic""#)
    );
    assert!(
        dev_manifest
            .content
            .contains(r#"android:label="Clinic Desk""#)
    );
    assert!(dev.content.contains("import com.example.clinic.R;"));
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
fn generates_intrinsic_brand_navigation_without_button_chrome() {
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
    let output = generate_android(
        &[brand_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let pages = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("pages");
    let dev = dev_java_source(&output);

    assert!(pages.content.contains("Row(modifier = Modifier"));
    assert!(
        pages
            .content
            .contains(".clickable(onClick = { navigate(\"push\", \"/\", null) })")
    );
    assert!(
        pages
            .content
            .contains(".semantics { contentDescription = \"Dowe home\" }")
    );
    assert!(pages.content.contains("DoweSize.Fixed(128.dp)"));
    assert!(pages.content.contains("DoweSize.Fixed(32.dp)"));
    assert!(!pages.content.contains(" Button(modifier ="));
    assert!(dev.content.contains("doweContainer(true)"));
    assert!(dev.content.contains("setContentDescription(\"Dowe home\")"));
    assert!(
        dev.content
            .contains("setOnClickListener(v -> doweNavigate(\"push\", \"/\", null))")
    );
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
    let output = generate_android(
        &[banner_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let pages = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("pages");
    let dev = dev_java_source(&output);

    assert!(pages.content.contains("Column(modifier = Modifier"));
    assert!(pages.content.contains(
        ".clickable(onClick = { openExternal(\"system\", \"https://dowe.dev/cloud\") })"
    ));
    assert!(
        pages
            .content
            .contains(".semantics { contentDescription = \"Explore Dowe Cloud\" }")
    );
    assert!(!pages.content.contains(" Button(modifier ="));
    assert!(dev.content.contains("doweContainer(false)"));
    assert!(
        dev.content
            .contains("setContentDescription(\"Explore Dowe Cloud\")")
    );
    assert!(dev.content.contains(
        "setOnClickListener(v -> doweOpenExternal(\"system\", \"https://dowe.dev/cloud\"))"
    ));
}
