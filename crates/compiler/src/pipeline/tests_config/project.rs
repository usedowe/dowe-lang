#[test]
fn compiles_example_project_and_writes_chunks() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());

    let project = compile_dev(temp.path()).expect("project");

    assert_eq!(project.backend.port, 8080);
    assert!(project.backend.has_websocket("/ws"));
    assert_eq!(project.web.pages[0].route_path, "/");
    assert_eq!(project.web.pages[0].layout_text, "Layout");
    assert_eq!(project.web.pages[0].page_text, "Login");
    assert!(
        project.web.pages[0]
            .body_html
            .contains(">Layout</p>")
    );
    assert!(
        project.web.pages[0]
            .body_html
            .contains(">Login</p>")
    );
    assert!(
        project.web.pages[0]
            .body_html
            .contains(r#"data-dowe-boundary="page:"#)
    );
    assert!(temp.path().join(".dowe/web/chunks/layouts").exists());
    assert!(temp.path().join(".dowe/web/chunks/pages").exists());
    assert!(temp.path().join(".dowe/web/index.html").exists());
    assert!(temp.path().join(".dowe/web/pages/index.html").exists());
    let manifest =
        fs::read_to_string(temp.path().join(".dowe/web/manifest.json")).expect("manifest");
    assert!(manifest.contains(r#""path":"/""#));
    assert!(manifest.contains(r#""staticFile":"web/pages/index.html""#));
    assert!(manifest.contains(r#""cssChunks""#));
    assert_eq!(project.web.chunks.len(), 2);
}

#[test]
fn compiles_database_registered_in_main_without_route_usage() {
    let temp = TempDir::new().expect("tempdir");
    fs::write(
        temp.path().join("server-config.dowe"),
        r#"database RegisteredDb provider:"dowe" host:"127.0.0.1" port:4147 account:"docs" secret:"secret" name:"registered" entities:[] seeders:[]"#,
    )
    .expect("database config");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import RegisteredDb from "@/server-config"

main
  server port:8080
    databases:[RegisteredDb]
    route "/health"
      response text:"ok""#,
    )
    .expect("registered main");

    let project = compile_dev(temp.path()).expect("project");

    assert_eq!(project.databases.len(), 1);
    assert_eq!(project.databases[0].binding, "RegisteredDb");
    assert_eq!(project.databases[0].connection.database, "registered");
}

#[test]
fn compiles_views_without_a_server() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  app name:"Dowe Ui" bundle:"dev.dowe.examples.ui"
  views:viewRoutes"#,
    )
    .expect("main");

    let project = compile_dev(temp.path()).expect("project");

    assert!(!project.capabilities.server);
    assert!(project.capabilities.views);
    assert_eq!(project.web.pages.len(), 1);
    assert!(temp.path().join(".dowe/web/index.html").is_file());
}

#[test]
fn server_compilation_ignores_view_configuration() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  design defaultTheme:"light"
    theme name:"light"
      colors:
        primary:"#000000""##,
    )
    .expect("invalid theme");

    let project = compile_for_server_environment(temp.path(), CompileEnvironment::Live)
        .expect("server project");

    assert!(project.capabilities.server);
    assert!(project.web.pages.is_empty());
    assert!(!temp.path().join(".dowe/web/index.html").exists());
    assert!(compile_for_environment(temp.path(), CompileEnvironment::Live).is_err());
}

#[test]
fn web_compilation_ignores_server_configuration() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    let main = fs::read_to_string(temp.path().join("main.dowe")).expect("main");
    fs::write(
        temp.path().join("main.dowe"),
        main.replace("server port:8080", "server port:\"invalid\""),
    )
    .expect("invalid server");

    let project = compile_for_web_environment(temp.path(), CompileEnvironment::Live)
        .expect("web project");

    assert!(project.capabilities.views);
    assert_eq!(project.web.pages.len(), 1);
    assert!(compile_for_environment(temp.path(), CompileEnvironment::Live).is_err());
}

#[test]
fn rejects_component_prop_on_view_routes() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("routes/view.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import loginPage from "../pages/login"

views viewRoutes
  group path:"/" layout:AuthLayout
    route path:"" component:loginPage"#,
    )
    .expect("views");

    let error = compile_dev(temp.path()).expect_err("component route prop");

    assert!(
        error
            .to_string()
            .contains("`route` does not support `component`")
    );
}

#[test]
fn compiles_multiple_view_and_endpoint_modules_in_declared_order() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("pages/login.dowe"),
        r#"page loginPage
  Box
    Text
      "Login"
    Button href:"/docs"
      "Docs""#,
    )
    .expect("login");
    fs::write(
        temp.path().join("pages/docs.dowe"),
        r#"page docsPage
  Text
    "Docs""#,
    )
    .expect("docs page");
    fs::write(
        temp.path().join("routes/site.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import loginPage from "../pages/login"

views siteRoutes
  group path:"/" layout:AuthLayout
    route path:"" page:loginPage"#,
    )
    .expect("site routes");
    fs::write(
        temp.path().join("routes/docs.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import docsPage from "../pages/docs"

views docsRoutes
  group path:"/" layout:AuthLayout
    route path:"docs" page:docsPage"#,
    )
    .expect("docs routes");
    fs::write(
        temp.path().join("routes/users.dowe"),
        r#"endpoints userRoutes
  get path:"/api/users"
    return text:"users""#,
    )
    .expect("user routes");
    fs::write(
        temp.path().join("routes/blogs.dowe"),
        r#"endpoints blogRoutes
  get path:"/api/blogs"
    return text:"blogs""#,
    )
    .expect("blog routes");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import siteRoutes from "@/routes/site"
import docsRoutes from "@/routes/docs"
import userRoutes from "@/routes/users"
import blogRoutes from "@/routes/blogs"

main
  views:[siteRoutes, docsRoutes]
  server port:8080
    endpoints:[userRoutes, blogRoutes]
  desktop
    server port:4500
      endpoints:[blogRoutes, userRoutes]"#,
    )
    .expect("main");

    let project = compile_dev(temp.path()).expect("project");

    assert_eq!(
        project
            .web
            .pages
            .iter()
            .map(|page| page.route_path.as_str())
            .collect::<Vec<_>>(),
        vec!["/", "/docs"]
    );
    assert_eq!(
        project
            .backend
            .endpoints
            .iter()
            .map(|endpoint| endpoint.path.as_str())
            .collect::<Vec<_>>(),
        vec!["/api/users", "/api/blogs"]
    );
    assert_eq!(
        project
            .desktop_server
            .expect("desktop server")
            .endpoints
            .iter()
            .map(|endpoint| endpoint.path.as_str())
            .collect::<Vec<_>>(),
        vec!["/api/blogs", "/api/users"]
    );
}

#[test]
fn rejects_empty_and_repeated_route_module_lists() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("main.dowe"),
        r#"main
  views:[]"#,
    )
    .expect("empty views");

    let empty_error = compile_dev(temp.path()).expect_err("empty list error");
    assert!(empty_error.to_string().contains("must not be empty"));

    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:["viewRoutes"]"#,
    )
    .expect("invalid views value");

    let invalid_error = compile_dev(temp.path()).expect_err("invalid list value error");
    assert!(
        invalid_error
            .to_string()
            .contains("list values must be imported symbols")
    );

    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:[viewRoutes, viewRoutes]"#,
    )
    .expect("repeated views");

    let repeated_error = compile_dev(temp.path()).expect_err("repeated list error");
    assert!(
        repeated_error
            .to_string()
            .contains("duplicate views reference `viewRoutes`")
    );

    fs::write(
        temp.path().join("main.dowe"),
        r#"main
  server port:8080
    endpoints:[]"#,
    )
    .expect("empty endpoints");

    let endpoint_error = compile_dev(temp.path()).expect_err("empty endpoint list error");
    assert!(endpoint_error.to_string().contains("must not be empty"));

    fs::write(
        temp.path().join("routes/api.dowe"),
        r#"endpoints apiRoutes
  get path:"/api/status"
    return text:"OK""#,
    )
    .expect("api routes");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import apiRoutes from "@/routes/api"

main
  server port:8080
    endpoints:[apiRoutes, apiRoutes]"#,
    )
    .expect("repeated endpoints");

    let repeated_endpoint_error =
        compile_dev(temp.path()).expect_err("repeated endpoint list error");
    assert!(
        repeated_endpoint_error
            .to_string()
            .contains("duplicate endpoints reference `apiRoutes`")
    );
}

#[test]
fn compiles_server_without_views_and_removes_stale_view_outputs() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    compile_dev(temp.path()).expect("views project");
    fs::write(
        temp.path().join("main.dowe"),
        r#"main
  app name:"Dowe Api" bundle:"dev.dowe.examples.api"
  server port:8080
    route "/api/status"
      response text:"OK""#,
    )
    .expect("main");

    let project = compile_dev(temp.path()).expect("project");

    assert!(project.capabilities.server);
    assert!(!project.capabilities.views);
    assert!(project.web.pages.is_empty());
    assert!(!temp.path().join(".dowe/web").exists());
    assert!(!temp.path().join(".dowe/apps").exists());
}

