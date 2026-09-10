r#"private struct DoweSvgImportMatrix {
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
    private let tokens = ["primary", "secondary", "accent", "muted", "success", "info", "warning", "danger"]
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

"#
