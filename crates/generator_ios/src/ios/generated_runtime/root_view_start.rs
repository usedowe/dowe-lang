fn swift_runtime_root_view_start() -> &'static str {
    r#"struct DoweRootView: View {
    @State private var rootEntry = DoweRouteEntry(path: DoweRoutes.initialPath, fragment: nil)
    @State private var navigationPath: [DoweRouteEntry] = []
    @State private var externalUrl: DoweExternalUrl?
    @State private var safeAreaInsets = EdgeInsets()

    var body: some View {
"#
}
