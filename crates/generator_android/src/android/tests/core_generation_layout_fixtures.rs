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

