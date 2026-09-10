fn signature_hash(namespace: &str, function: &str) -> Option<StdlibSignature> {
    let signature = match (namespace, function) {
        ("hash", "sha256") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::String,
            "Compute a SHA-256 digest for server-owned text.",
        ),
        _ => return None,
    };
    Some(signature)
}
