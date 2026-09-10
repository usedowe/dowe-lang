r#"
private func doweDynamicFontSize(_ value: String) -> Font {
    switch value { case "xs": return .caption; case "sm": return .subheadline; case "lg": return .title3; case "xl": return .title2; default: return .body }
}
private func doweDynamicFontWeight(_ value: String) -> Font.Weight {
    switch value { case "thin": return .thin; case "light": return .light; case "medium": return .medium; case "semibold": return .semibold; case "bold": return .bold; case "black": return .black; default: return .regular }
}

@MainActor
private func doweDynamicColor(_ value: String) -> Color {
    switch value {
    case "primary": return DoweDesign.primary
    case "secondary": return DoweDesign.secondary
    case "accent": return DoweDesign.accent
    case "muted": return DoweDesign.muted
    case "success": return DoweDesign.success
    case "info": return DoweDesign.info
    case "warning": return DoweDesign.warning
    case "danger": return DoweDesign.danger
    default: return DoweDesign.primary
    }
}
private let dowePropVariants = [__DOWE_VARIANTS__]
private let dowePropSchemes = [__DOWE_SCHEMES__]
private let dowePropSizes = [__DOWE_SIZES__]
private let dowePropRounded = [__DOWE_ROUNDED__]
private let dowePropColors = [__DOWE_COLORS__]
private let dowePropIcons = [__DOWE_ICON_NAMES__]

private func doweValidEnum(_ value: String, _ kind: String) -> String {
    let allowed: Set<String>
    let fallback: String
    switch kind {
    case "variant": allowed = Set(dowePropVariants); fallback = "solid"
    case "scheme": allowed = Set(dowePropSchemes); fallback = "primary"
    case "size": allowed = Set(dowePropSizes); fallback = "md"
    case "rounded": allowed = Set(dowePropRounded); fallback = "md"
    case "color": allowed = Set(dowePropColors); fallback = "primary"
    case "icon": allowed = Set(dowePropIcons); fallback = ""
    default: return value
    }
    return allowed.contains(value) ? value : fallback
}

struct DoweRow: Identifiable {
    let id: String
    let value: [String: Any]
}

struct DoweTreeNode: Identifiable {
    let id: String
    let label: String
    let path: String
    let branch: Bool
    let children: [DoweTreeNode]
    let value: [String: Any]
}

struct DoweInvokeAction {
    let function: String
    let args: [DoweStdlibArg]
    let update: String?
    let reset: String?
    let successAlert: String?
    let successMessage: String?
    let errorAlert: String?
    let errorMessage: String?
}

struct DoweRequestAction {
    let method: String
    let path: String
    let base: String
    let headers: [(String, String, String)]
    let body: String?
    let update: String?
    let reset: String?
    let successAlert: String?
    let successMessage: String?
    let errorAlert: String?
    let errorMessage: String?
}

struct DoweActionMetadata {
    let params: [String: String]
    let returnType: String?
}

enum DoweAction {
    case request(DoweRequestAction, DoweActionMetadata)
    case invoke(DoweInvokeAction, DoweActionMetadata)
    case assign(String, String, DoweStdlibCall?, DoweActionMetadata)
    case reset(String, DoweActionMetadata)
    case sequence([DoweStep], DoweActionMetadata)
}

enum DoweStep {
    case validate(String)
    case request(String, DoweRequestAction)
    case invoke(String, DoweInvokeAction)
    case branch(String, [DoweStep], [DoweStep])
    case assign(String, String, Any?, Bool, DoweStdlibCall?)
    case reset(String)
    case toast(String, String, String, Int?, String?, String?, String?)
    case redirect(String)
}

struct DoweToastState: Hashable {
    let id: Int
    let kind: String
    let title: String
    let message: String
    let duration: Int
    let scheme: String
    let variant: String
    let position: String
}

enum DoweNativeBridge {
    static var handler: ((String, [String: Any]) async -> (Bool, Any?))?
    static func install(_ handler: @escaping (String, [String: Any]) async -> (Bool, Any?)) {
        self.handler = handler
    }
    static func invoke(_ function: String, args: [String: Any]) async -> (Bool, Any?) {
        if let handler { return await handler(function, args) }
        return (false, nil)
    }
}

struct DoweStdlibCall {
    let namespace: String
    let function: String
    let args: [DoweStdlibArg]
}

struct DoweStdlibArg {
    let name: String
    let value: DoweStdlibValue
}

struct DoweStdlibValue {
    let kind: String
    let value: Any?
}

struct DoweSignalMetadata {
    let name: String
    let scope: String
    let storage: String
}

struct DoweFormFieldMetadata {
    let path: String
    let kind: String
    let rules: [DoweValidationRule]
}

"#
