fn signature_parse(namespace: &str, function: &str) -> Option<StdlibSignature> {
    let signature = match (namespace, function) {
        ("parse", "int") => sig(
            namespace,
            function,
            &["value"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Parse an integer or return fallback/null.",
        ),
        ("parse", "float") => sig(
            namespace,
            function,
            &["value"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Parse a finite number or return fallback/null.",
        ),
        ("parse", "bool") => sig(
            namespace,
            function,
            &["value"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Parse a boolean or return fallback/null.",
        ),
        ("parse", "json") => sig(
            namespace,
            function,
            &["value"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Parse JSON text or return fallback/null.",
        ),
        ("parse", "string") => sig(
            namespace,
            function,
            &["value"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Convert a value to string.",
        ),
        ("parse", "svg") => sig(
            namespace,
            function,
            &["value"],
            &["fallback", "colors", "format"],
            StdlibReturnKind::Unknown,
            "Convert portable SVG XML into Dowe source or normalized preview data.",
        ),
        _ => return None,
    };
    Some(signature)
}
