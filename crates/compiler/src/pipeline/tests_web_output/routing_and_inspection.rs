#[test]
fn scopes_design_capability_chunks_to_routes() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Box
    Text
      "Login""#,
    );
    fs::write(
        temp.path().join("pages/form.dowe"),
        r#"page formPage
  Box
    Text
      "Form"
    Input label:"Email""#,
    )
    .expect("form page");
    fs::write(
        temp.path().join("routes/view.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import loginPage from "../pages/login"
import formPage from "../pages/form"

views viewRoutes
  group path:"/" layout:AuthLayout
    route path:"" page:loginPage
    route path:"form" page:formPage"#,
    )
    .expect("routes");

    let project = compile_dev(temp.path()).expect("project");
    let basic = project
        .web
        .pages
        .iter()
        .find(|page| page.route_path == "/")
        .expect("basic route");
    let form = project
        .web
        .pages
        .iter()
        .find(|page| page.route_path == "/form")
        .expect("form route");

    assert!(
        basic
            .css_chunks
            .iter()
            .all(|path| !path.starts_with("chunks/design/"))
    );
    let form_style = form
        .css_chunks
        .iter()
        .find(|path| path.starts_with("chunks/design/forms-"))
        .expect("form style capability");
    assert_eq!(form.css_chunks.first(), Some(form_style));
    assert!(temp.path().join(".dowe/web").join(form_style).is_file());
    assert!(
        fs::read_to_string(temp.path().join(".dowe/web/manifest.json"))
            .expect("manifest")
            .contains(form_style)
    );
}

#[test]
fn emits_view_inspector_only_for_development_web_output() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    Text
      "Header"
    children"#,
        r#"page loginPage
  Section id:"login"
    CardContent"#,
    );
    fs::create_dir_all(temp.path().join("components")).expect("components");
    fs::write(
        temp.path().join("components/card-content.dowe"),
        r#"component CardContent
  Box
    Button
      "Login"
    Title
      "Welcome""#,
    )
    .expect("component");
    fs::write(
        temp.path().join("pages/login.dowe"),
        r#"import CardContent from "../components/card-content"

page loginPage
  Section id:"login"
    CardContent
    Empty type:"result" title:"No records" href:"/" actionLabel:"Open""#,
    )
    .expect("page import");

    let development = compile_dev(temp.path()).expect("development project");
    let development_body = &development.web.pages[0].body_html;
    assert!(development_body.contains("data-dowe-node=\"dn_"));
    assert!(development
        .web
        .chunks
        .iter()
        .any(|chunk| chunk.inspector.is_some()));
    let inspector = fs::read_to_string(temp.path().join(".dowe/web/inspector.json"))
        .expect("development inspector manifest");
    assert!(inspector.contains("\"version\":2"));
    assert!(inspector.contains("\"path\":\"pages/login.dowe\""));
    assert!(inspector.contains("\"path\":\"components/card-content.dowe\""));
    assert!(inspector.contains("\"usages\":[{\"path\":\"pages/login.dowe\""));
    assert!(inspector.contains("\"props\":[{\"name\":\"id\",\"value\":\"\\\"login\\\"\"}]"));
    assert!(inspector.contains("\"routes\":[{\"path\":\"/\""));
    assert!(inspector.contains("\"breakpoints\":[{\"name\":\"xs\",\"minWidth\":0}"));
    assert!(inspector.contains("\"startLine\":"));
    let inspector_value: serde_json::Value =
        serde_json::from_str(&inspector).expect("inspector json");
    let kinds = inspector_value["nodes"]
        .as_array()
        .expect("inspector nodes")
        .iter()
        .map(|node| node["kind"].as_str().expect("inspector kind"))
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            "Box", "Text", "Section", "Box", "Button", "Button", "Title", "Empty",
        ]
    );
    let marker_count = development_body.matches("data-dowe-node=").count();
    let inspector_node_count = development
        .web
        .chunks
        .iter()
        .filter_map(|chunk| chunk.inspector.as_ref())
        .map(|map| map.nodes.len())
        .sum::<usize>();
    assert_eq!(marker_count, inspector_node_count);
    let first_node = inspector_value["nodes"]
        .as_array()
        .expect("inspector nodes")
        .first()
        .expect("first inspector node");
    assert!(first_node["signals"].is_array());
    assert!(first_node["actions"].is_array());
    assert!(inspector_value["routes"].as_array().is_some());

    let live = compile_for_environment(temp.path(), CompileEnvironment::Live)
        .expect("live project");
    assert!(!live.web.pages[0].body_html.contains("data-dowe-node="));
    assert!(live.web.chunks.iter().all(|chunk| chunk.inspector.is_none()));
    assert!(!temp.path().join(".dowe/web/inspector.json").exists());
}

#[test]
fn emits_server_inspector_only_for_development_server_output() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());

    let development = compile_dev(temp.path()).expect("development project");
    assert!(development.server_inspector.is_some());
    let manifest = temp.path().join(".dowe/server/inspector.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest).expect("server inspector manifest"))
            .expect("server inspector json");
    assert_eq!(value["schema_version"], 2);
    assert_eq!(value["port"], 8080);
    assert!(value["routes"].as_array().is_some_and(|routes| !routes.is_empty()));
    assert!(value["nodes"].as_array().is_some_and(|nodes| !nodes.is_empty()));

    let live = compile_for_environment(temp.path(), CompileEnvironment::Live)
        .expect("live project");
    assert!(live.server_inspector.is_none());
    assert!(!manifest.exists());
}

#[test]
fn composes_and_emits_web_only_route_metadata() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  meta name:"title" content:"Dowe"
  meta name:"description" content:"Default & reliable"
  meta name:"canonical" content:"https://dowe.dev/?ref=docs&kind=web"
  meta name:"og:image" content:"https://dowe.dev/og.png"
  meta name:"twitter:card" content:"summary_large_image"
  Box
    children"#,
        r#"page loginPage
  meta name:"title" content:"Views < Dowe"
  meta name:"description" content:"Compose web views."
  Box
    Text
      "Login""#,
    );
    fs::write(
        temp.path().join("pages/inherited.dowe"),
        "page inheritedPage\n  Text\n    \"Inherited\"",
    )
    .expect("inherited page");
    fs::write(
        temp.path().join("layouts/other.dowe"),
        "layout OtherLayout\n  meta name:\"title\" content:\"Other layout\"\n  Box\n    children",
    )
    .expect("other layout");
    fs::write(
        temp.path().join("pages/other.dowe"),
        "page otherPage\n  Text\n    \"Other\"",
    )
    .expect("other page");
    fs::write(
        temp.path().join("routes/view.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import OtherLayout from "../layouts/other"
import loginPage from "../pages/login"
import inheritedPage from "../pages/inherited"
import otherPage from "../pages/other"

views viewRoutes
  group path:"/" layout:AuthLayout
    route path:"" page:loginPage
    route path:"inherited" page:inheritedPage
  group path:"/other" layout:OtherLayout
    route path:"" page:otherPage"#,
    )
    .expect("metadata routes");

    let project = compile_dev(temp.path()).expect("project");
    let page = project
        .web
        .pages
        .iter()
        .find(|page| page.route_path == "/")
        .expect("root page");
    assert_eq!(page.metadata[0].content, "Views < Dowe");
    assert_eq!(page.metadata[1].content, "Compose web views.");
    assert!(
        page.html_document
            .contains("<title data-dowe-meta>Views &lt; Dowe</title>")
    );
    assert!(
        page.html_document
            .contains(r#"<meta data-dowe-meta name="description" content="Compose web views.">"#)
    );
    assert!(page.html_document.contains(
        r#"<link data-dowe-meta rel="canonical" href="https://dowe.dev/?ref=docs&amp;kind=web">"#
    ));
    assert!(page.html_document.contains(
        r#"<meta data-dowe-meta property="og:image" content="https://dowe.dev/og.png">"#
    ));
    assert!(
        page.html_document
            .contains(r#"<meta data-dowe-meta name="twitter:card" content="summary_large_image">"#)
    );
    assert!(
        dowe_generator_web::manifest(&project.web)
            .contains(r#""metadata":[{"name":"title","content":"Views < Dowe"}"#)
    );
    assert!(
        project
            .web
            .router_js
            .contains("function applyRouteMetadata(route)")
    );
    assert!(project.web.router_js.contains("applyRouteMetadata(route)"));
    let inherited = project
        .web
        .pages
        .iter()
        .find(|page| page.route_path == "/inherited")
        .expect("inherited route");
    assert_eq!(inherited.metadata[0].content, "Dowe");
    assert_eq!(inherited.metadata[1].content, "Default & reliable");
    assert!(inherited.html_document.contains("Default &amp; reliable"));
    let changed_layout = project
        .web
        .pages
        .iter()
        .find(|page| page.route_path == "/other")
        .expect("changed layout route");
    assert_eq!(changed_layout.metadata[0].content, "Other layout");
    assert!(
        project
            .desktop_web
            .pages
            .iter()
            .all(|page| page.metadata.is_empty())
    );
    assert!(
        project
            .desktop_web
            .pages
            .iter()
            .all(|page| !page.html_document.contains("Views &lt; Dowe"))
    );
    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android pages");
    let ios =
        fs::read_to_string(temp.path().join(".dowe/apps/ios/DowePages.swift")).expect("ios pages");
    assert!(!android.contains("Compose web views."));
    assert!(!ios.contains("Compose web views."));
}

