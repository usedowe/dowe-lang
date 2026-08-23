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

    assert!(
        temp.path()
            .join(".dowe/apps/android/app/src/main/assets/avatars/ada.png")
            .is_file()
    );
    assert!(temp.path().join(".dowe/apps/ios/assets/avatars/ada.png").is_file());
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
    assert!(body.contains(
        r#"class="dowe-title title-2xl bg-softPrimary p-4 weight-extrabold tracking-tight""#
    ));
    assert!(body.contains(
            r#"class="dowe-text text-md bg-surface color-primaryText rounded-md border-1 weight-bold tracking-wide""#
        ));
    assert!(body.contains(
        r#"class="button button-md px-4 py-2.5 min-h-10 rounded-md is-solid is-danger""#
    ));
    assert!(body.contains(
        r#"class="button button-lg px-5 py-3 min-h-11 rounded-full is-soft is-warning""#
    ));
    assert!(
        body.contains(
            r#"<div class="control is-md is-outlined is-info"><input class="input"></div>"#
        )
    );
    assert!(body.contains(r#"class="card p-4 lg:p-5 rounded-md is-solid is-primary""#));

    let css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.design_file_name()),
    )
    .expect("css");
    assert!(css.contains("--dowe-primary"));
    assert!(css.contains("--dowe-softDanger"));
    assert!(!css.contains(".p-96"));
    let layout_css_path = temp
        .path()
        .join(".dowe/web")
        .join(generated_css_chunk(
            &project.web.pages[0].css_chunks,
            "chunks/layouts/",
        ));
    let layout_css = fs::read_to_string(layout_css_path).expect("layout css");
    assert!(layout_css.contains(".color-backgroundText{color:var(--dowe-backgroundText);}"));

    let page_css_path = temp
        .path()
        .join(".dowe/web")
        .join(generated_css_chunk(
            &project.web.pages[0].css_chunks,
            "chunks/pages/",
        ));
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
    assert!(ios.contains("leading: doweResponsive(viewportWidth, xs: CGFloat(20)) ?? CGFloat(0)"));
    assert!(ios.contains("top: doweResponsive(viewportWidth, xs: CGFloat(12)) ?? CGFloat(0)"));
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

