fn signature_id(namespace: &str, function: &str) -> Option<StdlibSignature> {
    let signature = match (namespace, function) {
        ("id", "ulid") => sig(
            namespace,
            function,
            &[],
            &[],
            StdlibReturnKind::String,
            "Generate a canonical server-only ULID.",
        ),
        _ => return None,
    };
    Some(signature)
}
