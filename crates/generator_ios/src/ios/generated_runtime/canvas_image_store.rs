r#"final class DoweCanvasImageStore: ObservableObject {
    @Published var images: [String: UIImage] = [:]

    func load(_ sources: [String]) {
        for source in sources where images[source] == nil {
            if let local = UIImage(named: source.trimmingCharacters(in: CharacterSet(charactersIn: "/"))) {
                images[source] = local
            } else if let url = URL(string: source), url.scheme == "https" {
                Task {
                    if let (data, _) = try? await URLSession.shared.data(from: url), let image = UIImage(data: data) {
                        images[source] = image
                    }
                }
            }
        }
    }
}

@MainActor
final class DoweCanvasSelection: ObservableObject {
    @Published var id = ""
}

struct DoweCanvasView: View {
    @ObservedObject var state: DoweReactiveState
"#
