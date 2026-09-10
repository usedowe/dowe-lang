fn signature_csv(namespace: &str, function: &str) -> Option<StdlibSignature> {
    let signature = match (namespace, function) {
        ("csv", "parse") => sig(
            namespace,
            function,
            &["value"],
            &["delimiter", "header", "maxRows", "maxColumns"],
            StdlibReturnKind::Object,
            "Parse CSV text into rows.",
        ),
        ("csv", "stringify") => sig(
            namespace,
            function,
            &["rows"],
            &["delimiter"],
            StdlibReturnKind::String,
            "Serialize rows to CSV text.",
        ),
        _ => return None,
    };
    Some(signature)
}
