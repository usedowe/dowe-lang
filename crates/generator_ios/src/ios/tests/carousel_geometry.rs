#[test]
fn carousel_geometry_is_emitted_for_every_variant() {
    for variant in dowe_components::CarouselVariant::all() {
        let mut route = media_display_form_route();
        let ViewNode::Box { children, .. } = &mut route.page_tree else {
            panic!("fixture root");
        };
        let carousel = children
            .iter_mut()
            .find(|node| matches!(node, ViewNode::Carousel { .. }))
            .unwrap();
        let ViewNode::Carousel { props, .. } = carousel else {
            unreachable!()
        };
        props.variant = *variant;
        props.slide_width = None;
        let output = generate_ios(
            &[route],
            &FontConfig::default(),
            &DesignConfig::default(),
            &[],
        );
        let source = swift_content(&output);
        assert!(
            source.contains(&format!(
                "DoweCarouselView(variant: \"{}\"",
                variant.as_str()
            )),
            "{variant:?}"
        );
        assert!(
            source.contains("geometry: DoweCarouselGeometry(contentGap: CGFloat(12)"),
            "{variant:?}"
        );
    }
}

#[test]
fn carousel_title_uses_shared_24_point_typography() {
    let output = generate_ios(
        &[media_display_form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let source = swift_content(&output);
    assert!(source.contains(".font(.system(size: CGFloat(24), weight: .bold))"));
}
