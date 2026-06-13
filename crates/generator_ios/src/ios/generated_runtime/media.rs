fn swift_runtime_media() -> &'static str {
    r#"struct DoweCoverImage: View {
    let source: String

    var body: some View {
        if source.hasPrefix("https://"), let url = URL(string: source) {
            AsyncImage(url: url) { image in
                image.resizable().scaledToFill()
            } placeholder: {
                Color.clear
            }
        } else {
            Image(source).resizable().scaledToFill()
        }
    }
}

struct DoweCodeToken {
    let text: String
    let color: Color
}

struct DoweVideoView: View {
    let poster: String?
    let autoplay: Bool
    let aspect: String
    let backgroundColor: Color
    let borderColor: Color?
    let radius: CGFloat
    @State private var player: AVPlayer
    @State private var started = false

    init(source: String, poster: String?, autoplay: Bool, aspect: String, backgroundColor: Color, borderColor: Color?, radius: CGFloat) {
        self.poster = poster
        self.autoplay = autoplay
        self.aspect = aspect
        self.backgroundColor = backgroundColor
        self.borderColor = borderColor
        self.radius = radius
        _player = State(initialValue: AVPlayer(url: URL(string: source)!))
    }

    var body: some View {
        ZStack {
            VideoPlayer(player: player)
            if let poster, !started {
                DoweCoverImage(source: poster)
                    .contentShape(Rectangle())
                    .onTapGesture(perform: play)
            }
        }
        .frame(maxWidth: .infinity)
        .aspectRatio(doweVideoAspect(aspect), contentMode: .fit)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
        .onAppear {
            if autoplay {
                play()
            }
        }
    }

    private func play() {
        started = true
        player.play()
    }
}

private func doweVideoAspect(_ value: String) -> CGFloat {
    switch value {
    case "vertical":
        return CGFloat(9) / CGFloat(16)
    case "square":
        return CGFloat(1)
    default:
        return CGFloat(16) / CGFloat(9)
    }
}

struct DoweAudioView: View {
    let source: String
    let subtitle: String?
    let avatarSource: String?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    @State private var playing = false

    var body: some View {
        HStack(spacing: CGFloat(12)) {
            Button(playing ? "Pause" : "Play") {
                playing.toggle()
            }
            .buttonStyle(.bordered)
            VStack(alignment: .leading, spacing: CGFloat(6)) {
                Text(subtitle ?? source)
                    .lineLimit(1)
                HStack(spacing: CGFloat(3)) {
                    ForEach(0..<24, id: \.self) { index in
                        RoundedRectangle(cornerRadius: CGFloat(2))
                            .fill(contentColor.opacity(playing ? 0.9 : 0.35))
                            .frame(width: CGFloat(3), height: CGFloat((index % 7) + 4))
                    }
                }
            }
            if let avatarSource {
                DoweCoverImage(source: avatarSource)
                    .frame(width: CGFloat(36), height: CGFloat(36))
                    .clipShape(Circle())
            }
        }
        .padding(.horizontal, CGFloat(12))
        .padding(.vertical, CGFloat(8))
        .foregroundStyle(contentColor)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
    }
}

struct DoweImageView: View {
    let source: String
    let alt: String
    let aspect: String
    let objectFit: String
    let loading: String
    let hideControls: Bool
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat

    var body: some View {
        ZStack(alignment: .bottomLeading) {
            AsyncImage(url: URL(string: source)) { image in
                if objectFit == "contain" {
                    image.resizable().scaledToFit()
                } else {
                    image.resizable().scaledToFill()
                }
            } placeholder: {
                Rectangle().fill(contentColor.opacity(0.12))
            }
            if !hideControls && !alt.isEmpty {
                Text(alt)
                    .lineLimit(1)
                    .padding(CGFloat(8))
                    .background(backgroundColor.opacity(0.72))
                    .foregroundStyle(contentColor)
            }
        }
        .frame(maxWidth: .infinity)
        .aspectRatio(doweImageAspect(aspect), contentMode: .fit)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
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
}
