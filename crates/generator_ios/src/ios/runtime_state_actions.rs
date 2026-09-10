r#"    private func value(_ path: String, item: [String: Any]? = nil) -> Any? {
        if let derived = formValue(path, item: item) { return derived }
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

    func appendChatMessage(_ path: String, text: String) {
        var next = (value(path) as? [[String: Any]]) ?? []
        next.append(["id": "local-\(Int(Date().timeIntervalSince1970 * 1000))-\(next.count)", "role": "user", "text": text, "own": true, "status": "sent"])
        write(path, value: next)
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
        touchFormField(path)
        persistRoot(root)
    }

    private func touchFormField(_ path: String) {
        for (form, fields) in forms {
            if path.hasPrefix(form + ".") {
                let field = String(path.dropFirst(form.count + 1))
                if fields.contains(where: { $0.path == field }) { formTouched[path] = true }
            }
        }
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
        case "str.equals": return text("value") == text("other")
        case "str.startsWith": return text("value").hasPrefix(text("prefix"))
        case "str.endsWith": return text("value").hasSuffix(text("suffix"))
        case "str.replace": return text("value").replacingOccurrences(of: text("from"), with: text("to"))
        case "str.truncate": return String(text("value").prefix(max(0, Int(number("max") ?? 0))))
        case "str.split": return stdlibSplit(text("value"), delimiter: text("delimiter"), limit: number("limit"))
        case "str.join": return list("values").map(stdlibText).joined(separator: text("delimiter"))
        case "math.add": return finite(number("left"), number("right"), +)
        case "math.sub": return finite(number("left"), number("right"), -)
        case "math.mul": return finite(number("left"), number("right"), *)
        case "math.div":
            guard let right = number("right"), right != 0 else { return nil }
            return finite(number("left"), right, /)
        case "math.gt": return compare(number("left"), number("right"), >)
        case "math.gte": return compare(number("left"), number("right"), >=)
        case "math.lt": return compare(number("left"), number("right"), <)
        case "math.lte": return compare(number("left"), number("right"), <=)
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
        case "parse.bool": return stdlibBool(args["value"] ?? nil) ?? args["fallback"] ?? nil
        case "parse.string": return stdlibText(args["value"] ?? nil)
        case "parse.svg": return DoweSvgImporter.convert(text("value"), colors: text("colors").isEmpty ? "tokens" : text("colors"), format: text("format").isEmpty ? "source" : text("format")) ?? args["fallback"] ?? nil
        case "parse.json", "json.parse":
            guard let data = text("value").data(using: .utf8) else { return args["fallback"] ?? nil }
            return (try? JSONSerialization.jsonObject(with: data)) ?? args["fallback"] ?? nil
        case "url.encode": return stdlibUrlEncode(text("value"))
        case "url.decode": return text("value").removingPercentEncoding ?? args["fallback"] ?? nil
        case "url.parse": return stdlibUrlParse(text("value"))
        case "url.queryGet": return URLComponents(string: text("value"))?.queryItems?.first(where: { $0.name == text("name") })?.value
        case "url.querySet": return stdlibUrlQuerySet(text("value"), name: text("name"), param: args["param"] ?? nil)
        case "csv.parse": return stdlibCsvParse(text("value"), delimiter: text("delimiter").isEmpty ? "," : text("delimiter"), header: (args["header"] as? Bool) ?? false, maxRows: Int(number("maxRows") ?? 1000), maxColumns: Int(number("maxColumns") ?? 100))
        case "csv.stringify": return stdlibCsvStringify(list("rows"), delimiter: text("delimiter").isEmpty ? "," : text("delimiter"))
        case "sort.asc": return stdlibSorted(list("values"), field: nil, descending: false, nulls: text("nulls"))
        case "sort.desc": return stdlibSorted(list("values"), field: nil, descending: true, nulls: text("nulls"))
        case "sort.by": return stdlibSorted(list("values"), field: text("field"), descending: text("direction") == "desc", nulls: text("nulls"))
        case "list.take": return Array(list("values").prefix(max(0, Int(number("count") ?? 0))))
        case "list.skip": return Array(list("values").dropFirst(max(0, Int(number("count") ?? 0))))
        case "list.first": return list("values").first
        case "list.last": return list("values").last
        case "list.count": return list("values").count
        case "list.filterEquals": return list("values").filter { stdlibEqual(read($0, path: text("field")), args["value"] ?? nil) }
        case "list.filterContains": return list("values").filter { stdlibText(read($0, path: text("field"))).lowercased().contains(text("value").lowercased()) }
        case "list.filterContainsAny":
            let needles = list("needles").map(stdlibText).filter { !$0.isEmpty }.map { $0.lowercased() }
            return list("values").filter { value in
                let fieldText = stdlibText(read(value, path: text("field"))).lowercased()
                return needles.contains { fieldText.contains($0) }
            }
        case "list.concat": return list("values") + list("other")
        case "list.mapField": return list("values").map { read($0, path: text("field")) as Any }
        case "list.sumBy": return list("values").compactMap { stdlibNumber(read($0, path: text("field"))) }.reduce(0, +)
        case "list.averageBy":
            let values = list("values").compactMap { stdlibNumber(read($0, path: text("field"))) }
            return values.isEmpty ? nil : values.reduce(0, +) / Double(values.count)
        case "json.get": return read(args["value"] ?? nil, path: text("path")) ?? args["fallback"] ?? nil
        case "json.set": return stdlibJsonSet(args["value"] ?? nil, path: text("path"), next: args["next"] ?? nil)
        case "json.pick": return stdlibJsonPick(args["value"] ?? nil, fields: list("fields").map(stdlibText))
        case "json.omit": return stdlibJsonOmit(args["value"] ?? nil, fields: list("fields").map(stdlibText))
        case "json.stringify":
            let options: JSONSerialization.WritingOptions = (args["pretty"] as? Bool) == true ? [.prettyPrinted, .sortedKeys] : [.sortedKeys]
            guard JSONSerialization.isValidJSONObject(args["value"] as Any) else { return stdlibText(args["value"] ?? nil) }
            let data = try? JSONSerialization.data(withJSONObject: args["value"] as Any, options: options)
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

    private func stdlibSplit(_ value: String, delimiter: String, limit: Double?) -> [String] {
        let values = value.components(separatedBy: delimiter)
        guard let limit else { return values }
        return Array(values.prefix(max(0, Int(limit))))
    }

    private func stdlibBool(_ value: Any?) -> Bool? {
        if let value = value as? Bool { return value }
        switch stdlibText(value).trimmingCharacters(in: .whitespacesAndNewlines).lowercased() {
        case "true", "1", "yes", "y": return true
        case "false", "0", "no", "n": return false
        default: return nil
        }
    }

    private func stdlibUrlEncode(_ value: String) -> String {
        var allowed = CharacterSet.alphanumerics
        allowed.insert(charactersIn: "-._~")
        return value.addingPercentEncoding(withAllowedCharacters: allowed) ?? value
    }

    private func stdlibUrlParse(_ value: String) -> [String: Any] {
        guard let components = URLComponents(string: value) else {
            return ["ok": false, "scheme": NSNull(), "host": NSNull(), "path": NSNull(), "query": [:], "fragment": NSNull(), "origin": NSNull(), "isRelative": false, "error": "invalid_url"]
        }
        var query: [String: String] = [:]
        for item in components.queryItems ?? [] { query[item.name] = item.value ?? "" }
        let origin = components.scheme.flatMap { scheme in components.host.map { "\(scheme)://\($0)" } }
        return ["ok": true, "scheme": components.scheme as Any, "host": components.host as Any, "path": components.path, "query": query, "fragment": components.fragment as Any, "origin": origin as Any, "isRelative": components.scheme == nil, "error": NSNull()]
    }

    private func stdlibUrlQuerySet(_ value: String, name: String, param: Any?) -> String {
        guard var components = URLComponents(string: value) else { return value }
        var items = components.queryItems ?? []
        items.removeAll { $0.name == name }
        if let param { items.append(URLQueryItem(name: name, value: stdlibText(param))) }
        components.queryItems = items
        return components.string ?? value
    }

    private func stdlibCsvParse(_ value: String, delimiter: String, header: Bool, maxRows: Int, maxColumns: Int) -> [String: Any] {
        let separator = delimiter.first ?? ","
        let characters = Array(value)
        var parsed: [[String]] = []
        var row: [String] = []
        var cell = ""
        var quoted = false
        var index = 0
        var truncated = false
        func finishCell() {
            row.append(cell)
            cell = ""
        }
        func finishRow() {
            finishCell()
            if parsed.count < max(0, maxRows) { parsed.append(Array(row.prefix(max(0, maxColumns)))) } else { truncated = true }
            row = []
        }
        while index < characters.count {
            let character = characters[index]
            if character == "\"" && quoted && index + 1 < characters.count && characters[index + 1] == "\"" {
                cell.append("\"")
                index += 1
            } else if character == "\"" {
                quoted.toggle()
            } else if !quoted && character == separator {
                finishCell()
            } else if !quoted && (character == "\n" || character == "\r") {
                finishRow()
                if character == "\r" && index + 1 < characters.count && characters[index + 1] == "\n" { index += 1 }
            } else {
                cell.append(character)
            }
            index += 1
        }
        if !cell.isEmpty || !row.isEmpty { finishRow() }
        let columns = header && !parsed.isEmpty ? parsed[0] : Array(0..<(parsed.map(\.count).max() ?? 0)).map { "column\($0 + 1)" }
        var rows: [Any] = []
        for position in (header && !parsed.isEmpty ? 1 : 0)..<parsed.count {
            let values = parsed[position]
            if header {
                var object: [String: Any] = [:]
                for (column, key) in columns.enumerated() { object[key] = column < values.count ? values[column] : "" }
                rows.append(object)
            } else {
                rows.append(values)
            }
        }
        return ["rows": rows, "columns": columns, "errors": [], "truncated": truncated, "rowCount": rows.count]
    }

    private func stdlibCsvStringify(_ rows: [Any], delimiter: String) -> String {
        let separator = String(delimiter.first ?? ",")
        let columns = (rows.first as? [String: Any])?.keys.sorted() ?? []
        func escape(_ value: Any?) -> String {
            let text = stdlibText(value)
            return text.contains(separator) || text.contains("\"") || text.contains("\n") || text.contains("\r") ? "\"\(text.replacingOccurrences(of: "\"", with: "\"\""))\"" : text
        }
        return rows.map { row in
            if let object = row as? [String: Any] { return columns.map { escape(object[$0]) }.joined(separator: separator) }
            if let values = row as? [Any] { return values.map(escape).joined(separator: separator) }
            return escape(row)
        }.joined(separator: "\n")
    }

    private func stdlibSorted(_ values: [Any], field: String?, descending: Bool, nulls: String) -> [Any] {
        let indexed = values.enumerated().sorted { left, right in
            let leftValue = field.flatMap { read(left.element, path: $0) } ?? (field == nil ? left.element : nil)
            let rightValue = field.flatMap { read(right.element, path: $0) } ?? (field == nil ? right.element : nil)
            let leftNull = leftValue == nil || leftValue is NSNull
            let rightNull = rightValue == nil || rightValue is NSNull
            if leftNull || rightNull {
                if leftNull && rightNull { return left.offset < right.offset }
                return leftNull == (nulls != "first")
            }
            let leftText = stdlibText(leftValue)
            let rightText = stdlibText(rightValue)
            if leftText == rightText { return left.offset < right.offset }
            return descending ? leftText > rightText : leftText < rightText
        }
        return indexed.map(\.element)
    }

    private func stdlibJsonSet(_ value: Any?, path: String, next: Any?) -> Any? {
        let parts = path.split(separator: ".").map(String.init)
        guard !parts.isEmpty else { return next }
        func set(_ object: [String: Any], _ remaining: ArraySlice<String>) -> [String: Any] {
            var result = object
            guard let first = remaining.first else { return result }
            if remaining.count == 1 {
                result[first] = next ?? NSNull()
            } else {
                result[first] = set(object[first] as? [String: Any] ?? [:], remaining.dropFirst())
            }
            return result
        }
        return set(value as? [String: Any] ?? [:], parts[...])
    }

    private func stdlibJsonPick(_ value: Any?, fields: [String]) -> [String: Any] {
        let source = value as? [String: Any] ?? [:]
        return fields.reduce(into: [:]) { output, field in if let value = source[field] { output[field] = value } }
    }

    private func stdlibJsonOmit(_ value: Any?, fields: [String]) -> [String: Any] {
        let source = value as? [String: Any] ?? [:]
        return source.filter { !fields.contains($0.key) }
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

    private func stdlibEqual(_ left: Any?, _ right: Any?) -> Bool {
        if left == nil || left is NSNull || right == nil || right is NSNull {
            return (left == nil || left is NSNull) && (right == nil || right is NSNull)
        }
        if let left = left as? NSNumber, let right = right as? NSNumber { return left == right }
        return stdlibText(left) == stdlibText(right)
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

    private func compare(_ left: Double?, _ right: Double?, _ op: (Double, Double) -> Bool) -> Bool? {
        guard let left, let right else { return nil }
        return op(left, right)
    }

    private func finite(_ left: Double?, _ right: Double?, _ op: (Double, Double) -> Double) -> Double? {
        guard let left, let right else { return nil }
        let value = op(left, right)
        return value.isFinite ? value : nil
    }

    private func execute(_ action: DoweInvokeAction, item: [String: Any]?) async -> (Bool, Any?) {
        var args: [String: Any] = [:]
        for arg in action.args {
            args[arg.name] = stdlibValue(arg.value, item: item) ?? NSNull()
        }
        let response = await DoweNativeBridge.invoke(action.function, args: args)
        if response.0 {
            if let update = action.update { write(update, value: response.1 ?? NSNull()) }
            if let reset = action.reset, let current = value(reset, in: initial) { write(reset, value: current) }
            setAlert(action.successAlert, type: "success", message: action.successMessage ?? "Invocation completed")
        } else {
            setAlert(action.errorAlert, type: "error", message: action.errorMessage ?? "Invocation failed")
        }
        return response
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
