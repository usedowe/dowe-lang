#[test]
fn compiles_reactive_button_props_across_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  children",
        r#"page loginPage
  signal variantChoice value:"solid"
  signal schemeChoice value:"primary"
  signal sizeChoice value:"md"
  signal roundedChoice value:"md"
  signal startIconVisible value:11
  Grid columns:1 show:{ when:startIconVisible gt:10 }
    Button variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice iconStart:{ when:startIconVisible gt:10 value:"add-circle" }
      "Create"
    Code:
      template:true
      content:"""
        Button variant:{variantChoice} scheme:{schemeChoice} size:{sizeChoice}
      """"#,
    );
    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    assert!(body.contains("data-dowe-button-variant="));
    assert!(body.contains("data-dowe-button-icon-start-when="));
    assert!(body.contains("data-dowe-show-operator=\">\""));
    assert!(body.contains("data-dowe-button-icon-start-operator=\">\""));
    assert!(body.contains("data-dowe-text="));
    assert!(body.contains(r#"code-token-type">Button</span>"#));
    assert!(body.contains(r#"code-token-attribute">variant</span>"#));
    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("doweButtonContainer(state.text("));
    assert!(android.contains(".toDoubleOrNull() ?: 0.0) > 10"));
    assert!(android.contains("DoweCode(source = \"Button variant:\" + state.text("));
    assert!(android.contains("listOf(DoweCodeToken(text = \"Button\", color = DoweDesign.info)"));
    assert!(android.contains("listOf(DoweCodeToken(text = state.text("));
    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("doweButtonContainer(doweTextValue("));
    assert!(android_dev.contains("renderCurrentRoute(false)"));
    assert!(android_dev.contains("DoweSvgView"));
    assert!(android_dev.contains("Double.parseDouble(doweTextValue("));
    assert!(android_dev.contains("private float doweButtonRadius(String value)"));
    assert!(android_dev.contains("new String[]{\"Button\""));
    assert!(!android_dev.contains("new String[]{}, new int[]{}"));
    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("@MainActor\nfunc doweButtonFamily(_ scheme: String) -> Color"));
    assert!(ios.contains("@MainActor\nfunc doweButtonRadius(_ value: String) -> CGFloat"));
    assert!(ios.contains("doweButtonContainer(state.text("));
    assert!(ios.contains("Double(state.text("));
    assert!(ios.contains("DoweCodeView(source: \"Button variant:\" + state.text("));
    assert!(ios.contains("[DoweCodeToken(text: \"Button\", color: DoweDesign.info)"));
    assert!(ios.contains("[DoweCodeToken(text: state.text("));
    assert!(ios.contains(".fixedSize(horizontal: true, vertical: true)"));
}

#[test]
fn compiles_reusable_view_component_inside_sidebar_and_drawer() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"import DocsNavigation from "../components/docs-navigation"

layout AuthLayout
  signal openDrawer value:false
  signal drawerVisible value:true
  Scaffold
    appBar
      AppBar variant:"solid" scheme:"background" bordered:true boxed:true
        start
          Button onClick:{ set:openDrawer value:!openDrawer } show:{ xs:true md:false } variant:"ghost" scheme:"primary" size:"xs"
            "Menu"
        center
          Text size:"xl" weight:"black" spacing:"normal"
            "DOWE"
    start
      Sidebar show:{ xs:false md:true } w:96 pr:6 variant:"solid" scheme:"surface" rounded:"lg" border:1 p:4
        body
          SideNav variant:"ghost" scheme:"primary" size:"sm" wide:true
            DocsNavigation
    main
      Drawer show:{ xs:true md:false } bind:openDrawer position:"start" variant:"solid" scheme:"surface" disableOverlayClose:false hideCloseButton:false p:4 w:80
        body
          SideNav variant:"ghost" scheme:"primary" size:"sm" wide:true
            DocsNavigation
      children"#,
        r#"page loginPage
  Box
    Text
      "Login""#,
    );
    fs::create_dir_all(temp.path().join("components")).expect("components");
    fs::write(
        temp.path().join("components/docs-navigation.dowe"),
        r#"component DocsNavigation
  header label:"Views" description:"Portable UI"
  item label:"Docs overview" href:"/"
  divider
  header label:"Deploy" description:"Distribution targets"
  item label:"Deploy overview" href:"/""#,
    )
    .expect("component");

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;

    assert!(body.contains("show-true md:show-false"));
    assert!(body.contains(r#"<aside class="sidebar show-false md:show-true"#));
    assert!(body.contains(r#"<nav class="sidenav"#));
    assert!(body.contains("is-ghost"));
    assert!(body.contains("sidenav"));
    assert!(body.contains(r#"<div class="drawer-panel show-true md:show-false"#));
    assert_eq!(body.matches("Docs overview").count(), 2);
    assert_eq!(body.matches("Deploy overview").count(), 2);

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("label = \"Docs overview\""));
    assert!(android.contains("label = \"Deploy overview\""));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("label: \"Docs overview\""));
    assert!(ios.contains("label: \"Deploy overview\""));
}

#[test]
fn compiles_fixed_appbar_with_automatic_scaffold_insets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Scaffold
    appBar
      AppBar position:"fixed" variant:"solid" scheme:"surface" bordered:true
        start
          Text
            "Dowe"
        center
          Text size:"xl"
            "Documentation"
    start
      Sidebar show:{ xs:false md:true } w:72
        body
          SideNav
            item label:"Start" href:"/"
    main
      children
    end
      Sidebar show:{ xs:false lg:true } w:64
        body
          SideNav
            item label:"End" href:"/""#,
        r#"page loginPage
  Section
    Text
      "Main content""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    let css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.design_file_name()),
    )
    .expect("css");

    assert!(body.contains("position-fixed"));
    assert!(body.contains(r#"<aside class="scaffold-start">"#));
    assert!(body.contains(r#"<main class="scaffold-main">"#));
    assert!(body.contains(r#"<aside class="scaffold-end">"#));
    assert!(!body.contains("vh-"));
    assert!(!css.is_empty());
    assert!(
        project
            .web
            .router_js
            .contains("function hydrateScaffoldInsets(root)")
    );
    assert!(
        project
            .web
            .router_js
            .contains("appBar.getBoundingClientRect().bottom")
    );
}

#[test]
fn compiles_navigation_components_with_appbar_aware_section_scroll() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r##"layout AuthLayout
  Scaffold
    appBar
      AppBar position:"fixed"
        center
          NavMenu
            item label:"Features" href:"#features"
    start
      SideNav
        item label:"Features" href:"#features"
      RailNav
        item label:"Features" href:"#features" icon:"stars-minimalistic"
    main
      children"##,
        r#"page loginPage
  Section id:"hero"
    Title
      "Landing"
  Section id:"features"
    Title
      "Features""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    let router = &project.web.router_js;
    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android pages");
    let ios = ios_swift_output(temp.path());

    assert_eq!(
        body.matches(r##"href="#features" data-dowe-nav="push""##)
            .count(),
        3
    );
    assert!(body.contains(r#"id="features""#));
    assert!(router.contains("function fragmentAppBarInset(target)"));
    assert!(router.contains(".appbar.position-fixed,.appbar.position-sticky"));
    assert!(router.contains("target.style.scrollMarginTop"));
    assert!(
        router.contains("scrollIntoView")
    );
    assert_eq!(
        android
            .matches(r#"{ navigate("push", "", "features") }"#)
            .count(),
        2
    );
    assert!(android.contains(r#"path = "", fragment = "features""#));
    assert_eq!(
        ios.matches(r#"{ navigate("push", "", "features") }"#)
            .count(),
        2
    );
    assert!(ios.contains(r#"path: "", fragment: "features""#));
}

#[test]
fn rejects_reusable_view_component_usage_shape_and_cycles() {
    let props = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        props.path(),
        r#"import DocsNavigation from "../components/docs-navigation"

layout AuthLayout
  Box
    DocsNavigation p:4
    children"#,
        r#"page loginPage
  Text
    "Login""#,
    );
    fs::create_dir_all(props.path().join("components")).expect("components");
    fs::write(
        props.path().join("components/docs-navigation.dowe"),
        r#"component DocsNavigation
  Text
    "Navigation""#,
    )
    .expect("component");
    let props_error = compile_dev(props.path()).expect_err("props error");
    assert!(!props_error.to_string().is_empty());

    let metadata = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        metadata.path(),
        r#"import DocsNavigation from "../components/docs-navigation"

layout AuthLayout
  Box
    DocsNavigation
    children"#,
        r#"page loginPage
  Text
    "Login""#,
    );
    fs::create_dir_all(metadata.path().join("components")).expect("components");
    fs::write(
        metadata.path().join("components/docs-navigation.dowe"),
        r#"component DocsNavigation
  meta name:"title" content:"Hidden override"
  Text
    "Navigation""#,
    )
    .expect("component");
    let metadata_error = compile_dev(metadata.path()).expect_err("metadata error");
    assert!(
        metadata_error
            .to_string()
            .contains("component exports cannot declare signal, fn, request or meta")
    );

    let cycle = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        cycle.path(),
        r#"import DocsNavigation from "../components/docs-navigation"

layout AuthLayout
  Box
    DocsNavigation
    children"#,
        r#"page loginPage
  Text
    "Login""#,
    );
    fs::create_dir_all(cycle.path().join("components")).expect("components");
    fs::write(
        cycle.path().join("components/docs-navigation.dowe"),
        r#"import NestedNavigation from "./nested-navigation"

component DocsNavigation
  NestedNavigation"#,
    )
    .expect("component");
    fs::write(
        cycle.path().join("components/nested-navigation.dowe"),
        r#"import DocsNavigation from "./docs-navigation"

component NestedNavigation
  DocsNavigation"#,
    )
    .expect("nested");
    let cycle_error = compile_dev(cycle.path()).expect_err("cycle error");
    assert!(
        cycle_error
            .to_string()
            .contains("component import cycle includes")
    );
}

#[test]
fn rejects_reusable_view_component_as_route_page() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Text
    "Login""#,
    );
    fs::create_dir_all(temp.path().join("components")).expect("components");
    fs::write(
        temp.path().join("components/docs-navigation.dowe"),
        r#"component DocsNavigation
  Text
    "Navigation""#,
    )
    .expect("component");
    fs::write(
        temp.path().join("routes/view.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import DocsNavigation from "../components/docs-navigation"

views viewRoutes
  group path:"/" layout:AuthLayout
    route path:"" page:DocsNavigation"#,
    )
    .expect("views");

    let error = compile_dev(temp.path()).expect_err("route page error");
    assert!(
        error
            .to_string()
            .contains("view modules must export a layout or page")
    );
}

