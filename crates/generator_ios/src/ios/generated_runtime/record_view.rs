r#"    let name: String
    let url: String?
    let disabled: Bool
    let maxDuration: UInt16?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let onStart: (() -> Void)?
    let onPause: (() -> Void)?
    let onResume: (() -> Void)?
    let onStop: (() -> Void)?
    let onDiscard: (() -> Void)?
    let onConfirm: (() -> Void)?
    @State private var state = "idle"
    @State private var elapsed = 0
    @State private var started = Date()
    @State private var now = Date()

    var body: some View {
        HStack(spacing: 12) {
            HStack(alignment: .bottom, spacing: 2) {
                ForEach(0..<50, id: \.self) { index in
                    Capsule()
                        .fill(contentColor.opacity(state == "recording" ? 0.85 : 0.34))
                        .frame(width: 2, height: CGFloat((index % 9) + 2) * 2)
                }
            }
            VStack(alignment: .leading, spacing: 2) {
                Text(recordTime).font(.caption.weight(.bold)).monospacedDigit()
                Text(recordStatus).font(.caption).opacity(0.72)
            }
            Spacer(minLength: 8)
            recordButtons
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: 16))
        .overlay(RoundedRectangle(cornerRadius: 16).stroke(borderColor ?? .clear, lineWidth: 1))
        .opacity(disabled ? 0.5 : 1)
        .disabled(disabled)
        .onAppear {
            if url != nil {
                state = "reviewing"
            }
        }
        .onReceive(Timer.publish(every: 1, on: .main, in: .common).autoconnect()) { value in
            now = value
            if let maxDuration = maxDuration, state == "recording", currentElapsed >= Int(maxDuration) {
                elapsed = Int(maxDuration)
                state = "reviewing"
                onStop?()
            }
        }
    }

    private var recordButtons: some View {
        HStack(spacing: 6) {
            if state == "idle" || state == "paused" {
                Button(state == "paused" ? "Resume" : "Record") {
                    let resume = state == "paused"
                    now = Date()
                    if !resume {
                        elapsed = 0
                    }
                    started = now
                    state = "recording"
                    if resume {
                        onResume?()
                    } else {
                        onStart?()
                    }
                }
            }
            if state == "recording" {
                Button("Pause") {
                    now = Date()
                    elapsed = currentElapsed
                    state = "paused"
                    onPause?()
                }
                Button("Stop") {
                    now = Date()
                    elapsed = currentElapsed
                    state = "reviewing"
                    onStop?()
                }
            }
            if state == "reviewing" {
                Button("Discard") {
                    elapsed = 0
                    state = "idle"
                    onDiscard?()
                }
                Button("Use") { onConfirm?() }
            }
        }
        .buttonStyle(.bordered)
        .font(.caption.weight(.semibold))
    }

    private var recordStatus: String {
        state == "recording" ? "Recording" : state == "paused" ? "Paused" : state == "reviewing" ? "Review" : "Ready"
    }

    private var recordTime: String {
        let value = currentElapsed
        return "\(value / 60):\(String(format: "%02d", value % 60))"
    }

    private var currentElapsed: Int {
        if state == "recording" {
            return elapsed + max(0, Int(now.timeIntervalSince(started)))
        }
        return elapsed
    }
}

"#
