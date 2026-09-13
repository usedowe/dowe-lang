#[test]
fn generates_shared_reactive_typography_for_swiftui() {
    let mut typography_route = route();
    typography_route.layout_tree = ViewNode::Children;
    typography_route.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "textSize01".to_string(),
            name: "textSize".to_string(),
            storage_key: "textSize".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::String("9xl".to_string()),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![
            ViewNode::Text {
                props: TextProps {
                    size: Some(ResponsiveValue::scalar(TextSize::Md)),
                    size_binding: Some(dowe_components::PropBinding::string("textSize")),
                    ..Default::default()
                },
                value: "Body".to_string(),
            },
            ViewNode::Title {
                props: TextProps {
                    size: Some(ResponsiveValue::scalar(TextSize::Md)),
                    size_binding: Some(dowe_components::PropBinding::string("textSize")),
                    ..Default::default()
                },
                value: "Title".to_string(),
            },
        ],
    };
    let output = generate_ios(
        &[typography_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains(
        "doweDynamicTextSize(state.text(\"textSize01\"), viewportWidth: viewportWidth, title: false)"
    ));
    assert!(views.contains(
        "doweDynamicTextSize(state.text(\"textSize01\"), viewportWidth: viewportWidth, title: true)"
    ));
    assert!(views.contains("doweDynamicTextLineHeight"));
    assert!(views.contains(
        "DoweTextMetrics(min: CGFloat(72), preferredBase: CGFloat(48), preferredViewport: CGFloat(7), max: CGFloat(128)"
    ));
    assert!(!views.contains("doweDynamicFontSize"));
}
