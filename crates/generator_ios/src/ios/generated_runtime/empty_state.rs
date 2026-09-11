r#"struct DoweEmpty: View {
    let kind: String
    let title: String?
    let description: String?
    let actionLabel: String
    let action: (() -> Void)?
    let iconViewBox: DoweSvgViewBox
    let iconPaths: [DoweSvgPathData]
    let backgroundColor: Color
    let contentColor: Color
    let accentColor: Color

    var body: some View {
        VStack(spacing: CGFloat(12)) {
            DoweSvgView(viewBox: iconViewBox, color: accentColor, paths: iconPaths)
                .frame(width: CGFloat(112), height: CGFloat(112))
            Text(title ?? defaultTitle)
                .font(.title3.weight(.semibold))
                .foregroundStyle(contentColor)
            Text(description ?? defaultDescription)
                .font(.subheadline)
                .foregroundStyle(contentColor.opacity(0.64))
                .multilineTextAlignment(.center)
            if let action {
                Button(actionLabel, action: action)
                    .buttonStyle(.plain)
                    .font(.subheadline.weight(.semibold))
                    .padding(.horizontal, CGFloat(16))
                    .padding(.vertical, CGFloat(9))
                    .background(accentColor.opacity(0.12))
                    .foregroundStyle(accentColor)
                    .clipShape(Capsule())
            }
        }
        .frame(maxWidth: .infinity)
        .padding(CGFloat(24))
    }

    private var defaultTitle: String {
        switch kind {
        case "playlist": return "No playlist items"
        case "result": return "No results"
        case "template": return "No templates"
        default: return "No data"
        }
    }

    private var defaultDescription: String {
        switch kind {
        case "playlist": return "Add items to start building this playlist."
        case "result": return "Try changing the search or filters."
        case "template": return "Create a template to reuse this workflow."
        default: return "There is nothing to show yet."
        }
    }
}

struct DoweMarquee<Content: View>: View {
"#
