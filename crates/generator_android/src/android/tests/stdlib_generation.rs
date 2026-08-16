#[test]
fn generates_portable_view_standard_library_for_compose_and_dev() {
    let output = generate_android(
        &[svg_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let source = all_android_source(&output);

    for fragment in [
        "\"url.querySet\"",
        "\"csv.parse\"",
        "\"sort.by\"",
        "\"list.filterContains\"",
        "\"json.pick\"",
        "\"date.addDays\"",
    ] {
        assert!(
            source.contains(fragment),
            "missing Android stdlib fragment: {fragment}"
        );
    }
}
