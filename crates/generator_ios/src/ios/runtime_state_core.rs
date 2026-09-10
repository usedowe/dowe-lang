r#"@MainActor
final class DoweReactiveState: ObservableObject {
    private static var globalValues: [String: Any] = [:]
    private static var globalStorage: [String: String] = [:]
    @Published private var values: [String: Any]
    @Published private(set) var toast: DoweToastState? = nil
    @Published private(set) var redirectPath: String? = nil
    private var toastSequence = 0
    private let constants: [String: Any]
    private let initial: [String: Any]
    private let signals: [String: DoweSignalMetadata]
    private let actions: [String: DoweAction]
    private let forms: [String: [DoweFormFieldMetadata]]
    private var formTouched: [String: Bool] = [:]
    private var loaded = Set<String>()

    init(constants: [String: Any], initial: [String: Any], signals: [String: DoweSignalMetadata], actions: [String: DoweAction], forms: [String: [DoweFormFieldMetadata]] = [:]) {
        self.constants = constants
        self.initial = initial
        self.signals = signals
        self.actions = actions
        self.forms = forms
        var hydrated = initial
        for (id, metadata) in signals where metadata.scope == "global" {
            Self.globalStorage[metadata.name] = metadata.storage
            if Self.globalValues[metadata.name] == nil {
                let fallback = initial[id] ?? NSNull()
                if let stored = Self.storedSignal(metadata), Self.compatibleSignalValue(stored, fallback) {
                    Self.globalValues[metadata.name] = stored
                } else {
                    Self.globalValues[metadata.name] = fallback
                }
            }
            hydrated[id] = Self.globalValues[metadata.name] ?? NSNull()
        }
        self.values = hydrated
    }

    private static func compatibleSignalValue(_ value: Any, _ initial: Any) -> Bool {
        if initial is NSNull {
            return value is NSNull
        }
        if let expected = initial as? [String: Any] {
            guard let actual = value as? [String: Any] else {
                return false
            }
            return expected.allSatisfy { key, expectedValue in
                guard let actualValue = actual[key] else {
                    return false
                }
                return compatibleSignalValue(actualValue, expectedValue)
            }
        }
        if let expected = initial as? [Any] {
            guard let actual = value as? [Any] else {
                return false
            }
            if expected.isEmpty {
                return true
            }
            return actual.allSatisfy { value in
                expected.contains { candidate in compatibleSignalValue(value, candidate) }
            }
        }
        if initial is Bool {
            return value is Bool
        }
        if initial is NSNumber {
            return value is NSNumber && !(value is Bool)
        }
        if initial is String {
            return value is String
        }
        return false
    }

    private static func storageKey(_ name: String) -> String {
        "dowe:signal:" + name
    }

    private static func storedSignal(_ metadata: DoweSignalMetadata) -> Any? {
        guard metadata.storage == "local",
              let data = UserDefaults.standard.data(forKey: Self.storageKey(metadata.name)) else {
            return nil
        }
        return (try? JSONSerialization.jsonObject(with: data) as? [String: Any])?["value"]
    }

    private func persistRoot(_ root: String) {
        guard let metadata = signals[root], metadata.scope == "global" else {
            return
        }
        let value = values[root] ?? NSNull()
        for (id, candidate) in signals where candidate.scope == "global" && candidate.name == metadata.name {
            values[id] = value
        }
        Self.globalValues[metadata.name] = value
        guard metadata.storage == "local", JSONSerialization.isValidJSONObject(["value": value]) else {
            return
        }
        if let data = try? JSONSerialization.data(withJSONObject: ["value": value]) {
            UserDefaults.standard.set(data, forKey: Self.storageKey(metadata.name))
        }
    }

"#
