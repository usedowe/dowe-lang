#[test]
fn generates_swiftui_svg_views() {
    let output = generate_ios(
        &[svg_route(), runtime_svg_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweSvgView: View"));
    assert!(views.contains("DoweRuntimeSvgView(payload: state.json(\"iconData01\")"));
    assert!(views.contains("private enum DoweRuntimeSvgParser"));
    assert!(views.contains("DoweSvgViewBox(minX: CGFloat(0), minY: CGFloat(0), width: CGFloat(24), height: CGFloat(24))"));
    assert!(views.contains("DoweSvgFill.currentColor"));
    assert!(views.contains(
        "doweResponsive(viewportWidth, xs: DoweDesign.accent) ?? DoweDesign.backgroundText"
    ));
    assert!(views.contains("private final class DoweSvgPathCache: @unchecked Sendable"));
    assert!(views.contains("DoweSvgPathCache.shared.path(for: data)"));
    assert!(views.contains("storage.countLimit = 2048"));
    assert!(views.contains("transform: CGAffineTransform(a: 2, b: 0, c: 0, d: 2, tx: 4, ty: 6)"));
    assert!(views.contains("private final class DoweSvgImporter: NSObject, XMLParserDelegate"));
    assert!(views.contains("private func rectangle(_ attrs: [String: String]) -> String?"));
    assert!(views.contains("private func sameColor(_ left: String, _ right: String) -> Bool"));
    assert!(views.contains("private func originalFill(_ source: String?) -> String?"));
    assert!(views.contains("let evenOdd: Bool"));
    assert!(views.contains("if path.evenOdd { value[\"evenOdd\"] = true }"));
    assert!(views.contains("path.evenOdd ? \" fillRule:\\\"evenodd\\\"\" : \"\""));
    assert!(
        views.contains(
            "case \"parse.svg\": return DoweSvgImporter.convert(text(\"value\"), colors:"
        )
    );
    assert!(views.contains("c.253.847.1 1.895-.62 2.618a.75.75"));
    assert!(views.contains("if characters[index] == \"-\" || characters[index] == \"+\""));
    assert!(views.contains(
            ".frame(width: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))))"
        ));
    assert!(views.contains(
            ".frame(maxWidth: doweMaxSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))))"
        ));
    assert!(views.contains(
            ".frame(height: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32))), viewportHeight: viewportHeight))"
        ));
    assert!(views.contains(
            ".frame(maxHeight: doweMaxSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))))"
        ));
    assert!(!views.contains(", maxWidth: doweMaxSize("));
    assert!(!views.contains(", maxHeight: doweMaxSize("));
}

#[test]
fn generates_svg_aspect_ratio_for_single_swiftui_dimension() {
    let mut tree = svg_tree();
    let ViewNode::Svg { props, .. } = &mut tree else {
        panic!("svg tree");
    };
    props.style.sizing.w = None;
    props.view_box.width = "48".to_string();
    props.view_box.height = "24".to_string();
    let route = ViewRoute {
        id: "svg-ratio".to_string(),
        route_path: "/svg-ratio".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: tree,
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };

    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains(".aspectRatio(CGFloat(2.000000), contentMode: .fit)"));
    let height = views
        .find(".frame(height: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))")
        .expect("svg height modifier");
    let ratio = views
        .find(".aspectRatio(CGFloat(2.000000), contentMode: .fit)")
        .expect("svg ratio modifier");
    assert!(height < ratio);
}

#[test]
fn preserves_full_svg_ratio_inside_single_dimension_brand_on_swiftui() {
    let mut child = svg_tree();
    let ViewNode::Svg { props, .. } = &mut child else {
        panic!("svg tree");
    };
    props.style.sizing.w = Some(ResponsiveValue::scalar(SizeValue::Full));
    props.style.sizing.h = Some(ResponsiveValue::scalar(SizeValue::Full));
    props.view_box.width = "562".to_string();
    props.view_box.height = "145".to_string();
    let route = ViewRoute {
        id: "brand-ratio".to_string(),
        route_path: "/brand-ratio".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Brand {
            props: BrandProps {
                style: StyleProps {
                    sizing: SizingProps {
                        h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                            ScaleValue::from_half_steps(24),
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
            children: vec![child],
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
    let views = swift_content(&output);

    assert!(views.contains(".aspectRatio(CGFloat(3.875862), contentMode: .fit)"));
}

#[test]
fn generates_animated_svg_spinner_views() {
    let spinner = icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String("svg-spinners:3-dots-bounce".to_string()),
    }])
    .expect("spinner");
    let route = ViewRoute {
        id: "spinner".to_string(),
        route_path: "/spinner".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: spinner,
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("animated: true"));
    assert!(views.contains("@Environment(\\.accessibilityReduceMotion)"));
    assert!(views.contains("TimelineView(.animation)"));
}

