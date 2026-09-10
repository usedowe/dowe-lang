fn ios_dev_host() -> String {
    r#"import Darwin
import Foundation
import SwiftUI
import UIKit
@main
struct DoweIosDevHostApp: App {
    var body: some Scene {
        WindowGroup {
            DoweIosDevModuleHost()
                .ignoresSafeArea()
        }
    }
}

struct DoweIosDevModuleHost: UIViewControllerRepresentable {
    func makeCoordinator() -> DoweIosDevModuleCoordinator {
        DoweIosDevModuleCoordinator()
    }

    func makeUIViewController(context: Context) -> UIViewController {
        let controller = UIViewController()
        controller.view.backgroundColor = .systemBackground
        context.coordinator.start(controller)
        return controller
    }

    func updateUIViewController(_ controller: UIViewController, context: Context) {
    }
}

final class DoweIosDevModuleCoordinator: NSObject {
    private let endpointKey = "dowe.hmr.endpoint"
    private let activeVersionKey = "dowe.hmr.version"
    private weak var container: UIViewController?
    private var activeController: UIViewController?
    private var activeVersion = ""
    private var activeRoute = "/"
    private var attemptedVersion = ""
    private var moduleEndpoint: String?
    private var handles: [UnsafeMutableRawPointer] = []
    private var waitingView: UIView?
    private var timer: Timer?
    private var loading = false

    func start(_ controller: UIViewController) {
        container = controller
        showWaitingState(in: controller)
        moduleEndpoint = resolveEndpoint()
        restoreCachedModule()
        poll()
        timer = Timer.scheduledTimer(withTimeInterval: 0.3, repeats: true) { [weak self] _ in
            self?.poll()
        }
    }

    private func resolveEndpoint() -> String? {
        let arguments = ProcessInfo.processInfo.arguments
        if let index = arguments.firstIndex(of: "--dowe-dev-server"), arguments.indices.contains(index + 1) {
            let value = arguments[index + 1]
            if !value.isEmpty {
                UserDefaults.standard.set(value, forKey: endpointKey)
                return value
            }
        }
        return UserDefaults.standard.string(forKey: endpointKey)
    }

    private func showWaitingState(in controller: UIViewController) {
        let spinner = UIActivityIndicatorView(style: .large)
        spinner.startAnimating()

        let title = UILabel()
        title.font = .preferredFont(forTextStyle: .headline)
        title.text = "Preparing Dowe app"
        title.textAlignment = .center
        title.textColor = .label

        let detail = UILabel()
        detail.font = .preferredFont(forTextStyle: .subheadline)
        detail.text = "The first iOS build can take a few minutes."
        detail.textAlignment = .center
        detail.textColor = .secondaryLabel
        detail.numberOfLines = 0

        let stack = UIStackView(arrangedSubviews: [spinner, title, detail])
        stack.axis = .vertical
        stack.alignment = .center
        stack.spacing = 12
        stack.translatesAutoresizingMaskIntoConstraints = false
        controller.view.addSubview(stack)
        NSLayoutConstraint.activate([
            stack.centerXAnchor.constraint(equalTo: controller.view.centerXAnchor),
            stack.centerYAnchor.constraint(equalTo: controller.view.centerYAnchor),
            stack.leadingAnchor.constraint(greaterThanOrEqualTo: controller.view.layoutMarginsGuide.leadingAnchor),
            stack.trailingAnchor.constraint(lessThanOrEqualTo: controller.view.layoutMarginsGuide.trailingAnchor),
        ])
        waitingView = stack
    }

    private func moduleFile(version: String) -> URL? {
        guard let root = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first else {
            return nil
        }
        let directory = root.appendingPathComponent("DoweModules", isDirectory: true)
        do {
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
            return directory.appendingPathComponent("dowe-module-\(version).dylib")
        } catch {
            return nil
        }
    }

    private func restoreCachedModule() {
        guard
            let version = UserDefaults.standard.string(forKey: activeVersionKey),
            !version.isEmpty,
            let file = moduleFile(version: version),
            FileManager.default.fileExists(atPath: file.path)
        else {
            return
        }
        _ = apply(file, version: version)
    }

    private func poll() {
        persistCurrentPath()
        guard !loading, let endpoint = moduleEndpoint, let url = URL(string: endpoint + "/_dowe/dev/modules/manifest.json?dowe_hmr=\(UUID().uuidString)") else {
            return
        }
        var request = URLRequest(url: url)
        request.cachePolicy = .reloadIgnoringLocalCacheData
        request.setValue("no-cache", forHTTPHeaderField: "Cache-Control")
        loading = true
        URLSession.shared.dataTask(with: request) { [weak self] data, _, _ in
            guard let self else { return }
            defer { self.loading = false }
            guard
                let data,
                let value = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                let targets = value["targets"] as? [String: Any],
                let ios = targets["ios"] as? [String: Any],
                let version = ios["version"] as? String,
                let path = ios["path"] as? String,
                version != self.activeVersion,
                let moduleUrl = URL(string: endpoint + path + "?dowe_hmr=\(version)")
            else {
                return
            }
            var moduleRequest = URLRequest(url: moduleUrl)
            moduleRequest.cachePolicy = .reloadIgnoringLocalCacheData
            moduleRequest.setValue("no-cache", forHTTPHeaderField: "Cache-Control")
            URLSession.shared.dataTask(with: moduleRequest) { [weak self] data, _, _ in
                guard let self, let data, let file = self.moduleFile(version: version) else { return }
                do {
                    try data.write(to: file, options: .atomic)
                    DispatchQueue.main.async {
                        _ = self.apply(file, version: version)
                    }
                } catch {
                }
            }.resume()
        }.resume()
    }

    @discardableResult
    private func apply(_ file: URL, version: String) -> Bool {
        guard let container else { return false }
        let handle = dlopen(file.path, RTLD_NOW | RTLD_LOCAL)
        guard let handle, let symbol = dlsym(handle, "dowe_create_root_view_controller") else {
            if let handle { dlclose(handle) }
            return false
        }
        typealias Factory = @convention(c) (UnsafePointer<CChar>?) -> UnsafeMutableRawPointer
        let factory = unsafeBitCast(symbol, to: Factory.self)
        let path = currentPath()
        let pointer = path.withCString { factory($0) }
        let next = Unmanaged<UIViewController>.fromOpaque(pointer).takeRetainedValue()
        activeController?.willMove(toParent: nil)
        activeController?.view.removeFromSuperview()
        activeController?.removeFromParent()
        container.addChild(next)
        next.view.frame = container.view.bounds
        next.view.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        container.view.addSubview(next.view)
        next.didMove(toParent: container)
        waitingView?.removeFromSuperview()
        waitingView = nil
        activeController = next
        activeVersion = version
        activeRoute = path
        UserDefaults.standard.set(version, forKey: activeVersionKey)
        handles.append(handle)
        attemptedVersion = version
        return true
    }

    private func currentPath() -> String {
        if
            let activeController,
            activeController.responds(to: NSSelectorFromString("doweCurrentPath")),
            let value = activeController.value(forKey: "doweCurrentPath") as? String
        {
            return value
        }
        return activeRoute
    }

    private func persistCurrentPath() {
        guard activeController != nil else { return }
        activeRoute = currentPath()
    }
}
"#
    .to_string()
}

fn ios_dev_module_factory() -> String {
    r#"import SwiftUI
import UIKit

final class DoweIosDevRouteTracker {
    var path: String

    init(path: String) {
        self.path = path
    }
}

@objc(DoweIosDevModuleController___DOWE_IOS_SOURCE_REVISION__)
final class DoweIosDevModuleController: UIHostingController<AnyView> {
    let routeTracker: DoweIosDevRouteTracker

    @objc dynamic var doweCurrentPath: String {
        routeTracker.path
    }

    init(path: String) {
        let tracker = DoweIosDevRouteTracker(path: path)
        routeTracker = tracker
        let root = DoweRootView(initialPath: path) { next in
            tracker.path = next
        }
        super.init(rootView: AnyView(root))
    }

    @MainActor required dynamic init?(coder: NSCoder) {
        nil
    }
}

@_cdecl("dowe_create_root_view_controller")
public func doweCreateRootViewController(_ path: UnsafePointer<CChar>?) -> UnsafeMutableRawPointer {
    let initialPath = path.map { String(cString: $0) } ?? DoweRoutes.initialPath
    let resolved = DoweRoutes.paths.contains(initialPath) ? initialPath : DoweRoutes.initialPath
    let controller = DoweIosDevModuleController(path: resolved)
    return Unmanaged.passRetained(controller).toOpaque()
}
"#
    .to_string()
}

