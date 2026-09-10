fn ios_layouts_index() -> String {
    "import SwiftUI\n".to_string()
}

fn ios_layout_artifacts(layouts: &[&ViewNode], font_config: &FontConfig) -> Vec<IosArtifact> {
    layouts
        .iter()
        .enumerate()
        .map(|(index, layout)| IosArtifact {
            relative_path: PathBuf::from(format!("apps/ios/DoweLayout{index}.swift")),
            content: ios_layout(index, layout, font_config),
            kind: IosArtifactKind::Layouts,
            target: "ios",
        })
        .collect()
}

fn ios_layout(index: usize, layout: &ViewNode, font_config: &FontConfig) -> String {
    let mut output = String::from("import SwiftUI\n\n");
    let sections = ios_layout_sections(layout);
    let expressions = sections
        .iter()
        .enumerate()
        .map(|(section_index, section)| {
            (
                swift_node_key(section.node),
                format!("layoutSection{section_index}()"),
            )
        })
        .collect();
    let context = SwiftReactiveContext::default()
        .with_children_expression("content")
        .with_node_expressions(expressions);
    output.push_str(&format!(
        r#"struct DoweLayout{index}<Content: View>: View {{
    let viewportWidth: CGFloat
    let viewportHeight: CGFloat
    let activePath: String
    @ObservedObject var state: DoweReactiveState
    let navigate: (String, String, String?) -> Void
    let goBack: () -> Void
    let openExternal: (String, String) -> Void
    let content: Content

    init(
        viewportWidth: CGFloat,
        viewportHeight: CGFloat,
        activePath: String,
        state: DoweReactiveState,
        navigate: @escaping (String, String, String?) -> Void,
        goBack: @escaping () -> Void,
        openExternal: @escaping (String, String) -> Void,
        @ViewBuilder content: () -> Content
    ) {{
        self.viewportWidth = viewportWidth
        self.viewportHeight = viewportHeight
        self.activePath = activePath
        self.state = state
        self.navigate = navigate
        self.goBack = goBack
        self.openExternal = openExternal
        self.content = content()
    }}

    var body: some View {{
        Group {{
"#
    ));
    render_swift_node_in_flow(
        layout,
        12,
        &mut output,
        NativeFlow::Block,
        None,
        font_config.default_family,
        &context,
    );
    output.push_str("        }\n    }\n\n");
    for (section_index, section) in sections.iter().enumerate() {
        output.push_str(&format!(
            "    @ViewBuilder\n    private func layoutSection{section_index}() -> some View {{\n"
        ));
        let section_context = section.scopes.iter().fold(
            context.without_node_expression(section.node),
            |context, scope| context.with_scope(scope.constants, scope.signals, scope.actions),
        );
        render_swift_node_in_flow(
            section.node,
            8,
            &mut output,
            section.flow,
            None,
            font_config.default_family,
            &section_context,
        );
        output.push_str("    }\n\n");
    }
    output.push_str("}\n");
    output
}

