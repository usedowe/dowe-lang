use super::source_parser::parse_source_file;
use std::path::Path;

#[test]
fn parses_indented_component_tree() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/src/pages/login.dowe"),
        "page loginPage\n  Box p:{ xs:4 md:8 }\n    Text size:\"md\"\n      \"Login\"\n"
            .to_string(),
    )
    .expect("source");

    assert_eq!(file.nodes[0].name, "page");
    assert_eq!(file.nodes[0].children[0].name, "Box");
    assert_eq!(file.nodes[0].children[0].props[0].name, "p");
}

#[test]
fn rejects_tabs_in_indentation() {
    let error = parse_source_file(
        Path::new("/project"),
        Path::new("/project/src/pages/login.dowe"),
        "page loginPage\n\tBox\n".to_string(),
    )
    .expect_err("error");

    assert!(error.to_string().contains("tabs are not valid indentation"));
}

#[test]
fn rejects_duplicate_props() {
    let error = parse_source_file(
        Path::new("/project"),
        Path::new("/project/src/pages/login.dowe"),
        "page loginPage\n  Box p:4 p:8\n".to_string(),
    )
    .expect_err("error");

    assert!(error.to_string().contains("duplicate prop `p`"));
}

#[test]
fn rejects_unquoted_import_paths() {
    let error = parse_source_file(
        Path::new("/project"),
        Path::new("/project/src/views.dowe"),
        "import AuthLayout from ./layouts/auth\n".to_string(),
    )
    .expect_err("error");

    assert!(error.to_string().contains("import path must be a string"));
}
