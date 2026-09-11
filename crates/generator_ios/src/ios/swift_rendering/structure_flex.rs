fn swift_static_flex_direction(value: &ResponsiveValue<FlexDirection>) -> Option<FlexDirection> {
    let base = value
        .entries
        .iter()
        .find(|entry| entry.breakpoint == Breakpoint::Xs)
        .map_or(FlexDirection::Row, |entry| entry.value);
    value
        .entries
        .iter()
        .all(|entry| entry.value == base)
        .then_some(base)
}

fn render_swift_structure_flex(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let ViewNode::Flex { props, .. } = node else {
        return;
    };
    let pad = " ".repeat(indent);
    output.push_str(&format!("{pad}Group {{\n"));
    if let Some(direction) = swift_static_flex_direction(&props.direction) {
        render_swift_flex_axis(
            node,
            direction,
            indent + 4,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
    } else {
        output.push_str(&format!(
            "{pad}    if {} == DoweFlexDirection.column {{\n",
            swift_flex_direction_value(&props.direction)
        ));
        render_swift_flex_axis(
            node,
            FlexDirection::Column,
            indent + 8,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}    }} else {{\n"));
        render_swift_flex_axis(
            node,
            FlexDirection::Row,
            indent + 8,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
        output.push_str(&format!("{pad}    }}\n"));
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_swift_flex_axis(
    node: &ViewNode,
    direction: FlexDirection,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let ViewNode::Flex { props, children } = node else {
        return;
    };
    let pad = " ".repeat(indent);
    let current_font = props.style.font.as_ref().or(inherited_font);
    let justify = swift_flex_justify(props.justify.as_ref());
    let gap = swift_gap(props.gap.as_ref());
    let column = direction == FlexDirection::Column;
    let wrapped = !column && props.wrap;
    if wrapped {
        output.push_str(&format!(
            "{pad}DoweFlowLayout(justify: {justify}, align: {}, gap: {gap}) {{\n",
            swift_flex_align(props.align.as_ref()),
        ));
    } else {
        let (stack, alignment) = if column {
            ("VStack", swift_horizontal_alignment(props.align.as_ref()))
        } else {
            ("HStack", swift_vertical_alignment(props.align.as_ref()))
        };
        output.push_str(&format!(
            "{pad}{stack}(alignment: {alignment}, spacing: doweFlexStackSpacing({justify}, gap: {gap})) {{\n"
        ));
        if !children.is_empty() {
            output.push_str(&format!(
                "{pad}    if let spacerGap = doweFlexLeadingSpacer({justify}, gap: {gap}) {{\n{pad}        Spacer(minLength: spacerGap)\n{pad}    }}\n"
            ));
        }
    }
    for (index, child) in children.iter().enumerate() {
        if index > 0 && !wrapped {
            output.push_str(&format!(
                "{pad}    if let spacerGap = doweFlexBetweenSpacer({justify}, gap: {gap}) {{\n{pad}        Spacer(minLength: spacerGap)\n{pad}    }}\n"
            ));
        }
        render_swift_node_in_flow(
            child,
            indent + 4,
            output,
            if column {
                NativeFlow::Block
            } else {
                NativeFlow::Inline
            },
            current_font,
            default_family,
            context,
        );
    }
    if !children.is_empty() && !wrapped {
        output.push_str(&format!(
            "{pad}    if let spacerGap = doweFlexTrailingSpacer({justify}, gap: {gap}) {{\n{pad}        Spacer(minLength: spacerGap)\n{pad}    }}\n"
        ));
    }
    output.push_str(&format!("{pad}}}\n"));
    let modifiers = if column {
        swift_modifiers_for_column_layout(props, flow)
    } else {
        swift_modifiers_for_layout(props, flow)
    };
    append_swift_modifiers(output, indent, &modifiers);
}
