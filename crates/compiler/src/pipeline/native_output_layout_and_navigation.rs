#[test]
fn compiles_flex_defaults_and_alignment_across_native_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Flex justify:"center" align:"center"
    Text
      "Flex""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    let ios = ios_swift_output(temp.path());
    let css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.design_file_name()),
    )
    .expect("css");
    let page_css_path = temp.path().join(".dowe/web").join(generated_css_chunk(
        &project.web.pages[0].css_chunks,
        "chunks/pages/",
    ));
    let page_css = fs::read_to_string(page_css_path).expect("page css");

    assert!(
        project.web.pages[0]
            .body_html
            .contains("flex direction-row justify-center align-center")
    );
    assert!(css.contains(".flex{--dowe-component-display:flex;display:var(--dowe-show,var(--dowe-component-display));width:100%;height:auto;}"));
    assert!(page_css.contains(".justify-center{justify-content:center;}"));
    assert!(page_css.contains(".align-center{align-items:center;}"));
    assert!(android.contains(
        "Column(modifier = Modifier.fillMaxWidth(), verticalArrangement = doweVerticalArrangement"
    ));
    assert!(android.contains("horizontalAlignment = doweHorizontalAlignment"));
    assert!(ios.contains("VStack(alignment:") || ios.contains("HStack(alignment:"));
    assert!(ios.contains(".frame(maxWidth: .infinity"));
    assert!(!android.contains("doweHeight(DoweSize"));
    assert!(!ios.contains("frame(height: doweFixedSize"));
}

#[test]
fn compiles_refactored_container_props() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Grid columns:{ xs:1 md:3 } rows:2 gap:"10px 20px" justify:"center" align:"end"
    Box colSpan:{ md:2 } cover:{ xs:"/mobile.jpg" md:"/desktop.jpg" } overlay:0.4
      Text
        "Hero"
    Card variant:"solid" scheme:"surface" rounded:"full" rowSpan:2 cover:"/images/card.jpg" overlay:0.6
      Text
        "Card""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;

    assert!(body.contains("grid-cols-1 md:grid-cols-3"));
    assert!(body.contains("grid-rows-"));
    assert!(body.contains("gap-"));
    assert!(body.contains("grid-justify-center"));
    assert!(body.contains("grid-align-end"));
    assert!(body.contains("md:col-span-2"));
    assert!(body.contains("has-cover"));
    assert!(body.contains("has-overlay"));
    assert!(body.contains("p-4 lg:p-5"));
    assert!(body.contains("is-solid is-surface"));

    let page_css_path = temp.path().join(".dowe/web").join(generated_css_chunk(
        &project.web.pages[0].css_chunks,
        "chunks/pages/",
    ));
    let page_css = fs::read_to_string(page_css_path).expect("page css");
    assert!(page_css.contains("grid-template-columns:repeat(3,minmax(0,1fr));"));
    assert!(page_css.contains("grid-template-rows:repeat(2,minmax(0,1fr));"));
    assert!(page_css.contains("row-gap:10px;column-gap:20px;"));
    assert!(page_css.contains("background-image:url(\"/mobile.jpg\")"));
    assert!(page_css.contains("background-image:url(\"/desktop.jpg\")"));
    assert!(page_css.contains(".lg\\:p-5"));
    assert!(page_css.contains("rgba(0,0,0,0.4)"));
    assert!(page_css.contains("rgba(0,0,0,0.6)"));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("DoweCoverBox"));
    assert!(android.contains("\"/desktop.jpg\""));
    assert!(android.contains("DoweOverlay.Solid(Color.Black.copy(alpha = 0.6f))"));
    assert!(android.contains("PaddingValues(start = doweResponsive(viewportWidth, xs = 16.dp, lg = 20.dp)"));
    assert!(android.contains("DoweGrid(modifier ="));
    assert!(android.contains("tracks = doweResponsive(viewportWidth, xs = listOf(1f), md = listOf(1f, 1f, 1f)) ?: listOf(1f)"));
    assert!(android.contains("horizontalGap = doweResponsive(viewportWidth, xs = 20.dp) ?: 0.dp"));
    assert!(android.contains("verticalGap = doweResponsive(viewportWidth, xs = 10.dp) ?: 0.dp"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("DoweCoverImage"));
    assert!(ios.contains("\"/desktop.jpg\""));
    assert!(ios.contains("DoweOverlay.color(Color.black.opacity(0.6))"));
    assert!(ios.contains(
            ".padding(EdgeInsets(top: doweResponsive(viewportWidth, xs: CGFloat(16), lg: CGFloat(20)) ?? CGFloat(0)"
        ));
    assert!(ios.contains("DoweGridLayout("));
}

#[test]
fn compiles_container_foreground_inheritance_for_all_view_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Box color:"primaryText"
    Text
      "Box inherited"
    Text color:"danger"
      "Box override"
  Card variant:"solid" scheme:"muted"
    Text
      "Card inherited"
    Title color:"warning"
      "Card override""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    assert!(body.contains("box color-primaryText"));
    assert!(body.contains("Box override"));
    assert!(body.contains("color-danger"));
    assert!(body.contains("card p-4 lg:p-5 rounded-md is-solid is-muted"));
    assert!(body.contains("Card override"));
    assert!(body.contains("color-warning"));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains(
            "CompositionLocalProvider(LocalContentColor provides (doweResponsive(viewportWidth, xs = DoweDesign.primaryText) ?: LocalContentColor.current))"
        ));
    assert!(
        android.contains("Text(\"Box inherited\", modifier = Modifier, color = Color.Unspecified")
    );
    assert!(android.contains("DoweDesign"));
    assert!(
        android.contains("Text(\"Card inherited\", modifier = Modifier, color = Color.Unspecified")
    );

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("Text(verbatim: \"Box inherited\")"));
    assert!(ios.contains(
            ".foregroundStyle(doweResponsive(viewportWidth, xs: DoweDesign.primaryText) ?? DoweDesign.backgroundText)"
        ));
    assert!(ios.contains("Text(verbatim: \"Card inherited\")"));
    assert!(ios.contains("DoweDesign"));
}

#[test]
fn compiles_layout_bars_without_ios_dividers() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    AppBar variant:"solid" scheme:"surface" position:"sticky" bordered:true boxed:true
      start
        Text
          "Dowe"
    children
    BottomBar variant:"solid" scheme:"surface" bordered:true boxed:true
      tab href:"/login" label:"Home"
        Icon name:"home"
    Footer scheme:"background" bordered:true boxed:true
      end
        Text
          "info@dowe.dev""#,
        r#"page loginPage
  Text
    "Login""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    assert!(project.web.pages[0].body_html.contains("position-sticky"));
    assert!(
        project.web.pages[0]
            .body_html
            .contains("footer px-4 md:px-6")
    );

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains(".zIndex(1)"));
    assert!(ios.contains("Text(verbatim: \"info@dowe.dev\")"));
    assert!(ios.contains(
        "leading: doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(24)) ?? CGFloat(0)"
    ));
    assert!(!ios.contains(".overlay(Rectangle().fill(DoweDesign.muted).frame(height: CGFloat(1))"));
    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains(".zIndex(1f)"));
    assert!(android.contains("horizontal = doweResponsive(viewportWidth, xs = 16.dp, md = 24.dp)"));
    let android_dev = android_dev_output(temp.path());
    assert!(
        android_dev
            .contains("PaddingX = doweResponsiveInt(viewportWidth, 16, null, 24, null, null)")
    );
    assert!(!ios.contains(
            ".overlay(RoundedRectangle(cornerRadius: CGFloat(0)).stroke(DoweDesign.muted, lineWidth: CGFloat(1)))"
        ));
}

#[test]
fn compiles_cross_target_typography_from_shared_metrics() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Box
    Text size:"9xl"
      "Body"
    Title size:"9xl"
      "Title""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let css = project.web.pages[0]
        .css_chunks
        .iter()
        .map(|chunk| {
            fs::read_to_string(temp.path().join(".dowe/web").join(chunk)).expect("css chunk")
        })
        .collect::<Vec<_>>()
        .join("");
    assert!(css.contains(
            ".text-9xl{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));font-size:clamp(2.5rem, 1.9rem + 2.8vw, 3.75rem);line-height:1.2;font-weight:400;margin:0;}"
        ));
    assert!(css.contains(
            ".title-9xl{--dowe-component-display:block;display:var(--dowe-show,var(--dowe-component-display));font-size:clamp(4.5rem, 3rem + 7vw, 8rem);line-height:1;font-weight:800;letter-spacing:-0.06em;margin:0;}"
        ));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains(
            "doweTextSize(viewportWidth, min = 40f, preferredBase = 30.4f, preferredViewport = 2.8f, max = 60f)"
        ));
    assert!(android.contains(
            "doweTextSize(viewportWidth, min = 72f, preferredBase = 48f, preferredViewport = 7f, max = 128f)"
        ));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("doweFluidTextSize(40f, 30.4f, 2.8f, 60f)"));
    assert!(android_dev.contains("doweFluidTextSize(72f, 48f, 7f, 128f)"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains(
            "doweTextSize(viewportWidth, min: CGFloat(40), preferredBase: CGFloat(30.4), preferredViewport: CGFloat(2.8), max: CGFloat(60))"
        ));
    assert!(ios.contains(
            "doweTextSize(viewportWidth, min: CGFloat(72), preferredBase: CGFloat(48), preferredViewport: CGFloat(7), max: CGFloat(128))"
        ));
}

