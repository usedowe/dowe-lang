fn generated_views_index() -> String {
    "import SwiftUI\n".to_string()
}

fn ios_routing(routes: &[ViewRoute]) -> String {
    let route_paths = routes
        .iter()
        .map(|route| format!("        \"{}\",", route.route_path))
        .collect::<Vec<_>>()
        .join("\n");
    let initial = routes_first_path(routes);
    let deep_links = routes
        .iter()
        .map(|route| {
            format!(
                "        \"dowe-dev://generated{}\",",
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
                .map(|section| format!("\"{}\"", escape_swift(&section.id)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("        \"{}\": [{values}],", route.route_path)
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"import SwiftUI

enum DoweRoutes {{
    static let initialPath = "{initial}"
    static let paths = [
{route_paths}
    ]
    static let sections: [String: [String]] = [
{sections}
    ]
    static let deepLinks = [
{deep_links}
    ]
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

