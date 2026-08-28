#[test]
fn emits_portable_view_standard_library_for_web() {
    let router = super::router_js(&super::WebOutput {
        chunks: Vec::new(),
        pages: Vec::new(),
        translation_chunks: Vec::new(),
        default_locale: None,
        router_js: String::new(),
        render_report: dowe_components::RenderReport::new(dowe_components::RenderTarget::Web, Vec::new()),
    });

    for fragment in [
        "case\"url.querySet\"",
        "case\"csv.parse\"",
        "case\"sort.by\"",
        "case\"list.filterContains\"",
        "case\"json.pick\"",
        "case\"date.addDays\"",
    ] {
        assert!(
            router.contains(fragment),
            "missing web stdlib fragment: {fragment}"
        );
    }
}
