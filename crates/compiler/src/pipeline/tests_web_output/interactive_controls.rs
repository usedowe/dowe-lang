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

    let css = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.design_file_name()),
    )
    .expect("css");
    assert!(css.contains("html.theme-transitioning"));
    assert!(css.contains("html.page-transitioning"));
    assert!(!css.is_empty());
    assert!(css.contains("select"));
    assert!(!css.contains(".theme-select-input"));
    assert!(!css.is_empty());
    assert!(!css.is_empty());
    assert!(!css.is_empty());

    let router = fs::read_to_string(
        temp.path()
            .join(".dowe/web")
            .join(project.web.router_file_name()),
    )
    .expect("router");
    assert!(router.contains("theme-preference"));
    assert!(router.contains("startViewTransition"));
    assert!(router.contains("hydrateThemeToggles"));
    assert!(router.contains("hydrateThemeSelects"));
    let controls = project
        .web
        .runtime_chunks()
        .into_iter()
        .find(|chunk| chunk.name == "controls")
        .expect("controls runtime")
        .content;
    let render_select = controls
        .split("function renderSelect(control,state,scope){")
        .nth(1)
        .and_then(|section| section.split("function renderSelects").next())
        .expect("renderSelect source");
    assert!(!render_select.contains("applyDoweTheme"));
    assert!(router.contains(
        "if(control.dataset.doweThemeSelect!==undefined&&value)applyDoweTheme(value,true);"
    ));
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
    assert!(android.contains("DoweSvg(viewBox = lightIconViewBox"));
    assert!(android.contains("DoweSvg(viewBox = darkIconViewBox"));
    assert!(android.contains("DoweThemeSelect("));
    assert!(android.contains("DoweThemeModule.names"));
    assert!(android.contains("DoweSliderField("));
    assert!(android.contains("DoweDropzone("));
    assert!(
        android
            .contains("rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument())")
    );
    assert!(android.contains(
        "rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments())"
    ));
    assert!(android.contains("doweDropzoneMimeTypes(accept)"));
    assert!(android.contains("maxSize = 4096L"));
    assert!(android.contains("\"theme-preference\""));
    assert!(android.contains("state.write("));
    assert!(android.contains("it.toDouble()"));
    assert!(android.contains("LaunchedEffect(currentEntry.path)"));
    assert!(android.contains("scrollState.scrollTo(0)"));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("getSharedPreferences(\"dowe\", 0)"));
    assert!(android_dev.contains("DoweSvgView"));
    assert!(!android_dev.contains("? \"sun\" : \"moon\""));
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
    assert!(ios.contains("DoweSvgView(viewBox:"));
    assert!(!ios.contains("Image(systemName: DoweDesign.shared.name == \"dark\""));
    assert!(ios.contains("DoweSelectField(value: Optional(Binding(get:"));
    assert!(ios.contains("DoweSelectOption(value: \"dark\", label: \"Dark\""));
    assert!(ios.contains("DoweSliderView(value:"));
    assert!(ios.contains("Binding<Double>.constant(40.0)"));
    assert!(ios.contains("Text(String(format: \"%.0f\", value.wrappedValue))"));
    assert!(!ios.contains(".constant(40).wrappedValue"));
    assert!(
        ios.contains("Image(systemName: selectedFiles.isEmpty ? \"paperclip\" : \"doc.on.doc\")")
    );
    assert!(ios.contains(".fileImporter("));
    assert!(ios.contains("doweDropzoneFileTypes(accept)"));
    assert!(ios.contains("allowsMultipleSelection: multiple"));
    assert!(ios.contains("resourceValues(forKeys: [.nameKey, .fileSizeKey])"));
    assert!(ios.contains("StrokeStyle(lineWidth: CGFloat(2), dash: [CGFloat(6)])"));
    assert!(ios.contains("proxy.scrollTo(\"__dowe_page_top\", anchor: .top)"));
    assert!(ios.contains(".id(\"__dowe_page_top\")"));
    assert!(ios.contains("Drop images"));
}

