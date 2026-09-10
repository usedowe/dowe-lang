fn signature_sort(namespace: &str, function: &str) -> Option<StdlibSignature> {
    let signature = match (namespace, function) {
        ("sort", "asc") => sig(
            namespace,
            function,
            &["values"],
            &[],
            StdlibReturnKind::Array,
            "Stable ascending sort.",
        ),
        ("sort", "desc") => sig(
            namespace,
            function,
            &["values"],
            &[],
            StdlibReturnKind::Array,
            "Stable descending sort.",
        ),
        ("sort", "by") => sig(
            namespace,
            function,
            &["values", "field"],
            &["direction", "nulls"],
            StdlibReturnKind::Array,
            "Stable sort objects by a field.",
        ),
        _ => return None,
    };
    Some(signature)
}
