#[test]
fn swiftui_large_route_dispatch_has_a_bounded_result_type() {
    let routes = (0..300)
        .map(|index| {
            let mut route = route();
            route.id = format!("page-{index}");
            route.route_path = format!("/page-{index}");
            route
        })
        .collect::<Vec<_>>();
    let output = generate_ios(
        &routes,
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let runtime = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.swift"))
        .expect("shared runtime");
    let start = runtime
        .content
        .find("    private func routeContent(")
        .unwrap();
    let end = runtime.content[start..]
        .find("    private func beginPageTransition()")
        .unwrap();
    let dispatch = &runtime.content[start..start + end];
    assert!(dispatch.contains("-> AnyView {"));
    assert!(!runtime.content[..start].ends_with("    @ViewBuilder\n"));
    assert_eq!(
        dispatch.matches("return AnyView(").count(),
        routes.len() + 1
    );
    for route in &routes {
        assert!(dispatch.contains(&format!("case \"{}\":", route.route_path)));
    }
    assert!(dispatch.contains("default:\n            return AnyView(Page0View("));
    assert_eq!(
        dispatch.matches("activeFragment: entry.fragment").count(),
        routes.len() + 1
    );
    assert!(runtime.content.contains(".id(routeRevision)"));
}
