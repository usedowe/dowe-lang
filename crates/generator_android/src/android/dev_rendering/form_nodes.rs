fn render_dev_android_form_node(
    node: &ViewNode,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    inherited_color: Option<String>,
    context: &ComposeReactiveContext,
    children_method: Option<&str>,
) {
    match node {
        ViewNode::Checkbox { .. }
        | ViewNode::Color { .. }
        | ViewNode::Date { .. }
        | ViewNode::DateRange { .. }
        | ViewNode::Dropzone { .. }
        | ViewNode::Fab { .. }
        | ViewNode::RadioGroup { .. }
        | ViewNode::Slider { .. }
        | ViewNode::Toggle { .. }
        | ViewNode::ToggleTheme { .. }
        | ViewNode::SelectTheme { .. } => render_dev_android_form_actions_node(
            node,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            inherited_color,
            context,
            children_method,
        ),
        ViewNode::Password { props } => render_dev_android_password(
            props,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            context,
        ),
        ViewNode::ImageCropper { props } => render_dev_android_image_cropper(
            props,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            context,
        ),
        ViewNode::Textarea { props } => render_dev_android_textarea(
            props,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            context,
        ),
        ViewNode::Phone { props } => render_dev_android_phone(
            props,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            context,
        ),
        ViewNode::Pin { props } => render_dev_android_pin(
            props,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            context,
        ),
        ViewNode::ComboBox { props, options } => render_dev_android_combo(
            props,
            options,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            context,
        ),
        ViewNode::Input { .. } | ViewNode::Select { .. } => render_dev_android_form_fields_node(
            node,
            parent,
            parent_gap,
            parent_horizontal,
            counter,
            output,
            inherited_font,
            inherited_color,
            context,
            children_method,
        ),
        _ => {}
    }
}

fn render_dev_android_image_cropper(
    props: &dowe_components::ImageCropperProps,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let view = next_dev_view(counter);
    let stage = next_dev_view(counter);
    let field = next_dev_view(counter);
    let actions = next_dev_view(counter);
    let key = props
        .style
        .element
        .bind
        .as_deref()
        .map(|path| context.signal_path(path))
        .or_else(|| props.name.clone())
        .unwrap_or_else(|| format!("image-cropper:{field}"));
    let initial = escape_java(props.src.as_deref().unwrap_or_default());
    let key = escape_java(&key);
    let accept = escape_java(&props.accept);
    let aspect = props
        .aspect_ratio
        .as_deref()
        .map(|value| format!("\"{}\"", escape_java(value)))
        .unwrap_or_else(|| "\"1\"".to_string());
    let max_width = props
        .max_width
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-1".to_string());
    let max_height = props
        .max_height
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-1".to_string());
    let min_width = props.min_width.to_string();
    let min_height = props.min_height.to_string();
    let size = props.style.size.unwrap_or(ButtonSize::Md).as_str();
    let shape = props.shape.as_str();
    let alt = escape_java(&props.alt);
    let placeholder = escape_java(props.style.placeholder.as_deref().unwrap_or("Upload"));
    let content = dev_variant_content(&props.style);
    let container = dev_variant_container(&props.style);
    let font = dev_font_value(props.style.style.font.as_ref().or(inherited_font));

    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n        String {view}Key = \"{key}\";\n        String {view}Value = doweTextValue({view}Key, null);\n        if (!doweState.containsKey({view}Key)) {view}Value = \"{initial}\";\n        boolean {view}HasImage = {view}Value != null && !{view}Value.isEmpty();\n"
    ));
    if let Some(label) = props.style.label.as_deref() {
        output.push_str(&format!(
            "        TextView {view}Label = doweControlLabel(\"{}\", {content}, {font});\n        doweAdd({view}, {view}Label);\n",
            escape_java(label)
        ));
    }
    let rendered = format!(
        "        FrameLayout {stage} = new FrameLayout(this);\n        {stage}.setMinimumHeight(doweDp(doweImageCropperSize(\"{size}\")));\n        {stage}.setContentDescription(\"{alt}\");\n        {stage}.setBackground(doweInputBackground({container}, {content}, DOWE_RADIUS));\n        if ({view}HasImage) {{\n            FrameLayout {field}Image = doweImage({view}Value, \"{alt}\", \"{aspect}\", \"cover\", {container}, null);\n            doweImageCropperShape({field}Image, \"{shape}\");\n            {field}Image.setOnClickListener(target -> doweOpenImageCropperPicker({view}Key, \"{accept}\", {aspect}, {min_width}, {min_height}, {max_width}, {max_height}));\n            {field}Image.setEnabled({});\n            {stage}.addView({field}Image, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n        }} else {{\n            Button {field}Upload = new Button(this);\n            {field}Upload.setText(\"Upload\\n{placeholder}\");\n            {field}Upload.setAllCaps(false);\n            {field}Upload.setTextColor({content});\n            {field}Upload.setBackgroundColor(Color.TRANSPARENT);\n            {field}Upload.setEnabled({});\n            {field}Upload.setOnClickListener(target -> doweOpenImageCropperPicker({view}Key, \"{accept}\", {aspect}, {min_width}, {min_height}, {max_width}, {max_height}));\n            {stage}.addView({field}Upload, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));\n        }}\n        doweAdd({view}, {stage}, 4, false);\n",
        !props.disabled,
        !props.disabled,
        min_width = min_width,
        min_height = min_height,
        max_width = max_width,
        max_height = max_height,
    );
    let rendered = rendered.replace(
        &format!("doweOpenImageCropperPicker({view}Key, \"{accept}\", {aspect}, "),
        &format!("doweOpenImageCropperPicker({view}Key, \"{accept}\", {aspect}, \"{shape}\", "),
    );
    output.push_str(&rendered.replace(
        &format!("doweImage({view}Value, \"{alt}\", \"{aspect}\","),
        &format!("doweImage({view}Value, \"{alt}\", {aspect},"),
    ));
    output.push_str(&format!(
        "        {stage}.setLayoutParams(new LinearLayout.LayoutParams(doweDp(doweImageCropperSize(\"{size}\")), doweDp(doweImageCropperSize(\"{size}\"))));\n"
    ));
    let actions_markup = format!(
        "        LinearLayout {actions} = doweContainer(true);\n        Button {actions}Change = new Button(this);\n        {actions}Change.setText(\"Change\");\n        {actions}Change.setAllCaps(false);\n        {actions}Change.setEnabled({} && {view}HasImage);\n        {actions}Change.setOnClickListener(target -> doweOpenImageCropperPicker({view}Key, \"{accept}\", {aspect}, {min_width}, {min_height}, {max_width}, {max_height}));\n        doweAdd({actions}, {actions}Change);\n        Button {actions}Remove = new Button(this);\n        {actions}Remove.setText(\"Remove\");\n        {actions}Remove.setAllCaps(false);\n        {actions}Remove.setEnabled({} && {view}HasImage);\n        {actions}Remove.setOnClickListener(target -> {{ doweWrite({view}Key, \"\"); renderCurrentRoute(false); }});\n        doweAdd({actions}, {actions}Remove, 8, true);\n        {actions}.setVisibility({view}HasImage ? View.VISIBLE : View.GONE);\n        doweAdd({view}, {actions}, 4, false);\n",
        !props.disabled,
        !props.disabled,
        min_width = min_width,
        min_height = min_height,
        max_width = max_width,
        max_height = max_height,
    );
    output.push_str(&actions_markup.replace(
        &format!("doweOpenImageCropperPicker({view}Key, \"{accept}\", {aspect}, "),
        &format!("doweOpenImageCropperPicker({view}Key, \"{accept}\", {aspect}, \"{shape}\", "),
    ));
    if let Some(text) = props.error_text.as_deref().or(props.help_text.as_deref()) {
        output.push_str(&format!(
            "        TextView {view}Help = doweText(\"{}\", {}, 12f, 400, 0f, 1.2f, {font});\n        doweAdd({view}, {view}Help, 4, false);\n",
            escape_java(text),
            if props.error_text.is_some() {
                java_color(ColorToken::Danger)
            } else {
                content
            }
        ));
    }
    apply_dev_android_style(&props.style.style, &view, true, output);
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}
