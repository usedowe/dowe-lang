r#"
struct DoweAudioView: View {
    let source: String
    let subtitle: String?
    let avatarSource: String?
    let playIcon: DoweVideoIcon
    let pauseIcon: DoweVideoIcon
    let backgroundColor: Color
    let contentColor: Color
    let buttonBackgroundColor: Color
    let buttonContentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    @State private var player: AVPlayer
    @State private var playing = false
    @State private var currentTime = Double(0)
    @State private var duration = Double(0)
    private let timer = Timer.publish(every: 0.25, on: .main, in: .common).autoconnect()

    init(source: String, subtitle: String?, avatarSource: String?, playIcon: DoweVideoIcon, pauseIcon: DoweVideoIcon, backgroundColor: Color, contentColor: Color, buttonBackgroundColor: Color, buttonContentColor: Color, borderColor: Color?, radius: CGFloat) {
        self.source = source
        self.subtitle = subtitle
        self.avatarSource = avatarSource
        self.playIcon = playIcon
        self.pauseIcon = pauseIcon
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.buttonBackgroundColor = buttonBackgroundColor
        self.buttonContentColor = buttonContentColor
        self.borderColor = borderColor
        self.radius = radius
        _player = State(initialValue: AVPlayer(url: doweAudioURL(source)))
    }

    var body: some View {
        HStack(spacing: CGFloat(12)) {
            DoweAudioControlButton(
                icon: playing ? pauseIcon : playIcon,
                label: playing ? "Pause audio" : "Play audio",
                backgroundColor: buttonBackgroundColor,
                contentColor: buttonContentColor,
                action: togglePlayback
            )
            VStack(alignment: .leading, spacing: CGFloat(2)) {
                GeometryReader { geometry in
                    HStack(spacing: CGFloat(2)) {
                        ForEach(0..<50, id: \.self) { index in
                            let active = duration > 0 && (Double(index) + 0.5) / 50 <= currentTime / duration
                            RoundedRectangle(cornerRadius: CGFloat(2))
                                .fill(contentColor.opacity(active ? 1 : 0.3))
                                .frame(maxWidth: .infinity)
                                .frame(height: doweAudioWaveform[index] * CGFloat(20))
                        }
                    }
                    .animation(.easeInOut(duration: 0.3), value: currentTime)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
                    .contentShape(Rectangle())
                    .gesture(
                        DragGesture(minimumDistance: 0)
                            .onChanged { value in seek(at: value.location.x, width: geometry.size.width) }
                    )
                }
                .frame(height: CGFloat(32))
                .accessibilityElement(children: .ignore)
                .accessibilityLabel(Text("Audio progress"))
                .accessibilityValue(Text(doweAudioTime(max(0, duration - currentTime))))
                .accessibilityAdjustableAction { direction in
                    guard duration > 0 else { return }
                    let step = min(5, duration / 20)
                    let next = direction == .increment ? min(duration, currentTime + step) : max(0, currentTime - step)
                    currentTime = next
                    player.seek(to: CMTime(seconds: next, preferredTimescale: 600))
                }
                HStack(spacing: CGFloat(12)) {
                    Text(doweAudioTime(max(0, duration - currentTime)))
                        .font(.system(size: CGFloat(12), weight: .semibold))
                        .monospacedDigit()
                    if let subtitle {
                        Text(subtitle)
                            .lineLimit(1)
                            .truncationMode(.tail)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
                .font(.system(size: CGFloat(12)))
                .opacity(0.72)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            if let avatarSource {
                DoweCoverImage(source: avatarSource)
                    .frame(width: CGFloat(48), height: CGFloat(48))
                    .clipShape(Circle())
            }
        }
        .padding(.horizontal, CGFloat(12))
        .padding(.vertical, CGFloat(6))
        .foregroundStyle(contentColor)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
        .onAppear {
            try? AVAudioSession.sharedInstance().setCategory(.playback, mode: .default)
            try? AVAudioSession.sharedInstance().setActive(true)
        }
        .onReceive(timer) { _ in
            let seconds = player.currentTime().seconds
            if seconds.isFinite { currentTime = max(0, seconds) }
            let total = player.currentItem?.duration.seconds ?? 0
            if total.isFinite { duration = max(0, total) }
            playing = player.timeControlStatus == .playing
        }
        .onDisappear {
            player.pause()
            playing = false
        }
    }

    private func togglePlayback() {
        if playing {
            player.pause()
            playing = false
            return
        }
        if duration > 0 && currentTime >= duration {
            seek(at: 0, width: 1)
        }
        player.play()
        playing = true
    }

    private func seek(at x: CGFloat, width: CGFloat) {
        guard duration > 0, width > 0 else { return }
        let value = max(0, min(1, x / width)) * duration
        currentTime = value
        player.seek(to: CMTime(seconds: value, preferredTimescale: 600))
    }
}

private struct DoweAudioControlButton: View {
    let icon: DoweVideoIcon
    let label: String
    let backgroundColor: Color
    let contentColor: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            DoweSvgView(viewBox: icon.viewBox, color: contentColor, paths: icon.paths)
                .frame(width: CGFloat(20), height: CGFloat(20))
                .contentShape(Rectangle())
        }
        .frame(width: CGFloat(40), height: CGFloat(40))
        .foregroundStyle(contentColor)
        .background(backgroundColor)
        .clipShape(Circle())
        .buttonStyle(.plain)
        .accessibilityLabel(Text(label))
    }
}

private func doweAudioURL(_ source: String) -> URL {
    if let url = URL(string: source), url.scheme != nil { return url }
    let path = source.trimmingCharacters(in: CharacterSet(charactersIn: "/")).replacingOccurrences(of: "assets/", with: "")
    let file = path as NSString
    if let bundled = Bundle.main.url(forResource: file.deletingPathExtension, withExtension: file.pathExtension.isEmpty ? nil : file.pathExtension) {
        return bundled
    }
    return URL(fileURLWithPath: source)
}

private func doweAudioTime(_ value: Double) -> String {
    let seconds = max(0, Int(value))
    return "\(seconds / 60):\(String(format: "%02d", seconds % 60))"
}

"#
