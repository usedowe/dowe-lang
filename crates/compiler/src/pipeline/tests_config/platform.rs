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

