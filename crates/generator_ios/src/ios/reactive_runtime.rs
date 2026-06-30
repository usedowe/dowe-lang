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
    case assign(String, String, DoweStdlibCall?)
    case reset(String)
}

struct DoweStdlibCall {
    let namespace: String
    let function: String
    let args: [DoweStdlibArg]
}

struct DoweStdlibArg {
    let name: String
    let value: DoweStdlibValue
}

struct DoweStdlibValue {
    let kind: String
    let value: Any?
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
        case .assign(let target, let source, let call):
            let current = call.map { stdlib($0, item: item) } ?? value(source, item: item)
            write(target, value: current ?? NSNull())
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

    private func stdlib(_ call: DoweStdlibCall, item: [String: Any]?) -> Any? {
        let args = Dictionary(uniqueKeysWithValues: call.args.map { ($0.name, stdlibValue($0.value, item: item)) })
        func text(_ name: String) -> String { stdlibText(args[name] ?? nil) }
        func number(_ name: String) -> Double? { stdlibNumber(args[name] ?? nil) }
        func list(_ name: String) -> [Any] { args[name] as? [Any] ?? [] }
        switch call.namespace + "." + call.function {
        case "str.trim": return text("value").trimmingCharacters(in: .whitespacesAndNewlines)
        case "str.lower": return text("value").lowercased()
        case "str.upper": return text("value").uppercased()
        case "str.length": return text("value").unicodeScalars.count
        case "str.contains": return text("value").contains(text("needle"))
        case "str.startsWith": return text("value").hasPrefix(text("prefix"))
        case "str.endsWith": return text("value").hasSuffix(text("suffix"))
        case "str.replace": return text("value").replacingOccurrences(of: text("from"), with: text("to"))
        case "str.split": return text("value").components(separatedBy: text("delimiter"))
        case "str.join": return list("values").map(stdlibText).joined(separator: text("delimiter"))
        case "math.add": return finite(number("left"), number("right"), +)
        case "math.sub": return finite(number("left"), number("right"), -)
        case "math.mul": return finite(number("left"), number("right"), *)
        case "math.div":
            guard let right = number("right"), right != 0 else { return nil }
            return finite(number("left"), right, /)
        case "math.round": return number("value").map { Foundation.round($0) }
        case "math.floor": return number("value").map { Foundation.floor($0) }
        case "math.ceil": return number("value").map { Foundation.ceil($0) }
        case "math.abs": return number("value").map { Swift.abs($0) }
        case "math.sum": return list("values").compactMap(stdlibNumber).reduce(0, +)
        case "math.average":
            let values = list("values").compactMap(stdlibNumber)
            return values.isEmpty ? nil : values.reduce(0, +) / Double(values.count)
        case "math.min": return list("values").compactMap(stdlibNumber).min()
        case "math.max": return list("values").compactMap(stdlibNumber).max()
        case "parse.int": return Int(text("value").trimmingCharacters(in: .whitespacesAndNewlines)) ?? args["fallback"] ?? nil
        case "parse.float": return number("value") ?? args["fallback"] ?? nil
        case "parse.string": return stdlibText(args["value"] ?? nil)
        case "parse.json", "json.parse":
            guard let data = text("value").data(using: .utf8) else { return args["fallback"] ?? nil }
            return (try? JSONSerialization.jsonObject(with: data)) ?? args["fallback"] ?? nil
        case "sort.asc": return list("values").sorted { stdlibText($0) < stdlibText($1) }
        case "sort.desc": return list("values").sorted { stdlibText($0) > stdlibText($1) }
        case "sort.by": return list("values").sorted { stdlibText(read($0, path: text("field"))) < stdlibText(read($1, path: text("field"))) }
        case "list.take": return Array(list("values").prefix(max(0, Int(number("count") ?? 0))))
        case "list.skip": return Array(list("values").dropFirst(max(0, Int(number("count") ?? 0))))
        case "list.first": return list("values").first
        case "list.last": return list("values").last
        case "list.count": return list("values").count
        case "list.filterContains": return list("values").filter { stdlibText(read($0, path: text("field"))).lowercased().contains(text("value").lowercased()) }
        case "list.mapField": return list("values").map { read($0, path: text("field")) as Any }
        case "list.sumBy": return list("values").compactMap { stdlibNumber(read($0, path: text("field"))) }.reduce(0, +)
        case "json.get": return read(args["value"] ?? nil, path: text("path")) ?? args["fallback"] ?? nil
        case "json.stringify":
            guard JSONSerialization.isValidJSONObject(args["value"] as Any) else { return stdlibText(args["value"] ?? nil) }
            let data = try? JSONSerialization.data(withJSONObject: args["value"] as Any)
            return data.flatMap { String(data: $0, encoding: .utf8) } ?? ""
        case "json.merge":
            var output = args["left"] as? [String: Any] ?? [:]
            for (key, value) in args["right"] as? [String: Any] ?? [:] { output[key] = value }
            return output
        case "date.now": return ISO8601DateFormatter().string(from: Date())
        case "date.formatIso":
            let formatter = ISO8601DateFormatter()
            return formatter.date(from: text("value")).map { formatter.string(from: $0) } ?? text("value")
        case "date.addDays":
            let formatter = ISO8601DateFormatter()
            guard let date = formatter.date(from: text("value")) else { return nil }
            return formatter.string(from: date.addingTimeInterval((number("days") ?? 0) * 86400))
        case "date.diffDays":
            let formatter = ISO8601DateFormatter()
            guard let start = formatter.date(from: text("start")), let end = formatter.date(from: text("end")) else { return 0 }
            return Int(end.timeIntervalSince(start) / 86400)
        default: return nil
        }
    }

    private func stdlibValue(_ value: DoweStdlibValue, item: [String: Any]?) -> Any? {
        switch value.kind {
        case "null": return nil
        case "bool": return value.value as? Bool
        case "number": return stdlibNumber(value.value)
        case "string": return value.value as? String ?? ""
        case "reference": return self.value(value.value as? String ?? "", item: item)
        case "array": return (value.value as? [DoweStdlibValue] ?? []).map { stdlibValue($0, item: item) as Any }
        case "object":
            var output: [String: Any] = [:]
            for entry in value.value as? [(String, DoweStdlibValue)] ?? [] {
                output[entry.0] = stdlibValue(entry.1, item: item) as Any
            }
            return output
        default: return nil
        }
    }

    private func stdlibText(_ value: Any?) -> String {
        guard let value, !(value is NSNull) else { return "" }
        if let text = value as? String { return text }
        return String(describing: value)
    }

    private func stdlibNumber(_ value: Any?) -> Double? {
        if let number = value as? NSNumber { return number.doubleValue.isFinite ? number.doubleValue : nil }
        if let number = value as? Double { return number.isFinite ? number : nil }
        if let text = value as? String, let number = Double(text.trimmingCharacters(in: .whitespacesAndNewlines)), number.isFinite { return number }
        return nil
    }

    private func read(_ value: Any?, path: String) -> Any? {
        var current = value
        for part in path.split(separator: ".").map(String.init) {
            guard let object = current as? [String: Any] else { return nil }
            current = object[part]
        }
        return current
    }

    private func finite(_ left: Double?, _ right: Double?, _ op: (Double, Double) -> Double) -> Double? {
        guard let left, let right else { return nil }
        let value = op(left, right)
        return value.isFinite ? value : nil
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
