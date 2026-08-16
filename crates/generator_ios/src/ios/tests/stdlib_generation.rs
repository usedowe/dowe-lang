#[test]
fn generates_portable_view_standard_library_for_swiftui() {
    let output = generate_ios(
        &[svg_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let source = swift_content(&output);

    for fragment in [
        "case \"url.querySet\"",
        "case \"csv.parse\"",
        "case \"sort.by\"",
        "case \"list.filterContains\"",
        "case \"json.pick\"",
        "case \"date.addDays\"",
    ] {
        assert!(
            source.contains(fragment),
            "missing iOS stdlib fragment: {fragment}"
        );
    }
}
