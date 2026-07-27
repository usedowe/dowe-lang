fn swift_runtime_root_view_start() -> &'static str {
    r#"struct DoweRootView: View {
    @StateObject private var design = DoweDesign.shared
    @State private var rootEntry: DoweRouteEntry
    @State private var navigationPath: [DoweRouteEntry] = []
    @State private var externalUrl: DoweExternalUrl?
    @State private var safeAreaInsets = EdgeInsets()
    private let routeChanged: (String) -> Void

    init(initialPath: String = DoweRoutes.initialPath, routeChanged: @escaping (String) -> Void = { _ in }) {
        let resolved = DoweRoutes.paths.contains(initialPath) ? initialPath : DoweRoutes.initialPath
        _rootEntry = State(initialValue: DoweRouteEntry(path: resolved, fragment: nil))
        self.routeChanged = routeChanged
    }

    var body: some View {
"#
}
