fn render_swift_editor(
    props: &EditorProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
output.push_str(&format!(
    "{pad}DoweEditorField(value: {}, language: {}, initialValue: {}, label: {}, placeholder: {}, minHeight: CGFloat({}), hideToolbar: {}, readOnly: {}, onSave: {}, backgroundColor: {}, contentColor: {})\n",
    swift_text_binding(props.style.element.bind.as_deref(), context),
    swift_string_literal(props.value.as_deref().unwrap_or_default()),
    swift_string_literal(props.language.as_str()),
    swift_optional_literal(props.style.label.as_deref()),
    swift_string_literal(props.style.placeholder.as_deref().unwrap_or_default()),
    props.min_height,
    props.hide_toolbar,
    props.readonly || props.disabled,
    swift_optional_component_action(props.on_save.as_deref(), None, context),
    variant_container(&props.style),
    variant_content(&props.style)
));
append_swift_modifiers(
    output,
    indent,
    &swift_modifiers_for_style(&props.style.style),
);
}

fn render_swift_image_cropper(
    props: &ImageCropperProps,
    indent: usize,
    output: &mut String,
    context: &SwiftReactiveContext,
) {
    let pad = " ".repeat(indent);
output.push_str(&format!(
    "{pad}DoweImageCropper(value: {}, initialValue: {}, label: {}, placeholder: {}, alt: {}, accept: {}, aspectRatio: {}, minWidth: {}, minHeight: {}, maxWidth: {}, maxHeight: {}, shape: {}, size: {}, disabled: {}, helpText: {}, errorText: {}, backgroundColor: {}, contentColor: {})\n",
    swift_text_binding(props.style.element.bind.as_deref(), context),
    swift_string_literal(props.src.as_deref().unwrap_or_default()),
    swift_optional_literal(props.style.label.as_deref()),
    swift_string_literal(props.style.placeholder.as_deref().unwrap_or("Upload")),
    swift_string_literal(&props.alt),
    swift_string_literal(&props.accept),
    swift_optional_literal(props.aspect_ratio.as_deref()),
    props.min_width,
    props.min_height,
    swift_optional_u16(props.max_width),
    swift_optional_u16(props.max_height),
    swift_string_literal(props.shape.as_str()),
    swift_string_literal(props.style.size.unwrap_or(ButtonSize::Md).as_str()),
    props.disabled,
    swift_optional_literal(props.help_text.as_deref()),
    swift_optional_literal(props.error_text.as_deref()),
    variant_container(&props.style),
    variant_content(&props.style)
));
append_swift_modifiers(
    output,
    indent,
    &swift_modifiers_for_style(&props.style.style),
);
}
