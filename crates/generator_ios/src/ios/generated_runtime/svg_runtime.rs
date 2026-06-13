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

    func resolved(_ current: Color) -> Color? {
        switch self {
        case .none:
            return nil
        case .currentColor:
            return current
        case .color(let color):
            return color
        }
    }
}

struct DoweSvgPathData {
    let data: String
    let fill: DoweSvgFill
}

struct DoweSvgShape: Shape {
    let data: String
    let viewBox: DoweSvgViewBox

    func path(in rect: CGRect) -> Path {
        var parser = DoweSvgPathParser(data)
        let parsed = parser.parse()
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
        return parsed.applying(transform)
    }
}

struct DoweSvgView: View {
    let viewBox: DoweSvgViewBox
    let color: Color
    let paths: [DoweSvgPathData]

    var body: some View {
        ZStack {
            ForEach(paths.indices, id: \.self) { index in
                if let fill = paths[index].fill.resolved(color) {
                    DoweSvgShape(data: paths[index].data, viewBox: viewBox)
                        .fill(fill)
                }
            }
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
