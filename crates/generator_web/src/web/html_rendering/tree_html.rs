fn render_tree_html(props: &dowe_components::TreeProps, context: &ReactiveRenderContext) -> String {
    let folder_icon = solar_control_icon("folder-with-files").expect("bundled Tree folder icon");
    let file_icon = solar_control_icon("file-text").expect("bundled Tree file icon");
    let arrow_icon = solar_control_icon("alt-arrow-down").expect("bundled Tree arrow icon");
    let mut extra = format!(
        r##" role="tree" aria-label="{}" data-dowe-tree data-dowe-tree-data="{}" data-dowe-tree-default-open="{}" data-dowe-tree-empty-label="{}""##,
        escape_attr(&props.aria_label),
        escape_attr(&context.signal_path(&props.data)),
        props.default_open,
        escape_attr(&props.empty_label),
    );
    if let Some(bind) = props.bind.as_deref() {
        extra.push_str(&format!(
            r##" data-dowe-tree-bind="{}""##,
            escape_attr(&context.signal_path(bind))
        ));
    }
    if let Some(action) = props.on_select.as_deref() {
        extra.push_str(&format!(
            r##" data-dowe-tree-on-select="{}""##,
            escape_attr(&context.action_id(action))
        ));
    }
    format!(
        r#"<div{}><template data-dowe-tree-icon="folder">{}</template><template data-dowe-tree-icon="file">{}</template><template data-dowe-tree-icon="arrow">{}</template><div class="tree-content" data-dowe-tree-content></div><div class="tree-empty" data-dowe-tree-empty role="status" hidden></div></div>"#,
        attrs(
            tree_classes(props),
            Some(&props.style.element),
            Some(&extra),
            context
        ),
        render_svg_html(&folder_icon.props, &folder_icon.paths, context),
        render_svg_html(&file_icon.props, &file_icon.paths, context),
        render_svg_html(&arrow_icon.props, &arrow_icon.paths, context),
    )
}
