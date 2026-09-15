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
    let panelPadding: CGFloat
    let contentGap: CGFloat
    let iconSize: CGFloat
    let titleSize: CGFloat
    let descriptionSize: CGFloat
    let actionHorizontalPadding: CGFloat
    let actionVerticalPadding: CGFloat

    var body: some View {
        VStack(spacing: contentGap) {
            DoweSvgView(viewBox: iconViewBox, color: accentColor, paths: iconPaths)
                .frame(width: iconSize, height: iconSize)
            Text(title ?? defaultTitle)
                .font(.system(size: titleSize, weight: .semibold))
                .foregroundStyle(contentColor)
            Text(description ?? defaultDescription)
                .font(.system(size: descriptionSize))
                .foregroundStyle(contentColor.opacity(0.64))
                .multilineTextAlignment(.center)
            if let action {
                Button(actionLabel, action: action)
                    .buttonStyle(.plain)
                    .font(.system(size: descriptionSize, weight: .semibold))
                    .padding(.horizontal, actionHorizontalPadding)
                    .padding(.vertical, actionVerticalPadding)
                    .background(accentColor.opacity(0.12))
                    .foregroundStyle(accentColor)
                    .clipShape(Capsule())
            }
        }
        .frame(maxWidth: .infinity)
        .padding(panelPadding)
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
