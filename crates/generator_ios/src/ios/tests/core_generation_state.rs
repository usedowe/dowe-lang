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
                        sizing: SizingProps {
                            h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                                ScaleValue::from_half_steps(112),
                            ))),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("Grid item")],
            },
            ViewNode::Box {
                props: StyleProps::default(),
                children: vec![text("Flexible grid item")],
            }],
        }],
    };
    let generated = swift_content(&generate_ios(
        &[flex_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    ));

    assert!(generated.contains("enum DoweFlexItem: Equatable"));
    assert!(generated.contains("DoweFlexItem.fill"));
    assert!(generated.contains("DoweFlexItem.none"));
    assert_eq!(generated.matches(".doweFlexItem(").count(), 1);
    assert!(generated.contains(
        ".frame(maxHeight: .infinity, alignment: .top).layoutPriority(1)"
    ));
    assert!(generated.contains(
        "fillHeight: (false) || ((doweResponsive(viewportWidth, xs: true, md: false) ?? false))"
    ));
    assert!(generated.contains(
        "let stretches = stretchesByDefault && subviews[index][DoweGridItemStretchKey.self]"
    ));
    assert!(generated.contains(
        ".doweGridItemStretches((doweResponsive(viewportWidth, xs: false) ?? true))"
    ));
    assert!(generated.contains(".doweGridItemStretches(true)"));
    assert!(generated.contains("image.resizable().scaledToFill().clipped()"));
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
    assert!(generated.contains("DoweOverlayCloseIcon(color: DoweDesign.mutedText)"));
    assert!(generated.contains(".accessibilityLabel(\"Close toast\")"));
    assert!(!generated.contains("UIAlertController"));
}

