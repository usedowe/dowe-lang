#[test]
fn compiles_maximum_sizing_limits_across_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  Box\n    children",
        r#"page loginPage
  Box w:"full" maxW:{ xs:"full" md:64 } maxH:"vh-16"
    Text
      "Bounded""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let page = project.web.pages.first().expect("page");
    assert!(
        page.body_html
            .contains("max-w-full md:max-w-64 max-h-vh-16")
    );
    let page_css = page
        .css_chunks
        .iter()
        .map(|path| fs::read_to_string(temp.path().join(".dowe/web").join(path)).expect("css"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(page_css.contains(".max-w-full{max-width:100%;}"));
    assert!(page_css.contains(".md\\:max-w-64{max-width:16rem;}"));
    assert!(page_css.contains(".max-h-vh-16{max-height:calc(100vh - 4rem);}"));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains(
        ".doweMaxWidth(doweResponsive(viewportWidth, xs = DoweSize.Full, md = DoweSize.Fixed(256.dp)))"
    ));
    assert!(android.contains(
        ".doweMaxHeight(doweResponsive(viewportWidth, xs = DoweSize.ViewportMinus(64.dp)))"
    ));
    let android_max_width = android.find(".doweMaxWidth(").expect("android max width");
    let android_width = android[android_max_width..]
        .find(".doweWidth(")
        .map(|index| index + android_max_width)
        .expect("android width");
    assert!(android_max_width < android_width);

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains(
        ".frame(maxWidth: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.full, md: DoweSize.fixed(CGFloat(256)))))"
    ));
    assert!(ios.contains(
        ".frame(maxHeight: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.viewportMinus(CGFloat(64))), viewportHeight: viewportHeight))"
    ));
}
