fn swift_runtime_route_helpers() -> &'static str {
    r#"struct DoweExternalUrl: Identifiable {
    let url: URL
    var id: String {
        url.absoluteString
    }
}

struct DoweRouteEntry: Hashable {
    let path: String
    let fragment: String?
}

struct DoweExternalWebView: UIViewControllerRepresentable {
    let url: URL

    func makeUIViewController(context: Context) -> SFSafariViewController {
        SFSafariViewController(url: url)
    }

    func updateUIViewController(_ controller: SFSafariViewController, context: Context) {
    }
}

struct DoweSafeAreaReporter: UIViewRepresentable {
    let onChange: (EdgeInsets) -> Void

    func makeUIView(context: Context) -> DoweSafeAreaReportingView {
        let view = DoweSafeAreaReportingView()
        view.onChange = onChange
        return view
    }

    func updateUIView(_ view: DoweSafeAreaReportingView, context: Context) {
        view.onChange = onChange
        view.reportSafeArea()
    }
}

final class DoweSafeAreaReportingView: UIView {
    var onChange: ((EdgeInsets) -> Void)?

    override func didMoveToWindow() {
        super.didMoveToWindow()
        reportSafeArea()
    }

    override func safeAreaInsetsDidChange() {
        super.safeAreaInsetsDidChange()
        reportSafeArea()
    }

    func reportSafeArea() {
        let uiInsets = window?.safeAreaInsets ?? safeAreaInsets
        let insets = EdgeInsets(top: uiInsets.top, leading: uiInsets.left, bottom: uiInsets.bottom, trailing: uiInsets.right)
        DispatchQueue.main.async {
            self.onChange?(insets)
        }
    }
}

"#
}
