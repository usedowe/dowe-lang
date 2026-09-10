fn path_unit(path: &str) -> Option<&'static str> {
    let path = path.trim_start_matches("./");
    if path == "theme.dowe" {
        return Some("theme");
    }
    if path == "main.dowe" || path == ".gitignore" || path.starts_with(".env") {
        return Some("core/configuration");
    }
    if path == "readme.md" || path.starts_with("docs/") {
        return Some("core");
    }
    for (prefix, unit) in [
        ("views/layouts/", "views/layouts"),
        ("views/pages/", "views/pages"),
        ("views/components/", "views/components"),
        ("views/requests/", "views/requests"),
        ("server/entities/", "server/entities"),
        ("server/handlers/", "server/handlers"),
        ("server/functions/", "server/functions"),
        ("server/routes/", "server/routes"),
    ] {
        if path.starts_with(prefix) {
            return Some(unit);
        }
    }
    None
}

