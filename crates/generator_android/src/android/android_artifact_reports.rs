fn android_view_consumption_manifest(routes: &[ViewRoute]) -> String {
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
        "{{\"schemaVersion\":{},\"target\":\"android-dev\",\"routes\":[{}],\"consumedProps\":[{}]}}\n",
        dowe_components::VIEW_IR_SCHEMA_VERSION,
        routes
            .iter()
            .map(|route| format!("\"{}\"", route.route_path))
            .collect::<Vec<_>>()
            .join(","),
        entries.into_iter().collect::<Vec<_>>().join(",")
    )
}

fn android_translation_artifacts(catalog: &TranslationCatalog) -> Vec<AndroidArtifact> {
    catalog
        .locales
        .iter()
        .map(|locale| {
            let directory = if Some(locale.locale.as_str()) == catalog.default_locale.as_deref() {
                "values".to_string()
            } else {
                format!("values-{}", locale.locale)
            };
            AndroidArtifact {
                relative_path: PathBuf::from(format!(
                    "apps/android/app/src/main/res/{directory}/strings.xml"
                )),
                content: android_strings_xml(locale),
                kind: AndroidArtifactKind::Localization,
                target: "android",
            }
        })
        .collect()
}

fn android_strings_xml(locale: &dowe_components::TranslationLocale) -> String {
    let values = locale
        .values
        .iter()
        .map(|value| {
            format!(
                "    <string name=\"{}\">{}</string>",
                translation_resource_name(&value.key),
                escape_android_xml(&value.value)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("<resources>\n{values}\n</resources>\n")
}

fn escape_android_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "\\'")
}

fn android_environment(environment: &[(String, String)]) -> String {
    let mut values = environment
        .iter()
        .map(|(name, value)| format!("    const val {} = \"{}\"", name, escape_kotlin(value)))
        .collect::<Vec<_>>();
    if !environment.iter().any(|(name, _)| name == "BACKEND_URL") {
        values.push("    const val BACKEND_URL = \"\"".to_string());
    }
    let values = values.join("\n");
    format!(
        r#"package dev.dowe.generated

object DoweEnvironment {{
{values}
}}
"#
    )
}

fn generated_views_index() -> String {
    "package dev.dowe.generated\n".to_string()
}

fn android_routing(routes: &[ViewRoute]) -> String {
    let route_paths = routes
        .iter()
        .map(|route| format!("    \"{}\",", route.route_path))
        .collect::<Vec<_>>()
        .join("\n");
    let initial = routes_first_path(routes);
    let deep_links = routes
        .iter()
        .map(|route| {
            format!(
                "    \"dowe-dev://generated{}\",",
                if route.route_path == "/" {
                    "/"
                } else {
                    route.route_path.as_str()
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let sections = routes
        .iter()
        .map(|route| {
            let values = route
                .sections
                .iter()
                .map(|section| format!("\"{}\"", escape_kotlin(&section.id)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("    \"{}\" to listOf({values}),", route.route_path)
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"package dev.dowe.generated

object DoweRoutes {{
    const val initialPath = "{initial}"
    val paths = listOf(
{route_paths}
    )
    val sections: Map<String, List<String>> = mapOf(
{sections}
    )
    val deepLinks = listOf(
{deep_links}
    )
}}
"#
    )
}

fn routes_first_path(routes: &[ViewRoute]) -> &str {
    routes
        .first()
        .map(|route| route.route_path.as_str())
        .unwrap_or("/")
}

fn android_layouts() -> String {
    r#"package dev.dowe.generated

import androidx.compose.runtime.Composable

@Composable
fun DoweLayoutBoundary(content: @Composable () -> Unit) {
    content()
}
"#
    .to_string()
}

