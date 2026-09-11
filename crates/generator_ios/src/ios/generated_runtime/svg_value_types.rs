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
"#
