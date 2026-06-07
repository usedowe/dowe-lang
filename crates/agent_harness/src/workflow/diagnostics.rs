fn error_diagnostic(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
    action: impl Into<String>,
) -> Diagnostic {
    Diagnostic {
        code: code.into(),
        severity: DiagnosticSeverity::Error,
        path: path.into(),
        message: message.into(),
        action: action.into(),
    }
}
