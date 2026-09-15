fn swift_runtime_map() -> &'static str {
    r#"struct DoweMapMarker: Identifiable {
    let id: String
    let lat: String
    let lng: String
    let label: String?
    let popup: String?
    let icon: String
    let action: (() -> Void)?
}

struct DoweMapWaypoint {
    let lat: String
    let lng: String
}

struct DoweMap: View {
    let centerLat: String
    let centerLng: String
    let zoom: UInt16
    let height: String
    let width: String
    let showControls: Bool
    let showScale: Bool
    let showLocationControl: Bool
    let interactive: Bool
    let markers: [DoweMapMarker]
    let waypoints: [DoweMapWaypoint]
    let backgroundColor: Color
    let contentColor: Color
    let onLocation: (() -> Void)?
    let onLocationError: (() -> Void)?
    let onRoute: (() -> Void)?

    var body: some View {
        GeometryReader { proxy in
            ZStack {
                backgroundColor.opacity(0.18)
                GridPattern().stroke(contentColor.opacity(0.16), lineWidth: 1)
                if !waypoints.isEmpty {
                    Capsule().fill(contentColor.opacity(0.6)).frame(width: proxy.size.width * 0.7, height: 4).rotationEffect(.degrees(-10))
                }
                ForEach(Array(markers.enumerated()), id: \.element.id) { index, marker in
                    Button(action: { marker.action?() }) {
                        VStack(spacing: 4) {
                            Image(systemName: "mappin.circle.fill").font(.title2)
                            if let label = marker.label ?? marker.popup {
                                Text(label).font(.caption.weight(.semibold)).padding(.horizontal, 8).padding(.vertical, 2).background(.ultraThinMaterial).clipShape(Capsule())
                            }
                        }
                    }
                    .buttonStyle(.plain)
                    .foregroundStyle(marker.icon == "start" ? DoweDesign.success : marker.icon == "end" ? DoweDesign.danger : contentColor)
                    .position(mapPoint(index: index, total: max(markers.count, 1), size: proxy.size))
                }
                if showControls {
                    VStack(spacing: 0) { Text("+").frame(width: CGFloat(34), height: CGFloat(34)).foregroundStyle(contentColor); Rectangle().fill(contentColor.opacity(0.12)).frame(width: CGFloat(34), height: CGFloat(1)); Text("-").frame(width: CGFloat(34), height: CGFloat(34)).foregroundStyle(contentColor) }
                        .font(.headline.weight(.bold))
                        .frame(width: 34)
                        .background(backgroundColor.opacity(0.92))
                        .clipShape(RoundedRectangle(cornerRadius: 10))
                        .position(x: proxy.size.width - 28, y: 46)
                }
                if showScale {
                    Text("1 km").font(.caption.weight(.bold)).padding(.horizontal, 10).padding(.vertical, 4).background(backgroundColor.opacity(0.92)).clipShape(Capsule()).position(x: 42, y: proxy.size.height - 24)
                }
                if showLocationControl {
                    Button(action: { onLocation?() }) { Image(systemName: "location.fill") }
                        .frame(width: 34, height: 34)
                        .foregroundStyle(contentColor)
                        .background(backgroundColor.opacity(0.92))
                        .clipShape(Circle())
                        .position(x: proxy.size.width - 28, y: proxy.size.height - 28)
                }
            }
        }
        .frame(height: doweMapHeight(height))
        .clipShape(RoundedRectangle(cornerRadius: 16))
    }

    private func mapPoint(index: Int, total: Int, size: CGSize) -> CGPoint {
        let step = size.width / CGFloat(total + 1)
        let x = min(max(step * CGFloat(index + 1), 36), size.width - 36)
        let y = min(max(size.height * (0.3 + CGFloat((index * 23) % 46) / 100), 36), size.height - 36)
        return CGPoint(x: x, y: y)
    }
}

struct GridPattern: Shape {
    func path(in rect: CGRect) -> Path {
        var path = Path()
        let step: CGFloat = 32
        stride(from: CGFloat(0), through: rect.width, by: step).forEach { x in
            path.move(to: CGPoint(x: x, y: 0))
            path.addLine(to: CGPoint(x: x, y: rect.height))
        }
        stride(from: CGFloat(0), through: rect.height, by: step).forEach { y in
            path.move(to: CGPoint(x: 0, y: y))
            path.addLine(to: CGPoint(x: rect.width, y: y))
        }
        return path
    }
}

func doweMapHeight(_ value: String) -> CGFloat {
    if value.hasSuffix("px") {
        return CGFloat(Double(value.dropLast(2)) ?? 400)
    }
    return CGFloat(Double(value) ?? 400)
}

"#
}
