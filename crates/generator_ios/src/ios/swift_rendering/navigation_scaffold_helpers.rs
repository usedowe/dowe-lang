fn render_swift_scaffold(
    props: &ScaffoldProps,
    app_bar: &[ViewNode],
    start: &[ViewNode],
    main: &[ViewNode],
    end: &[ViewNode],
    bottom_bar: &[ViewNode],
    overlays: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let docking_app_bar = app_bar
        .iter()
        .any(|node| matches!(node, ViewNode::AppBar { props, .. } if props.dock_on_scroll));
    if docking_app_bar {
        let pad = " ".repeat(indent);
        output.push_str(&format!("{pad}DoweDockingScaffold {{\n"));
        render_swift_scaffold_content(
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
            true,
        );
        output.push_str(&format!("{pad}}}\n"));
    } else {
        render_swift_scaffold_content(
            props,
            app_bar,
            start,
            main,
            end,
            bottom_bar,
            overlays,
            indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
            false,
        );
    }
}

fn render_swift_scaffold_content(
    props: &ScaffoldProps,
    app_bar: &[ViewNode],
    start: &[ViewNode],
    main: &[ViewNode],
    end: &[ViewNode],
    bottom_bar: &[ViewNode],
    overlays: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
    docking_app_bar: bool,
) {
    let pad = " ".repeat(indent);
    let current_font = props.style.font.as_ref().or(inherited_font);
    let persistent_app_bar = app_bar.iter().any(|node| {
        matches!(node, ViewNode::AppBar { props, .. } if props.position != BarPosition::Static)
    });
    output.push_str(&format!("{pad}VStack(spacing: CGFloat(0)) {{\n"));
    for child in app_bar {
        render_swift_node_in_flow(
            child,
            indent + 4,
            output,
            NativeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    if persistent_app_bar {
        output.push_str(&format!("{pad}    ScrollView {{\n"));
    }
    output.push_str(&format!(
        "{pad}    HStack(alignment: .top, spacing: CGFloat(0)) {{\n"
    ));
    if !start.is_empty() {
        output.push_str(&format!(
            "{pad}        VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
        ));
        for child in start {
            render_swift_node_in_flow(
                child,
                indent + 12,
                output,
                NativeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}        }}\n"));
    }
    output.push_str(&format!(
        "{pad}        VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
    ));
    for child in main {
        render_swift_node_in_flow(
            child,
            indent + 12,
            output,
            NativeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}        }}\n"));
    if persistent_app_bar {
        output.push_str(&format!(
            "{pad}        .frame(maxWidth: .infinity, alignment: .topLeading)\n"
        ));
    } else {
        output.push_str(&format!("{pad}        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"));
    }
    if !end.is_empty() {
        output.push_str(&format!(
            "{pad}        VStack(alignment: .leading, spacing: CGFloat(0)) {{\n"
        ));
        for child in end {
            render_swift_node_in_flow(
                child,
                indent + 12,
                output,
                NativeFlow::Block,
                current_font,
                default_family,
                context,
            );
        }
        output.push_str(&format!("{pad}        }}\n"));
    }
    output.push_str(&format!("{pad}    }}\n"));
    if props.boxed && persistent_app_bar {
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: CGFloat(1536), alignment: .topLeading)\n{pad}    .frame(maxWidth: .infinity, alignment: .top)\n"
        ));
    } else if props.boxed {
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: CGFloat(1536), maxHeight: .infinity, alignment: .topLeading)\n{pad}    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)\n"
        ));
    } else if persistent_app_bar {
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: .infinity, alignment: .topLeading)\n"
        ));
    } else {
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"
        ));
    }
    if persistent_app_bar {
        if docking_app_bar {
            output.push_str(&format!(
                "{pad}    .background(DoweDockingScrollObserver())\n"
            ));
        }
        output.push_str(&format!("{pad}    }}\n"));
        output.push_str(&format!(
            "{pad}    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n"
        ));
    }
    for child in bottom_bar {
        render_swift_node_in_flow(
            child,
            indent + 4,
            output,
            NativeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    for child in overlays {
        render_swift_node_in_flow(
            child,
            indent + 4,
            output,
            NativeFlow::Block,
            current_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{pad}}}\n"));
    append_swift_modifiers(
        output,
        indent,
        &swift_modifiers_for_container_style(&props.style, flow),
    );
}

