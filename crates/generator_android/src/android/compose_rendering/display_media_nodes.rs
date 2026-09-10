#[allow(unused_variables)]
fn render_compose_display_audio(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Audio { props } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_audio(props, indent, output);
}

#[allow(unused_variables)]
fn render_compose_display_camera(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Camera { props } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_camera(props, indent, output, context);
}

#[allow(unused_variables)]
fn render_compose_display_microphone(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Microphone { props } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_microphone(props, indent, output, context);
}

#[allow(unused_variables)]
fn render_compose_display_image(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Image { props } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_image(props, indent, output, context);
}

#[allow(unused_variables)]
fn render_compose_display_accordion(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Accordion { props, items } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_accordion(
                props,
                items,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
}

#[allow(unused_variables)]
fn render_compose_display_carousel(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Carousel { props, slides } = node else { return; };
    let pad = " ".repeat(indent);
            render_compose_carousel(
                props,
                slides,
                indent,
                output,
                flow,
                inherited_font,
                default_family,
                context,
            );
}

#[allow(unused_variables)]
fn render_compose_display_code(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Code { props } = node else { return; };
    let pad = " ".repeat(indent);
            let source = if props.template_segments.is_empty() {
                compose_string_literal(&props.source)
            } else {
                props
                    .template_segments
                    .iter()
                    .map(|segment| match segment {
                        CodeTemplateSegment::Static { text, .. } => compose_string_literal(text),
                        CodeTemplateSegment::Binding(path) => format!(
                            "state.text(\"{}\", \"\")",
                            escape_kotlin(&context.signal_path(path))
                        ),
                    })
                    .collect::<Vec<_>>()
                    .join(" + ")
            };
            let tokens = if props.template_segments.is_empty() {
                compose_code_tokens(&props.tokens, card_variant_content(&props.style))
            } else {
                props
                    .template_segments
                    .iter()
                    .map(|segment| match segment {
                        CodeTemplateSegment::Static { tokens, .. } => {
                            compose_code_tokens(tokens, card_variant_content(&props.style))
                        }
                        CodeTemplateSegment::Binding(path) => format!(
                            "listOf(DoweCodeToken(text = state.text(\"{}\", \"\"), color = {}))",
                            escape_kotlin(&context.signal_path(path)),
                            card_variant_content(&props.style)
                        ),
                    })
                    .collect::<Vec<_>>()
                    .join(" + ")
            };
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                card_variant_content(&props.style)
            } else {
                "null"
            };
            output.push_str(&format!(
                        "{pad}DoweCode(source = {}, language = {}, tokens = {}, copyLabel = {}, copiedLabel = {}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
                        source,
                        compose_string_literal(props.language.as_str()),
                        tokens,
                        compose_string_literal(&props.copy_label),
                        compose_string_literal(&props.copied_label),
                        modifier_for_style(&props.style.style),
                        compose_card_radius(&props.style.style),
                        card_variant_container(&props.style),
                        card_variant_content(&props.style),
                    ));
}

#[allow(unused_variables)]
fn render_compose_display_video(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let ViewNode::Video { props } = node else { return; };
    let pad = " ".repeat(indent);
            let border = if props.style.variant.unwrap_or(ComponentVariant::Solid)
                == ComponentVariant::Outlined
            {
                card_variant_content(&props.style)
            } else {
                "null"
            };
            let icons = compose_video_icons();
            output.push_str(&format!(
                        "{pad}DoweVideo(source = {}, poster = {}, autoplay = {}, aspect = {}, icons = {icons}, modifier = {}, shape = RoundedCornerShape({}), backgroundColor = {}, borderColor = {border})\n",
                        compose_string_literal(&props.src),
                        compose_optional_string(props.poster.as_deref()),
                        props.autoplay,
                        compose_string_literal(props.aspect.as_str()),
                        modifier_for_style(&props.style.style),
                        compose_card_radius(&props.style.style),
                        card_variant_container(&props.style),
                    ));
}

