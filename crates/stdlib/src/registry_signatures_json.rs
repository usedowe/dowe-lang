fn signature_json(namespace: &str, function: &str) -> Option<StdlibSignature> {
    let signature = match (namespace, function) {
        ("json", "get") => sig(
            namespace,
            function,
            &["value", "path"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Read a JSON-compatible path.",
        ),
        ("json", "set") => sig(
            namespace,
            function,
            &["value", "path", "next"],
            &[],
            StdlibReturnKind::Object,
            "Return an object with a path set.",
        ),
        ("json", "pick") => sig(
            namespace,
            function,
            &["value", "fields"],
            &[],
            StdlibReturnKind::Object,
            "Pick object fields.",
        ),
        ("json", "omit") => sig(
            namespace,
            function,
            &["value", "fields"],
            &[],
            StdlibReturnKind::Object,
            "Omit object fields.",
        ),
        ("json", "merge") => sig(
            namespace,
            function,
            &["left", "right"],
            &[],
            StdlibReturnKind::Object,
            "Merge two objects shallowly.",
        ),
        ("json", "stringify") => sig(
            namespace,
            function,
            &["value"],
            &["pretty"],
            StdlibReturnKind::String,
            "Serialize JSON-compatible value.",
        ),
        ("json", "parse") => sig(
            namespace,
            function,
            &["value"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Parse JSON text or return fallback/null.",
        ),
        _ => return None,
    };
    Some(signature)
}
