fn signature_url(namespace: &str, function: &str) -> Option<StdlibSignature> {
    let signature = match (namespace, function) {
        ("url", "encode") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::String,
            "Percent-encode text.",
        ),
        ("url", "decode") => sig(
            namespace,
            function,
            &["value"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Percent-decode text.",
        ),
        ("url", "parse") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::Object,
            "Parse URL text into serializable parts.",
        ),
        ("url", "queryGet") => sig(
            namespace,
            function,
            &["value", "name"],
            &[],
            StdlibReturnKind::Unknown,
            "Read a query parameter.",
        ),
        ("url", "querySet") => sig(
            namespace,
            function,
            &["value", "name", "param"],
            &[],
            StdlibReturnKind::String,
            "Set a query parameter.",
        ),
        _ => return None,
    };
    Some(signature)
}
