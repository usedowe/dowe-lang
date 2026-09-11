#[test]
fn generates_compose_and_dev_camera_microphone_capture_contract() {
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
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let source = all_android_source(&output);

    assert!(source.contains("DoweCamera(state = state, facing = \"user\""));
    assert!(source.contains("DoweMicrophone(state = state, label = \"Record audio\""));
    assert!(source.contains("MediaStore.ACTION_IMAGE_CAPTURE"));
    assert!(source.contains("MediaRecorder()"));
    assert!(source.contains("android.permission.CAMERA"));
    assert!(source.contains("android.permission.RECORD_AUDIO"));
    assert!(source.contains("requestPermissions(new String[]{Manifest.permission.CAMERA}"));
    assert!(source.contains("requestPermissions(new String[]{Manifest.permission.RECORD_AUDIO}"));
    assert!(source.contains("handlePermissionResult(int requestCode"));

    let activity = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDevActivity.java"))
        .expect("dev activity");
    assert!(
        activity
            .content
            .contains("void doweOpenCamera(String facing")
    );
    assert!(
        !activity
            .content
            .contains("private void doweOpenCamera(String facing")
    );
    let route_shard = output
        .files
        .iter()
        .find(|file| {
            file.relative_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("DoweDevRoute") && name.ends_with(".java"))
                && file.content.contains("runtime.doweOpenCamera(")
        })
        .expect("capture route shard");
    assert!(route_shard.content.contains("runtime.doweOpenCamera("));
}
