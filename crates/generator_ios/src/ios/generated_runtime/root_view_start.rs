fn swift_runtime_root_view_start() -> &'static str {
    r#"@MainActor
private struct DoweLayoutStateHost<Content: View>: View {
    let makeState: @MainActor () -> DoweReactiveState
    let content: (DoweReactiveState) -> Content
    @StateObject private var state: DoweReactiveState

    init(makeState: @escaping @MainActor () -> DoweReactiveState, @ViewBuilder content: @escaping (DoweReactiveState) -> Content) {
        self.makeState = makeState
        self.content = content
        _state = StateObject(wrappedValue: makeState())
    }

    var body: some View {
        content(state)
    }
}

struct DoweRootView: View {
    @StateObject private var design = DoweDesign.shared
    @State private var rootEntry: DoweRouteEntry
    @State private var navigationPath: [DoweRouteEntry] = []
    @State private var routeRevision = 0
    @State private var pageEntranceSuppressed = false
    @State private var pageTransitionSequence = 0
    @State private var externalUrl: DoweExternalUrl?
    @State private var safeAreaInsets = EdgeInsets()
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    private let routeChanged: (String) -> Void

    init(initialPath: String = DoweRoutes.initialPath, routeChanged: @escaping (String) -> Void = { _ in }) {
        let resolved = DoweRoutes.paths.contains(initialPath) ? initialPath : DoweRoutes.initialPath
        _rootEntry = State(initialValue: DoweRouteEntry(path: resolved, fragment: nil))
        self.routeChanged = routeChanged
    }

    var body: some View {
"#
}
