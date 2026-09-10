fn render_swift_csv_field(
    props: &CsvFieldProps,
    columns: &[CsvColumn],
    indent: usize,
    output: &mut String,
) {
    let pad = " ".repeat(indent);
output.push_str(&format!(
    "{pad}DoweCsvField(label: {}, buttonText: {}, modalTitle: {}, instructions: {}, columns: {}, backgroundColor: {}, contentColor: {})\n",
    swift_optional_literal(props.style.label.as_deref()),
    swift_string_literal(&props.button_text),
    swift_string_literal(&props.modal_title),
    swift_string_literal(&props.instructions),
    swift_csv_columns(columns),
    variant_container(&props.style),
    variant_content(&props.style)
));
append_swift_modifiers(
    output,
    indent,
    &swift_modifiers_for_style(&props.style.style),
);
}

fn render_swift_drag_drop(
    props: &DragDropProps,
    items: &[DragItem],
    groups: &[DragGroup],
    indent: usize,
    output: &mut String,
) {
    let pad = " ".repeat(indent);
output.push_str(&format!(
    "{pad}DoweDragDrop(label: {}, emptyText: {}, direction: {}, items: {}, groups: {}, backgroundColor: {}, contentColor: {})\n",
    swift_optional_literal(props.style.label.as_deref()),
    swift_string_literal(&props.empty_text),
    swift_string_literal(props.direction.as_str()),
    swift_drag_items(items),
    swift_drag_groups(groups),
    variant_container(&props.style),
    variant_content(&props.style)
));
append_swift_modifiers(
    output,
    indent,
    &swift_modifiers_for_style(&props.style.style),
);
}
