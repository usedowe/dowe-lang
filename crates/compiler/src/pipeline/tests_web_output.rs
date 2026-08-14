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
    assert!(page.html_document.contains("<title data-dowe-meta>Views &lt; Dowe</title>"));
    assert!(page.html_document.contains(r#"<meta data-dowe-meta name="description" content="Compose web views.">"#));
    assert!(page.html_document.contains(r#"<link data-dowe-meta rel="canonical" href="https://dowe.dev/?ref=docs&amp;kind=web">"#));
    assert!(page.html_document.contains(r#"<meta data-dowe-meta property="og:image" content="https://dowe.dev/og.png">"#));
    assert!(page.html_document.contains(r#"<meta data-dowe-meta name="twitter:card" content="summary_large_image">"#));
    assert!(dowe_generator_web::manifest(&project.web).contains(r#""metadata":[{"name":"title","content":"Views < Dowe"}"#));
    assert!(project.web.router_js.contains("function applyRouteMetadata(route)"));
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
    assert!(project
        .desktop_web
        .pages
        .iter()
        .all(|page| page.metadata.is_empty()));
    assert!(project
        .desktop_web
        .pages
        .iter()
        .all(|page| !page.html_document.contains("Views &lt; Dowe")));
    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android pages");
    let ios = fs::read_to_string(temp.path().join(".dowe/apps/ios/DowePages.swift"))
        .expect("ios pages");
    assert!(!android.contains("Compose web views."));
    assert!(!ios.contains("Compose web views."));
}

#[test]
fn resolves_component_defaults_before_all_target_generators() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Button
    "Default"
  Button scheme:"secondary"
    "Explicit scheme"
  Card
    Text
      "Card"
  Input
  Section
    Text
      "Band""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    assert!(body.contains(
        r#"class="button button-md px-4 py-2.5 min-h-10 rounded-md is-solid is-primary""#
    ));
    assert!(body.contains(
        r#"class="button button-md px-4 py-2.5 min-h-10 rounded-md is-solid is-secondary""#
    ));
    assert!(body.contains(r#"class="card p-4 lg:p-5 rounded-md is-solid is-surface""#));
    assert!(body.contains(r#"class="control is-md is-outlined is-primary""#));
    assert!(body.contains(r#"class="section bg-background""#));
    assert!(!body.contains("button-md border-"));
    assert!(!body.contains("button-md shadow-"));
    let desktop_body = &project.desktop_web.pages[0].body_html;
    assert!(desktop_body.contains(
        r#"class="button button-md px-4 py-2.5 min-h-10 rounded-md is-solid is-primary""#
    ));
    assert!(desktop_body.contains(
        r#"class="button button-md px-4 py-2.5 min-h-10 rounded-md is-solid is-secondary""#
    ));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("DoweDesign.primary"));
    assert!(android.contains("DoweDesign.secondary"));
    assert!(android.contains("DoweDesign.surface"));
    assert!(android.contains("outlined") || android.contains("Outlined"));
    assert!(android.contains("DoweDesign.background"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("DoweDesign.primary"));
    assert!(ios.contains("DoweDesign.secondary"));
    assert!(ios.contains("DoweDesign.surface"));
    assert!(ios.contains("outlined") || ios.contains("Outlined"));
    assert!(ios.contains("DoweDesign.background"));
}

#[test]
fn lets_design_defaults_override_builtin_component_defaults() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Button
    "Default variant"
  Button scheme:"secondary"
    "Explicit scheme""#,
    );
    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  design defaultTheme:"light"
    Button variant:"outlined"
    theme name:"light"
      colors:
        primary color:"#2563eb" text:"#ffffff" title:"#ffffff""##,
    )
    .expect("theme");

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    assert!(body.contains(
        r#"class="button button-md px-4 py-2.5 min-h-10 rounded-md is-outlined is-primary""#
    ));
    assert!(body.contains(
        r#"class="button button-md px-4 py-2.5 min-h-10 rounded-md is-outlined is-secondary""#
    ));
    assert!(!body.contains("button-md border-"));
    assert!(!body.contains("button-md shadow-"));
    let desktop_body = &project.desktop_web.pages[0].body_html;
    assert!(desktop_body.contains(
        r#"class="button button-md px-4 py-2.5 min-h-10 rounded-md is-outlined is-primary""#
    ));
    assert!(desktop_body.contains(
        r#"class="button button-md px-4 py-2.5 min-h-10 rounded-md is-outlined is-secondary""#
    ));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("DoweDesign.primary"));
    assert!(android.contains("DoweDesign.secondary"));
    assert!(android.contains("outlined") || android.contains("Outlined"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("DoweDesign.primary"));
    assert!(ios.contains("DoweDesign.secondary"));
    assert!(ios.contains("outlined") || ios.contains("Outlined"));
}

#[test]
fn applies_toast_design_defaults_to_global_function_statements() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  fn showDefault
    toast value:{ type:"success" title:"Saved" message:"Saved" visible:true }
  fn showExplicit
    toast value:{ type:"success" title:"Saved" message:"Saved" visible:true } variant:"outlined"
  Button onClick:showDefault
    "Default"
  Button onClick:showExplicit
    "Explicit""#,
    );
    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  design defaultTheme:"light"
    Toast variant:"soft"
    theme name:"light""#,
    )
    .expect("theme");

    let project = compile_dev(temp.path()).expect("project");
    let toast_variants = |tree: &dowe_components::ViewNode| {
        let dowe_components::ViewNode::Scope { actions, .. } = tree else {
            panic!("scope");
        };
        actions
            .iter()
            .filter_map(|action| {
                let dowe_components::ViewActionKind::Sequence(statements) = &action.kind else {
                    return None;
                };
                let [dowe_components::ViewFunctionStatement::Toast(toast)] = statements.as_slice()
                else {
                    return None;
                };
                Some(toast.variant.clone())
            })
            .collect::<Vec<_>>()
    };

    assert_eq!(
        toast_variants(&project.web.pages[0].page_tree),
        vec![Some("soft".to_string()), Some("outlined".to_string())]
    );
    assert_eq!(
        project.view_routes.web[0].page_tree,
        project.view_routes.desktop[0].page_tree
    );
    assert_eq!(
        project.view_routes.web[0].page_tree,
        project.view_routes.android[0].page_tree
    );
    assert_eq!(
        project.view_routes.web[0].page_tree,
        project.view_routes.ios[0].page_tree
    );
}

#[test]
fn copies_project_assets_to_android_bundle() {
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
    fs::create_dir_all(temp.path().join("assets/avatars")).expect("assets");
    fs::write(temp.path().join("assets/avatars/ada.png"), [1_u8, 2, 3]).expect("asset");

    compile_dev(temp.path()).expect("project");

    assert!(temp
        .path()
        .join(".dowe/apps/android/app/src/main/assets/avatars/ada.png")
        .is_file());
}

#[test]
fn compiles_design_system_components_and_responsive_props() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box bg:"background" color:"backgroundText" p:{ xs:2 md:4 }
    Text size:"sm"
      "Shell"
    children"#,
        r#"page loginPage
  Box p:10 px:0.5 w:"full"
    Flex direction:{ xs:"column" md:"row" } wrap:true justify:"center" align:"center" gap:{ xs:2 lg:6 }
      Card variant:"soft" scheme:"primary" rounded:"lg" border:1 p:{ xs:4 md:8 }
        Title size:"2xl" bg:"softPrimary" weight:"extrabold" spacing:"tight" p:4
          "Welcome"
        Text size:"md" bg:"surface" color:"primaryText" weight:"bold" spacing:"wide" rounded:"md" border:1
          "Login"
        Button variant:"solid" scheme:"danger"
          "Save"
        Button variant:"soft" scheme:"warning" size:"lg" rounded:"full"
          "Warn"
        Input variant:"outlined" scheme:"info"
        Card scheme:"primary"
          Text
            "Default""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;

    assert!(body.contains(r#"class="box bg-background color-backgroundText p-2 md:p-4""#));
    assert!(body.contains(r#"class="box p-10 px-0.5 w-full""#));
    assert!(body.contains(r#"class="flex direction-column md:direction-row flex-wrap justify-center align-center gap-2 lg:gap-6""#));
    assert!(body.contains(r#"class="card p-4 md:p-8 rounded-lg border-1 is-soft is-primary""#));
    assert!(
        body.contains(r#"class="dowe-title title-2xl bg-softPrimary p-4 weight-extrabold tracking-tight""#)
    );
    assert!(body.contains(
            r#"class="dowe-text text-md bg-surface color-primaryText rounded-md border-1 weight-bold tracking-wide""#
        ));
    assert!(body.contains(r#"class="button button-md px-4 py-2.5 min-h-10 rounded-md is-solid is-danger""#));
    assert!(body.contains(
        r#"class="button button-lg px-5 py-3 min-h-11 rounded-full is-soft is-warning""#
    ));
    assert!(
        body.contains(r#"<div class="control is-md is-outlined is-info"><input class="input"></div>"#)
    );
    assert!(body.contains(r#"class="card p-4 lg:p-5 rounded-md is-solid is-primary""#));

    let css = fs::read_to_string(temp.path().join(".dowe/web/design.css")).expect("css");
    assert!(css.contains("--dowe-primary"));
    assert!(css.contains("--dowe-softDanger"));
    assert!(!css.contains(".p-96"));
    let layout_css_path = temp
        .path()
        .join(".dowe/web")
        .join(&project.web.pages[0].css_chunks[0]);
    let layout_css = fs::read_to_string(layout_css_path).expect("layout css");
    assert!(layout_css.contains(".color-backgroundText{color:var(--dowe-backgroundText);}"));

    let page_css_path = temp
        .path()
        .join(".dowe/web")
        .join(&project.web.pages[0].css_chunks[1]);
    let page_css = fs::read_to_string(page_css_path).expect("page css");
    assert!(page_css.contains(".p-10{padding:2.5rem;}"));
    assert!(page_css.contains(".px-0\\.5{padding-left:0.125rem;padding-right:0.125rem;}"));
    assert!(page_css.contains(".md\\:p-8"));
    assert!(page_css.contains(".lg\\:p-5"));
    assert!(!page_css.contains(".lg\\:p-8"));
    assert!(page_css.contains(".lg\\:gap-6"));
    assert!(page_css.contains(".direction-column{flex-direction:column;}"));
    assert!(page_css.contains(".md\\:direction-row{flex-direction:row;}"));
    assert!(page_css.contains(".flex-wrap{flex-wrap:wrap;}"));
    assert!(page_css.contains(".title-2xl{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));font-size:clamp(1.75rem, 1.4rem + 1vw, 2.25rem);line-height:1.2;font-weight:700;letter-spacing:-0.025em;margin:0;}"));
    assert!(page_css.contains(".text-md{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));font-size:clamp(0.875rem, 0.82rem + 0.25vw, 1rem);line-height:1.6;font-weight:400;margin:0;}"));
    assert!(page_css.contains(".color-primaryText{color:var(--dowe-primaryText);}"));
    assert!(page_css.contains(".button-md{padding:0.625rem 1rem;min-height:2.5rem;}"));
    assert!(page_css.contains(".button-lg{padding:0.75rem 1.25rem;min-height:2.75rem;}"));
    assert!(page_css.contains(".min-h-10{min-height:2.5rem;}"));
    assert!(page_css.contains(".rounded-full{border-radius:9999px;}"));
    assert!(page_css.contains(".weight-extrabold{font-weight:800;}"));
    assert!(page_css.contains(".tracking-wide{letter-spacing:0.02em;}"));
    assert!(!page_css.contains(".p-96"));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("object DoweDesign"));
    assert!(android.contains("var softWarning"));
    assert!(android.contains("Button("));
    assert!(android.contains("DoweDesign.danger"));
    assert!(android.contains("DoweDesign.softWarning"));
    assert!(android.contains("DoweDesign.softWarningText"));
    assert!(android.contains("all = doweResponsive(viewportWidth, xs = 16.dp, md = 32.dp)"));
    assert!(android.contains("all = doweResponsive(viewportWidth, xs = 16.dp, lg = 20.dp)"));
    assert!(android.contains(
        "contentPadding = PaddingValues(start = doweResponsive(viewportWidth, xs = 20.dp) ?: 0.dp, top = doweResponsive(viewportWidth, xs = 12.dp) ?: 0.dp, end = doweResponsive(viewportWidth, xs = 20.dp) ?: 0.dp, bottom = doweResponsive(viewportWidth, xs = 12.dp) ?: 0.dp)"
    ));
    assert!(android.contains("doweResponsive(viewportWidth, xs = DoweSize.Fixed(44.dp))"));
    assert!(android.contains(
        "RoundedCornerShape(doweResponsive(viewportWidth, xs = 999.dp) ?: DoweDesign.radius)"
    ));
    assert!(android.contains("DoweInput("));
    assert!(android.contains("DoweDesign.info"));
    assert!(android.contains("FontWeight.ExtraBold"));
    assert!(android.contains("xs = (-0.02f).em"));
    assert!(android.contains("DoweDesign.softPrimary"));
    assert!(android.contains("if ((doweResponsive(viewportWidth, xs = DoweFlexDirection.Column, md = DoweFlexDirection.Row) ?: DoweFlexDirection.Row) == DoweFlexDirection.Column)"));
    assert!(android.contains("Column(modifier ="));
    assert!(android.contains("Row(modifier ="));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("final class DoweDesign: ObservableObject"));
    assert!(ios.contains("static var softWarning"));
    assert!(ios.contains("Button(action: {})"));
    assert!(ios.contains("DoweDesign.danger"));
    assert!(ios.contains("DoweDesign.softWarning"));
    assert!(ios.contains("DoweDesign.softWarningText"));
    assert!(ios.contains("if (doweResponsive(viewportWidth, xs: DoweFlexDirection.column, md: DoweFlexDirection.row) ?? DoweFlexDirection.row) == DoweFlexDirection.column"));
    assert!(ios.contains("VStack(alignment:"));
    assert!(ios.contains("HStack(alignment:"));
    assert!(ios.contains(
        ".padding(EdgeInsets(top: doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(32)) ?? CGFloat(0)"
    ));
    assert!(ios.contains(
        ".padding(EdgeInsets(top: doweResponsive(viewportWidth, xs: CGFloat(16), lg: CGFloat(20)) ?? CGFloat(0)"
    ));
    assert!(ios.contains(
        "leading: doweResponsive(viewportWidth, xs: CGFloat(20)) ?? CGFloat(0)"
    ));
    assert!(ios.contains(
        "top: doweResponsive(viewportWidth, xs: CGFloat(12)) ?? CGFloat(0)"
    ));
    assert!(ios.contains("DoweSize.fixed(CGFloat(44))"));
    assert!(ios.contains(
            "RoundedRectangle(cornerRadius: doweResponsive(viewportWidth, xs: CGFloat(999)) ?? DoweDesign.radius)"
        ));
    assert!(
        ios.contains("DoweInputField(value: nil, label: nil, placeholder: \"\", floating: false")
    );
    assert!(ios.contains("Font.Weight.heavy"));
    assert!(ios.contains("doweTextTracking"));
    assert!(ios.contains("DoweDesign.softPrimary"));
}

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
    let android = fs::read_to_string(temp.path().join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt")).expect("android");
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
      Sidebar show:{ xs:false md:true } w:96 pr:6 variant:"soft" scheme:"surface" rounded:"lg" border:1 p:4
        body
          SideNav variant:"ghost" scheme:"muted" size:"sm" wide:true
            DocsNavigation
    main
      Drawer show:{ xs:true md:false } open:openDrawer position:"start" variant:"soft" scheme:"surface" disableOverlayClose:false hideCloseButton:false p:4 w:80
        body
          SideNav variant:"ghost" scheme:"muted" size:"sm" wide:true
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
    assert!(body.contains(r#"<nav class="sidenav is-ghost is-muted sidenav-sm is-wide""#));
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
      AppBar position:"fixed" variant:"soft" scheme:"surface" bordered:true
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
    let css = fs::read_to_string(temp.path().join(".dowe/web/design.css")).expect("css");

    assert!(body.contains("position-fixed"));
    assert!(body.contains(r#"<aside class="scaffold-start">"#));
    assert!(body.contains(r#"<main class="scaffold-main">"#));
    assert!(body.contains(r#"<aside class="scaffold-end">"#));
    assert!(!body.contains("vh-"));
    assert!(css.contains("padding-top:var(--dowe-scaffold-top-inset,0px)"));
    assert!(css.contains("max-height:calc(100vh - var(--dowe-scaffold-top-inset,0px))"));
    assert!(project.web.router_js.contains("function hydrateScaffoldInsets(root)"));
    assert!(project.web.router_js.contains("appBar.getBoundingClientRect().bottom"));
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
    assert!(router.contains(
        "scrollIntoView({behavior:reduce?\"auto\":\"smooth\",block:\"start\"})"
    ));
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
    assert!(props_error
        .to_string()
        .contains("component `DocsNavigation` cannot declare args, props or children"));

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
    assert!(metadata_error
        .to_string()
        .contains("component exports cannot declare signal, fn, request or meta"));

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
    assert!(cycle_error
        .to_string()
        .contains("component import cycle includes"));
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
    assert!(error
        .to_string()
        .contains("view modules must export a layout or page"));
}

#[test]
fn compiles_theme_fab_slider_and_dropzone_across_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r##"page loginPage
  signal volume value:40
  fn resetVolume
    reset volume
  Box id:"top"
    ToggleTheme variant:"soft" scheme:"secondary" size:"sm" lightLabel:"Light mode" darkLabel:"Dark mode"
    SelectTheme label:"Theme palette" placeholder:"Choose a palette" variant:"outlined" scheme:"surface" size:"sm"
    Fab position:"bottom-left" offsetX:6 offsetY:8 icon:"settings" label:"Actions" variant:"soft" scheme:"primary" size:"lg" onClick:resetVolume
      fabAction label:"Top" icon:"link" href:"#top" navigate:"replace" scheme:"info"
      fabAction label:"Reset" icon:"dismiss" onClick:resetVolume scheme:"danger"
    Slider bind:volume value:40 min:0 max:100 step:5 label:"Volume" name:"volume" scheme:"warning" size:"lg"
    Slider value:40 min:0 max:100 step:5 label:"Static volume" scheme:"warning" size:"md"
    Dropzone accept:"image/*" multiple:false maxSize:4096 name:"images" label:"Images" helpText:"PNG only" placeholder:"Drop images" variant:"outlined" scheme:"surface" size:"sm""##,
    );
    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  design defaultTheme:"light"
    theme name:"light"
      colors:
        primary color:"#000000" text:"#ffffff" title:"#ffffff"
    theme name:"dark"
      colors:
        primary color:"#ffffff" text:"#000000" title:"#000000""##,
    )
    .expect("theme");

    let project = compile_dev(temp.path()).expect("project");
    let page = &project.web.pages[0];
    let body = &page.body_html;

    assert!(page.html_document.contains("theme-preference"));
    assert!(page.html_document.contains("prefers-color-scheme"));
    assert!(body.contains("data-dowe-theme-toggle"));
    assert!(body.contains("data-dowe-theme-select"));
    assert!(body.contains("data-dowe-select"));
    assert!(body.contains(r#"data-dowe-option-value="light" data-dowe-option-label="Light""#));
    assert!(body.contains(r#"data-dowe-option-value="dark" data-dowe-option-label="Dark""#));
    assert!(body.contains(r#"data-dowe-light-label="Light mode""#));
    assert!(body.contains(r#"data-dowe-dark-label="Dark mode""#));
    assert!(body.contains(r#"class="fab-container is-bottom-left is-fixed""#));
    assert!(body.contains("data-dowe-fab-trigger"));
    assert!(body.contains("data-dowe-fab-action"));
    assert!(body.contains(r##"href="#top" data-dowe-nav="replace""##));
    assert!(body.contains("data-dowe-slider"));
    assert!(body.contains("data-dowe-bind="));
    assert!(body.contains(r#"style="--dowe-slider-progress:40%""#));
    assert!(body.contains("data-dowe-dropzone"));
    assert!(body.contains(r#"data-dowe-dropzone-max-size="4096""#));
    assert!(body.contains("Drop images"));

    let css = fs::read_to_string(temp.path().join(".dowe/web/design.css")).expect("css");
    assert!(css.contains("html.theme-transitioning"));
    assert!(css.contains("html.page-transitioning"));
    assert!(css.contains(".theme-toggle"));
    assert!(css.contains(".select-control"));
    assert!(!css.contains(".theme-select-input"));
    assert!(css.contains(".fab-container"));
    assert!(css.contains(".slider-wrapper"));
    assert!(css.contains(".dropzone-input"));

    let router = fs::read_to_string(temp.path().join(".dowe/web/router.js")).expect("router");
    assert!(router.contains("theme-preference"));
    assert!(router.contains("startViewTransition"));
    assert!(router.contains("hydrateThemeToggles"));
    assert!(router.contains("hydrateThemeSelects"));
    let render_select = router
        .split("function renderSelect(control,state,scope){")
        .nth(1)
        .and_then(|section| section.split("function renderSelects").next())
        .expect("renderSelect source");
    assert!(!render_select.contains("applyDoweTheme"));
    assert!(router.contains("if(control.dataset.doweThemeSelect!==undefined&&value)applyDoweTheme(value,true);"));
    assert!(router.contains("hydrateFabs"));
    assert!(router.contains("hydrateSliders"));
    assert!(router.contains("hydrateDropzones"));
    assert!(router.contains("function pageScrollViewport()"));
    assert!(router.contains("scrollToPageDestination(currentFragment)"));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("DoweThemeToggle("));
    assert!(android.contains("DoweThemeSelect("));
    assert!(android.contains("DoweThemeModule.names"));
    assert!(android.contains("DoweSliderField("));
    assert!(android.contains("DoweDropzone("));
    assert!(android.contains("rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument())"));
    assert!(android.contains("rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments())"));
    assert!(android.contains("doweDropzoneMimeTypes(accept)"));
    assert!(android.contains("maxSize = 4096L"));
    assert!(android.contains("\"theme-preference\""));
    assert!(android.contains("state.write("));
    assert!(android.contains("it.toDouble()"));
    assert!(android.contains("LaunchedEffect(currentEntry.path)"));
    assert!(android.contains("scrollState.scrollTo(0)"));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("getSharedPreferences(\"dowe\", 0)"));
    assert!(android_dev.contains("doweSelectTrigger"));
    assert!(android_dev.contains("doweBindSelect("));
    assert!(!android_dev.contains("android.widget.Spinner"));
    assert!(android_dev.contains("new SeekBar(this)"));
    assert!(android_dev.contains("Drop images"));
    assert!(android_dev.contains("Intent.ACTION_OPEN_DOCUMENT"));
    assert!(android_dev.contains("handleActivityResult(int requestCode"));
    assert!(android_dev.contains("scrollView.scrollTo(0, 0);"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("DoweDesign.applyTheme"));
    assert!(ios.contains("DoweSelectField(value: Optional(Binding(get:"));
    assert!(ios.contains("DoweSelectOption(value: \"dark\", label: \"Dark\""));
    assert!(ios.contains("DoweSliderView(value:"));
    assert!(ios.contains("Binding<Double>.constant(40.0)"));
    assert!(ios.contains("Text(String(format: \"%.0f\", value.wrappedValue))"));
    assert!(!ios.contains(".constant(40).wrappedValue"));
    assert!(ios.contains("Image(systemName: selectedFiles.isEmpty ? \"paperclip\" : \"doc.on.doc\")"));
    assert!(ios.contains(".fileImporter("));
    assert!(ios.contains("doweDropzoneFileTypes(accept)"));
    assert!(ios.contains("allowsMultipleSelection: multiple"));
    assert!(ios.contains("resourceValues(forKeys: [.nameKey, .fileSizeKey])"));
    assert!(ios.contains("StrokeStyle(lineWidth: CGFloat(2), dash: [CGFloat(6)])"));
    assert!(ios.contains("proxy.scrollTo(\"__dowe_page_top\", anchor: .top)"));
    assert!(ios.contains(".id(\"__dowe_page_top\")"));
    assert!(ios.contains("Drop images"));
}

#[test]
fn compiles_expanded_text_weight_overrides() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Box
    Text weight:{ xs:"thin" md:"extralight" lg:"black" }
      "Weighted text"
    Title weight:"black"
      "Weighted title""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;

    assert!(body.contains(r#"class="dowe-text text-md weight-thin md:weight-extralight lg:weight-black""#));
    assert!(body.contains(r#"class="dowe-title title-md weight-black""#));

    let page_css_path = temp
        .path()
        .join(".dowe/web")
        .join(&project.web.pages[0].css_chunks[1]);
    let page_css = fs::read_to_string(page_css_path).expect("page css");
    assert!(page_css.contains(".weight-thin{font-weight:100;}"));
    assert!(page_css.contains(".md\\:weight-extralight{font-weight:200;}"));
    assert!(page_css.contains(".lg\\:weight-black{font-weight:900;}"));
    assert!(page_css.contains(".weight-black{font-weight:900;}"));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("FontWeight.Thin"));
    assert!(android.contains("FontWeight.ExtraLight"));
    assert!(android.contains("FontWeight.Black"));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains(
        "doweTextWeight(doweResponsiveInt(viewportWidth, 100, null, 200, 900, null), 400)"
    ));
    assert!(android_dev.contains(
        "doweTextWeight(doweResponsiveInt(viewportWidth, 900, null, null, null, null), 400)"
    ));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("Font.Weight.ultraLight"));
    assert!(ios.contains("Font.Weight.thin"));
    assert!(ios.contains("Font.Weight.black"));
}

#[test]
fn compiles_platform_reset_and_font_tokens() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box font:"roboto"
    Text
      "Layout"
    children"#,
        r#"page loginPage
  Box font:{ xs:"inter" md:"lato" }
    Text
      "Inherited"
    Text font:"manrope"
      "Lead"
    Title font:"poppins"
      "Login"
    Button font:"montserrat"
      "Submit"
    Input font:"roboto"
    Text font:"lora"
      "Caption""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    assert!(body.contains(r#"class="box font-roboto""#));
    assert!(body.contains(r#"class="box font-inter md:font-lato""#));
    assert!(body.contains(r#"class="dowe-text text-md font-manrope""#));
    assert!(body.contains(r#"class="dowe-title title-md font-poppins""#));
    assert!(body.contains(
        r#"class="button button-md font-montserrat px-4 py-2.5 min-h-10 rounded-md is-solid is-primary""#
    ));
    assert!(body.contains("font-roboto"));
    assert!(body.contains("is-md font-roboto is-outlined is-primary"));
    assert!(body.contains(r#"class="dowe-text text-md font-lora""#));

    let css = fs::read_to_string(temp.path().join(".dowe/web/design.css")).expect("css");
    assert!(css.contains("body{--dowe-content-text:var(--dowe-backgroundText);--dowe-content-title:var(--dowe-backgroundTitle);margin:0;"));
    assert!(css.contains("scroll-behavior:smooth;"));
    assert!(css.contains("p,h1,h2,h3,h4,h5,h6{margin:0;"));
    assert!(css.contains("a{color:inherit;text-decoration:inherit;}"));
    assert!(css.contains("button,input,textarea,select{font:inherit;color:inherit;margin:0;}"));
    assert!(css.contains("::-webkit-scrollbar{width:var(--dowe-scrollbar-size,6px);"));
    assert!(css.contains("::-webkit-scrollbar-thumb{background:color-mix(in oklch,currentColor 80%,transparent);border-radius:999px;}"));
    assert!(css.contains("@supports not selector(::-webkit-scrollbar){*{scrollbar-width:thin;"));
    assert!(css.contains("--dowe-font-inter"));
    assert!(css.contains("@font-face{font-family:\"Dowe Inter\""));
    assert!(css.contains("font-weight:100;src:url(\"/fonts/inter/inter-light.ttf\")"));
    assert!(css.contains("src:url(\"/fonts/inter/inter-regular.ttf\") format(\"truetype\")"));
    assert!(css.contains("font-weight:900;src:url(\"/fonts/inter/inter-extrabold.ttf\")"));
    assert!(temp
        .path()
        .join(".dowe/fonts/inter/inter-regular.ttf")
        .is_file());
    assert!(!temp.path().join(".dowe/fonts/quicksand").exists());

    let page_css_path = temp
        .path()
        .join(".dowe/web")
        .join(&project.web.pages[0].css_chunks[1]);
    let page_css = fs::read_to_string(page_css_path).expect("page css");
    assert!(page_css.contains(".font-poppins{font-family:var(--dowe-font-poppins);}"));
    assert!(page_css.contains(".font-manrope{font-family:var(--dowe-font-manrope);}"));
    assert!(page_css.contains(".font-lora{font-family:var(--dowe-font-lora);}"));
    assert!(page_css.contains(".md\\:font-lato{font-family:var(--dowe-font-lato);}"));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("private enum class DoweFont"));
    assert!(android.contains("Font(R.font.inter_light, FontWeight.Thin)"));
    assert!(android.contains("Font(R.font.inter_regular, FontWeight.Normal)"));
    assert!(android.contains("Font(R.font.inter_extrabold, FontWeight.Black)"));
    assert!(android.contains("DoweFont.Lato -> DoweFonts.lato"));
    assert!(android.contains("DoweFont.Manrope -> DoweFonts.manrope"));
    assert!(android.contains("DoweFont.Lora -> DoweFonts.lora"));
    assert!(
        android.contains("doweResponsive(viewportWidth, xs = DoweFont.Inter, md = DoweFont.Lato)")
    );
    assert!(android.contains("xs = DoweFont.Poppins"));
    assert!(android.contains("contentPadding = PaddingValues(0.dp)"));
    assert!(temp
        .path()
        .join(".dowe/apps/android/app/src/main/res/font/inter_regular.ttf")
        .is_file());
    assert!(temp
        .path()
        .join(".dowe/apps/android/app/src/main/res/font/manrope_regular.ttf")
        .is_file());
    assert!(temp
        .path()
        .join(".dowe/apps/android/app/src/main/res/font/lora_regular.ttf")
        .is_file());

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("setAllCaps(false)"));
    assert!(android_dev
        .contains("doweResponsiveString(viewportWidth, \"Inter\", null, \"Lato\", null, null)"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("enum DoweFont"));
    assert!(ios.contains("doweResponsive(viewportWidth, xs: .inter, md: .lato)"));
    assert!(ios.contains("xs: .poppins"));
    assert!(ios.contains("xs: .manrope"));
    assert!(ios.contains("xs: .lora"));
    assert!(ios.contains(".buttonStyle(.plain)"));
    assert!(ios.contains(".textFieldStyle(.plain)"));
    assert!(temp
        .path()
        .join(".dowe/apps/ios/Fonts/inter-regular.ttf")
        .is_file());
    assert!(temp
        .path()
        .join(".dowe/apps/ios/Fonts/manrope-regular.ttf")
        .is_file());
    assert!(temp
        .path()
        .join(".dowe/apps/ios/Fonts/lora-regular.ttf")
        .is_file());
    let plist = fs::read_to_string(temp.path().join(".dowe/apps/ios/Info.plist")).expect("plist");
    assert!(plist.contains("UIAppFonts"));
    assert!(plist.contains("Fonts/inter-regular.ttf"));
}

#[test]
fn compiles_configured_font_install_set() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    Text
      "Layout"
    children"#,
        r#"page loginPage
  Box
    Text
      "Login""#,
    );
    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  fonts default:"manrope" install:["lora"]"#,
    )
    .expect("config");

    let project = compile_dev(temp.path()).expect("project");
    assert_eq!(
        project.font_config.default_family,
        dowe_components::FontFamily::Manrope
    );

    let css = fs::read_to_string(temp.path().join(".dowe/web/design.css")).expect("css");
    assert!(css.contains("html{font-family:var(--dowe-font-manrope);"));
    assert!(css.contains("body{--dowe-content-text:var(--dowe-backgroundText);--dowe-content-title:var(--dowe-backgroundTitle);margin:0;"));
    assert!(css.contains("--dowe-font-manrope"));
    assert!(css.contains("--dowe-font-lora"));
    assert!(!css.contains("--dowe-font-inter"));
    assert!(!css.contains("--dowe-font-poppins"));
    assert!(temp
        .path()
        .join(".dowe/fonts/manrope/manrope-regular.ttf")
        .is_file());
    assert!(temp
        .path()
        .join(".dowe/fonts/lora/lora-regular.ttf")
        .is_file());
    assert!(!temp.path().join(".dowe/fonts/inter").exists());

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("null -> DoweFonts.manrope"));
    assert!(android.contains("DoweFont.Lora -> DoweFonts.lora"));
    assert!(!android.contains("R.font.inter_regular"));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("return value == null ? \"Manrope\" : value;"));

    let plist = fs::read_to_string(temp.path().join(".dowe/apps/ios/Info.plist")).expect("plist");
    assert!(plist.contains("Fonts/manrope-regular.ttf"));
    assert!(plist.contains("Fonts/lora-regular.ttf"));
    assert!(!plist.contains("Fonts/inter-regular.ttf"));
}

#[test]
fn compiles_syne_jost_and_puritan_across_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Box
    Text font:"jost"
      "Jost"
    Text font:"puritan"
      "Puritan""#,
    );
    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  fonts default:"syne" install:["jost","puritan"]"#,
    )
    .expect("theme");

    compile_dev(temp.path()).expect("project");

    let css = fs::read_to_string(temp.path().join(".dowe/web/design.css")).expect("css");
    assert!(css.contains("--dowe-font-syne"));
    assert!(css.contains("/fonts/syne/syne-variable.ttf"));
    assert!(css.contains("--dowe-font-jost"));
    assert!(css.contains("--dowe-font-puritan"));
    assert!(temp
        .path()
        .join(".dowe/apps/android/app/src/main/res/font/jost_variable.ttf")
        .is_file());
    assert!(temp
        .path()
        .join(".dowe/apps/android/app/src/main/res/font/puritan_bold.ttf")
        .is_file());

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("case syne"));
    assert!(ios.contains("case jost"));
    assert!(ios.contains("case puritan"));
    assert!(temp
        .path()
        .join(".dowe/apps/ios/Fonts/syne-variable.ttf")
        .is_file());
}

#[test]
fn compiles_design_tokens_from_theme_dowe() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    Text
      "Layout"
    children"#,
    r#"page loginPage
  Box
    Card
      Text
        "Default"
      Button
        "Go"
    Tabs
      tab id:"overview" label:"Overview"
        Text
          "Overview"
    Card variant:"soft" scheme:"success" p:5 rounded:"md"
      Text
        "Override""#,
    );
    fs::write(
        temp.path().join("theme.dowe"),
        r##"theme
  fonts default:"inter" install:["inter"]
  design defaultTheme:"light"
    Card radius:"md" shadow:"xs" shadowColor:"primary" border:1 borderColor:"primary" scheme:"surface" variant:"outline"
    Button radius:"md" scheme:"primary" variant:"solid" size:"md"
    Tabs variant:"line"
    Chip radius:"md"
    Avatar radius:"full"
    theme name:"light"
      colors:
        primary color:"#000000" text:"#ffffff" title:"#ffffff"
    theme name:"dark"
      colors:
        primary color:"#ffffff" text:"#000000" title:"#000000""##,
    )
    .expect("config");

    let project = compile_dev(temp.path()).expect("project");

    assert_eq!(project.design_config.default_theme, "light");
    assert!(project.design_config.theme("dark").is_some());
    assert_eq!(
        project
            .design_config
            .defaults
            .tabs_variant
            .get(&dowe_components::DesignComponentSlot::Tabs),
        Some(&dowe_components::TabsVariant::Line)
    );

    let body = &project.web.pages[0].body_html;
    assert!(body.contains("card p-4 lg:p-5 rounded-md border-1 border-color-primary shadow-xs shadow-color-primary is-outlined is-surface"));
    assert!(body.contains("button button-md px-4 py-2.5 min-h-10 rounded-md is-solid is-primary"));
    assert!(body.contains("card p-5 rounded-md border-1 border-color-primary shadow-xs shadow-color-primary is-soft is-success"));
    assert!(body.contains("tabs"));

    let css = fs::read_to_string(temp.path().join(".dowe/web/design.css")).expect("css");
    assert!(css.contains("--dowe-primary:#000000;"));
    assert!(css.contains("--dowe-radius:8px;"));
    assert!(css.contains("[data-dowe-theme=\"dark\"]{"));
    assert!(css.contains("--dowe-primary:#ffffff;"));
    let page_css_path = temp
        .path()
        .join(".dowe/web")
        .join(&project.web.pages[0].css_chunks[1]);
    let page_css = fs::read_to_string(page_css_path).expect("page css");
    assert!(page_css.contains(".border-color-primary{border-color:var(--dowe-primary);"));
    assert!(page_css.contains(".shadow-xs{box-shadow:"));
    assert!(page_css.contains(".shadow-color-primary{--dowe-shadow-color:"));

    let android_theme = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DoweTheme.kt"),
    )
    .expect("android theme");
    assert!(android_theme.contains("const val defaultTheme = \"light\""));
    assert!(android_theme.contains("\"dark\""));
    assert!(android_theme.contains("\"primary\" to Color(0xFF000000)"));
    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("private static int DOWE_PRIMARY = Color.rgb(0, 0, 0);"));
    assert!(android_dev.contains("private static float DOWE_RADIUS = 8f;"));
    let android_pages = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android pages");
    assert!(android_pages.contains(".doweShadow("));
    assert!(android_pages.contains("color = DoweDesign.primary"));

    let ios_theme =
        fs::read_to_string(temp.path().join(".dowe/apps/ios/DoweTheme.swift")).expect("ios theme");
    assert!(ios_theme.contains("static let defaultTheme = \"light\""));
    assert!(ios_theme.contains("\"dark\""));
    assert!(ios_theme.contains("\"primary\": Color(red: 0.000, green: 0.000, blue: 0.000)"));
    let ios_pages = ios_swift_output(temp.path());
    assert!(ios_pages.contains(
        ".background(DoweShadowSurface(shadow: DoweShadowSpec(color: DoweDesign.primary.opacity(0.28), blurRadius: doweResponsive(viewportWidth, xs: CGFloat(2)) ?? CGFloat(0), offsetY: doweResponsive(viewportWidth, xs: CGFloat(1)) ?? CGFloat(0))"
    ));
}

#[test]
fn compiles_text_and_title_font_defaults_from_theme_dowe() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    Text
      "Layout"
    children"#,
        r#"page loginPage
  Box
    Text
      "Default text"
    Text font:"inter"
      "Override text"
    Title
      "Default title"
    Title font:"inter"
      "Override title""#,
    );
    fs::write(
        temp.path().join("theme.dowe"),
        r#"theme
  fonts default:"inter" install:["inter"]
  design defaultTheme:"light"
    Text font:"manrope"
    Title font:"syne"
    theme name:"light""#,
    )
    .expect("theme");

    let project = compile_dev(temp.path()).expect("project");
    let defaults = &project.design_config.defaults;
    assert_eq!(
        defaults.font.get(&dowe_components::DesignComponentSlot::Text),
        Some(&dowe_components::FontFamily::Manrope)
    );
    assert_eq!(
        defaults.font.get(&dowe_components::DesignComponentSlot::Title),
        Some(&dowe_components::FontFamily::Syne)
    );

    let body = &project.web.pages[0].body_html;
    assert!(body.contains(r#"class="dowe-text text-md font-manrope">Default text"#));
    assert!(body.contains(r#"class="dowe-text text-md font-inter">Override text"#));
    assert!(body.contains(r#"class="dowe-title title-md font-syne">Default title"#));
    assert!(body.contains(r#"class="dowe-title title-md font-inter">Override title"#));

    assert!(temp.path().join(".dowe/fonts/manrope/manrope-regular.ttf").is_file());
    assert!(temp.path().join(".dowe/fonts/syne/syne-variable.ttf").is_file());
    assert!(temp
        .path()
        .join(".dowe/apps/android/app/src/main/res/font/manrope_regular.ttf")
        .is_file());
    assert!(temp
        .path()
        .join(".dowe/apps/android/app/src/main/res/font/syne_variable.ttf")
        .is_file());
    assert!(temp
        .path()
        .join(".dowe/apps/ios/Fonts/manrope-regular.ttf")
        .is_file());
    assert!(temp
        .path()
        .join(".dowe/apps/ios/Fonts/syne-variable.ttf")
        .is_file());

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("xs = DoweFont.Manrope"));
    assert!(android.contains("xs = DoweFont.Syne"));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("\"Manrope\".equals(font)"));
    assert!(android_dev.contains("return R.font.manrope_regular;"));
    assert!(android_dev.contains("\"Syne\".equals(font)"));
    assert!(android_dev.contains("return R.font.syne_variable;"));
    assert!(android_dev.contains("Typeface bundled = getResources().getFont(resource);"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("xs: .manrope"));
    assert!(ios.contains("xs: .syne"));
}

#[test]
fn compiles_mobile_responsive_runtime_values() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box p:{ xs:4 md:8 }
    Text
      "Layout"
    children"#,
        r#"page loginPage
  Box p:{ md:8 }
    Text size:{ md:"lg" }
      "Login""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;

    assert!(body.contains(r#"class="box p-4 md:p-8""#));
    assert!(body.contains(r#"class="box md:p-8""#));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("BoxWithConstraints"));
    assert!(android.contains(
        "fun IndexScreen(viewportWidth: Dp, scrollState: ScrollState, sectionRegistry: DoweSectionRegistry, navigate:"
    ));
    assert!(android.contains("doweResponsive(viewportWidth, xs = 16.dp, md = 32.dp)"));
    assert!(android.contains("doweResponsive(viewportWidth, md = 32.dp)"));
    assert!(android.contains(
            "doweResponsive(viewportWidth, md = doweTextSize(viewportWidth, min = 16f, preferredBase = 15.2f, preferredViewport = 0.3f, max = 18f)) ?: doweTextSize(viewportWidth, min = 14f, preferredBase = 13.12f, preferredViewport = 0.25f, max = 16f)"
        ));

    let android_dev = android_dev_output(temp.path());
    assert!(
        android_dev.contains("viewportWidth = getResources().getConfiguration().screenWidthDp;")
    );
    assert!(android_dev.contains("int viewportWidth = this.viewportWidth;"));
    assert!(android_dev.contains("doweResponsiveInt(viewportWidth, 16, null, 32, null, null)"));
    assert!(android_dev.contains("doweResponsiveInt(viewportWidth, null, null, 32, null, null)"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("GeometryReader { geometry in"));
    assert!(ios.contains("let viewportWidth: CGFloat"));
    assert!(ios.contains(
        ".padding(EdgeInsets(top: doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(32)) ?? CGFloat(0)"
    ));
    assert!(ios.contains("top: doweResponsive(viewportWidth, md: CGFloat(32)) ?? CGFloat(0)"));
    assert!(ios.contains(
            ".font(doweFont(.inter, size: doweResponsive(viewportWidth, md: doweTextSize(viewportWidth, min: CGFloat(16), preferredBase: CGFloat(15.2), preferredViewport: CGFloat(0.3), max: CGFloat(18))) ?? doweTextSize(viewportWidth, min: CGFloat(14), preferredBase: CGFloat(13.12), preferredViewport: CGFloat(0.25), max: CGFloat(16))))"
        ));
    assert!(ios.contains(".fontWeight(Font.Weight.regular)"));
}
