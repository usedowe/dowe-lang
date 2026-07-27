include!("web/artifacts.rs");
include!("web/routes_and_css_collection.rs");
include!("web/css_rules.rs");
include!("web/html_rendering.rs");
include!("web/html_classes_and_escape.rs");

#[cfg(test)]
mod tests {
    include!("web/tests.rs");
}
