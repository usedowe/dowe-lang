use super::{
    compile_dev, compile_for_environment, compile_for_server_environment,
    compile_for_web_environment,
};
use crate::model::{
    CompileEnvironment, EndpointBehavior, EnvironmentValueSource, EnvironmentVisibility,
    HttpMethod, ServerLogLevel, ServerLogValue, ServerStatement,
};
use crate::parser::validate_design_copilot_dowe;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn generated_css_chunk<'a>(paths: &'a [String], prefix: &str) -> &'a str {
    paths
        .iter()
        .find(|path| path.starts_with(prefix))
        .map(String::as_str)
        .expect("generated css chunk")
}

fn android_dev_output(root: &Path) -> String {
    let source_root = root.join(".dowe/apps/android/dev/src/dev/dowe/generated");
    let core = fs::read_to_string(source_root.join("DoweDevActivity.java"))
        .expect("android dev activity");
    let mut output = core
        .lines()
        .map(|line| {
            if let Some(declaration) = line.strip_prefix("    ")
                && !declaration.starts_with(' ')
            {
                format!("    private {declaration}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    output.push('\n');
    let mut shards = fs::read_dir(&source_root)
        .expect("android dev sources")
        .map(|entry| entry.expect("android dev source").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    (name.starts_with("DoweDevRoute") || name.starts_with("DoweDevLayout"))
                        && name.ends_with(".java")
                })
        })
        .collect::<Vec<_>>();
    shards.sort();
    for path in shards {
        output.push_str(
            &fs::read_to_string(path)
                .expect("android dev source")
                .replace(
                    "int viewportWidth = runtime.viewportWidth;",
                    "int viewportWidth = this.viewportWidth;",
                )
                .replace("runtime.", "")
                .replace("runtime", "this")
                .replace("DoweDevActivity.", ""),
        );
        output.push('\n');
    }
    output
}

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
            .contains(r#"<p class="dowe-text text-md">Layout</p>"#)
    );
    assert!(
        project.web.pages[0]
            .body_html
            .contains(r#"<p class="dowe-text text-md">Login</p>"#)
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

#[test]
fn compiles_i18n_catalogs_for_web_and_native_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Box
    Title i18n:"home.hero.title"
      "Dowe builds systems."
    NavMenu
      item label:"Home" i18n:"home.hero.title" description:"Start" descriptionI18n:"home.hero.title"
    Button i18n:"home.hero.title"
      "Save"
    SideNav
      item label:"Views" i18n:"home.hero.title" description:"Catalog" descriptionI18n:"home.hero.title" status:"Ready" statusI18n:"home.hero.title"
    Tabs
      tab id:"overview" label:"Overview" i18n:"home.hero.title"
        Text
          "Panel""#,
    );
    write_translation_catalogs(temp.path());

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;

    assert_eq!(project.translations.default_locale.as_deref(), Some("en"));
    assert_eq!(project.web.translation_chunks.len(), 2);
    assert!(body.contains(r#"data-dowe-i18n="home.hero.title""#));
    assert!(body.contains(r#"class="navmenu-label" data-text="Home" data-dowe-i18n="home.hero.title""#));
    assert!(body.contains(r#"class="sidenav-label" data-dowe-i18n="home.hero.title""#));
    assert!(body.contains(r#"class="tabs-label" data-dowe-i18n="home.hero.title""#));
    assert!(body.contains("Dowe builds systems."));
    assert!(project.web.router_js.contains("navigator.languages"));
    assert!(project.web.router_js.contains("hydrateTranslations"));
    assert!(
        project
            .web
            .translation_chunks
            .iter()
            .any(|chunk| chunk.content.contains("Dowe construye sistemas."))
    );

    let manifest =
        fs::read_to_string(temp.path().join(".dowe/web/manifest.json")).expect("manifest");
    assert!(manifest.contains(r#""translationChunks""#));
    assert!(manifest.contains(r#""defaultLocale":"en""#));
    assert!(
        temp.path()
            .join(".dowe/apps/desktop/web/chunks/i18n")
            .is_dir()
    );

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("stringResource(R.string.dowe_home_hero_title_"));
    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("getString(R.string.dowe_home_hero_title_"));
    let android_spanish = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/res/values-es/strings.xml"),
    )
    .expect("android spanish");
    assert!(android_spanish.contains("Dowe construye sistemas."));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains(r#"String(localized: "home.hero.title")"#));
    let ios_spanish = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/ios/es.lproj/Localizable.strings"),
    )
    .expect("ios spanish");
    assert!(ios_spanish.contains("Dowe construye sistemas."));
}

#[test]
fn compiles_platform_specific_view_routes() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::create_dir_all(temp.path().join("layouts")).expect("layouts");
    fs::create_dir_all(temp.path().join("pages")).expect("pages");
    fs::write(
        temp.path().join("layouts/marketing.dowe"),
        r#"layout MarketingLayout
  Box
    Text
      "Marketing"
    children"#,
    )
    .expect("marketing layout");
    fs::write(
        temp.path().join("pages/landing.dowe"),
        r#"page landingPage
  Box
    Text
      "Landing""#,
    )
    .expect("landing");
    fs::write(
        temp.path().join("routes/view.dowe"),
        r#"import AuthLayout from "@/layouts/auth"
import MarketingLayout from "@/layouts/marketing"
import loginPage from "@/pages/login"
import landingPage from "@/pages/landing"

views viewRoutes
  group path:"/" layout:MarketingLayout platform:"web"
    route path:"" page:landingPage
  group path:"/" layout:AuthLayout platform:["desktop","ios","android"]
    route path:"" page:loginPage"#,
    )
    .expect("views");

    let project = compile_dev(temp.path()).expect("project");

    assert_eq!(project.web.pages[0].layout_text, "Marketing");
    assert_eq!(project.web.pages[0].page_text, "Landing");
    assert_eq!(project.desktop_web.pages[0].layout_text, "Layout");
    assert_eq!(project.desktop_web.pages[0].page_text, "Login");
    assert_eq!(project.view_routes.web.len(), 1);
    assert_eq!(project.view_routes.desktop.len(), 1);
    assert_eq!(project.view_routes.android.len(), 1);
    assert_eq!(project.view_routes.ios.len(), 1);
    let web_index =
        fs::read_to_string(temp.path().join(".dowe/web/index.html")).expect("web index");
    let desktop_index = fs::read_to_string(temp.path().join(".dowe/apps/desktop/web/index.html"))
        .expect("desktop index");
    assert!(web_index.contains("Landing"));
    assert!(!web_index.contains("Login"));
    assert!(desktop_index.contains("Login"));
    assert!(!desktop_index.contains("Landing"));
    let manifest =
        fs::read_to_string(temp.path().join(".dowe/apps/manifest.json")).expect("manifest");
    assert!(manifest.contains(r#""routesByTarget""#));
    assert!(manifest.contains(r#""desktopWebManifest":"apps/desktop/web/manifest.json""#));
}

#[test]
fn rejects_overlapping_platform_route_paths() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("routes/view.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import loginPage from "../pages/login"

views viewRoutes
  group path:"/" layout:AuthLayout platform:["web","desktop"]
    route path:"" page:loginPage
  group path:"/" layout:AuthLayout platform:"web"
    route path:"" page:loginPage"#,
    )
    .expect("views");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(
        error
            .to_string()
            .contains("duplicate view path `/` for platform `web`")
    );
}

#[test]
fn rejects_invalid_platform_values() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("routes/view.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import loginPage from "../pages/login"

views viewRoutes
  group path:"/" layout:AuthLayout platform:["web","watch"]
    route path:"" page:loginPage"#,
    )
    .expect("views");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("got `watch`"));

    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("routes/view.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import loginPage from "../pages/login"

views viewRoutes
  group path:"/" layout:AuthLayout platform:web
    route path:"" page:loginPage"#,
    )
    .expect("views");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("quoted static string literal"));

    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("routes/view.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import loginPage from "../pages/login"

views viewRoutes
  group path:"/" layout:AuthLayout platform:["web","web"]
    route path:"" page:loginPage"#,
    )
    .expect("views");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("duplicate platform `web`"));
}

#[test]
fn writes_source_language_artifacts() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());

    compile_dev(temp.path()).expect("project");

    let source = fs::read_to_string(temp.path().join(".dowe/language/source-format.json"))
        .expect("source format");
    let server = fs::read_to_string(temp.path().join(".dowe/language/server-surface.json"))
        .expect("server surface");
    let views = fs::read_to_string(temp.path().join(".dowe/language/views-surface.json"))
        .expect("views surface");
    let config = fs::read_to_string(temp.path().join(".dowe/language/config-surface.json"))
        .expect("config surface");

    assert!(source.contains(r#""extension": ".dowe""#));
    assert!(source.contains(r#""declaredTypes""#));
    assert!(source.contains(r#""unsupportedAuthoringExtensions""#));
    assert!(source.contains(r#""packages": "rejected""#));
    assert!(source.contains(r#""moduleRoot": ".""#));
    assert!(source.contains(r#""projectRootAlias": "@/""#));
    assert!(source.contains(r#""assetsImportable": false"#));
    assert!(source.contains("dynamic text uses exactly one braced binding"));
    assert!(views.contains("main views:[dashboardRoutes docsRoutes]"));
    assert!(views.contains("server endpoints:[userRoutes,blogRoutes]"));
    assert!(server.contains(r#""root": "main.dowe""#));
    assert!(server.contains(r#""req.json""#));
    assert!(server.contains(r#""resolvedLogValues": true"#));
    assert!(server.contains(r#""const body:Type value:req.json""#));
    assert!(server.contains("functionName result args:{ input:value }"));
    assert!(!server.contains("let result = functionName"));
    assert!(server.contains(r#""nodeRuntime": false"#));
    assert!(views.contains(r#""root": "main.dowe""#));
    assert!(views.contains(r#""Box""#));
    assert!(views.contains(r#""Alert""#));
    assert!(views.contains("BottomBar tab href label Icon featured"));
    assert!(views.contains(r#""Svg""#));
    assert!(views.contains(r#""Path""#));
    assert!(views.contains(r#""Code""#));
    assert!(views.contains(r#""Video""#));
    assert!(views.contains(r#""Divider""#));
    assert!(views.contains("Section boxed:true"));
    assert!(views.contains("center:{ xs:false md:true }"));
    assert!(views.contains("gap:{ xs:2 md:4 }"));
    assert!(views.contains(r#""Input bind:signal.field""#));
    assert!(views.contains(r#""signalPathValidation""#));
    assert!(views.contains(r#"\"{blog.title}\" dynamic text child"#));
    assert!(views.contains(r#""signal rows type:Row[] value:[]""#));
    assert!(views.contains("any imported .dowe module"));
    assert!(views.contains("classified by store declaration"));
    assert!(!views.contains("views/store/**/*.dowe"));
    assert!(views.contains(r#""routing""#));
    assert!(views.contains("platform values"));
    assert!(views.contains(r#""metadata""#));
    assert!(views.contains(r#"meta name:\"title\" content:\"Page title\""#));
    assert!(views.contains("active layout then page by name"));
    assert!(views.contains("web SSR and browser routing only"));
    assert!(!views.contains(r#""Body""#));
    assert!(views.contains(r#""children""#));
    assert!(views.contains(r#""serverApisAvailable": false"#));
    assert!(config.contains(r#""themeRoot": "theme.dowe""#));
    assert!(config.contains(r#""envRoot": ".env""#));
    assert!(config.contains(r#""envExampleRoot": ".env.example""#));
    assert!(config.contains(r#""serverRoot": "main.dowe""#));
    assert!(config.contains(
        r#""obsoleteConfig": ["dowe.json", "env.dowe", "src/config.dowe", "src/main.dowe", "src/theme.dowe", "src/env.dowe"]"#
    ));
    assert!(config.contains(r#""defaultTheme": "light""#));
    assert!(config.contains(
        r##""declaration": "colors: -> primary color:\"#2563eb\" text:\"#ffffff\" title:\"#ffffff\"""##
    ));
    assert!(config.contains(r#""roles": ["color", "text", "title"]"#));
    assert!(config.contains(r#""flatRoleAuthoring": "rejected""#));
    assert!(config.contains(r#""fontSlots": ["text", "title"]"#));
    assert!(config.contains(r#""cors""#));
    assert!(config.contains(r#""devOrigins""#));
    assert!(!temp.path().join(".dowe/tsconfig.json").exists());
    assert!(!temp.path().join(".dowe/types").exists());
}

#[test]
fn rejects_root_dowe_json_configuration() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("dowe.json"),
        r#"{"fonts":{"default":"inter","install":["inter"]}}"#,
    )
    .expect("json config");

    let error = compile_dev(temp.path()).expect_err("error");
    let message = error.to_string();

    assert!(message.contains("dowe.json"));
    assert!(message.contains("theme.dowe"));
    assert!(message.contains("no longer supported"));
}

#[test]
fn rejects_legacy_server_dowe_entry() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::create_dir_all(temp.path().join("src")).expect("legacy source directory");
    fs::write(temp.path().join("src/server.dowe"), "main\n").expect("legacy server");

    let error = compile_dev(temp.path()).expect_err("error");
    let message = error.to_string();

    assert!(message.contains("src/server.dowe"));
    assert!(message.contains("main.dowe"));
}

#[test]
fn rejects_entry_and_configuration_files_under_src() {
    for file_name in ["main.dowe", "theme.dowe"] {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        fs::create_dir_all(temp.path().join("src")).expect("legacy source directory");
        fs::write(temp.path().join("src").join(file_name), "legacy\n").expect("legacy source");

        let error = compile_dev(temp.path()).expect_err("legacy location");
        let message = error.to_string();

        assert!(message.contains(&format!("src/{file_name}")));
        assert!(message.contains(&format!("project-root `{file_name}`")));
    }
}

#[test]
fn rejects_removed_environment_dowe_files() {
    for relative in ["env.dowe", "src/env.dowe"] {
        let temp = TempDir::new().expect("tempdir");
        write_fixture(temp.path());
        let path = temp.path().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent");
        }
        fs::write(&path, "env\n").expect("removed env source");

        let error = compile_dev(temp.path()).expect_err("removed env.dowe");
        let message = error.to_string();

        assert!(message.contains(relative));
        assert!(message.contains(".env"));
        assert!(message.contains("no longer supported"));
    }
}

#[test]
fn rejects_invalid_theme_dowe_theme() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  fonts default:"inter" install:["inter"]
  design defaultTheme:"dark"
    theme name:"light"
      colors:
        primary color:"#000000""##,
    )
    .expect("config");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(
        error
            .to_string()
            .contains("default theme `dark` is not declared")
    );
}

#[test]
fn accepts_grouped_theme_color_families_and_rejects_flat_roles() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  design defaultTheme:"light"
    theme name:"light"
      colors:
        primary color:"#1F3A5F" text:"#FFFFFF" title:"#FFFFFE"
        background color:"#FFFFFF" text:"#17263A" title:"#17263E"
        surface color:"#F7F9FC" text:"#17263A" title:"#17263E"
        softPrimary color:"#CCFBF3" text:"#073B35" title:"#073B35"
    theme name:"brand" extends:"light"
      colors:
        primary title:"#FFFEEE""##,
    )
    .expect("theme");

    compile_dev(temp.path()).expect("grouped theme color families");

    for flat in [
        "primary:\"#1F3A5F\"",
        "primaryText:\"#FFFFFF\"",
        "primaryTitle:\"#FFFFFE\"",
        "onPrimary:\"#FFFFFF\"",
        "onSuccess:\"#FFFFFF\"",
        "onSoftPrimary:\"#073B35\"",
    ] {
        fs::write(
            temp.path().join("theme.dowe"),
            format!(
                r##"theme
  design defaultTheme:"light"
    theme name:"light"
      colors:
        {flat}"##
            ),
        )
        .expect("flat theme");

        let message = compile_dev(temp.path())
            .expect_err("flat theme role")
            .to_string();
        assert!(message.contains("grouped color families"), "{message}");
    }

    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  design defaultTheme:"light"
    theme name:"light"
      colors:
        softBackground color:"#FFFFFF" text:"#17263A" title:"#17263E""##,
    )
    .expect("unknown family");
    let message = compile_dev(temp.path())
        .expect_err("unknown color family")
        .to_string();
    assert!(message.contains("unknown color family `softBackground`"), "{message}");
}

#[test]
fn compiles_custom_theme_color_families_for_all_view_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  Box\n    children",
        r#"page loginPage
  Card scheme:"happy"
    Title
      "Saved"
    Text
      "Your changes are ready."
  Card variant:"soft" scheme:"happy"
    Title
      "Gentle success""#,
    );
    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  design defaultTheme:"light"
    theme name:"light"
      colors:
        happy color:"#176c75" text:"#fffffe" title:"#fffefe"
        softHappy color:"#d9f3f1" text:"#124d53" title:"#124d53"
    theme name:"dark"
      colors:
        happy color:"#55c2cc" text:"#071e20" title:"#071e20""##,
    )
    .expect("theme");

    let project = compile_dev(temp.path()).expect("custom theme family");
    let happy = dowe_components::ColorFamily::from_name("happy").expect("happy family");
    let light = project.design_config.theme("light").expect("light theme");
    let dark = project.design_config.theme("dark").expect("dark theme");

    assert_eq!(light.color_value(happy.color_token()), "#176c75");
    assert_eq!(light.color_value(happy.text_token()), "#fffffe");
    assert_eq!(light.color_value(happy.title_token()), "#fffefe");
    assert_eq!(light.color_value(happy.soft_color_token()), "#d9f3f1");
    assert_eq!(dark.color_value(happy.color_token()), "#55c2cc");
    assert_eq!(dark.color_value(happy.soft_color_token()), "#d9f3f1");

    let body = &project.web.pages[0].body_html;
    assert!(body.contains("is-solid is-happy"), "{body}");
    assert!(body.contains("is-soft is-happy"), "{body}");

    let design_css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.design_file_name()),
    )
    .expect("design css");
    assert!(design_css.contains("--dowe-happy:#176c75;"), "{design_css}");
    assert!(design_css.contains("--dowe-happyText:#fffffe;"), "{design_css}");
    assert!(design_css.contains("--dowe-happyTitle:#fffefe;"), "{design_css}");
    assert!(design_css.contains("--dowe-softHappy:#d9f3f1;"), "{design_css}");
    assert!(
        design_css.contains("[data-dowe-theme=\"dark\"]{")
            && design_css.contains("--dowe-happy:#55c2cc;"),
        "{design_css}"
    );
    let page_css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(generated_css_chunk(
                &project.web.pages[0].css_chunks,
                "chunks/pages/",
            )),
    )
    .expect("page css");
    assert!(
        page_css.contains(".card.is-solid.is-happy")
            && page_css.contains("var(--dowe-happyText)")
            && page_css.contains("var(--dowe-happyTitle)"),
        "{page_css}"
    );
    assert!(
        page_css.contains(".card.is-soft.is-happy")
            && page_css.contains("var(--dowe-softHappyText)")
            && page_css.contains("var(--dowe-softHappyTitle)"),
        "{page_css}"
    );

    let android_theme = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DoweTheme.kt"),
    )
    .expect("android theme");
    assert!(android_theme.contains("\"happy\" to Color(0xFF176C75)"));
    assert!(android_theme.contains("\"softHappy\" to Color(0xFFD9F3F1)"));
    let android_pages = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android pages");
    assert!(android_pages.contains("DoweDesign.happy"), "{android_pages}");
    assert!(android_pages.contains("DoweDesign.happyText"), "{android_pages}");
    assert!(android_pages.contains("DoweDesign.happyTitle"), "{android_pages}");
    assert!(android_pages.contains("DoweDesign.softHappy"), "{android_pages}");
    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("DOWE_HAPPY"), "{android_dev}");
    assert!(android_dev.contains("DOWE_SOFT_HAPPY"), "{android_dev}");

    let ios_theme =
        fs::read_to_string(temp.path().join(".dowe/apps/ios/DoweTheme.swift")).expect("ios theme");
    assert!(ios_theme.contains("\"happy\": Color("), "{ios_theme}");
    assert!(ios_theme.contains("\"softHappy\": Color("), "{ios_theme}");
    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("DoweDesign.happy"), "{ios}");
    assert!(ios.contains("DoweDesign.happyText"), "{ios}");
    assert!(ios.contains("DoweDesign.happyTitle"), "{ios}");
    assert!(ios.contains("DoweDesign.softHappy"), "{ios}");
}

#[test]
fn rejects_undeclared_custom_scheme_and_missing_custom_soft_family() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  Box\n    children",
        "page loginPage\n  Card scheme:\"happy\"\n    Text\n      \"Missing\"",
    );
    let undeclared = match compile_dev(temp.path()) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("expected undeclared custom scheme to fail"),
    };
    assert!(undeclared.contains("happy"), "{undeclared}");

    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  design defaultTheme:"light"
    theme name:"light"
      colors:
        happy color:"#176c75" text:"#fffffe" title:"#fffffe""##,
    )
    .expect("theme");
    fs::write(
        temp.path().join("pages/login.dowe"),
        "page loginPage\n  Card variant:\"soft\" scheme:\"happy\"\n    Text\n      \"Missing soft\"",
    )
    .expect("page");
    let missing_soft = match compile_dev(temp.path()) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("expected missing custom soft family to fail"),
    };
    assert!(missing_soft.contains("softHappy"), "{missing_soft}");
}

#[test]
fn rejects_removed_slot_defaults_and_unknown_component_default_props() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  fonts default:"inter" install:["inter"]
  design defaultTheme:"light"
    radius panel:md
    theme name:"light""##,
    )
    .expect("config");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("`radius` is not valid inside `design`"));

    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  fonts default:"inter" install:["inter"]
  design defaultTheme:"light"
    Card columns:3
    theme name:"light""##,
    )
    .expect("config");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(
        error
            .to_string()
            .contains("`columns` is not a theme default prop for `Card`")
    );
}

#[test]
fn rejects_invalid_text_and_title_theme_defaults() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    for (line, expected) in [
        ("Text variant:\"soft\"", "`variant` is not a theme default prop for `Text`"),
        ("Title radius:\"md\"", "`radius` is not a theme default prop for `Title`"),
        ("Text font:\"arial\"", "unknown font token `arial` in `Text.font`"),
        ("Title font:\"Inter\"", "unknown font token `Inter` in `Title.font`"),
    ] {
        fs::write(
            temp.path().join("theme.dowe"),
            format!(
                "theme\n  design defaultTheme:\"light\"\n    {line}\n    theme name:\"light\""
            ),
        )
        .expect("theme");

        let error = compile_dev(temp.path()).expect_err("invalid text theme default");

        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn rejects_removed_theme_radius_surface() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  design defaultTheme:"light"
    theme name:"light" radius:10"#,
    )
    .expect("theme");

    assert!(
        compile_dev(temp.path())
            .expect_err("removed radius prop")
            .to_string()
            .contains("`radius` is not valid on `theme`")
    );

    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  design defaultTheme:"light"
    theme name:"light" radiusBox:12"#,
    )
    .expect("theme");
    assert!(
        compile_dev(temp.path())
            .expect_err("removed radius prop")
            .to_string()
            .contains("`radiusBox` is not valid on `theme`")
    );

    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  design defaultTheme:"light"
    theme name:"light"
      radii radius:10"#,
    )
    .expect("theme");
    assert!(
        compile_dev(temp.path())
            .expect_err("removed radii block")
            .to_string()
            .contains("`radii` is not valid inside `theme`")
    );
}

#[test]
fn rejects_unquoted_static_config_strings() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  fonts default:inter install:["inter"]"#,
    )
    .expect("config");

    let font_error = compile_dev(temp.path()).expect_err("font error");
    assert!(
        font_error
            .to_string()
            .contains("quoted static string literal")
    );

    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    cors target:server methods:["GET"]
    route "/api/status"
      response text:"OK""#,
    )
    .expect("config");

    let cors_error = compile_dev(temp.path()).expect_err("cors error");
    assert!(
        cors_error
            .to_string()
            .contains("quoted static string literal")
    );
}

#[test]
fn compiles_app_metadata_from_main() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    let main = fs::read_to_string(temp.path().join("main.dowe")).expect("main");
    fs::write(
        temp.path().join("main.dowe"),
        main.replace(
            "main\n  views:viewRoutes",
            "main\n  app name:\"Clinic Desk\" bundle:\"com.example.clinic\"\n  views:viewRoutes",
        ),
    )
    .expect("main");

    let project = compile_dev(temp.path()).expect("project");
    let apps_manifest =
        fs::read_to_string(temp.path().join(".dowe/apps/manifest.json")).expect("manifest");
    let android_gradle =
        fs::read_to_string(temp.path().join(".dowe/apps/android/app/build.gradle.kts"))
            .expect("android gradle");
    let android_manifest = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/dev/AndroidManifest.xml"),
    )
    .expect("android manifest");
    let android_activity = android_dev_output(temp.path());
    let ios_plist =
        fs::read_to_string(temp.path().join(".dowe/apps/ios/Info.plist")).expect("ios plist");
    let desktop_manifest = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/desktop/macos/dowe-desktop.json"),
    )
    .expect("desktop manifest");

    assert_eq!(project.app_config.name, "Clinic Desk");
    assert_eq!(project.app_config.bundle, "com.example.clinic");
    assert!(apps_manifest.contains(r#""name":"Clinic Desk""#));
    assert!(apps_manifest.contains(r#""bundle":"com.example.clinic""#));
    assert!(android_gradle.contains(r#"applicationId = "com.example.clinic""#));
    assert!(android_gradle.contains("create(\"release\")"));
    assert!(android_gradle.contains("DOWE_ANDROID_KEYSTORE"));
    assert!(android_gradle.contains("DOWE_APP_BUILD_NUMBER"));
    assert!(android_gradle.contains("DOWE_APP_VERSION"));
    assert!(android_gradle.contains("signingConfigs.getByName(\"release\")"));
    assert!(android_manifest.contains(r#"package="com.example.clinic""#));
    assert!(android_manifest.contains(r#"android:label="Clinic Desk""#));
    assert!(android_activity.contains("import com.example.clinic.R;"));
    assert!(ios_plist.contains("<string>Clinic Desk</string>"));
    assert!(ios_plist.contains("<string>com.example.clinic</string>"));
    assert!(desktop_manifest.contains(r#""title":"Clinic Desk""#));
}

#[test]
fn rejects_invalid_app_metadata() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    let main = fs::read_to_string(temp.path().join("main.dowe")).expect("main");
    fs::write(
        temp.path().join("main.dowe"),
        main.replace(
            "main\n  views:viewRoutes",
            "main\n  app name:\"\" bundle:\"example\"\n  views:viewRoutes",
        ),
    )
    .expect("main");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("app.name"));

    let main = fs::read_to_string(temp.path().join("main.dowe")).expect("main");
    fs::write(
        temp.path().join("main.dowe"),
        main.replace(
            "main\n  app name:\"\" bundle:\"example\"\n  views:viewRoutes",
            "main\n  app name:\"Example\" bundle:\"example\"\n  views:viewRoutes",
        ),
    )
    .expect("main");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("app.bundle"));
}

#[test]
fn replaces_stale_typecheck_artifacts_with_language_support() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());

    fs::create_dir_all(temp.path().join(".dowe/types")).expect("types");
    fs::write(temp.path().join(".dowe/tsconfig.json"), "stale").expect("root config");
    fs::write(temp.path().join(".dowe/types/views.d.ts"), "stale").expect("views types");

    compile_dev(temp.path()).expect("project");

    let first_source = fs::read_to_string(temp.path().join(".dowe/language/source-format.json"))
        .expect("source format");

    assert!(!temp.path().join(".dowe/tsconfig.json").exists());
    assert!(!temp.path().join(".dowe/types").exists());
    assert!(first_source.contains("dowe-source-format"));

    compile_dev(temp.path()).expect("project");

    let second_source = fs::read_to_string(temp.path().join(".dowe/language/source-format.json"))
        .expect("source format");

    assert_eq!(first_source, second_source);
}

#[test]
fn ignores_agents_as_application_source_and_output() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::create_dir_all(temp.path().join(".agents/pages")).expect("agents");
    fs::write(temp.path().join(".agents/pages/bad.dowe"), "Stack\n").expect("bad agent source");

    compile_dev(temp.path()).expect("project");

    let web_manifest =
        fs::read_to_string(temp.path().join(".dowe/web/manifest.json")).expect("web");
    let apps_manifest =
        fs::read_to_string(temp.path().join(".dowe/apps/manifest.json")).expect("apps");

    assert!(!web_manifest.contains(".agents"));
    assert!(!apps_manifest.contains(".agents"));
}

#[test]
fn parses_expected_endpoints() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());

    let project = compile_dev(temp.path()).expect("project");
    let status = project
        .backend
        .find_endpoint(&HttpMethod::Get, "/api/status")
        .expect("status");
    let posts = project
        .backend
        .find_endpoint(&HttpMethod::Post, "/api/posts")
        .expect("posts");

    assert_eq!(
        status.endpoint.behavior,
        EndpointBehavior::StaticText("OK".to_string())
    );
    assert_eq!(posts.endpoint.behavior, EndpointBehavior::CreatePostJson);
}

#[test]
fn compiles_blog_crud_signals_actions_and_handlers() {
    let temp = TempDir::new().expect("tempdir");
    write_blog_fixture(temp.path());

    let project = compile_dev(temp.path()).expect("project");
    let list = project
        .backend
        .find_endpoint(&HttpMethod::Get, "/api/blogs")
        .expect("list blogs");
    let create = project
        .backend
        .find_endpoint(&HttpMethod::Post, "/api/blogs")
        .expect("create blog");
    let update = project
        .backend
        .find_endpoint(&HttpMethod::Patch, "/api/blogs/01HX")
        .expect("update blog");

    assert!(matches!(
        list.endpoint.behavior,
        EndpointBehavior::StoreActionJson(_)
    ));
    assert!(matches!(
        create.endpoint.behavior,
        EndpointBehavior::StoreActionJson(_)
    ));
    assert!(matches!(
        update.endpoint.behavior,
        EndpointBehavior::StoreActionJson(_)
    ));

    let page = project
        .web
        .pages
        .iter()
        .find(|page| page.route_path == "/blogs")
        .expect("blogs page");
    let alert_paths = attribute_values(&page.body_html, "data-dowe-alert-visible");
    let click_actions = attribute_values(&page.body_html, "data-dowe-click");
    let bindings = attribute_values(&page.body_html, "data-dowe-bind");
    let collections = attribute_values(&page.body_html, "data-dowe-each");

    assert_eq!(alert_paths.len(), 2);
    assert_ne!(alert_paths[0], alert_paths[1]);
    assert!(alert_paths.iter().all(|path| short_root(path, ".visible")));
    assert!(bindings.iter().any(|path| short_root(path, ".title")));
    assert!(collections.iter().any(|path| short_root(path, "")));
    assert!(click_actions.len() >= 3);
    assert!(!click_actions.iter().any(|action| *action == "close"));
    assert!(page.body_html.contains(r#"data-dowe-alert"#));
    assert!(page.body_html.contains(">item.literal</p>"));

    let page_chunk = project
        .web
        .chunks
        .iter()
        .find(|chunk| chunk.id == page.page_chunk_id)
        .expect("page chunk");
    let layout_chunk = project
        .web
        .chunks
        .iter()
        .find(|chunk| chunk.id == page.layout_chunk_id)
        .expect("layout chunk");
    assert!(page_chunk.content.contains("dowePage"));
    assert!(page_chunk.content.contains(r#""id":"#));
    assert!(layout_chunk.content.contains("doweLayout"));
    assert!(page_chunk.content.contains("/api/blogs"));
    assert!(page_chunk.content.contains("\"create\""));
    assert!(project.web.router_js.contains("dowe:request"));
    assert!(project.web.router_js.contains("doweLayout"));

    let android_pages = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android pages");
    let android_dev = android_dev_output(temp.path());
    let ios_pages = ios_swift_output(temp.path());

    assert!(android_pages.contains("DoweReactiveState"));
    assert!(android_pages.contains("state.text("));
    assert!(android_pages.contains("state.rows("));
    assert!(android_pages.contains("state.run("));
    assert!(android_pages.contains("DoweEnvironment.BACKEND_URL"));
    assert!(android_pages.contains(".verticalScroll(scrollState)"));
    assert!(android_pages.contains("if (path == \"item\" && item != null)"));
    assert!(!android_pages.contains("Text(\"alert.message\""));
    assert!(!android_pages.contains("Text(\"item.title\""));
    assert!(android_pages.contains("Text(\"item.literal\""));

    assert!(android_dev.contains("doweState"));
    assert!(android_dev.contains("doweRows("));
    assert!(android_dev.contains("doweRunAction("));
    assert!(android_dev.contains("DoweEnvironment.BACKEND_URL"));
    assert!(android_dev.contains("ScrollView scrollView"));
    assert!(android_dev.contains("doweInputBackground("));
    assert!(android_dev.contains("doweAdd(ViewGroup parent, View child, Integer gap"));
    assert!(!android_dev.contains("doweText(\"alert.message\""));
    assert!(!android_dev.contains("doweText(\"item.title\""));
    assert!(android_dev.contains("doweText(\"item.literal\""));

    assert!(ios_pages.contains("DoweReactiveState"));
    assert!(ios_pages.contains("state.binding("));
    assert!(ios_pages.contains("state.rows("));
    assert!(ios_pages.contains("state.run("));
    assert!(ios_pages.contains("DoweEnvironment.BACKEND_URL"));
    assert!(ios_pages.contains("ScrollView {"));
    assert!(ios_pages.contains("DoweInputField(value: state.binding("));
    assert!(ios_pages.contains("minHeight: CGFloat(40), horizontalPadding: CGFloat(12)"));
    assert!(ios_pages.contains("if path == \"item\", let item"));
    assert!(!ios_pages.contains("Text(verbatim: \"alert.message\")"));
    assert!(!ios_pages.contains("Text(verbatim: \"item.title\")"));
    assert!(ios_pages.contains("Text(verbatim: \"item.literal\")"));
}

#[test]
fn writes_app_targets_from_shared_view_tree() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());

    compile_dev(temp.path()).expect("project");

    let root = temp.path().join(".dowe/apps");
    assert!(root.join("desktop/macos/DoweMacOSApp.swift").exists());
    assert!(root.join("desktop/windows/DoweWindowsApp.cs").exists());
    assert!(root.join("desktop/linux/dowe_linux_app.c").exists());
    assert!(
        root.join("android/app/src/main/AndroidManifest.xml")
            .exists()
    );
    assert!(
        root.join("android/app/src/main/java/dev/dowe/generated/MainActivity.kt")
            .exists()
    );
    assert!(
        root.join("android/app/src/main/java/dev/dowe/generated/DoweRouting.kt")
            .exists()
    );
    assert!(
        root.join("android/app/src/main/java/dev/dowe/generated/DoweLayouts.kt")
            .exists()
    );
    assert!(
        root.join("android/app/src/main/java/dev/dowe/generated/DowePages.kt")
            .exists()
    );
    assert!(
        root.join("android/app/src/main/java/dev/dowe/generated/DoweTheme.kt")
            .exists()
    );
    assert!(
        root.join("android/app/src/main/java/dev/dowe/generated/DoweResponsive.kt")
            .exists()
    );
    assert!(root.join("ios/DoweIosApp.swift").exists());
    assert!(root.join("ios/DoweRouting.swift").exists());
    assert!(root.join("ios/DoweLayouts.swift").exists());
    assert!(root.join("ios/DowePages.swift").exists());
    assert!(root.join("ios/DoweTheme.swift").exists());
    assert!(root.join("ios/DoweResponsive.swift").exists());
    assert!(root.join("manifest.json").exists());

    let android =
        fs::read_to_string(root.join("android/app/src/main/java/dev/dowe/generated/DowePages.kt"))
            .expect("android views");
    let ios = ios_apps_swift_output(&root);
    let manifest = fs::read_to_string(root.join("manifest.json")).expect("apps manifest");

    assert!(android.contains("Column(modifier = Modifier.fillMaxWidth()) {"));
    assert!(android.contains("Text(\"Layout\", modifier = Modifier, color = Color.Unspecified"));
    assert!(android.contains("Text(\"Login\", modifier = Modifier, color = Color.Unspecified"));
    assert!(ios.contains("VStack(alignment: .leading, spacing: 0)"));
    assert!(!ios.contains("VStack(alignment: .leading) {"));
    assert!(ios.contains("Text(verbatim: \"Layout\")"));
    assert!(ios.contains("Text(verbatim: \"Login\")"));
    assert!(manifest.contains("desktop-macos"));
    assert!(manifest.contains("desktop-windows"));
    assert!(manifest.contains("desktop-linux"));
    assert!(manifest.contains("android"));
    assert!(manifest.contains("ios"));
    assert!(manifest.contains("web/manifest.json"));
    assert!(manifest.contains(r#""deepLinks""#));
    assert!(manifest.contains(r#""scheme":"dowe-dev""#));
}

#[test]
fn preserves_nested_box_order_and_children_position() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    Text
      "Before"
    children
    Text
      "After""#,
        r#"page loginPage
  Box
    Box
      Text
        "Login""#,
    );

    let project = compile_dev(temp.path()).expect("project");

    assert!(
        project.web.pages[0]
            .body_html
            .contains(r#"<p class="dowe-text text-md">Before</p>"#)
    );
    assert!(
        project.web.pages[0].body_html.contains(
            r#"<div class="box"><div class="box"><p class="dowe-text text-md">Login</p></div></div>"#
        )
    );
    assert!(
        project.web.pages[0]
            .body_html
            .contains(r#"<p class="dowe-text text-md">After</p>"#)
    );
}

#[test]
fn compiles_client_environment_for_request_base() {
    let temp = TempDir::new().expect("tempdir");
    write_blog_fixture(temp.path());
    fs::write(
        temp.path().join(".env.example"),
        "BACKEND_URL=\nINTERNAL_TOKEN=\n",
    )
    .expect("env example");
    fs::write(
        temp.path().join(".env"),
        "BACKEND_URL=\"https://api.example.com\"\nINTERNAL_TOKEN=secret\n",
    )
    .expect("env");

    let project = compile_dev(temp.path()).expect("project");
    let backend = project
        .environment_config
        .variable("BACKEND_URL")
        .expect("backend url");
    let internal = project
        .environment_config
        .variable("INTERNAL_TOKEN")
        .expect("internal token");
    assert_eq!(backend.visibility, EnvironmentVisibility::Client);
    assert_eq!(backend.resolved_source, EnvironmentValueSource::DotEnv);
    assert_eq!(
        backend.resolved_value.as_deref(),
        Some("https://api.example.com")
    );
    assert_eq!(internal.visibility, EnvironmentVisibility::Server);

    let page = project
        .web
        .pages
        .iter()
        .find(|page| page.route_path == "/blogs")
        .expect("blogs page");
    let page_chunk = project
        .web
        .chunks
        .iter()
        .find(|chunk| chunk.id == page.page_chunk_id)
        .expect("page chunk");
    assert!(page_chunk.content.contains(r#""baseEnv":"BACKEND_URL""#));
    assert!(project.web.router_js.contains("env.json"));

    let env_json = fs::read_to_string(temp.path().join(".dowe/web/env.json")).expect("env");
    assert!(env_json.contains(r#""BACKEND_URL":"https://api.example.com""#));
    assert!(!env_json.contains("INTERNAL_TOKEN"));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DoweEnvironment.kt"),
    )
    .expect("android env");
    let ios = fs::read_to_string(temp.path().join(".dowe/apps/ios/DoweEnvironment.swift"))
        .expect("ios env");
    assert!(android.contains(r#"const val BACKEND_URL = "https://api.example.com""#));
    assert!(ios.contains(r#"static let BACKEND_URL = "https://api.example.com""#));
    assert!(!android.contains("INTERNAL_TOKEN"));
    assert!(!ios.contains("INTERNAL_TOKEN"));
}

#[test]
fn isolates_development_and_deploy_environment_files() {
    let temp = TempDir::new().expect("tempdir");
    write_blog_fixture(temp.path());
    fs::write(temp.path().join(".env.example"), "BACKEND_URL=\n").expect("env example");
    fs::write(
        temp.path().join(".env"),
        "BACKEND_URL=https://dev.example.com\n",
    )
    .expect("development env");
    fs::write(
        temp.path().join(".env.live"),
        "BACKEND_URL=https://live.example.com\n",
    )
    .expect("live env");
    fs::write(
        temp.path().join(".env.stage"),
        "BACKEND_URL=https://stage.example.com\n",
    )
    .expect("stage env");
    fs::write(
        temp.path().join(".env.uat"),
        "BACKEND_URL=https://uat.example.com\n",
    )
    .expect("uat env");

    let development = compile_dev(temp.path()).expect("development project");
    let live = compile_for_environment(temp.path(), CompileEnvironment::Live).expect("live");
    let stage = compile_for_environment(temp.path(), CompileEnvironment::Stage).expect("stage");
    let uat = compile_for_environment(temp.path(), CompileEnvironment::Uat).expect("uat");

    assert_eq!(
        development
            .environment_config
            .variable("BACKEND_URL")
            .and_then(|variable| variable.resolved_value.as_deref()),
        Some("https://dev.example.com")
    );
    assert_eq!(
        live
            .environment_config
            .variable("BACKEND_URL")
            .and_then(|variable| variable.resolved_value.as_deref()),
        Some("https://live.example.com")
    );
    assert_eq!(
        stage
            .environment_config
            .variable("BACKEND_URL")
            .and_then(|variable| variable.resolved_value.as_deref()),
        Some("https://stage.example.com")
    );
    assert_eq!(
        uat.environment_config
            .variable("BACKEND_URL")
            .and_then(|variable| variable.resolved_value.as_deref()),
        Some("https://uat.example.com")
    );
}

#[test]
fn rejects_view_environment_references_in_server_only_http_configuration() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join(".env.example"),
        "SHARED_URL=\n",
    )
    .expect("env example");
    fs::write(
        temp.path().join("pages/login.dowe"),
        r#"page loginPage
  fn load
    request status method:"GET" route:"/status" base:env.SHARED_URL
  Text
    "Login""#,
    )
    .expect("page");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    route "/api/proxy"
      handler req
        http upstream method:"get" base:env.SHARED_URL path:"/status" mode:"json"
        return json:upstream.json"#,
    )
    .expect("main");

    let error = compile_dev(temp.path()).expect_err("public server HTTP base");

    assert!(error.to_string().contains("SHARED_URL"));
    assert!(error.to_string().contains("must be server-only"));
}

#[test]
fn resolves_environment_from_operating_system_before_dotenv() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join(".env.example"),
        "DOWE_TEST_BACKEND_URL=\n",
    )
    .expect("config");
    fs::write(
        temp.path().join(".env"),
        "DOWE_TEST_BACKEND_URL=https://local.example.com\n",
    )
    .expect("local env");
    unsafe {
        std::env::set_var("DOWE_TEST_BACKEND_URL", "https://os.example.com");
    }

    let project = compile_dev(temp.path()).expect("project");

    unsafe {
        std::env::remove_var("DOWE_TEST_BACKEND_URL");
    }
    let variable = project
        .environment_config
        .variable("DOWE_TEST_BACKEND_URL")
        .expect("variable");
    assert_eq!(variable.resolved_source, EnvironmentValueSource::Os);
    assert_eq!(
        variable.resolved_value.as_deref(),
        Some("https://os.example.com")
    );
}

#[test]
fn rejects_process_environment_names_without_dotenv_declaration() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("pages/login.dowe"),
        r#"page loginPage
  fn load
    request status method:"GET" route:"/status" base:env.DOWE_OS_ONLY_URL
  Text
    "Login""#,
    )
    .expect("page");
    unsafe {
        std::env::set_var("DOWE_OS_ONLY_URL", "https://os.example.com");
    }

    let error = compile_dev(temp.path()).expect_err("undeclared process variable");

    unsafe {
        std::env::remove_var("DOWE_OS_ONLY_URL");
    }
    assert!(error.to_string().contains("DOWE_OS_ONLY_URL"));
    assert!(error.to_string().contains("unknown environment variable"));
}

#[test]
fn ignores_env_example_values_when_local_and_process_values_are_absent() {
    let temp = TempDir::new().expect("tempdir");
    write_blog_fixture(temp.path());
    fs::write(
        temp.path().join(".env.example"),
        "BACKEND_URL=https://placeholder.example\nINTERNAL_TOKEN=replace-me\n",
    )
    .expect("env example");
    fs::remove_file(temp.path().join(".env")).expect("remove local env");

    let project = compile_dev(temp.path()).expect("project");
    let backend = project
        .environment_config
        .variable("BACKEND_URL")
        .expect("backend url");

    assert_eq!(backend.visibility, EnvironmentVisibility::Client);
    assert_eq!(backend.resolved_source, EnvironmentValueSource::Missing);
    assert_eq!(backend.resolved_value, None);
    assert_eq!(project.environment_config.client_json(), r#"{"BACKEND_URL":""}"#);
}

#[test]
fn rejects_invalid_request_base_url() {
    let temp = TempDir::new().expect("tempdir");
    write_blog_fixture(temp.path());
    fs::write(
        temp.path().join(".env"),
        "BACKEND_URL=file:///tmp/api\nINTERNAL_TOKEN=\n",
    )
    .expect("env");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("http or https URL"));
    assert!(error.to_string().contains("BACKEND_URL"));
}

#[test]
fn rejects_unknown_environment_variable() {
    let temp = TempDir::new().expect("tempdir");
    write_blog_fixture(temp.path());
    fs::write(temp.path().join(".env.example"), "INTERNAL_TOKEN=\n").expect("env example");
    fs::write(temp.path().join(".env"), "INTERNAL_TOKEN=secret\n").expect("env");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("BACKEND_URL"));
    assert!(error.to_string().contains("unknown environment variable"));
}

#[test]
fn rejects_duplicate_dotenv_keys() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join(".env"),
        "BACKEND_URL=\nBACKEND_URL=https://api.example.com\n",
    )
    .expect("env");

    let error = compile_dev(temp.path()).expect_err("duplicate dotenv key");

    assert!(error.to_string().contains(".env"));
    assert!(error.to_string().contains("duplicate environment variable `BACKEND_URL`"));
}

#[test]
fn compiles_backend_cors_config() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
            temp.path().join("main.dowe"),
            r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    cors target:"server" devOrigins:true origins:["http://127.0.0.1:56035"] methods:["GET","POST","PATCH","DELETE"] headers:["Content-Type"] exposeHeaders:["X-Request-Id"] credentials:false maxAge:600
    route "/api/status"
      response text:"OK""#,
        )
        .expect("config");

    let project = compile_dev(temp.path()).expect("project");

    assert!(project.backend.cors.enabled);
    assert!(project.backend.cors.allow_dev_origins);
    assert_eq!(
        project.backend.cors.origins,
        vec!["http://127.0.0.1:56035".to_string()]
    );
    assert!(project.backend.cors.methods.contains(&"GET".to_string()));
    assert!(project.backend.cors.methods.contains(&"POST".to_string()));
    assert_eq!(
        project.backend.cors.headers,
        vec!["Content-Type".to_string()]
    );
    assert_eq!(
        project.backend.cors.expose_headers,
        vec!["X-Request-Id".to_string()]
    );
    assert_eq!(project.backend.cors.max_age, Some(600));
}

#[test]
fn expands_cors_target_all_to_backend_and_desktop() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    cors target:"all" origins:["https://app.example.com"] headers:["Content-Type"]
    route "/api/status"
      response text:"OK"
  desktop
    server port:4500
      cors target:"all" origins:["https://app.example.com"] headers:["Content-Type"]
      route "/api/status"
        response text:"OK""#,
    )
    .expect("config");

    let project = compile_dev(temp.path()).expect("project");

    assert!(project.backend.cors.enabled);
    assert!(
        project
            .desktop_server
            .as_ref()
            .expect("desktop")
            .cors
            .enabled
    );
}

#[test]
fn rejects_invalid_cors_origin() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    cors target:"server" origins:["https://app.example.com/path"]
    route "/api/status"
      response text:"OK""#,
    )
    .expect("config");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("invalid CORS origin"));
}

#[test]
fn rejects_wildcard_cors_with_credentials() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    cors target:"server" origins:["*"] credentials:true
    route "/api/status"
      response text:"OK""#,
    )
    .expect("config");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("credentials:true"));
}

#[test]
fn rejects_duplicate_cors_policy_for_target() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    cors target:"server" origins:["https://one.example.com"]
    cors target:"server" origins:["https://two.example.com"]
    route "/api/status"
      response text:"OK""#,
    )
    .expect("config");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("duplicate `cors` block"));
}

fn write_blog_fixture(root: &Path) {
    write_fixture_with_views(
        root,
        r#"layout AuthLayout
  signal alert value:{ type:"info" message:"Layout alert" visible:true }
  fn close
    reset alert
  Box
    Text
      "Layout"
    Alert type:"info" message:alert.message visible:alert.visible onClose:close
    children"#,
        r#"page loginPage
  Box
    Text
      "Login""#,
    );
    fs::write(
        root.join("theme.dowe"),
        r#"theme
  fonts default:"inter" install:["inter"]"#,
    )
    .expect("theme");
    fs::write(root.join(".env.example"), "BACKEND_URL=\nINTERNAL_TOKEN=\n")
        .expect("env example");
    fs::write(root.join(".env"), "BACKEND_URL=\nINTERNAL_TOKEN=\n").expect("env");
    fs::create_dir_all(root.join("handlers")).expect("handlers");
    fs::write(
        root.join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"
import listBlogs from "@/handlers/blogs"
import createBlog from "@/handlers/blogs"
import readBlog from "@/handlers/blogs"
import updateBlog from "@/handlers/blogs"
import deleteBlog from "@/handlers/blogs"

main
  views:viewRoutes
  server port:8080
    route "/api/blogs"
      method GET handler:listBlogs
      method POST handler:createBlog
    route "/api/blogs/:id"
      method GET handler:readBlog
      method PATCH handler:updateBlog
      method DELETE handler:deleteBlog"#,
    )
    .expect("server");
    fs::write(
            root.join("handlers/blogs.dowe"),
        r#"handler listBlogs req
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query blogs conn:db.list table:"blogs"
  return json:{ ok:true data:blogs }

handler createBlog
  const body value:req.json
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query created conn:db.insert table:"blogs" value:{ title:body.title content:body.content createdAt:now updatedAt:now } required:["title","content"]
  query blogs conn:db.list table:"blogs"
  return status:201 json:{ ok:true data:blogs }

handler readBlog req
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query blog conn:db.read table:"blogs" where:{ id:req.params.id } required:true
  return json:{ ok:true data:blog }

handler updateBlog
  const body value:req.json
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query updated conn:db.update table:"blogs" where:{ id:req.params.id } value:{ title:body.title content:body.content updatedAt:now } required:true match:{ id:req.params.id }
  query blogs conn:db.list table:"blogs"
  return json:{ ok:true data:blogs }

handler deleteBlog req
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query deleted conn:db.delete table:"blogs" where:{ id:req.params.id } required:true
  query blogs conn:db.list table:"blogs"
  return json:{ ok:true data:blogs }"#,
        )
        .expect("handlers");
    fs::write(
        root.join("routes/view.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import loginPage from "../pages/login"
import blogsPage from "../pages/blogs"

views viewRoutes
  group path:"/" layout:AuthLayout
    route path:"" page:loginPage
    route path:"blogs" page:blogsPage"#,
    )
    .expect("views");
    fs::write(
        root.join("pages/blogs.dowe"),
        r#"page blogsPage
  signal blog value:{ id:null title:"" content:"" }
  signal blogs value:[]
  signal alert value:{ type:"info" message:"" visible:false }
  fn load
    request GET route:"/api/blogs" update:blogs autoload:true
      onError alert:"No se pudieron cargar los blogs"
  fn create
    request POST route:"/api/blogs" body:blog update:blogs reset:blog
      onSuccess alert:"Blog creado"
      onError alert:"No se pudo crear el blog"
  fn edit
    set blog value:item
  fn update
    request PATCH route:"/api/blogs/:id" body:blog update:blogs reset:blog
      onSuccess alert:"Blog actualizado"
      onError alert:"No se pudo actualizar el blog"
  fn delete
    request DELETE route:"/api/blogs/:id" body:item update:blogs
      onSuccess alert:"Blog eliminado"
      onError alert:"No se pudo eliminar el blog"
  fn close
    reset alert
  Box
    Title
      "Blogs"
    Alert type:"info" message:alert.message visible:alert.visible onClose:close
    Input bind:blog.title
    Button onClick:create
      "Crear"
    each in:blogs as:item key:item.id
      Card
        Title
          "{item.title}"
        Text
          "{item.content}"
        Text
          "item.literal"
        Button onClick:edit
          "Editar""#,
    )
    .expect("blogs");
}

fn attribute_values<'a>(html: &'a str, name: &str) -> Vec<&'a str> {
    let prefix = format!(r#"{name}=""#);
    html.match_indices(&prefix)
        .filter_map(|(start, _)| {
            let value = &html[start + prefix.len()..];
            value.find('"').map(|end| &value[..end])
        })
        .collect()
}

fn short_root(value: &str, suffix: &str) -> bool {
    value.strip_suffix(suffix).is_some_and(|root| {
        root.len() == 8
            && root
                .chars()
                .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    })
}

fn ios_swift_output(root: &Path) -> String {
    ios_swift_output_from(&root.join(".dowe/apps/ios"))
}

fn ios_apps_swift_output(root: &Path) -> String {
    ios_swift_output_from(&root.join("ios"))
}

fn ios_swift_output_from(ios_root: &Path) -> String {
    let mut swift_files = fs::read_dir(ios_root)
        .expect("ios output")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("swift"))
        .collect::<Vec<_>>();
    swift_files.sort();
    swift_files
        .into_iter()
        .map(|path| fs::read_to_string(path).expect("ios swift"))
        .collect::<Vec<_>>()
        .join("\n")
}
