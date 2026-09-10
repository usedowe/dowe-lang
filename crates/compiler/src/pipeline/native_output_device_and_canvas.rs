#[test]
fn compiles_device_preview_with_fixed_profiles_and_zoom() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  Text
    "Device preview"
  Device device:"laptop" w:"full" rounded:"md" border:1
    Iframe src:"/examples/appbar-one" title:"Responsive preview""#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    assert!(body.contains("data-dowe-device-profile=\"laptop\""));
    assert!(body.contains("data-dowe-device-option=\"mobile\""));
    assert!(body.contains("width:1440px;height:900px"));
    assert!(body.contains("class=\"device"));
    assert!(body.contains("border-1"));
    assert!(body.contains("button icon-button button-md device-toggle"));
    assert!(body.contains("data-dowe-button-icon-start"));
    assert!(project.web.router_js.contains("ResizeObserver"));
    assert!(
        project
            .web
            .router_js
            .contains("Math.min(1,width/dimensions[0])")
    );

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("private fun DoweDevicePreview("));
    assert!(android.contains("1440f to 900f"));
    assert!(android.contains("DoweDeviceIcon(profile = \"mobile\""));
    assert!(android.contains("DoweSvg(viewBox = icon.viewBox"));
    assert!(android.contains(".border(doweResponsive("));
    assert!(android.contains("private fun DoweDeviceIconButton("));
    assert!(android.contains("Modifier.size(40.dp)"));
    assert!(android.contains("Modifier.size(24.dp)"));
    assert!(android.contains("containerColor = if (selected) DoweDesign.muted else Color.Transparent"));
    assert!(android.contains("contentColor = if (selected) DoweDesign.primary else DoweDesign.backgroundText"));
    assert!(android.contains("Row(modifier = Modifier.padding(4.dp)"));
    assert!(!android.contains("Text(option.second)"));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("private FrameLayout doweDevice("));
    assert!(android_dev.contains("DoweDeviceOption[] options"));
    assert!(android_dev.contains("new DoweSvgView(this, 0f, 0f, 24f, 24f"));
    assert!(
        android_dev
            .contains("doweDevice(\"laptop\", \"/examples/appbar-one\", \"Responsive preview\"")
    );
    assert!(android_dev.contains("new DoweDeviceOption(\"mobile\""));
    assert!(android_dev.contains("doweStyledBackground(Color.TRANSPARENT, DOWE_BACKGROUND_TEXT,"));
    assert!(android_dev.contains("selected ? DOWE_MUTED : DOWE_BACKGROUND, selected ? DOWE_PRIMARY : DOWE_BACKGROUND_TEXT"));
    assert!(android_dev.contains(".setPadding(doweDp(1), doweDp(1), doweDp(1), doweDp(1));"));
    assert!(android_dev.contains("doweDeviceIconButtonBackground"));
    assert!(
        android_dev
            .contains("new FrameLayout.LayoutParams(doweDp(24), doweDp(24), Gravity.CENTER)")
    );
    assert!(android_dev.contains("setMargins(doweDp(2), doweDp(4), doweDp(2), doweDp(4))"));
    assert!(!android_dev.contains("button.setText(option[1])"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("struct DoweDevicePreview: View"));
    assert!(ios.contains("CGSize(width: 1920, height: 1080)"));
    assert!(ios.contains("DoweDeviceIcon(profile: \"mobile\""));
    assert!(ios.contains("DoweSvgView(viewBox: option.viewBox"));
    assert!(ios.contains(".frame(width: CGFloat(40), height: CGFloat(40))"));
    assert!(ios.contains(".frame(width: CGFloat(24), height: CGFloat(24))"));
    assert!(ios.contains(".background(profile == option.profile ? DoweDesign.muted : Color.clear)"));
    assert!(ios.contains(".padding(CGFloat(4))"));
    assert!(!ios.contains("Button(option.1)"));
}

#[test]
fn compiles_canvas_with_cross_target_scene_runtime() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture_with_views(
        temp.path(),
        r#"layout AuthLayout
  Box
    children"#,
        r#"page loginPage
  signal input value:{ x:48 y:90 }
  signal scene value:[{ type:"rect" x:0 y:0 width:320 height:180 fill:"surface" },{ type:"circle" x:48 y:90 radius:18 fill:"primary" bind:{ x:"input.x" y:"input.y" } },{ type:"text" x:160 y:28 text:"Canvas" fill:"surfaceText" size:18 align:"center" }]
  fn capture
    set input value:item
  signal layers value:[{ id:"circle" type:"circle" x:100 y:80 radius:24 fill:"primary" }]
  signal selected value:""
  signal mode value:"select"
  fn recordLayer
    set input value:item
  Box
    Text
      "Canvas demo"
    Canvas scene:scene viewWidth:320 viewHeight:180 fit:"contain" fps:60 autoplay:true background:"background" pixelated:true label:"Animated Canvas" onPointer:capture onKey:capture onMotion:capture motionRate:30 w:"full" h:48 rounded:"md" border:1
    Draw bind:layers selected:selected viewWidth:320 viewHeight:180 label:"Layer editor" draw:true drawMode:mode onLayerAdd:recordLayer onLayerChange:recordLayer onLayerRemove:recordLayer onLayerSelect:recordLayer w:"full" h:48"#,
    );

    let project = compile_dev(temp.path()).expect("project");
    let body = &project.web.pages[0].body_html;
    let runtime_path = project.web.pages[0]
        .runtime_chunks
        .iter()
        .find(|path| path.contains("canvas-"))
        .expect("canvas route dependency");
    assert!(body.contains("data-dowe-canvas"));
    assert!(body.contains("data-dowe-canvas-scene="));
    assert!(body.contains("aria-label=\"Animated Canvas\""));
    let canvas = project
        .web
        .runtime_chunks()
        .into_iter()
        .find(|chunk| chunk.name == "canvas")
        .expect("canvas runtime");
    assert!(canvas.content.contains("drawCanvasCommand"));
    assert!(canvas.content.contains("devicePixelRatio"));
    assert!(canvas.content.contains("closeCanvasFrames"));
    assert!(
        project.web.pages[0]
            .html_document
            .contains(&format!(r#"rel="modulepreload" href="/{runtime_path}""#))
    );
    assert!(
        dowe_generator_web::manifest(&project.web)
            .contains(&format!(r#""runtimeChunks":["{runtime_path}"]"#))
    );
    assert!(temp.path().join(".dowe/web").join(runtime_path).is_file());
    assert!(body.contains("data-dowe-canvas-on-pointer="));
    assert!(body.contains("data-dowe-canvas-on-key="));
    assert!(body.contains("data-dowe-canvas-on-motion="));
    assert!(body.contains("data-dowe-canvas-layer-bind="));
    assert!(body.contains("data-dowe-canvas-selected="));
    assert!(body.contains("data-dowe-canvas-on-layer-add="));
    assert!(canvas.content.contains("boundCanvasCommand"));
    assert!(canvas.content.contains("canvasLogicalPoint"));
    assert!(canvas.content.contains("canvasRemoveSelected"));
    assert!(canvas.content.contains("canvasTopLayerAt"));

    let android = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt"),
    )
    .expect("android");
    assert!(android.contains("private fun DoweCanvas("));
    assert!(android.contains("doweDrawCanvasCommand"));
    assert!(android.contains("DoweCanvas(state = state, scenePath = \""));
    assert!(android.contains("doweBoundCanvasCommand"));
    assert!(android.contains(
        "pointerInput(state, draw, layersPath, selectedPath, drawModePath, onPointer, viewWidth, viewHeight, fit)"
    ));
    assert!(android.contains("Sensor.TYPE_ROTATION_VECTOR"));
    assert!(android.contains("layersPath = \""));
    assert!(android.contains("onLayerRemove = \""));

    let android_dev = android_dev_output(temp.path());
    assert!(android_dev.contains("private DoweCanvasView doweCanvas("));
    assert!(android_dev.contains("doweDrawCanvasCommand"));
    assert!(android_dev.contains("onTouchEvent(MotionEvent event)"));
    assert!(android_dev.contains("doweStartCanvasSensors"));
    assert!(android_dev.contains("doweUpdateCanvasLayer"));

    let ios = ios_swift_output(temp.path());
    assert!(ios.contains("struct DoweCanvasView: View"));
    assert!(ios.contains("TimelineView(.animation"));
    assert!(ios.contains("DoweCanvasView(state: state, scenePath: \""));
    assert!(ios.contains("pixelated: true"));
    assert!(ios.contains("interpolation(pixelated ? .none : .medium)"));
    assert!(ios.contains("accessibilityAddTraits(.isImage)"));
    assert!(ios.contains("DoweCanvasInputBridge"));
    assert!(ios.contains("CMMotionManager"));
    assert!(ios.contains("boundCommand"));
    assert!(ios.contains("layersPath: \""));
    assert!(ios.contains("removeSelectedIfNeeded"));
    let ios_info =
        fs::read_to_string(temp.path().join(".dowe/apps/ios/Info.plist")).expect("ios info");
    assert!(ios_info.contains("NSMotionUsageDescription"));
}
