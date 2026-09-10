fn signature_date(namespace: &str, function: &str) -> Option<StdlibSignature> {
    let signature = match (namespace, function) {
        ("date", "now") => sig(
            namespace,
            function,
            &[],
            &[],
            StdlibReturnKind::String,
            "Return the current UTC instant.",
        ),
        ("date", "formatIso") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::String,
            "Normalize an ISO-like instant string.",
        ),
        ("date", "addDays") => sig(
            namespace,
            function,
            &["value", "days"],
            &[],
            StdlibReturnKind::Unknown,
            "Add days to an ISO instant.",
        ),
        ("date", "diffDays") => sig(
            namespace,
            function,
            &["start", "end"],
            &[],
            StdlibReturnKind::Number,
            "Return whole-day difference.",
        ),
        _ => return None,
    };
    Some(signature)
}
