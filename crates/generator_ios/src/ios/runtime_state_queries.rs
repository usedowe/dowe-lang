r#"    func text(_ path: String, item: [String: Any]? = nil) -> String {
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

    func treeNodes(_ path: String) -> [DoweTreeNode] {
        treeChildren(value(path)).compactMap(treeNode)
    }

    private func treeChildren(_ value: Any?) -> [Any] {
        if let list = value as? [Any] {
            return list
        }
        guard let map = value as? [String: Any] else {
            return []
        }
        if let children = map["children"] as? [Any] {
            return children
        }
        return (map["folders"] as? [Any] ?? []) + (map["files"] as? [Any] ?? [])
    }

    private func treeNode(_ value: Any) -> DoweTreeNode? {
        guard let map = value as? [String: Any] else {
            return nil
        }
        let rawId = map["id"] ?? map["path"] ?? map["name"] ?? map["label"]
        guard let rawId else {
            return nil
        }
        let id = String(describing: rawId)
        guard !id.isEmpty else {
            return nil
        }
        guard let rawLabel = map["name"] ?? map["label"] else {
            return nil
        }
        let label = String(describing: rawLabel)
        guard !label.isEmpty else {
            return nil
        }
        let path = String(describing: map["path"] ?? id)
        let children = treeChildren(value).compactMap(treeNode)
        let kind = String(describing: map["type"] ?? map["kind"] ?? "").lowercased()
        let explicitChildren = map["children"] is [Any] || map["folders"] is [Any] || map["files"] is [Any]
        let branch = !children.isEmpty || explicitChildren || ["folder", "directory", "branch"].contains(kind)
        return DoweTreeNode(id: id, label: label, path: path, branch: branch, children: children, value: map)
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
        case .invoke(let invoke, _):
            _ = await execute(invoke, item: item)
        case .sequence(let steps, _):
            _ = await runSteps(steps, item: item, results: [:])
        }
    }

    private func runSteps(_ steps: [DoweStep], item: [String: Any]?, results: [String: Any]) async -> Bool {
        var results = results
        for step in steps {
            switch step {
            case .validate(let target):
                if !validateForm(target, item: item) { return true }
            case .request(let result, let action):
                let response = await execute(action, item: item)
                results[result] = ["ok": response.0, "data": response.1 ?? NSNull()]
            case .invoke(let result, let action):
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

    private func formError(_ form: String, _ field: DoweFormFieldMetadata, item: [String: Any]?) -> String? {
        let current = value(form + "." + field.path, item: item)
        let rules = field.rules.map { rule in
            guard rule.kind == "matches", let argument = rule.argument else { return rule }
            return DoweValidationRule(kind: rule.kind, argument: text(argument, item: item), message: rule.message)
        }
        if field.kind == "boolean" {
            return doweBooleanValidationError((current as? Bool) ?? false, rules: rules)
        }
        return doweValidationError(String(describing: current ?? ""), rules: rules)
    }

    private func formValue(_ path: String, item: [String: Any]?) -> Any? {
        let parts = path.split(separator: ".").map(String.init)
        guard parts.count >= 2, let fields = forms[parts[0]] else { return nil }
        let form = parts[0]
        if parts[1] == "isValid" && parts.count == 2 { return fields.allSatisfy { formError(form, $0, item: item) == nil } }
        if parts[1] == "isInvalid" && parts.count == 2 { return fields.contains { formError(form, $0, item: item) != nil } }
        if parts[1] == "errors" {
            var errors: [String: Any] = [:]
            for field in fields { if let error = formError(form, field, item: item) { errors[field.path] = error } }
            return parts.count == 2 ? errors : errors[parts.dropFirst(2).joined(separator: ".")]
        }
        if parts[1] == "touched" {
            if parts.count == 2 { return Dictionary(uniqueKeysWithValues: fields.map { ($0.path, formTouched[form + "." + $0.path] ?? false) }) }
            return formTouched[form + "." + parts.dropFirst(2).joined(separator: ".")] ?? false
        }
        return nil
    }

    private func validateForm(_ target: String, item: [String: Any]?) -> Bool {
        guard let fields = forms[target] else { return true }
        for field in fields { formTouched[target + "." + field.path] = true }
        return (formValue(target + ".isValid", item: item) as? Bool) ?? true
    }

"#
