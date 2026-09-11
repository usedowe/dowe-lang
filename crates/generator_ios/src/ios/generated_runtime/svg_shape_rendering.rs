r#"    let data: String
    let viewBox: DoweSvgViewBox
    let pathTransform: CGAffineTransform?

    func path(in rect: CGRect) -> Path {
        let parsed = DoweSvgPathCache.shared.path(for: data)
        let scale = min(rect.width / viewBox.width, rect.height / viewBox.height)
        let transform = CGAffineTransform(
            a: scale,
            b: 0,
            c: 0,
            d: scale,
            tx: rect.midX - (viewBox.minX + viewBox.width / 2) * scale,
            ty: rect.midY - (viewBox.minY + viewBox.height / 2) * scale
        )
        return parsed.applying(pathTransform ?? .identity).applying(transform)
    }
}

private struct DoweRuntimeSvgRecord {
    let viewBox: DoweSvgViewBox
    let paths: [DoweSvgPathData]
}

private enum DoweRuntimeSvgParser {
    private static let allowedPathCharacters = CharacterSet(charactersIn: "MmZzLlHhVvCcSsQqTtAa0123456789eE.,+- \t\r\n")

    static func parse(_ payload: String) -> DoweRuntimeSvgRecord? {
        guard !payload.isEmpty,
              payload.utf8.count <= 131072,
              let data = payload.data(using: .utf8),
              let source = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let viewBoxSource = source["viewBox"] as? String else {
            return nil
        }
        let values = numbers(viewBoxSource)
        guard values.count == 4,
              values.allSatisfy(\.isFinite),
              values[2] > 0,
              values[3] > 0,
              let sourcePaths = source["paths"] as? [[String: Any]],
              (1...64).contains(sourcePaths.count) else {
            return nil
        }
        var paths: [DoweSvgPathData] = []
        for sourcePath in sourcePaths {
            guard let path = path(sourcePath) else {
                return nil
            }
            paths.append(path)
        }
        return DoweRuntimeSvgRecord(
            viewBox: DoweSvgViewBox(
                minX: CGFloat(values[0]),
                minY: CGFloat(values[1]),
                width: CGFloat(values[2]),
                height: CGFloat(values[3])
            ),
            paths: paths
        )
    }

    private static func path(_ source: [String: Any]) -> DoweSvgPathData? {
        guard let data = source["d"] as? String,
              !data.isEmpty,
              data.utf8.count <= 32768,
              data.rangeOfCharacter(from: allowedPathCharacters.inverted) == nil else {
            return nil
        }
        let paint = source["paint"] as? String ?? "currentColor"
        guard ["fill", "stroke", "none", "currentColor"].contains(paint) else {
            return nil
        }
        let colorSource = source["color"] as? String ?? "currentColor"
        guard colorSource == "currentColor" || hexColor(colorSource) != nil,
              let opacity = integer(source["opacity"], fallback: 255, range: 0...255),
              let width = integer(source["width"], fallback: 100, range: 1...10000) else {
            return nil
        }
        let cap = source["lineCap"] as? String ?? "butt"
        let join = source["lineJoin"] as? String ?? "miter"
        guard ["butt", "round", "square"].contains(cap),
              ["miter", "round", "bevel"].contains(join) else {
            return nil
        }
        let transform: CGAffineTransform?
        if let sourceTransform = source["transform"] {
            guard let value = sourceTransform as? String,
                  let resolved = matrix(value) else {
                return nil
            }
            transform = resolved
        } else {
            transform = nil
        }
        let color = colorSource == "currentColor" ? nil : hexColor(colorSource)
        let fill: DoweSvgFill
        switch paint {
        case "none":
            fill = .none
        case "currentColor":
            fill = .currentColor
        case "stroke":
            fill = .stroke(color, Double(opacity) / 255, CGFloat(width) / 100, cap, join)
        default:
            let evenOdd = source["evenOdd"] as? Bool ?? false
            fill = .fill(color, Double(opacity) / 255, evenOdd)
        }
        return DoweSvgPathData(data: data, fill: fill, transform: transform)
    }

    private static func integer(_ source: Any?, fallback: Int, range: ClosedRange<Int>) -> Int? {
        guard let source else {
            return fallback
        }
        guard !(source is Bool), let number = source as? NSNumber else {
            return nil
        }
        let value = number.doubleValue
        let integer = number.intValue
        return value == Double(integer) && range.contains(integer) ? integer : nil
    }

    private static func numbers(_ source: String) -> [Double] {
        source.components(separatedBy: CharacterSet(charactersIn: " ,\t\r\n"))
            .filter { !$0.isEmpty }
            .compactMap(Double.init)
    }

    private static func matrix(_ source: String) -> CGAffineTransform? {
        guard source.hasPrefix("matrix("), source.hasSuffix(")") else {
            return nil
        }
        let values = numbers(String(source.dropFirst(7).dropLast()))
        guard values.count == 6, values.allSatisfy(\.isFinite) else {
            return nil
        }
        return CGAffineTransform(
            a: CGFloat(values[0]),
            b: CGFloat(values[1]),
            c: CGFloat(values[2]),
            d: CGFloat(values[3]),
            tx: CGFloat(values[4]),
            ty: CGFloat(values[5])
        )
    }

    private static func hexColor(_ source: String) -> Color? {
        guard source.first?.asciiValue == 35 else {
            return nil
        }
        var hex = String(source.dropFirst())
        if hex.count == 3 {
            hex = hex.map { "\($0)\($0)" }.joined()
        }
        guard hex.count == 6 || hex.count == 8,
              let value = UInt64(hex, radix: 16) else {
            return nil
        }
        let red = Double((value >> (hex.count == 8 ? 24 : 16)) & 0xff) / 255
        let green = Double((value >> (hex.count == 8 ? 16 : 8)) & 0xff) / 255
        let blue = Double((value >> (hex.count == 8 ? 8 : 0)) & 0xff) / 255
        let alpha = hex.count == 8 ? Double(value & 0xff) / 255 : 1
        return Color(.sRGB, red: red, green: green, blue: blue, opacity: alpha)
    }
}

struct DoweRuntimeSvgView: View {
"#
