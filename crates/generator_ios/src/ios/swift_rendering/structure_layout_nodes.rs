#[allow(unused_variables)]
fn render_swift_structure_section(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let ViewNode::Section { props, children } = node else { return; };
    let pad = " ".repeat(indent);
            let current_font = props.font.as_ref().or(inherited_font);
            let section_spacing = swift_section_vertical_spacing(props.gap.as_ref());
            if props.cover.is_some() {
                output.push_str(&format!("{pad}ZStack(alignment: .topLeading) {{\n"));
                output.push_str(&format!(
                    "{pad}    DoweCoverImage(source: {} ?? \"\")\n",
                    swift_cover_value(props.cover.as_ref().expect("cover"))
                ));
                if let Some(overlay) = props.overlay.as_ref() {
                    output.push_str(&format!(
                        "{pad}    if let overlay = {} {{\n{pad}        DoweOverlayView(overlay: overlay)\n{pad}    }}\n",
                        swift_overlay_value(overlay)
                    ));
                }
                output.push_str(&format!(
                    "{pad}    VStack(alignment: {}, spacing: {section_spacing}) {{\n",
                    swift_section_horizontal_alignment(props.center_x.as_ref())
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 8,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}    }}\n"));
                append_swift_modifiers(
                    output,
                    indent + 4,
                    &swift_modifiers_for_section_content(props),
                );
                output.push_str(&format!("{pad}}}\n"));
            } else if let Some(background) = props.background.as_ref() {
                output.push_str(&format!("{pad}ZStack(alignment: .topLeading) {{\n"));
                output.push_str(&format!(
                    "{pad}    if let background = {} {{\n{pad}        DoweSectionBackgroundView(background: background)\n{pad}    }}\n",
                    swift_section_background_value(background)
                ));
                output.push_str(&format!(
                    "{pad}    VStack(alignment: {}, spacing: {section_spacing}) {{\n",
                    swift_section_horizontal_alignment(props.center_x.as_ref())
                ));
                for child in children {
                    render_swift_node_in_flow(
                        child,
                        indent + 8,
                        output,
                        NativeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}    }}\n"));
                append_swift_modifiers(
                    output,
                    indent + 4,
                    &swift_modifiers_for_section_content(props),
                );
                output.push_str(&format!("{pad}}}\n"));
            } else {
                output.push_str(&format!(
                    "{pad}VStack(alignment: {}, spacing: {section_spacing}) {{\n",
                    swift_section_horizontal_alignment(props.center_x.as_ref())
                ));
                for child in children {
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
                append_swift_modifiers(output, indent, &swift_modifiers_for_section_content(props));
            }
            append_swift_modifiers(
                output,
                indent,
                &swift_modifiers_for_section_container(props, flow),
            );
}

include!("structure_flex.rs");

#[allow(unused_variables)]
fn render_swift_structure_grid(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    let ViewNode::Grid { props, children } = node else { return; };
    let pad = " ".repeat(indent);
            let current_font = props.style.font.as_ref().or(inherited_font);
            output.push_str(&format!(
                "{pad}DoweGridLayout(tracks: {}, rowGap: {}, columnGap: {}, justify: {}, align: {}, fillHeight: {}) {{\n",
                swift_grid_tracks(props.columns.as_ref()),
                swift_grid_row_gap(props.gap.as_ref()),
                swift_grid_column_gap(props.gap.as_ref()),
                swift_grid_alignment(props.justify.as_ref()),
                swift_grid_alignment(props.align.as_ref()),
                swift_grid_fills_height(props, flow)
            ));
            for child in children {
                render_swift_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    NativeFlow::GridItem,
                    current_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
            append_swift_modifiers(output, indent, &swift_modifiers_for_grid(props, flow));
}

