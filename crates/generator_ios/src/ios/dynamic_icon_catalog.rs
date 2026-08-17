const IOS_DYNAMIC_ICON_CATALOG_SHARD_BYTES: usize = 384 * 1024;

fn ios_has_dynamic_icon(routes: &[ViewRoute]) -> bool {
    routes.iter().any(|route| {
        dowe_components::tree_has_dynamic_icon(&route.layout_tree)
            || dowe_components::tree_has_dynamic_icon(&route.page_tree)
    })
}

fn ios_dynamic_icon_catalog_artifacts() -> Vec<IosArtifact> {
    let entries = dowe_components::runtime_icon_catalog_shared()
        .expect("validated runtime icon catalog")
        .iter()
        .map(|(name, payload)| {
            format!(
                "        ({}, {}),",
                swift_string_literal(name),
                swift_string_literal(payload)
            )
        })
        .collect::<Vec<_>>();
    let entry_count = entries.len();
    let shards = ios_dynamic_icon_catalog_shards(entries);
    let mut files = shards
        .iter()
        .enumerate()
        .map(|(index, entries)| IosArtifact {
            relative_path: std::path::PathBuf::from(format!(
                "apps/ios/DoweDynamicIconCatalogShard{index}.swift"
            )),
            content: format!(
                "enum DoweDynamicIconCatalogShard{index} {{\n    static let entries: [(String, String)] = [\n{}\n    ]\n}}\n",
                entries.join("\n")
            ),
            kind: IosArtifactKind::GeneratedView,
            target: "ios",
        })
        .collect::<Vec<_>>();
    let append_shards = shards
        .iter()
        .enumerate()
        .map(|(index, _)| {
            format!(
                "    for (name, payload) in DoweDynamicIconCatalogShard{index}.entries {{\n        catalog[name] = payload\n    }}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    files.push(IosArtifact {
        relative_path: std::path::PathBuf::from("apps/ios/DoweDynamicIconCatalog.swift"),
        content: format!(
            "let DoweDynamicIconCatalog: [String: String] = {{\n    var catalog: [String: String] = [:]\n    catalog.reserveCapacity({entry_count})\n{append_shards}\n    return catalog\n}}()\n"
        ),
        kind: IosArtifactKind::GeneratedView,
        target: "ios",
    });
    files
}

fn ios_dynamic_icon_catalog_shards(entries: Vec<String>) -> Vec<Vec<String>> {
    let mut shards = Vec::new();
    let mut current = Vec::new();
    let mut current_bytes: usize = 0;
    for entry in entries {
        let entry_bytes = entry.len() + 1;
        if !current.is_empty()
            && current_bytes.saturating_add(entry_bytes) > IOS_DYNAMIC_ICON_CATALOG_SHARD_BYTES
        {
            shards.push(std::mem::take(&mut current));
            current_bytes = 0;
        }
        current_bytes += entry_bytes;
        current.push(entry);
    }
    if !current.is_empty() {
        shards.push(current);
    }
    shards
}
