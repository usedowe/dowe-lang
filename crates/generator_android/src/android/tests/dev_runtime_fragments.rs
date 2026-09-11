#[test]
fn preserves_svg_class_declarations_when_joining_runtime_fragments() {
    let source = super::dev_activity_svg_parser();
    assert!(source.starts_with("    private static final class DoweSvgPathEntry {\n"));
    assert_eq!(source.matches("private final String source;").count(), 1);
}

#[test]
fn preserves_chart_fields_and_blocks_when_joining_runtime_fragments() {
    let source = super::dev_activity_chart_runtime();
    assert_eq!(source.matches("private final String chartType;").count(), 1);
    assert_eq!(
        source
            .matches("if (dataPath != null && !dataPath.isEmpty()) {")
            .count(),
        1
    );
}

#[test]
fn emits_java_only_when_joining_diagram_runtime_fragments() {
    let source = super::dev_activity_diagram_view();
    assert!(source.starts_with("    private final class DoweDiagramView extends View {\n"));
    assert!(!source.contains("r#\""));
}
