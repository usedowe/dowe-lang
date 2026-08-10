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
    let headers: [(String, String, String)]
    let body: String?
    let update: String?
    let reset: String?
    let successAlert: String?
    let successMessage: String?
    let errorAlert: String?
    let errorMessage: String?
}

struct DoweActionMetadata {
    let params: [String: String]
    let returnType: String?
}

enum DoweAction {
    case request(DoweRequestAction, DoweActionMetadata)
    case assign(String, String, DoweStdlibCall?, DoweActionMetadata)
    case reset(String, DoweActionMetadata)
    case sequence([DoweStep], DoweActionMetadata)
}

enum DoweStep {
    case request(String, DoweRequestAction)
    case branch(String, [DoweStep], [DoweStep])
    case assign(String, String, Any?, Bool, DoweStdlibCall?)
    case reset(String)
    case toast(String, String, String, Int?, String?, String?, String?)
    case redirect(String)
}

struct DoweToastState: Hashable {
    let id: Int
    let kind: String
    let title: String
    let message: String
    let duration: Int
    let scheme: String
    let variant: String
    let position: String
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

struct DoweSignalMetadata {
    let name: String
    let scope: String
    let storage: String
}

private struct DoweSvgImportMatrix {
    let a: Double
    let b: Double
    let c: Double
    let d: Double
    let e: Double
    let f: Double

    func multiplied(by next: DoweSvgImportMatrix) -> DoweSvgImportMatrix {
        DoweSvgImportMatrix(
            a: a * next.a + c * next.b,
            b: b * next.a + d * next.b,
            c: a * next.c + c * next.d,
            d: b * next.c + d * next.d,
            e: a * next.e + c * next.f + e,
            f: b * next.e + d * next.f + f
        )
    }
}

private struct DoweSvgImportContext {
    let matrix: DoweSvgImportMatrix
    let fill: String?
    let evenOdd: Bool
    let hidden: Bool
}

private struct DoweSvgImportedPath {
    let data: String
    let fill: String
    let evenOdd: Bool
    let transform: String?
}

private final class DoweSvgImporter: NSObject, XMLParserDelegate {
    private let identity = DoweSvgImportMatrix(a: 1, b: 0, c: 0, d: 1, e: 0, f: 0)
    private let tokens = ["primary", "secondary", "tertiary", "muted", "success", "info", "warning", "danger"]
    private var stack: [DoweSvgImportContext] = []
    private var colors: [String] = []
    private var paths: [DoweSvgImportedPath] = []
    private var viewBox: String?
    private var valid = true
    private var originalColors = false

    static func convert(_ source: String, colors: String = "tokens", format: String = "source") -> String? {
        guard source.utf8.count <= 262_144,
              ["tokens", "original"].contains(colors),
              ["source", "data"].contains(format),
              format != "data" || colors == "original",
              !source.localizedCaseInsensitiveContains("<!entity"),
              let data = source.data(using: .utf8) else { return nil }
        let importer = DoweSvgImporter()
        importer.originalColors = colors == "original"
        importer.stack = [DoweSvgImportContext(matrix: importer.identity, fill: nil, evenOdd: false, hidden: false)]
        let parser = XMLParser(data: data)
        parser.shouldResolveExternalEntities = false
        parser.delegate = importer
        guard parser.parse(), importer.valid, let viewBox = importer.viewBox, !importer.paths.isEmpty else { return nil }
        if format == "data" {
            let paths = importer.paths.map { path -> [String: Any] in
                var value: [String: Any] = [
                    "d": path.data,
                    "paint": path.fill == "none" ? "none" : path.fill == "currentColor" ? "currentColor" : "fill"
                ]
                if path.fill != "none" && path.fill != "currentColor" { value["color"] = path.fill }
                if path.evenOdd { value["evenOdd"] = true }
                if let transform = path.transform { value["transform"] = transform }
                return value
            }
            guard let data = try? JSONSerialization.data(withJSONObject: ["viewBox": viewBox, "paths": paths]) else { return nil }
            return String(data: data, encoding: .utf8)
        }
        return "Svg viewBox:\"" + viewBox + "\" w:\"full\" h:\"full\"\n" + importer.paths.map { path in
            "  Path d:\"" + path.data + "\" fill:\"" + path.fill + "\"" + (path.evenOdd ? " fillRule:\"evenodd\"" : "") + (path.transform.map { " transform:\"" + $0 + "\"" } ?? "")
        }.joined(separator: "\n")
    }

    func parser(_ parser: XMLParser, didStartElement elementName: String, namespaceURI: String?, qualifiedName qName: String?, attributes attributeDict: [String: String] = [:]) {
        guard valid, let parent = stack.last else { return }
        let name = elementName.lowercased()
        let attrs = Dictionary(uniqueKeysWithValues: attributeDict.map { ($0.key.lowercased(), $0.value) })
        let local: DoweSvgImportMatrix
        if let transform = attrs["transform"] {
            guard let parsed = matrix(transform) else {
                valid = false
                return
            }
            local = parsed
        } else {
            local = identity
        }
        let combined = parent.matrix.multiplied(by: local)
        let styleFill = attrs["style"]?.split(separator: ";").compactMap { entry -> String? in
            let pair = entry.split(separator: ":", maxSplits: 1).map(String.init)
            return pair.count == 2 && pair[0].trimmingCharacters(in: .whitespaces).lowercased() == "fill"
                ? pair[1].trimmingCharacters(in: .whitespaces)
                : nil
        }.first
        let fill = attrs["fill"] ?? styleFill ?? parent.fill
        let styleFillRule = attrs["style"]?.split(separator: ";").compactMap { entry -> String? in
            let pair = entry.split(separator: ":", maxSplits: 1).map(String.init)
            return pair.count == 2 && pair[0].trimmingCharacters(in: .whitespaces).lowercased() == "fill-rule"
                ? pair[1].trimmingCharacters(in: .whitespaces)
                : nil
        }.first
        let fillRule = attrs["fill-rule"] ?? styleFillRule
        let evenOdd: Bool
        if let fillRule {
            switch fillRule.trimmingCharacters(in: .whitespacesAndNewlines).lowercased() {
            case "nonzero": evenOdd = false
            case "evenodd": evenOdd = true
            default:
                valid = false
                return
            }
        } else {
            evenOdd = parent.evenOdd
        }
        let hidden = parent.hidden || ["defs", "clippath", "mask", "symbol", "script", "style"].contains(name)
        if name == "svg" && viewBox == nil {
            let raw: String
            if let sourceViewBox = attrs["viewbox"] {
                raw = sourceViewBox
            } else if let width = dimension(attrs["width"]), let height = dimension(attrs["height"]) {
                raw = "0 0 " + width + " " + height
            } else {
                valid = false
                return
            }
            let values = numbers(raw)
            guard values.count == 4, values.allSatisfy({ $0.isFinite }), values[2] > 0, values[3] > 0 else {
                valid = false
                return
            }
            viewBox = values.map(number).joined(separator: " ")
        }
        let drawable = name == "path" || (name == "rect" && attrs["rx"] == nil && attrs["ry"] == nil)
        if drawable && !hidden {
            let data = name == "path"
                ? attrs["d"]?.trimmingCharacters(in: .whitespacesAndNewlines)
                : rectangle(attrs)
            guard paths.count < 1_024,
                  let data,
                  !data.isEmpty,
                  data.range(of: "^[0-9\\sMmZzLlHhVvCcSsQqTtAa+.,eE-]+$", options: .regularExpression) != nil else {
                valid = false
                return
            }
            let transform = same(combined, identity) ? nil : matrixSource(combined)
            guard let portableFill = originalColors ? originalFill(fill) : portableFill(fill) else {
                valid = false
                return
            }
            paths.append(DoweSvgImportedPath(data: data, fill: portableFill, evenOdd: evenOdd, transform: transform))
        }
        stack.append(DoweSvgImportContext(matrix: combined, fill: fill, evenOdd: evenOdd, hidden: hidden))
    }

    func parser(_ parser: XMLParser, didEndElement elementName: String, namespaceURI: String?, qualifiedName qName: String?) {
        if stack.count > 1 { stack.removeLast() }
    }

    func parser(_ parser: XMLParser, resolveExternalEntityName name: String, systemID: String?) -> Data? {
        nil
    }

    private func matrix(_ source: String) -> DoweSvgImportMatrix? {
        var rest = source.trimmingCharacters(in: .whitespacesAndNewlines)
        var output = identity
        while !rest.isEmpty {
            guard rest.hasPrefix("matrix"),
                  let open = rest.firstIndex(of: "("),
                  let close = rest[open...].firstIndex(of: ")") else { return nil }
            let values = numbers(String(rest[rest.index(after: open)..<close]))
            guard values.count == 6, values.allSatisfy({ $0.isFinite }) else { return nil }
            output = output.multiplied(by: DoweSvgImportMatrix(a: values[0], b: values[1], c: values[2], d: values[3], e: values[4], f: values[5]))
            rest = String(rest[rest.index(after: close)...]).trimmingCharacters(in: .whitespacesAndNewlines)
        }
        return output
    }

    private func numbers(_ source: String) -> [Double] {
        source.split { $0.isWhitespace || $0 == "," }.compactMap { Double($0) }
    }

    private func dimension(_ source: String?) -> String? {
        guard let source else { return nil }
        let text = source.trimmingCharacters(in: .whitespacesAndNewlines).replacingOccurrences(of: "px", with: "", options: [.caseInsensitive, .anchored, .backwards])
        guard let value = Double(text), value.isFinite, value > 0 else { return nil }
        return number(value)
    }

    private func rectangle(_ attrs: [String: String]) -> String? {
        let x = attrs["x"].flatMap { Double($0.trimmingCharacters(in: .whitespacesAndNewlines)) } ?? 0
        let y = attrs["y"].flatMap { Double($0.trimmingCharacters(in: .whitespacesAndNewlines)) } ?? 0
        guard let width = attrs["width"].flatMap({ Double($0.trimmingCharacters(in: .whitespacesAndNewlines)) }),
              let height = attrs["height"].flatMap({ Double($0.trimmingCharacters(in: .whitespacesAndNewlines)) }),
              x.isFinite,
              y.isFinite,
              width.isFinite,
              height.isFinite,
              width > 0,
              height > 0 else { return nil }
        let right = x + width
        let bottom = y + height
        guard right.isFinite, bottom.isFinite else { return nil }
        return "M" + number(x) + " " + number(y) + "H" + number(right) + "V" + number(bottom) + "H" + number(x) + "Z"
    }

    private func portableFill(_ source: String?) -> String? {
        let value = source?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if value.lowercased() == "none" { return "none" }
        if value.isEmpty || value.lowercased() == "currentcolor" { return "currentColor" }
        let key = value.lowercased()
        let index: Int
        if let existing = colors.firstIndex(where: { sameColor($0, key) }) {
            index = existing
        } else {
            colors.append(key)
            index = colors.count - 1
        }
        return tokens[index % tokens.count]
    }

    private func originalFill(_ source: String?) -> String? {
        let value = source?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if value.lowercased() == "none" { return "none" }
        if value.isEmpty || value.lowercased() == "currentcolor" { return "currentColor" }
        let normalized = value.lowercased()
        if normalized.range(of: "^#[0-9a-f]{3,4}$|^#[0-9a-f]{6}([0-9a-f]{2})?$", options: .regularExpression) != nil {
            return normalized
        }
        guard let channels = rgb(normalized) else { return nil }
        return String(UnicodeScalar(35)!) + String(format: "%02x%02x%02x", channels[0], channels[1], channels[2])
    }

    private func sameColor(_ left: String, _ right: String) -> Bool {
        if left == right { return true }
        guard let leftChannels = rgb(left), let rightChannels = rgb(right) else { return false }
        return zip(leftChannels, rightChannels).allSatisfy { abs($0 - $1) <= 1 }
    }

    private func rgb(_ source: String) -> [Int]? {
        let value = source.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        guard value.hasPrefix("rgb("), value.hasSuffix(")") else { return nil }
        let body = value.dropFirst(4).dropLast()
        let channels = body.split(separator: ",").compactMap { part in
            Int(part.trimmingCharacters(in: .whitespacesAndNewlines))
        }
        guard channels.count == 3, channels.allSatisfy({ 0...255 ~= $0 }) else { return nil }
        return channels
    }

    private func matrixSource(_ value: DoweSvgImportMatrix) -> String {
        "matrix(" + [value.a, value.b, value.c, value.d, value.e, value.f].map(number).joined(separator: " ") + ")"
    }

    private func same(_ left: DoweSvgImportMatrix, _ right: DoweSvgImportMatrix) -> Bool {
        [left.a - right.a, left.b - right.b, left.c - right.c, left.d - right.d, left.e - right.e, left.f - right.f].allSatisfy { abs($0) < 0.000_000_1 }
    }

    private func number(_ value: Double) -> String {
        if abs(value) < 0.000_000_1 { return "0" }
        var output = String(format: "%.6f", value)
        while output.contains(".") && output.last == "0" { output.removeLast() }
        if output.last == "." { output.removeLast() }
        return output
    }
}

@MainActor
final class DoweReactiveState: ObservableObject {
    private static var globalValues: [String: Any] = [:]
    private static var globalStorage: [String: String] = [:]
    @Published private var values: [String: Any]
    @Published private(set) var toast: DoweToastState? = nil
    @Published private(set) var redirectPath: String? = nil
    private var toastSequence = 0
    private let constants: [String: Any]
    private let initial: [String: Any]
    private let signals: [String: DoweSignalMetadata]
    private let actions: [String: DoweAction]
    private var loaded = Set<String>()

    init(constants: [String: Any], initial: [String: Any], signals: [String: DoweSignalMetadata], actions: [String: DoweAction]) {
        self.constants = constants
        self.initial = initial
        self.signals = signals
        self.actions = actions
        var hydrated = initial
        for (id, metadata) in signals where metadata.scope == "global" {
            Self.globalStorage[metadata.name] = metadata.storage
            if Self.globalValues[metadata.name] == nil {
                let fallback = initial[id] ?? NSNull()
                if let stored = Self.storedSignal(metadata), Self.compatibleSignalValue(stored, fallback) {
                    Self.globalValues[metadata.name] = stored
                } else {
                    Self.globalValues[metadata.name] = fallback
                }
            }
            hydrated[id] = Self.globalValues[metadata.name] ?? NSNull()
        }
        self.values = hydrated
    }

    private static func compatibleSignalValue(_ value: Any, _ initial: Any) -> Bool {
        if initial is NSNull {
            return value is NSNull
        }
        if let expected = initial as? [String: Any] {
            guard let actual = value as? [String: Any] else {
                return false
            }
            return expected.allSatisfy { key, expectedValue in
                guard let actualValue = actual[key] else {
                    return false
                }
                return compatibleSignalValue(actualValue, expectedValue)
            }
        }
        if let expected = initial as? [Any] {
            guard let actual = value as? [Any] else {
                return false
            }
            if expected.isEmpty {
                return true
            }
            return actual.allSatisfy { value in
                expected.contains { candidate in compatibleSignalValue(value, candidate) }
            }
        }
        if initial is Bool {
            return value is Bool
        }
        if initial is NSNumber {
            return value is NSNumber && !(value is Bool)
        }
        if initial is String {
            return value is String
        }
        return false
    }

    private static func storageKey(_ name: String) -> String {
        "dowe:signal:" + name
    }

    private static func storedSignal(_ metadata: DoweSignalMetadata) -> Any? {
        guard metadata.storage == "local",
              let data = UserDefaults.standard.data(forKey: Self.storageKey(metadata.name)) else {
            return nil
        }
        return (try? JSONSerialization.jsonObject(with: data) as? [String: Any])?["value"]
    }

    private func persistRoot(_ root: String) {
        guard let metadata = signals[root], metadata.scope == "global" else {
            return
        }
        let value = values[root] ?? NSNull()
        Self.globalValues[metadata.name] = value
        guard metadata.storage == "local", JSONSerialization.isValidJSONObject(["value": value]) else {
            return
        }
        if let data = try? JSONSerialization.data(withJSONObject: ["value": value]) {
            UserDefaults.standard.set(data, forKey: Self.storageKey(metadata.name))
        }
    }

    func text(_ path: String, item: [String: Any]? = nil) -> String {
        guard let current = value(path, item: item), !(current is NSNull) else {
            return ""
        }
        return String(describing: current)
    }

    func json(_ path: String, item: [String: Any]? = nil) -> String {
        guard let current = value(path, item: item), !(current is NSNull) else {
            return ""
        }
        if let current = current as? String {
            return current
        }
        guard JSONSerialization.isValidJSONObject(current),
              let data = try? JSONSerialization.data(withJSONObject: current),
              let text = String(data: data, encoding: .utf8) else {
            return ""
        }
        return text
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
        var pending: [String] = []
        for id in actionIds where !loaded.contains(id) {
            loaded.insert(id)
            pending.append(id)
        }
        Task {
            for id in pending {
                await runAction(id)
            }
        }
    }

    func run(_ id: String, item: [String: Any]? = nil) {
        Task {
            await runAction(id, item: item)
        }
    }

    private func runAction(_ id: String, item: [String: Any]? = nil) async {
        guard let action = actions[id] else {
            return
        }
        switch action {
        case .assign(let target, let source, let call, _):
            let current = call.map { stdlib($0, item: item) } ?? (source == "$dowe:bool:true" ? true : source == "$dowe:bool:false" ? false : source.hasPrefix("$dowe:string:") ? String(source.dropFirst(13)) : source.hasPrefix("!") ? !(value(String(source.dropFirst()), item: item) as? Bool ?? false) : value(source, item: item))
            write(target, value: current ?? NSNull())
        case .reset(let target, _):
            if let current = value(target, in: initial) {
                write(target, value: current)
            }
        case .request(let request, _):
            _ = await execute(request, item: item)
        case .sequence(let steps, _):
            _ = await runSteps(steps, item: item, results: [:])
        }
    }

    private func runSteps(_ steps: [DoweStep], item: [String: Any]?, results: [String: Any]) async -> Bool {
        var results = results
        for step in steps {
            switch step {
            case .request(let result, let action):
                let response = await execute(action, item: item)
                results[result] = ["ok": response.0, "data": response.1 ?? NSNull()]
            case .branch(let result, let success, let error):
                let ok = stepValue(result + ".ok", item: item, results: results) as? Bool ?? false
                if await runSteps(ok ? success : error, item: item, results: results) { return true }
            case .assign(let target, let source, let literal, let hasLiteral, let call):
                let current = hasLiteral ? literal : call.map { stdlib($0, item: item) } ?? stepValue(source, item: item, results: results)
                write(target, value: current ?? NSNull())
            case .reset(let target):
                if let current = value(target, in: initial) {
                    write(target, value: current)
                }
            case .toast(let kind, let title, let message, let duration, let scheme, let variant, let position):
                showToast(kind: kind, title: title, message: message, duration: duration, scheme: scheme, variant: variant, position: position)
            case .redirect(let path):
                redirectPath = path
                return true
            }
        }
        return false
    }

    private func stepValue(_ source: String, item: [String: Any]?, results: [String: Any]) -> Any? {
        if source == "$dowe:bool:true" { return true }
        if source == "$dowe:bool:false" { return false }
        if source.hasPrefix("$dowe:string:") { return String(source.dropFirst(13)) }
        if source.hasPrefix("!") {
            return !(stepValue(String(source.dropFirst()), item: item, results: results) as? Bool ?? false)
        }
        return value(source, in: results) ?? value(source, item: item)
    }

    private func showToast(kind: String, title: String, message: String, duration: Int?, scheme: String?, variant: String?, position: String?) {
        toastSequence += 1
        toast = DoweToastState(
            id: toastSequence,
            kind: kind,
            title: title,
            message: message,
            duration: max(500, duration ?? 4000),
            scheme: scheme ?? (kind == "error" ? "danger" : kind),
            variant: variant ?? "solid",
            position: position ?? "top-right"
        )
    }

    func closeToast() {
        toast = nil
    }

    func consumeRedirect() {
        redirectPath = nil
    }

    func canvasValue(_ path: String) -> Any? {
        if let current = value(path) { return current }
        let parts = path.split(separator: ".").map(String.init)
        guard let root = parts.first, let id = signals.first(where: { $0.value.name == root })?.key else { return nil }
        return value(([id] + parts.dropFirst()).joined(separator: "."))
    }

    func text(_ path: String, fallback: String) -> String {
        guard let current = value(path) else { return fallback }
        let text = String(describing: current)
        return text.isEmpty ? fallback : text
    }

    func bool(_ path: String, fallback: Bool) -> Bool {
        value(path) as? Bool ?? fallback
    }

    private func value(_ path: String, item: [String: Any]? = nil) -> Any? {
        if path == "item", let item {
            return item
        }
        if path.hasPrefix("item."), let item {
            return value(String(path.dropFirst(5)), in: item)
        }
        return value(path, in: values) ?? value(path, in: constants)
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
            persistRoot(root)
            return
        }
        var object = values[root] as? [String: Any] ?? [:]
        object[parts[1]] = value
        values[root] = object
        persistRoot(root)
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
        case "parse.svg": return DoweSvgImporter.convert(text("value"), colors: text("colors").isEmpty ? "tokens" : text("colors"), format: text("format").isEmpty ? "source" : text("format")) ?? args["fallback"] ?? nil
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

    private func execute(_ action: DoweRequestAction, item: [String: Any]?) async -> (Bool, Any?) {
        let body = action.body.flatMap { value($0, item: item) }
        let path = filledPath(action.path, body: body, item: item)
        let base = action.base.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        let address = base.isEmpty ? path : base + (path.hasPrefix("/") ? path : "/" + path)
        guard let url = URL(string: address), url.scheme != nil else {
            setAlert(action.errorAlert, type: "error", message: action.errorMessage ?? "Request failed")
            return (false, nil)
        }
        var request = URLRequest(url: url)
        request.httpMethod = action.method
        for header in action.headers {
            let rawValue = header.1 == "signal" ? text(header.2, item: item) : header.2
            if !rawValue.isEmpty {
                request.setValue(rawValue, forHTTPHeaderField: header.0)
            }
        }
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
            return (true, payload["data"] ?? payload)
        } catch {
            setAlert(action.errorAlert, type: "error", message: action.errorMessage ?? "Request failed")
            return (false, nil)
        }
    }

    private func filledPath(_ path: String, body: Any?, item: [String: Any]?) -> String {
        guard let pattern = try? NSRegularExpression(pattern: ":[A-Za-z_][A-Za-z0-9_]*") else {
            return path
        }
        var output = path
        let range = NSRange(path.startIndex..<path.endIndex, in: path)
        let matches = pattern.matches(in: path, range: range)
        var allowed = CharacterSet.urlPathAllowed
        allowed.remove(charactersIn: "/")
        for match in matches.reversed() {
            guard let sourceRange = Range(match.range, in: output) else { continue }
            let name = String(output[sourceRange].dropFirst())
            let fromBody = (body as? [String: Any])?[name]
            let signal = signals.reversed().first(where: { $0.value.name == name })?.key
            let current = fromBody ?? signal.flatMap { value($0, item: item) } ?? value(name, item: item)
            let text = current.flatMap { $0 is NSNull ? nil : String(describing: $0) } ?? ""
            output.replaceSubrange(sourceRange, with: text.addingPercentEncoding(withAllowedCharacters: allowed) ?? "")
        }
        return output
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
