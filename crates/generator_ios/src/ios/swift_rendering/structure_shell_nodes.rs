#[allow(unused_variables)]
fn render_swift_structure_splash(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let ViewNode::Splash { binding, content, children, .. } = node else { return; };
    let pad = " ".repeat(indent);
            output.push_str(&format!(
                "{pad}if state.bool(\"{}\") {{\n",
                escape_swift(&context.signal_path(binding))
            ));
            for child in children {
                render_swift_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}}} else {{\n"));
            for child in content {
                render_swift_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
}

#[allow(unused_variables)]
fn render_swift_structure_scope(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let ViewNode::Scope { constants, signals, actions, children, } = node else { return; };
    let pad = " ".repeat(indent);
            let context = context.with_scope(constants, signals, actions);
            for child in children {
                render_swift_node_in_flow(
                    child,
                    indent,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    &context,
                );
            }
}

#[allow(unused_variables)]
fn render_swift_structure_each(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let ViewNode::Each { item, collection, children, .. } = node else { return; };
    let pad = " ".repeat(indent);
            output.push_str(&format!(
                "{pad}ForEach(state.rows(\"{}\")) {{ row in\n",
                escape_swift(&context.signal_path(collection))
            ));
            let context = context.with_item(item, "row.value".to_string());
            for child in children {
                render_swift_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    &context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
}

#[allow(unused_variables)]
fn render_swift_structure_box(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let ViewNode::Box { props, children } = node else { return; };
    let pad = " ".repeat(indent);
            if props.position().mode != BoxPosition::Fixed {
                render_swift_box(
                    props,
                    children,
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

