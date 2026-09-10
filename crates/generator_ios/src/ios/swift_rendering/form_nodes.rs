include!("form_button_node.rs");
include!("form_input_node.rs");
include!("form_select_node.rs");
include!("form_csv_and_drag_nodes.rs");
include!("form_editor_and_cropper_nodes.rs");
include!("form_password_phone.rs");
include!("form_pin_textarea.rs");
include!("form_node_helpers.rs");

fn render_swift_form_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: NativeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &SwiftReactiveContext,
) {
    match node {
        ViewNode::Button { props, children } => render_swift_button_node(
            props, children, indent, output, flow, inherited_font, default_family, context,
        ),
        ViewNode::ToggleTheme { props } => render_swift_theme_toggle(props, indent, output),
        ViewNode::SelectTheme { props } => {
            render_swift_theme_select(props, indent, output, inherited_font, default_family)
        }
        ViewNode::Fab { props, actions } => {
            render_swift_fab(props, actions, indent, output, context, None)
        }
        ViewNode::Input { props } => render_swift_input_node(
            props, indent, output, inherited_font, default_family, context,
        ),
        ViewNode::Slider { props } => render_swift_slider(props, indent, output, context),
        ViewNode::Dropzone { props } => render_swift_dropzone(props, indent, output),
        ViewNode::Select { props, options, option_each } => render_swift_select_node(
            props, options, option_each.as_ref(), indent, output, inherited_font, default_family, context,
        ),
        ViewNode::ComboBox { props, options } => render_swift_combo_box(
            props, options, indent, output, inherited_font, default_family, context,
        ),
        ViewNode::CsvField { props, columns } => render_swift_csv_field(props, columns, indent, output),
        ViewNode::DragDrop { props, items, groups } => {
            render_swift_drag_drop(props, items, groups, indent, output)
        }
        ViewNode::Editor { props } => render_swift_editor(props, indent, output, context),
        ViewNode::ImageCropper { props } => {
            render_swift_image_cropper(props, indent, output, context)
        }
        ViewNode::Password { props } => render_swift_password(props, indent, output, context),
        ViewNode::Phone { props } => render_swift_phone(props, indent, output, context),
        ViewNode::Pin { props } => render_swift_pin(props, indent, output, context),
        ViewNode::Textarea { props } => render_swift_textarea(props, indent, output, context),
        ViewNode::Checkbox { props } => render_swift_checkbox(props, indent, output, context),
        ViewNode::Color { props } => render_swift_color(props, indent, output, context),
        ViewNode::Date { props } => render_swift_date(props, indent, output, context),
        ViewNode::DateRange { props } => render_swift_date_range(props, indent, output, context),
        ViewNode::RadioGroup { props, options } => {
            render_swift_radio_group(props, options, indent, output, context)
        }
        ViewNode::Toggle { props } => render_swift_toggle(props, indent, output, context),
        _ => unreachable!(),
    }
}
