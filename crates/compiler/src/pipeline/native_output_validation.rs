#[test]
fn compiles_divider_across_native_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Box
    Text
      "Divider"
    Divider scheme:"primary"
    Divider orientation:"vertical" scheme:"secondary""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    assert!(body.contains("divider divider-horizontal is-primary"));
    assert!(body.contains("divider divider-vertical is-secondary"));
    assert!(project.web.chunks.iter().any(|chunk| {
        chunk
            .css_content
            .contains(".divider.is-primary{background-color:var(--dowe-primary);")
    }));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains(
        "Box(modifier = Modifier.fillMaxWidth().height(1.dp).background(DoweDesign.primary))"
    ));
    assert!(android.contains(
        "Box(modifier = Modifier.width(1.dp).fillMaxHeight().background(DoweDesign.secondary))"
    ));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("setBackgroundColor(DOWE_PRIMARY)"));
    assert!(android_dev.contains("setBackgroundColor(DOWE_SECONDARY)"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains(".fill(DoweDesign.primary)"));
    assert!(ios.contains(".fill(DoweDesign.secondary)"));
    assert!(ios.contains(".frame(height: CGFloat(1))"));
    assert!(ios.contains(".frame(width: CGFloat(1))"));
}

#[test]
fn rejects_invalid_divider_components() {
    assert_compile_error(
        r#"page loginPage
  Text
    "Divider"
  Divider orientation:"diagonal""#,
        "expected horizontal or vertical",
    );
    assert_compile_error(
        r#"page loginPage
  Text
    "Divider"
  Divider
    Text
      "Child""#,
        "children are not valid for this component",
    );
}

#[test]
fn rejects_invalid_video_components() {
    assert_compile_error(
        r#"page loginPage
  Text
    "Video"
  Video"#,
        "invalid value for prop `src`: expected https URL",
    );
    assert_compile_error(
        r#"page loginPage
  Text
    "Video"
  Video src:"http://example.com/video.mp4""#,
        "invalid value for prop `src`: expected https URL",
    );
    assert_compile_error(
        r#"page loginPage
  Text
    "Video"
  Video src:"https://example.com/video.mp4" aspect:"wide""#,
        "invalid value for prop `aspect`: expected horizontal, vertical or square",
    );
    assert_compile_error(
        r#"page loginPage
  Text
    "Video"
  Video src:"https://example.com/video.mp4" autoplay:"true""#,
        "invalid value for prop `autoplay`: expected boolean",
    );
    assert_compile_error(
        r#"page loginPage
  Text
    "Video"
  Video src:"https://example.com/video.mp4"
    Text
      "Child""#,
        "children are not valid for this component",
    );
}

#[test]
fn rejects_invalid_iframe_components() {
    assert_compile_error(
        r#"page loginPage
  Iframe src:"https://example.com""#,
        "invalid value for prop `title`: expected non-empty string",
    );
    assert_compile_error(
        r#"page loginPage
  Iframe src:"http://example.com" title:"Example""#,
        "invalid value for prop `src`: expected https URL or internal route",
    );
    assert_compile_error(
        r#"page loginPage
  Iframe src:"//example.com" title:"Example""#,
        "invalid value for prop `src`: expected https URL or internal route",
    );
    assert_compile_error(
        r#"page loginPage
  Iframe src:"/examples/../admin" title:"Example""#,
        "invalid value for prop `src`: expected https URL or internal route",
    );
    assert_compile_error(
        r#"page loginPage
  Iframe src:"https://example.com" title:"Example" sandbox:"scripts unknown""#,
        "invalid value for prop `sandbox`: expected portable iframe policy tokens",
    );
    assert_compile_error(
        r#"page loginPage
  Iframe src:"https://example.com" title:"Example"
    Text
      "Child""#,
        "children are not valid for this component",
    );
}

#[test]
fn rejects_invalid_device_components() {
    assert_compile_error(
        r#"page loginPage
  Device device:"watch"
    Iframe src:"/preview" title:"Preview""#,
        "invalid value for prop `device`: expected mobile, tablet, laptop or monitor",
    );
    assert_compile_error(
        r#"page loginPage
  Device"#,
        "Device requires exactly one Iframe child",
    );
    assert_compile_error(
        r#"page loginPage
  Device
    Text
      "Invalid""#,
        "Device can only contain one Iframe child",
    );
}

#[test]
fn rejects_empty_text() {
    assert_compile_error(
        r#"page loginPage
  Box
    Text"#,
        "Text requires a text child",
    );

    assert_compile_error(
        r#"page loginPage
  Box
    Button"#,
        "Button requires a text child",
    );

    assert_compile_error(
        r#"page loginPage
  Box
    Text
      "   ""#,
        "Text requires static text",
    );
}

#[test]
fn rejects_component_children_inside_text() {
    assert_compile_error(
        r#"page loginPage
  Box
    Text
      Box
        Text
          "Nested""#,
        "must be a quoted static string literal",
    );
}

#[test]
fn rejects_children_inside_page() {
    assert_compile_error(
        r#"page loginPage
  Box
    children"#,
        "children can only be used inside layouts",
    );
}
