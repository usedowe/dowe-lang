fn swift_runtime_svg_runtime() -> &'static str {
    r#"struct DoweSvgViewBox {
    let minX: CGFloat
    let minY: CGFloat
    let width: CGFloat
    let height: CGFloat
}

enum DoweSvgFill {
    case none
    case currentColor
    case color(Color)
    case fill(Color?, Double, Bool)
    case stroke(Color?, Double, CGFloat, String, String)

    func resolved(_ current: Color) -> Color? {
        switch self {
        case .none:
            return nil
        case .currentColor:
            return current
        case .color(let color):
            return color
        case .fill(let color, let opacity, _):
            return (color ?? current).opacity(opacity)
        case .stroke(let color, let opacity, _, _, _):
            return (color ?? current).opacity(opacity)
        }
    }
}

struct DoweSvgPathData {
    let data: String
    let fill: DoweSvgFill
    let transform: CGAffineTransform?

    init(data: String, fill: DoweSvgFill, transform: CGAffineTransform? = nil) {
        self.data = data
        self.fill = fill
        self.transform = transform
    }
}

private final class DoweSvgCachedPath {
    let value: Path

    init(_ value: Path) {
        self.value = value
    }
}

private final class DoweSvgPathCache: @unchecked Sendable {
    static let shared = DoweSvgPathCache()
    private let storage = NSCache<NSString, DoweSvgCachedPath>()

    private init() {
        storage.countLimit = 2048
    }

    func path(for data: String) -> Path {
        let key = data as NSString
        if let cached = storage.object(forKey: key) {
            return cached.value
        }
        var parser = DoweSvgPathParser(data)
        let value = parser.parse()
        storage.setObject(DoweSvgCachedPath(value), forKey: key)
        return value
    }
}

struct DoweSvgShape: Shape {
    let data: String
    let viewBox: DoweSvgViewBox
    let pathTransform: CGAffineTransform?

    func path(in rect: CGRect) -> Path {
        let parsed = DoweSvgPathCache.shared.path(for: data)
        let scaleX = rect.width / viewBox.width
        let scaleY = rect.height / viewBox.height
        let transform = CGAffineTransform(
            a: scaleX,
            b: 0,
            c: 0,
            d: scaleY,
            tx: rect.minX - viewBox.minX * scaleX,
            ty: rect.minY - viewBox.minY * scaleY
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
    let payload: String
    let color: Color
    let animated: Bool

    @ViewBuilder var body: some View {
        if let record = DoweRuntimeSvgParser.parse(payload) {
            DoweSvgView(
                viewBox: record.viewBox,
                color: color,
                paths: record.paths,
                animated: animated
            )
        }
    }
}

struct DoweSvgView: View {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    let viewBox: DoweSvgViewBox
    let color: Color
    let paths: [DoweSvgPathData]
    let animated: Bool

    init(viewBox: DoweSvgViewBox, color: Color, paths: [DoweSvgPathData], animated: Bool = false) {
        self.viewBox = viewBox
        self.color = color
        self.paths = paths
        self.animated = animated
    }

    private var vector: some View {
        ZStack {
            ForEach(paths.indices, id: \.self) { index in
                if let fill = paths[index].fill.resolved(color) {
                    if case .stroke(_, _, let width, let cap, let join) = paths[index].fill {
                        DoweSvgShape(data: paths[index].data, viewBox: viewBox, pathTransform: paths[index].transform)
                            .stroke(fill, style: StrokeStyle(lineWidth: width, lineCap: cap == "round" ? .round : cap == "square" ? .square : .butt, lineJoin: join == "round" ? .round : join == "bevel" ? .bevel : .miter))
                    } else if case .fill(_, _, let evenOdd) = paths[index].fill {
                        DoweSvgShape(data: paths[index].data, viewBox: viewBox, pathTransform: paths[index].transform)
                            .fill(fill, style: FillStyle(eoFill: evenOdd))
                    } else {
                    DoweSvgShape(data: paths[index].data, viewBox: viewBox, pathTransform: paths[index].transform)
                        .fill(fill)
                    }
                }
            }
        }
    }

    @ViewBuilder var body: some View {
        if animated && !reduceMotion {
            TimelineView(.animation) { timeline in
                let elapsed = timeline.date.timeIntervalSinceReferenceDate.truncatingRemainder(dividingBy: 0.9)
                vector.rotationEffect(.degrees(elapsed * 400))
            }
        } else {
            vector
        }
    }
}

private enum DoweSvgPathToken {
    case command(Character)
    case number(CGFloat)
}

private struct DoweSvgPathParser {
    private var tokens: [DoweSvgPathToken]
    private var index = 0
    private var command: Character?

    init(_ source: String) {
        tokens = Self.tokenize(source)
    }

    mutating func parse() -> Path {
        var path = Path()
        var current = CGPoint.zero
        var start = CGPoint.zero
        var lastCubic: CGPoint?
        var lastQuad: CGPoint?

        while index < tokens.count {
            if let next = peekCommand() {
                command = next
                index += 1
            }
            guard let command else {
                break
            }
            let relative = String(command).lowercased() == String(command)
            let normalized = Character(String(command).uppercased())
            switch normalized {
            case "M":
                guard let first = nextPoint(relative: relative, current: current) else {
                    return path
                }
                path.move(to: first)
                current = first
                start = first
                self.command = relative ? "l" : "L"
                while let point = nextPoint(relative: relative, current: current) {
                    path.addLine(to: point)
                    current = point
                }
                lastCubic = nil
                lastQuad = nil
            case "L":
                while let point = nextPoint(relative: relative, current: current) {
                    path.addLine(to: point)
                    current = point
                }
                lastCubic = nil
                lastQuad = nil
            case "H":
                while let x = nextNumber() {
                    let point = CGPoint(x: relative ? current.x + x : x, y: current.y)
                    path.addLine(to: point)
                    current = point
                }
                lastCubic = nil
                lastQuad = nil
            case "V":
                while let y = nextNumber() {
                    let point = CGPoint(x: current.x, y: relative ? current.y + y : y)
                    path.addLine(to: point)
                    current = point
                }
                lastCubic = nil
                lastQuad = nil
            case "C":
                while let x1 = nextNumber(), let y1 = nextNumber(), let x2 = nextNumber(), let y2 = nextNumber(), let x = nextNumber(), let y = nextNumber() {
                    let c1 = point(x1, y1, relative: relative, current: current)
                    let c2 = point(x2, y2, relative: relative, current: current)
                    let end = point(x, y, relative: relative, current: current)
                    path.addCurve(to: end, control1: c1, control2: c2)
                    current = end
                    lastCubic = c2
                    lastQuad = nil
                }
            case "S":
                while let x2 = nextNumber(), let y2 = nextNumber(), let x = nextNumber(), let y = nextNumber() {
                    let c1 = lastCubic.map { reflected($0, around: current) } ?? current
                    let c2 = point(x2, y2, relative: relative, current: current)
                    let end = point(x, y, relative: relative, current: current)
                    path.addCurve(to: end, control1: c1, control2: c2)
                    current = end
                    lastCubic = c2
                    lastQuad = nil
                }
            case "Q":
                while let x1 = nextNumber(), let y1 = nextNumber(), let x = nextNumber(), let y = nextNumber() {
                    let control = point(x1, y1, relative: relative, current: current)
                    let end = point(x, y, relative: relative, current: current)
                    path.addQuadCurve(to: end, control: control)
                    current = end
                    lastQuad = control
                    lastCubic = nil
                }
            case "T":
                while let x = nextNumber(), let y = nextNumber() {
                    let control = lastQuad.map { reflected($0, around: current) } ?? current
                    let end = point(x, y, relative: relative, current: current)
                    path.addQuadCurve(to: end, control: control)
                    current = end
                    lastQuad = control
                    lastCubic = nil
                }
            case "A":
                while let rx = nextNumber(), let ry = nextNumber(), let angle = nextNumber(), let large = nextNumber(), let sweep = nextNumber(), let x = nextNumber(), let y = nextNumber() {
                    let end = point(x, y, relative: relative, current: current)
                    addArc(to: &path, from: current, rx: rx, ry: ry, angle: angle, largeArc: large != 0, sweep: sweep != 0, end: end)
                    current = end
                    lastCubic = nil
                    lastQuad = nil
                }
            case "Z":
                path.closeSubpath()
                current = start
                lastCubic = nil
                lastQuad = nil
                self.command = nil
            default:
                index += 1
            }
        }

        return path
    }

    private func peekCommand() -> Character? {
        guard index < tokens.count else {
            return nil
        }
        if case .command(let value) = tokens[index] {
            return value
        }
        return nil
    }

    private mutating func nextNumber() -> CGFloat? {
        guard index < tokens.count else {
            return nil
        }
        if case .number(let value) = tokens[index] {
            index += 1
            return value
        }
        return nil
    }

    private mutating func nextPoint(relative: Bool, current: CGPoint) -> CGPoint? {
        guard let x = nextNumber(), let y = nextNumber() else {
            return nil
        }
        return point(x, y, relative: relative, current: current)
    }

    private func point(_ x: CGFloat, _ y: CGFloat, relative: Bool, current: CGPoint) -> CGPoint {
        relative ? CGPoint(x: current.x + x, y: current.y + y) : CGPoint(x: x, y: y)
    }

    private func reflected(_ point: CGPoint, around current: CGPoint) -> CGPoint {
        CGPoint(x: current.x * 2 - point.x, y: current.y * 2 - point.y)
    }

    private func addArc(to path: inout Path, from current: CGPoint, rx rawRx: CGFloat, ry rawRy: CGFloat, angle: CGFloat, largeArc: Bool, sweep: Bool, end: CGPoint) {
        var rx = abs(rawRx)
        var ry = abs(rawRy)
        if rx == 0 || ry == 0 || current == end {
            path.addLine(to: end)
            return
        }
        let phi = angle * CGFloat.pi / 180
        let cosPhi = cos(phi)
        let sinPhi = sin(phi)
        let dx = (current.x - end.x) / 2
        let dy = (current.y - end.y) / 2
        let x1p = cosPhi * dx + sinPhi * dy
        let y1p = -sinPhi * dx + cosPhi * dy
        let lambda = x1p * x1p / (rx * rx) + y1p * y1p / (ry * ry)
        if lambda > 1 {
            let factor = sqrt(lambda)
            rx *= factor
            ry *= factor
        }
        let rx2 = rx * rx
        let ry2 = ry * ry
        let x1p2 = x1p * x1p
        let y1p2 = y1p * y1p
        let denominator = rx2 * y1p2 + ry2 * x1p2
        if denominator == 0 {
            path.addLine(to: end)
            return
        }
        let sign: CGFloat = largeArc == sweep ? -1 : 1
        let factor = sign * sqrt(max(0, (rx2 * ry2 - rx2 * y1p2 - ry2 * x1p2) / denominator))
        let cxp = factor * rx * y1p / ry
        let cyp = factor * -ry * x1p / rx
        let cx = cosPhi * cxp - sinPhi * cyp + (current.x + end.x) / 2
        let cy = sinPhi * cxp + cosPhi * cyp + (current.y + end.y) / 2
        let theta1 = vectorAngle(1, 0, (x1p - cxp) / rx, (y1p - cyp) / ry)
        var delta = vectorAngle((x1p - cxp) / rx, (y1p - cyp) / ry, (-x1p - cxp) / rx, (-y1p - cyp) / ry)
        if !sweep && delta > 0 {
            delta -= 2 * CGFloat.pi
        } else if sweep && delta < 0 {
            delta += 2 * CGFloat.pi
        }
        let segments = max(1, Int(ceil(abs(delta) / (CGFloat.pi / 2))))
        let step = delta / CGFloat(segments)
        var theta = theta1
        for _ in 0..<segments {
            let next = theta + step
            addArcSegment(to: &path, cx: cx, cy: cy, rx: rx, ry: ry, phi: phi, start: theta, end: next)
            theta = next
        }
    }

    private func addArcSegment(to path: inout Path, cx: CGFloat, cy: CGFloat, rx: CGFloat, ry: CGFloat, phi: CGFloat, start: CGFloat, end: CGFloat) {
        let alpha = 4 / 3 * tan((end - start) / 4)
        let cosStart = cos(start)
        let sinStart = sin(start)
        let cosEnd = cos(end)
        let sinEnd = sin(end)
        let c1 = arcPoint(cx, cy, rx, ry, phi, cosStart - alpha * sinStart, sinStart + alpha * cosStart)
        let c2 = arcPoint(cx, cy, rx, ry, phi, cosEnd + alpha * sinEnd, sinEnd - alpha * cosEnd)
        let p = arcPoint(cx, cy, rx, ry, phi, cosEnd, sinEnd)
        path.addCurve(to: p, control1: c1, control2: c2)
    }

    private func arcPoint(_ cx: CGFloat, _ cy: CGFloat, _ rx: CGFloat, _ ry: CGFloat, _ phi: CGFloat, _ x: CGFloat, _ y: CGFloat) -> CGPoint {
        CGPoint(
            x: cx + rx * cos(phi) * x - ry * sin(phi) * y,
            y: cy + rx * sin(phi) * x + ry * cos(phi) * y
        )
    }

    private func vectorAngle(_ ux: CGFloat, _ uy: CGFloat, _ vx: CGFloat, _ vy: CGFloat) -> CGFloat {
        let dot = ux * vx + uy * vy
        let length = sqrt((ux * ux + uy * uy) * (vx * vx + vy * vy))
        let value = max(-1, min(1, dot / length))
        let sign: CGFloat = ux * vy - uy * vx < 0 ? -1 : 1
        return sign * acos(value)
    }

    private static func tokenize(_ source: String) -> [DoweSvgPathToken] {
        let characters = Array(source)
        var tokens: [DoweSvgPathToken] = []
        var index = 0
        while index < characters.count {
            let value = characters[index]
            if isCommand(value) {
                tokens.append(.command(value))
                index += 1
            } else if isNumberStart(value) {
                let start = index
                if characters[index] == "-" || characters[index] == "+" {
                    index += 1
                }
                while index < characters.count && characters[index].isNumber {
                    index += 1
                }
                if index < characters.count && characters[index] == "." {
                    index += 1
                    while index < characters.count && characters[index].isNumber {
                        index += 1
                    }
                }
                if index < characters.count && (characters[index] == "e" || characters[index] == "E") {
                    index += 1
                    if index < characters.count && (characters[index] == "-" || characters[index] == "+") {
                        index += 1
                    }
                    while index < characters.count && characters[index].isNumber {
                        index += 1
                    }
                }
                let text = String(characters[start..<index])
                if let value = Double(text) {
                    tokens.append(.number(CGFloat(value)))
                }
            } else {
                index += 1
            }
        }
        return tokens
    }

    private static func isCommand(_ value: Character) -> Bool {
        "MmZzLlHhVvCcSsQqTtAa".contains(value)
    }

    private static func isNumberStart(_ value: Character) -> Bool {
        value.isNumber || value == "-" || value == "+" || value == "."
    }
}

"#
}
