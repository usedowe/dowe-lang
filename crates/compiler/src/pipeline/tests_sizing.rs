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

#[test]
fn compiles_container_width_values_with_cross_target_parity() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        "layout AuthLayout\n  Box\n    children",
        r#"page loginPage
  Box w:"sm" minW:"md" maxW:"lg"
    Box w:"xl" minW:"2xl" maxW:"3xl"
      Box w:"4xl" minW:"5xl" maxW:"6xl"
        Box w:"7xl"
          Text
            "Container widths""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let page = project.web.pages.first().expect("page");
    for class_name in [
        "w-sm",
        "min-w-md",
        "max-w-lg",
        "w-xl",
        "min-w-2xl",
        "max-w-3xl",
        "w-4xl",
        "min-w-5xl",
        "max-w-6xl",
        "w-7xl",
    ] {
        assert!(page.body_html.contains(class_name), "{class_name}");
    }

    let page_css = page
        .css_chunks
        .iter()
        .map(|path| fs::read_to_string(temp.path().join(".dowe/web").join(path)).expect("css"))
        .collect::<Vec<_>>()
        .join("\n");
    for (class_name, property, variable) in [
        ("w-sm", "width", "sm"),
        ("min-w-md", "min-width", "md"),
        ("max-w-lg", "max-width", "lg"),
        ("w-xl", "width", "xl"),
        ("min-w-2xl", "min-width", "2xl"),
        ("max-w-3xl", "max-width", "3xl"),
        ("w-4xl", "width", "4xl"),
        ("min-w-5xl", "min-width", "5xl"),
        ("max-w-6xl", "max-width", "6xl"),
        ("w-7xl", "width", "7xl"),
    ] {
        assert!(
            page_css.contains(&format!(".{class_name}{{{property}:var(--container-{variable});}}")),
            "{class_name}"
        );
    }
    let design_css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.design_file_name()),
    )
    .expect("design css");
    for (name, rem) in [
        ("sm", "24rem"),
        ("md", "28rem"),
        ("lg", "32rem"),
        ("xl", "36rem"),
        ("2xl", "42rem"),
        ("3xl", "48rem"),
        ("4xl", "56rem"),
        ("5xl", "64rem"),
        ("6xl", "72rem"),
        ("7xl", "80rem"),
    ] {
        assert!(design_css.contains(&format!("--container-{name}:{rem};")), "{name}");
    }

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    for units in [384, 448, 512, 576, 672, 768, 896, 1024, 1152, 1280] {
        assert!(android.contains(&format!("DoweSize.Fixed({units}.dp)")), "{units}");
    }

    let ios = ios_swift_output(temp.path());
    for units in [384, 448, 512, 576, 672, 768, 896, 1024, 1152, 1280] {
        assert!(
            ios.contains(&format!("DoweSize.fixed(CGFloat({units}))")),
            "{units}"
        );
    }
}
