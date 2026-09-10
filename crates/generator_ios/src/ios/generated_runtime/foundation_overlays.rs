r#"    let overlay: DoweOverlay

    var body: some View {
        switch overlay {
        case .color(let color):
            color
        case .gradient(let start, let end):
            LinearGradient(colors: [start, end], startPoint: .top, endPoint: .bottom)
        }
    }
}

struct DoweSectionBackgroundView: View {
    let background: DoweSectionBackground

    var body: some View {
        switch background {
        case .soft:
            LinearGradient(colors: [DoweDesign.surface, DoweDesign.background], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .aurora:
            LinearGradient(colors: [DoweDesign.primary, DoweDesign.secondary, DoweDesign.accent], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .sunrise:
            LinearGradient(colors: [DoweDesign.warning, DoweDesign.danger, DoweDesign.surface], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .ocean:
            LinearGradient(colors: [DoweDesign.info, DoweDesign.primary, DoweDesign.accent], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .meadow:
            LinearGradient(colors: [DoweDesign.success, DoweDesign.accent, DoweDesign.surface], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .slate:
            LinearGradient(colors: [DoweDesign.muted, DoweDesign.surface, DoweDesign.background], startPoint: .topLeading, endPoint: .bottomTrailing)
        }
    }
}

"#
