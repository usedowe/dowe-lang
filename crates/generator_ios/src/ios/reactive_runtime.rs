fn swift_reactive_runtime() -> &'static str {
    r#"
struct DoweRow: Identifiable {
    let id: String
    let value: [String: Any]
}

struct DoweRequestAction {
    let method: String
    let path: String
    let base: String
    let body: String?
    let update: String?
    let reset: String?
    let successAlert: String?
    let successMessage: String?
    let errorAlert: String?
    let errorMessage: String?
}

enum DoweAction {
    case request(DoweRequestAction)
    case assign(String, String)
    case reset(String)
}

@MainActor
final class DoweReactiveState: ObservableObject {
    @Published private var values: [String: Any]
    private let initial: [String: Any]
    private let actions: [String: DoweAction]
    private var loaded = Set<String>()

    init(initial: [String: Any], actions: [String: DoweAction]) {
        self.initial = initial
        self.values = initial
        self.actions = actions
    }

    func text(_ path: String, item: [String: Any]? = nil) -> String {
        guard let current = value(path, item: item), !(current is NSNull) else {
            return ""
        }
        return String(describing: current)
    }

    func bool(_ path: String, item: [String: Any]? = nil) -> Bool {
        value(path, item: item) as? Bool ?? false
    }

    func binding(_ path: String) -> Binding<String> {
        Binding(
            get: { self.text(path) },
            set: { self.write(path, value: $0) }
        )
    }

    func boolBinding(_ path: String) -> Binding<Bool> {
        Binding(
            get: { self.bool(path) },
            set: { self.write(path, value: $0) }
        )
    }

    func rows(_ path: String) -> [DoweRow] {
        let rows = value(path) as? [[String: Any]] ?? []
        return rows.enumerated().map { index, row in
            let id = row["id"].map { String(describing: $0) } ?? String(index)
            return DoweRow(id: id, value: row)
        }
    }

    func candles(_ path: String) -> [[String: Any]] {
        value(path) as? [[String: Any]] ?? []
    }

    func upsertCandles(_ path: String, payload: Any, maxPoints: Int) {
        var rows = candles(path)
        for candle in candlePayloads(payload) where isCandlePayload(candle) {
            if let key = candleKey(candle),
               let index = rows.firstIndex(where: { candleKey($0) == key }) {
                rows[index] = candle
            } else {
                rows.append(candle)
            }
        }
        if maxPoints > 0 && rows.count > maxPoints {
            rows = Array(rows.suffix(maxPoints))
        }
        write(path, value: rows)
    }

    private func candlePayloads(_ payload: Any) -> [[String: Any]] {
        if let candles = payload as? [[String: Any]] {
            return candles
        }
        guard let object = payload as? [String: Any] else {
            return []
        }
        if let data = object["data"] as? [[String: Any]] {
            return data
        }
        if let candle = object["data"] as? [String: Any] {
            return [candle]
        }
        return [object]
    }

    private func isCandlePayload(_ value: [String: Any]) -> Bool {
        guard candleKey(value) != nil,
              let open = candleNumber(value["open"]),
              let high = candleNumber(value["high"]),
              let low = candleNumber(value["low"]),
              let close = candleNumber(value["close"]) else {
            return false
        }
        return high >= low && high >= open && high >= close && low <= open && low <= close
    }

    private func candleKey(_ value: [String: Any]) -> String? {
        value["time"].map { String(describing: $0) }
    }

    private func candleNumber(_ value: Any?) -> Double? {
        if let number = value as? NSNumber {
            return number.doubleValue
        }
        if let text = value as? String {
            return Double(text)
        }
        return nil
    }

    func load(_ actionIds: [String]) {
        for id in actionIds where !loaded.contains(id) {
            loaded.insert(id)
            run(id)
        }
    }

    func run(_ id: String, item: [String: Any]? = nil) {
        guard let action = actions[id] else {
            return
        }
        switch action {
        case .assign(let target, let source):
            if let current = value(source, item: item) {
                write(target, value: current)
            }
        case .reset(let target):
            if let current = value(target, in: initial) {
                write(target, value: current)
            }
        case .request(let request):
            Task {
                await execute(request, item: item)
            }
        }
    }

    private func value(_ path: String, item: [String: Any]? = nil) -> Any? {
        if path == "item", let item {
            return item
        }
        if path.hasPrefix("item."), let item {
            return value(String(path.dropFirst(5)), in: item)
        }
        return value(path, in: values)
    }

    private func value(_ path: String, in source: [String: Any]) -> Any? {
        let parts = path.split(separator: ".").map(String.init)
        guard let root = parts.first, var current = source[root] else {
            return nil
        }
        for part in parts.dropFirst() {
            guard let object = current as? [String: Any], let next = object[part] else {
                return nil
            }
            current = next
        }
        return current
    }

    func write(_ path: String, value: Any) {
        let parts = path.split(separator: ".").map(String.init)
        guard let root = parts.first else {
            return
        }
        if parts.count == 1 {
            values[root] = value
            return
        }
        var object = values[root] as? [String: Any] ?? [:]
        object[parts[1]] = value
        values[root] = object
    }

    private func execute(_ action: DoweRequestAction, item: [String: Any]?) async {
        let body = action.body.flatMap { value($0, item: item) }
        let path = filledPath(action.path, body: body)
        let base = action.base.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        let address = base.isEmpty ? path : base + (path.hasPrefix("/") ? path : "/" + path)
        guard let url = URL(string: address), url.scheme != nil else {
            setAlert(action.errorAlert, type: "error", message: action.errorMessage ?? "Request failed")
            return
        }
        var request = URLRequest(url: url)
        request.httpMethod = action.method
        if let body, action.method != "GET", JSONSerialization.isValidJSONObject(body) {
            request.setValue("application/json", forHTTPHeaderField: "content-type")
            request.httpBody = try? JSONSerialization.data(withJSONObject: body)
        }
        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            let status = (response as? HTTPURLResponse)?.statusCode ?? 500
            let payload = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] ?? [:]
            if status < 200 || status >= 300 || payload["ok"] as? Bool == false {
                throw URLError(.badServerResponse)
            }
            if let update = action.update {
                write(update, value: payload["data"] ?? payload)
            }
            if let reset = action.reset, let current = value(reset, in: initial) {
                write(reset, value: current)
            }
            setAlert(action.successAlert, type: "success", message: action.successMessage ?? "Request completed")
        } catch {
            setAlert(action.errorAlert, type: "error", message: action.errorMessage ?? "Request failed")
        }
    }

    private func filledPath(_ path: String, body: Any?) -> String {
        guard path.contains(":id"), let object = body as? [String: Any], let id = object["id"] else {
            return path
        }
        return path.replacingOccurrences(of: ":id", with: String(describing: id))
    }

    private func setAlert(_ path: String?, type: String, message: String) {
        guard let path else {
            return
        }
        write(path, value: ["type": type, "message": message, "visible": true])
    }
}
"#
}
