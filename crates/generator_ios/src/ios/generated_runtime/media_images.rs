r#"struct DoweImageView: View {
    let source: String
    let alt: String
    let aspect: String
    let objectFit: String
    let loading: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat

    var body: some View {
        Group {
            if aspect == "auto" {
                doweImageContent(source: source, alt: alt, objectFit: objectFit, contentColor: contentColor)
            } else {
                DoweImageAspectLayout(ratio: doweImageAspect(aspect)) {
                    doweImageContent(source: source, alt: alt, objectFit: objectFit, contentColor: contentColor)
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(backgroundColor)
        .clipped()
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
        .accessibilityElement(children: .ignore)
        .accessibilityAddTraits(.isImage)
        .accessibilityLabel(Text(alt))
        .accessibilityHidden(alt.isEmpty)
    }
}

@ViewBuilder
private func doweImageContent(source: String, alt: String, objectFit: String, contentColor: Color) -> some View {
                if let url = doweImageURL(source) {
                    AsyncImage(url: url) { image in
                        if objectFit == "contain" {
                            image.resizable().scaledToFit()
                        } else {
                            image.resizable().scaledToFill()
                        }
                    } placeholder: {
                        Rectangle().fill(contentColor.opacity(0.12))
                    }
                } else {
                    Image(source)
                        .resizable()
                        .aspectRatio(contentMode: objectFit == "contain" ? .fit : .fill)
                }
}

struct DoweImageAspectLayout: Layout {
    let ratio: CGFloat

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let resolvedRatio = ratio > 0 ? ratio : CGFloat(1)
        if let width = proposal.width, width.isFinite {
            let resolvedWidth = Swift.max(width, 0)
            return CGSize(width: resolvedWidth, height: resolvedWidth / resolvedRatio)
        }
        if let height = proposal.height, height.isFinite {
            let resolvedHeight = Swift.max(height, 0)
            return CGSize(width: resolvedHeight * resolvedRatio, height: resolvedHeight)
        }
        let idealWidth = subviews.first?.sizeThatFits(.unspecified).width ?? CGFloat(0)
        let resolvedWidth = Swift.max(idealWidth, 0)
        return CGSize(width: resolvedWidth, height: resolvedWidth / resolvedRatio)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        for subview in subviews {
            subview.place(at: bounds.origin, anchor: .topLeading, proposal: ProposedViewSize(bounds.size))
        }
    }
}

private func doweImageAspect(_ value: String) -> CGFloat {
    switch value {
    case "vertical":
        return CGFloat(9) / CGFloat(16)
    case "square":
        return CGFloat(1)
    default:
        return CGFloat(16) / CGFloat(9)
    }
}

"#
