#[test]
fn generates_svg_compose_and_dev_views() {
    let output = generate_android(
        &[svg_route(), runtime_svg_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");

    assert!(views.content.contains("private fun DoweSvg("));
    assert!(
        views
            .content
            .contains("DoweRuntimeSvg(payload = state.json(\"iconData01\")")
    );
    assert!(
        views
            .content
            .contains("private fun doweRuntimeSvgRecord(payload: String)")
    );
    assert!(views.content.contains("DoweSvgViewBox(0f, 0f, 24f, 24f)"));
    assert!(views.content.contains("DoweSvgFill.CurrentColor"));
    assert!(views.content.contains(
        "doweResponsive(viewportWidth, xs = DoweDesign.accent) ?: LocalContentColor.current"
    ));
    assert!(
        views
            .content
            .contains("PathParser().parsePathString(entry.data).toPath()")
    );
    assert!(
        views
            .content
            .contains("DoweSvgTransform(2f, 0f, 0f, 2f, 4f, 6f)")
    );
    assert!(views.content.contains("private object DoweSvgImporter"));
    assert!(
        views
            .content
            .contains("private fun rectangle(attrs: Map<String, String>): String")
    );
    assert!(
        views
            .content
            .contains("private fun sameColor(left: String, right: String): Boolean")
    );
    assert!(
        views
            .content
            .contains("private fun originalFill(value: String?): String")
    );
    assert!(
        views
            .content
            .contains("val evenOdd = when (fillRule?.trim()?.lowercase())")
    );
    assert!(
        views
            .content
            .contains("if (path.evenOdd) value.put(\"evenOdd\", true)")
    );
    assert!(
        views
            .content
            .contains("if (path.evenOdd) \" fillRule:\\\"evenodd\\\"\" else \"\"")
    );
    assert!(
        views
            .content
            .contains("\"parse.svg\" -> DoweSvgImporter.convert(text(\"value\"), text(\"colors\")")
    );

    let dev = dev_java_source(&output);
    assert!(
        dev.content
            .contains("private static final class DoweSvgView extends View")
    );
    assert!(
        dev.content
            .contains("doweRuntimeSvg(doweTextValue(\"iconData01\", null)")
    );
    assert!(dev.content.contains(
        "private DoweSvgView doweRuntimeSvg(String payload, int currentColor, boolean animated)"
    ));
    assert!(
        dev.content
            .contains("private static final class DoweSvgPathParser")
    );
    assert!(
        dev.content
            .contains("Path path = DoweSvgPathParser.parse(entry.data)")
    );
    assert!(dev.content.contains("if (entry.transform != null)"));
    assert!(
        dev.content
            .contains("private static Object doweParseSvg(String source, Object fallback, String colorsMode, String format)")
    );
    let parse_svg_start = dev
        .content
        .find("private static Object doweParseSvg")
        .expect("parse.svg development runtime");
    let parse_svg_end = dev.content[parse_svg_start..]
        .find("private static DoweSvgImportMatrix doweSvgMatrix")
        .expect("parse.svg runtime boundary");
    let parse_svg_runtime = &dev.content[parse_svg_start..parse_svg_start + parse_svg_end];
    assert!(parse_svg_runtime.contains("catch (Exception error)"));
    assert!(!parse_svg_runtime.contains("catch (RuntimeException error)"));
    assert!(parse_svg_runtime.contains("boolean evenOdd = parent.evenOdd;"));
    assert!(
        parse_svg_runtime.contains("if (pathEvenOdds.get(index)) path.put(\"evenOdd\", true);")
    );
    assert!(
        parse_svg_runtime
            .contains("pathEvenOdds.get(index) ? \" fillRule:\\\"evenodd\\\"\" : \"\"")
    );
    assert!(
        dev.content
            .contains("private static String doweSvgRectangle(HashMap<String, String> attrs)")
    );
    assert!(
        dev.content
            .contains("private static boolean doweSvgSameColor(String left, String right)")
    );
    assert!(
        dev.content
            .contains("if (\"parse.svg\".equals(name)) return doweParseSvg")
    );
    assert!(dev.content.contains(
        "Integer fill = entry.currentColor ? Integer.valueOf(currentColor) : entry.color;"
    ));
    assert!(!dev.content.contains("import android.graphics.PathParser;"));
    assert!(
        dev.content
            .contains("protected void onMeasure(int widthMeasureSpec, int heightMeasureSpec)")
    );
    assert!(dev.content.contains("new DoweSvgView(this, 0f, 0f, 24f, 24f, doweColor(doweResponsiveInt(viewportWidth, DOWE_ACCENT, null, null, null, null), DOWE_BACKGROUND_TEXT)"));
}

