use super::source_parser::parse_source_file;
use std::path::Path;

#[test]
fn parses_indented_component_tree() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/pages/login.dowe"),
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
        Path::new("/project/pages/login.dowe"),
        "page loginPage\n\tBox\n".to_string(),
    )
    .expect_err("error");

    assert!(error.to_string().contains("tabs are not valid indentation"));
}

#[test]
fn rejects_duplicate_props() {
    let error = parse_source_file(
        Path::new("/project"),
        Path::new("/project/pages/login.dowe"),
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

#[test]
fn parses_multiple_imports_from_one_module() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/server/api.dowe"),
        "import listBlogs, createBlog from \"../handlers/blogs\"\nimport { readBlog, updateBlog } from \"../handlers/blogs\"\n"
            .to_string(),
    )
    .expect("source");

    assert_eq!(
        file.imports
            .iter()
            .map(|import| import.local.as_str())
            .collect::<Vec<_>>(),
        ["listBlogs", "createBlog", "readBlog", "updateBlog"]
    );
    assert!(
        file.imports
            .iter()
            .all(|import| import.path == "../handlers/blogs")
    );
}

#[test]
fn rejects_empty_or_duplicate_multiple_imports() {
    for source in [
        "import {} from \"../handlers/blogs\"\n",
        "import listBlogs, listBlogs from \"../handlers/blogs\"\n",
        "import { listBlogs, } from \"../handlers/blogs\"\n",
    ] {
        let error = parse_source_file(
            Path::new("/project"),
            Path::new("/project/server/api.dowe"),
            source.to_string(),
        )
        .expect_err("invalid import");
        assert!(error.to_string().contains("invalid import syntax"));
    }
}

#[test]
fn parses_multiline_property_suites_and_structured_values() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/pages/canvas.dowe"),
        "page canvasPage\n  signal gameScene:\n    value:[\n      {\n        type:\"rect\"\n        x:0\n        motion:{\n          vx:-18\n          wrap:true\n        }\n      },\n    ]\n  Canvas:\n    scene:gameScene\n    viewWidth:640\n    label:\"Animated scene\"\n"
            .to_string(),
    )
    .expect("multiline source");

    let signal = &file.nodes[0].children[0];
    let canvas = &file.nodes[0].children[1];
    assert_eq!(signal.name, "signal");
    assert_eq!(signal.props.len(), 1);
    assert!(matches!(
        signal.props[0].value,
        super::SourceValue::Array(_)
    ));
    assert_eq!(canvas.name, "Canvas");
    assert_eq!(canvas.props.len(), 3);
    assert_eq!(canvas.props[0].name, "scene");
}

#[test]
fn multiline_property_suite_matches_inline_ast() {
    let inline = parse_source_file(
        Path::new("/project"),
        Path::new("/project/pages/canvas.dowe"),
        "page canvasPage\n  Canvas scene:gameScene viewWidth:640 label:\"Animated scene\"\n"
            .to_string(),
    )
    .expect("inline source");
    let multiline = parse_source_file(
        Path::new("/project"),
        Path::new("/project/pages/canvas.dowe"),
        "page canvasPage\n  Canvas:\n    scene:gameScene\n    viewWidth:640\n    label:\"Animated scene\"\n"
            .to_string(),
    )
    .expect("multiline source");

    let inline_canvas = &inline.nodes[0].children[0];
    let multiline_canvas = &multiline.nodes[0].children[0];
    assert_eq!(inline_canvas.name, multiline_canvas.name);
    assert_eq!(inline_canvas.args, multiline_canvas.args);
    assert_eq!(
        inline_canvas
            .props
            .iter()
            .map(|prop| (&prop.name, &prop.value))
            .collect::<Vec<_>>(),
        multiline_canvas
            .props
            .iter()
            .map(|prop| (&prop.name, &prop.value))
            .collect::<Vec<_>>()
    );
}

#[test]
fn parses_property_suite_child_after_parent_suite_props() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/pages/hero.dowe"),
        "page heroPage\n  Grid:\n    columns:2\n    gap:4\n    Box show:false\n    Box:\n      cover:\"/hero.jpg\"\n      minH:96\n"
            .to_string(),
    )
    .expect("nested property suite");

    let grid = &file.nodes[0].children[0];
    let box_node = &grid.children[1];
    assert_eq!(grid.name, "Grid");
    assert_eq!(grid.props.len(), 2);
    assert_eq!(grid.children.len(), 2);
    assert_eq!(box_node.name, "Box");
    assert_eq!(box_node.props.len(), 2);
}

#[test]
fn rejects_multiple_props_on_property_suite_line() {
    let error = parse_source_file(
        Path::new("/project"),
        Path::new("/project/pages/canvas.dowe"),
        "page canvasPage\n  Canvas:\n    scene:gameScene viewWidth:640\n".to_string(),
    )
    .expect_err("invalid suite");

    assert!(
        error
            .to_string()
            .contains("property suites require one prop per line")
    );
}

#[test]
fn rejects_duplicate_property_suite_props() {
    let error = parse_source_file(
        Path::new("/project"),
        Path::new("/project/pages/canvas.dowe"),
        "page canvasPage\n  Canvas:\n    scene:gameScene\n    scene:otherScene\n".to_string(),
    )
    .expect_err("duplicate prop");

    assert!(error.to_string().contains("duplicate prop `scene`"));
}

#[test]
fn rejects_inline_props_on_property_suite_header() {
    let error = parse_source_file(
        Path::new("/project"),
        Path::new("/project/pages/canvas.dowe"),
        "page canvasPage\n  Canvas scene:gameScene:\n    label:\"Animated scene\"\n".to_string(),
    )
    .expect_err("inline suite prop");

    assert!(
        error
            .to_string()
            .contains("property suite headers cannot contain inline props")
    );
}

#[test]
fn rejects_property_suites_on_type_declarations() {
    let error = parse_source_file(
        Path::new("/project"),
        Path::new("/project/types/user.dowe"),
        "type User:\n  name:string\n".to_string(),
    )
    .expect_err("type suite");

    assert!(
        error
            .to_string()
            .contains("`type` declarations do not accept property suites")
    );
}

#[test]
fn rejects_property_suite_props_after_children() {
    let error = parse_source_file(
        Path::new("/project"),
        Path::new("/project/pages/card.dowe"),
        "page cardPage\n  Card:\n    variant:\"soft\"\n    Text\n      \"Status\"\n    p:4\n"
            .to_string(),
    )
    .expect_err("late prop");

    assert!(
        error
            .to_string()
            .contains("property suite props must appear before child nodes")
    );
}

#[test]
fn rejects_unclosed_multiline_structured_value() {
    let error = parse_source_file(
        Path::new("/project"),
        Path::new("/project/pages/canvas.dowe"),
        "page canvasPage\n  signal scene:\n    value:[\n      { type:\"circle\" },\n".to_string(),
    )
    .expect_err("unclosed value");

    assert!(error.to_string().contains("unclosed structured value"));
}
