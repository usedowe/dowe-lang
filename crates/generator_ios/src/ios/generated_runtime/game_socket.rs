fn swift_runtime_game_socket() -> &'static str {
    r#"private final class DoweGameSocketDelegate: NSObject, URLSessionWebSocketDelegate {
    let opened: (URLSessionWebSocketTask) -> Void
    let closed: (URLSessionWebSocketTask, URLSessionWebSocketTask.CloseCode, Data?) -> Void

    init(
        opened: @escaping (URLSessionWebSocketTask) -> Void,
        closed: @escaping (URLSessionWebSocketTask, URLSessionWebSocketTask.CloseCode, Data?) -> Void
    ) {
        self.opened = opened
        self.closed = closed
    }

    func urlSession(_ session: URLSession, webSocketTask: URLSessionWebSocketTask, didOpenWithProtocol protocol: String?) {
        opened(webSocketTask)
    }

    func urlSession(_ session: URLSession, webSocketTask: URLSessionWebSocketTask, didCloseWith closeCode: URLSessionWebSocketTask.CloseCode, reason: Data?) {
        closed(webSocketTask, closeCode, reason)
    }
}

@MainActor
final class DoweGameSocket: ObservableObject {
    private let state: DoweReactiveState
    private let sendPath: String?
    private let statusPath: String?
    private let onOpen: String?
    private let onMessage: String?
    private let onClose: String?
    private let onError: String?
    private let reconnect: Bool
    private let reconnectDelay: Int
    private var task: URLSessionWebSocketTask?
    private var session: URLSession?
    private var delegate: DoweGameSocketDelegate?
    private var reconnectWork: DispatchWorkItem?
    private var currentURL: URL?
    private var stopped = true
    private var isOpen = false
    private var lastSent: String?

    init(
        state: DoweReactiveState,
        sendPath: String?,
        statusPath: String?,
        onOpen: String?,
        onMessage: String?,
        onClose: String?,
        onError: String?,
        reconnect: Bool,
        reconnectDelay: Int
    ) {
        self.state = state
        self.sendPath = sendPath
        self.statusPath = statusPath
        self.onOpen = onOpen
        self.onMessage = onMessage
        self.onClose = onClose
        self.onError = onError
        self.reconnect = reconnect
        self.reconnectDelay = reconnectDelay
    }

    private func item(_ kind: String, data: Any? = nil) -> [String: Any] {
        ["source": "game", "kind": kind, "data": data ?? NSNull()]
    }

    private func run(_ action: String?, item: [String: Any]) {
        if let action {
            state.run(action, item: item)
        }
    }

    private func status(_ value: String) {
        if let statusPath {
            state.write(statusPath, value: value)
        }
    }

    private func socketURL(_ raw: String?) -> URL? {
        guard let value = raw?.trimmingCharacters(in: .whitespacesAndNewlines), !value.isEmpty else {
            return nil
        }
        if value.hasPrefix("ws://") || value.hasPrefix("wss://") {
            return URL(string: value)
        }
        guard value.hasPrefix("/"), !value.hasPrefix("//") else {
            return nil
        }
        let development = (UserDefaults.standard.string(forKey: "dowe.hmr.endpoint") ?? "").trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        let configured = DoweEnvironment.BACKEND_URL.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        guard let base = [development, configured].first(where: { $0.hasPrefix("http://") || $0.hasPrefix("https://") }) else {
            return nil
        }
        let socketBase = base.hasPrefix("https://") ? "wss://" + base.dropFirst(8) : "ws://" + base.dropFirst(7)
        return URL(string: String(socketBase) + value)
    }

    func start(_ raw: String?) {
        stop()
        stopped = false
        guard let url = socketURL(raw) else {
            status("error")
            run(onError, item: item("error", data: "Invalid WebSocket URL"))
            return
        }
        currentURL = url
        connect()
    }

    private func connect() {
        guard let currentURL, !stopped else { return }
        status("connecting")
        isOpen = false
        lastSent = nil
        let delegate = DoweGameSocketDelegate(
            opened: { [weak self] socket in
                Task { @MainActor in self?.handleOpen(socket) }
            },
            closed: { [weak self] socket, code, reason in
                Task { @MainActor in self?.handleClose(socket, code: code, reason: reason) }
            }
        )
        let session = URLSession(configuration: .default, delegate: delegate, delegateQueue: OperationQueue.main)
        self.delegate = delegate
        self.session = session
        let next = session.webSocketTask(with: currentURL)
        task = next
        next.resume()
        receive(from: next)
    }

    private func handleOpen(_ socket: URLSessionWebSocketTask) {
        guard !stopped, task === socket else { return }
        isOpen = true
        status("open")
        sendCurrent()
        run(onOpen, item: item("open"))
    }

    private func handleClose(_ socket: URLSessionWebSocketTask, code: URLSessionWebSocketTask.CloseCode, reason: Data?) {
        guard !stopped, task === socket else { return }
        isOpen = false
        task = nil
        session?.invalidateAndCancel()
        session = nil
        delegate = nil
        let reasonText = reason.flatMap { String(data: $0, encoding: .utf8) } ?? ""
        status("closed")
        run(onClose, item: item("close", data: ["code": code.rawValue, "reason": reasonText]))
        scheduleReconnect()
    }

    private func handleFailure(_ socket: URLSessionWebSocketTask, error: Error) {
        guard !stopped, task === socket else { return }
        isOpen = false
        task = nil
        session?.invalidateAndCancel()
        session = nil
        delegate = nil
        status("error")
        run(onError, item: item("error", data: error.localizedDescription))
        scheduleReconnect()
    }

    private func receive(from socket: URLSessionWebSocketTask) {
        socket.receive { [weak self] result in
            Task { @MainActor in
                guard let self, !self.stopped, self.task === socket else { return }
                switch result {
                case .success(.string(let text)):
                    self.run(self.onMessage, item: self.item("message", data: text))
                    self.receive(from: socket)
                case .success(.data(let data)):
                    let text = String(data: data, encoding: .utf8) ?? ""
                    self.run(self.onMessage, item: self.item("message", data: text))
                    self.receive(from: socket)
                case .failure(let error):
                    self.handleFailure(socket, error: error)
                @unknown default:
                    self.handleFailure(socket, error: NSError(domain: "DoweGame", code: -1, userInfo: [NSLocalizedDescriptionKey: "WebSocket failure"]))
                }
            }
        }
    }

    private func scheduleReconnect() {
        guard reconnect, !stopped, currentURL != nil, reconnectWork == nil else { return }
        let work = DispatchWorkItem { [weak self] in
            Task { @MainActor in
                guard let self, !self.stopped else { return }
                self.reconnectWork = nil
                self.connect()
            }
        }
        reconnectWork = work
        DispatchQueue.main.asyncAfter(
            deadline: .now() + .milliseconds(reconnectDelay.clamped(to: 100...60000)),
            execute: work
        )
    }

    private func sendCurrent() {
        send(sendPath.map { state.json($0) } ?? "")
    }

    func send(_ payload: String) {
        guard !payload.isEmpty, payload != lastSent, isOpen, task?.state == .running else { return }
        lastSent = payload
        task?.send(.string(payload)) { [weak self] error in
            guard let error else { return }
            Task { @MainActor in
                guard let self, !self.stopped else { return }
                self.status("error")
                self.run(self.onError, item: self.item("error", data: error.localizedDescription))
            }
        }
    }

    func stop() {
        stopped = true
        isOpen = false
        reconnectWork?.cancel()
        reconnectWork = nil
        task?.cancel(with: .normalClosure, reason: nil)
        session?.invalidateAndCancel()
        task = nil
        session = nil
        delegate = nil
        currentURL = nil
        lastSent = nil
    }
}

private extension Int {
    func clamped(to range: ClosedRange<Int>) -> Int {
        Swift.max(range.lowerBound, Swift.min(range.upperBound, self))
    }
}

struct DoweGameView: View {
    @ObservedObject var state: DoweReactiveState
    let scenePath: String?
    let renderer: String
    let worldPath: String?
    let cameraPath: String?
    let controls: String
    let moveSpeed: CGFloat
    let turnSpeed: CGFloat
    let viewWidth: CGFloat
    let viewHeight: CGFloat
    let fit: String
    let fps: Int
    let autoplay: Bool
    let pixelated: Bool
    let backgroundColor: Color
    let label: String
    let onPointer: String?
    let onKey: String?
    let onFire: String?
    let onMotion: String?
    let motionRate: Int
    let socketPath: String?
    let socketBinding: Bool
    let sendPath: String?
    let statusPath: String?
    let onOpen: String?
    let onMessage: String?
    let onClose: String?
    let onError: String?
    let reconnect: Bool
    let reconnectDelay: Int
    @StateObject private var socket: DoweGameSocket

    init(
        state: DoweReactiveState,
        scenePath: String?,
        renderer: String,
        worldPath: String?,
        cameraPath: String?,
        controls: String,
        moveSpeed: CGFloat,
        turnSpeed: CGFloat,
        viewWidth: CGFloat,
        viewHeight: CGFloat,
        fit: String,
        fps: Int,
        autoplay: Bool,
        pixelated: Bool,
        backgroundColor: Color,
        label: String,
        onPointer: String?,
        onKey: String?,
        onFire: String?,
        onMotion: String?,
        motionRate: Int,
        socketPath: String?,
        socketBinding: Bool,
        sendPath: String?,
        statusPath: String?,
        onOpen: String?,
        onMessage: String?,
        onClose: String?,
        onError: String?,
        reconnect: Bool,
        reconnectDelay: Int
    ) {
        self.state = state
        self.scenePath = scenePath
        self.renderer = renderer
        self.worldPath = worldPath
        self.cameraPath = cameraPath
        self.controls = controls
        self.moveSpeed = moveSpeed
        self.turnSpeed = turnSpeed
        self.viewWidth = viewWidth
        self.viewHeight = viewHeight
        self.fit = fit
        self.fps = fps
        self.autoplay = autoplay
        self.pixelated = pixelated
        self.backgroundColor = backgroundColor
        self.label = label
        self.onPointer = onPointer
        self.onKey = onKey
        self.onFire = onFire
        self.onMotion = onMotion
        self.motionRate = motionRate
        self.socketPath = socketPath
        self.socketBinding = socketBinding
        self.sendPath = sendPath
        self.statusPath = statusPath
        self.onOpen = onOpen
        self.onMessage = onMessage
        self.onClose = onClose
        self.onError = onError
        self.reconnect = reconnect
        self.reconnectDelay = reconnectDelay
        _socket = StateObject(wrappedValue: DoweGameSocket(
            state: state,
            sendPath: sendPath,
            statusPath: statusPath,
            onOpen: onOpen,
            onMessage: onMessage,
            onClose: onClose,
            onError: onError,
            reconnect: reconnect,
            reconnectDelay: reconnectDelay
        ))
    }

    private var rawSocket: String? {
        guard let socketPath else { return nil }
        return socketBinding ? state.text(socketPath) : socketPath
    }

    var body: some View {
        let outbound = sendPath.map { state.json($0) } ?? ""
        Group {
            if renderer == "raycast3d", let worldPath, let cameraPath {
                DoweRaycastGameView(
                    state: state,
                    worldPath: worldPath,
                    cameraPath: cameraPath,
                    controls: controls,
                    moveSpeed: moveSpeed,
                    turnSpeed: turnSpeed,
                    viewWidth: viewWidth,
                    viewHeight: viewHeight,
                    fit: fit,
                    fps: fps,
                    autoplay: autoplay,
                    pixelated: pixelated,
                    backgroundColor: backgroundColor,
                    label: label,
                    onPointer: onPointer,
                    onKey: onKey,
                    onFire: onFire
                )
            } else {
                DoweCanvasView(
                    state: state,
                    scenePath: scenePath ?? "",
                    viewWidth: viewWidth,
                    viewHeight: viewHeight,
                    fit: fit,
                    fps: fps,
                    autoplay: autoplay,
                    pixelated: pixelated,
                    backgroundColor: backgroundColor,
                    label: label,
                    onPointer: onPointer,
                    onKey: onKey,
                    onMotion: onMotion,
                    motionRate: motionRate,
                    draw: false,
                    drawMode: "pen",
                    drawModePath: nil,
                    layersPath: nil,
                    selectedPath: nil,
                    onLayerAdd: nil,
                    onLayerChange: nil,
                    onLayerRemove: nil,
                    onLayerSelect: nil
                )
            }
        }
        .onAppear {
            if let rawSocket { socket.start(rawSocket) } else { socket.stop() }
        }
        .onDisappear { socket.stop() }
        .onChange(of: rawSocket) { _, next in
            if let next { socket.start(next) } else { socket.stop() }
        }
        .onChange(of: outbound) { _, next in socket.send(next) }
    }
}
"#
}
