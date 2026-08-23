#[test]
fn generates_swiftui_view_motion() {
    let output = generate_ios(
        &[motion_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("enum DoweAnimationPreset"));
    assert!(views.contains(".modifier(DoweAnimationModifier(preset: .fadeIn))"));
    assert!(views.contains(".modifier(DoweAnimationModifier(preset: .slideUp))"));
    assert!(views.contains(".animation(.easeOut(duration: 0.22), value: active)"));
}

#[test]
fn generates_swiftui_form_validation_contract() {
    let mut props = VariantProps {
        label: Some("Email".to_string()),
        variant: Some(ComponentVariant::Outlined),
        ..Default::default()
    };
    let validation = props.element.form_validation_mut();
    validation.help_text = Some("Use your work email".to_string());
    validation.rules = vec![
        dowe_components::form_validation_rule("required", "Email is required").expect("rule"),
        dowe_components::form_validation_rule("email", "Enter a valid email").expect("rule"),
    ];
    let route = ViewRoute {
        id: "validation".to_string(),
        route_path: "/validation".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Input { props },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let source = swift_content(&output);
    assert!(source.contains("struct DoweValidationRule"));
    assert!(source.contains("private func doweValidationError"));
    assert!(source.contains("message: \"Email is required\""));
    assert!(source.contains("helpText: \"Use your work email\""));
    assert!(source.contains("touched ? doweValidationError"));
    assert!(source.contains("DoweDesign.danger"));
    let date_start = source
        .find("struct DoweDateField: View")
        .expect("date field");
    let date_end = source[date_start..]
        .find("struct DoweDateRangeField: View")
        .map(|offset| date_start + offset)
        .expect("date range field");
    let date_source = &source[date_start..date_end];
    assert!(date_source.contains("let validationRules: [DoweValidationRule]"));
    assert!(date_source.contains("@State private var touched = false"));
}

#[test]
fn generates_swiftui_camera_and_microphone_capture_contract() {
    let route = ViewRoute {
        id: "capture".to_string(),
        route_path: "/capture".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps::default(),
            children: vec![
                ViewNode::Camera {
                    props: CameraProps {
                        style: VariantProps::default(),
                        facing: CameraFacing::User,
                        label: "Take photo".to_string(),
                        disabled: false,
                        on_start: Some("cameraStart".to_string()),
                        on_capture: Some("cameraCapture".to_string()),
                        on_error: Some("cameraError".to_string()),
                    },
                },
                ViewNode::Microphone {
                    props: MicrophoneProps {
                        style: VariantProps::default(),
                        label: "Record audio".to_string(),
                        max_duration: Some(30),
                        disabled: false,
                        on_start: Some("microphoneStart".to_string()),
                        on_stop: Some("microphoneStop".to_string()),
                        on_error: Some("microphoneError".to_string()),
                    },
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let source = swift_content(&output);
    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("Info.plist")
        .content
        .clone();

    assert!(source.contains("DoweCameraView(state: state, facing: \"user\""));
    assert!(source.contains("DoweMicrophoneView(state: state, label: \"Record audio\""));
    assert!(source.contains("struct DoweCameraPicker: UIViewControllerRepresentable"));
    assert!(source.contains("let sourceType: UIImagePickerController.SourceType"));
    assert!(source.contains("if sourceType == .camera"));
    assert!(source.contains("AVAudioRecorderDelegate"));
    assert!(source.contains(
        "AVAudioApplication.requestRecordPermission(completionHandler: handlePermission)"
    ));
    assert!(source.contains("let handlePermission: @Sendable (Bool) -> Void"));
    assert!(source.contains("nonisolated func audioRecorderDidFinishRecording"));
    assert!(source.contains("Task { @MainActor [weak self] in"));
    assert!(plist.contains("NSCameraUsageDescription"));
    assert!(plist.contains("NSMicrophoneUsageDescription"));
}
