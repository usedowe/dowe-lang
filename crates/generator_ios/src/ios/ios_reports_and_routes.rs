fn ios_view_consumption_manifest(routes: &[ViewRoute]) -> String {
    let mut entries = BTreeSet::new();
    for route in routes {
        for tree in [&route.layout_tree, &route.page_tree] {
            for entry in consumed_props_for_tree(tree) {
                let owner = entry
                    .item
                    .map(|item| format!("Item:{}", item.as_str()))
                    .unwrap_or_else(|| entry.component.as_str().to_string());
                entries.insert(format!(
                    "{{\"component\":\"{}\",\"owner\":\"{}\",\"prop\":\"{}\",\"irField\":\"{}\"}}",
                    entry.component.as_str(),
                    owner,
                    entry.prop,
                    entry.ir_field.as_str()
                ));
            }
        }
    }
    format!(
        "{{\"schemaVersion\":{},\"target\":\"ios-dev\",\"routes\":[{}],\"consumedProps\":[{}]}}\n",
        dowe_components::VIEW_IR_SCHEMA_VERSION,
        routes
            .iter()
            .map(|route| format!("\"{}\"", route.route_path))
            .collect::<Vec<_>>()
            .join(","),
        entries.into_iter().collect::<Vec<_>>().join(",")
    )
}

fn ios_route_artifacts(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    route_layouts: &[Option<usize>],
) -> Vec<IosArtifact> {
    routes
        .iter()
        .zip(route_layouts)
        .map(|(route, layout_index)| IosArtifact {
            relative_path: PathBuf::from(format!(
                "apps/ios/DowePage{}.swift",
                swift_view_name(&route.route_path)
            )),
            content: generated_route_view(route, font_config, *layout_index),
            kind: IosArtifactKind::Pages,
            target: "ios",
        })
        .collect()
}

