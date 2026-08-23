fn render_dev_android_pin(
    props: &dowe_components::PinProps,
    parent: &str,
    parent_gap: Option<&str>,
    parent_horizontal: bool,
    counter: &mut usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    context: &ComposeReactiveContext,
) {
    let view = next_dev_view(counter);
    let row = next_dev_view(counter);
    let pin_cells = format!("{view}PinCells");
    let pin_updating = format!("{view}PinUpdating");
    let has_validation = dev_has_validation(&props.style.element)
        || props.help_text.is_some()
        || props.error_text.is_some();
    let size = props.style.size.unwrap_or(ButtonSize::Md);
    let text_size = dev_text_size_expr(false, form_control_text_size(size));
    let (width, height) = match size {
        ButtonSize::Sm => (40, 32),
        ButtonSize::Lg => (52, 48),
        _ => (44, 40),
    };
    let radius = dev_style_radius(&props.style.style);
    let background =
        if props.style.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
            format!(
                "doweInputBackground({}, {}, {radius})",
                dev_variant_container(&props.style),
                java_color(ColorToken::Muted)
            )
        } else {
            format!(
                "doweBackground({}, {radius})",
                dev_variant_container(&props.style)
            )
        };
    let content = dev_variant_content(&props.style);
    let font = dev_font_value(props.style.style.font.as_ref().or(inherited_font));
    let input_type = match props.kind {
        dowe_components::PinKind::Number => "android.text.InputType.TYPE_CLASS_NUMBER",
        dowe_components::PinKind::Password => {
            "android.text.InputType.TYPE_CLASS_TEXT | android.text.InputType.TYPE_TEXT_VARIATION_PASSWORD"
        }
        dowe_components::PinKind::Text => "android.text.InputType.TYPE_CLASS_TEXT",
    };
    let sanitize = if props.kind == dowe_components::PinKind::Number {
        "next = next.replaceAll(\"[^0-9]\", \"\");"
    } else {
        ""
    };
    let bind_path = props
        .style
        .element
        .bind
        .as_deref()
        .map(|path| context.signal_path(path));
    output.push_str(&format!(
        "        LinearLayout {view} = doweContainer(false);\n        LinearLayout {row} = doweContainer(true);\n        {row}.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));\n        EditText[] {pin_cells} = new EditText[{}];\n        boolean[] {pin_updating} = new boolean[] {{ false }};\n",
        props.length
    ));
    if let Some(label) = props.style.label.as_deref() {
        output.push_str(&format!(
            "        TextView {view}Label = doweControlLabel(\"{}\", {content}, {font});\n        doweAdd({view}, {view}Label);\n",
            escape_java(label)
        ));
    }
    for index in 0..props.length {
        let cell = next_dev_view(counter);
        let initial = if let Some(path) = bind_path.as_deref() {
            format!(
                "doweTextValue(\"{}\", null).length() > {} ? String.valueOf(doweTextValue(\"{}\", null).charAt({})) : \"\"",
                escape_java(path),
                index,
                escape_java(path),
                index
            )
        } else {
            props
                .value
                .as_deref()
                .and_then(|value| value.chars().nth(index as usize))
                .map(|value| format!("\"{}\"", escape_java(&value.to_string())))
                .unwrap_or_else(|| "\"\"".to_string())
        };
        output.push_str(&format!(
            "        EditText {cell} = new EditText(this);\n        {cell}.setLayoutParams(new LinearLayout.LayoutParams(doweDp({width}), doweDp({height})));\n        {cell}.setTypeface(Typeface.create({font}, android.graphics.Typeface.NORMAL));\n        {cell}.setTextSize({});\n        {cell}.setIncludeFontPadding(false);\n        {cell}.setGravity(Gravity.CENTER);\n        {cell}.setTextColor({content});\n        {cell}.setSingleLine(true);\n        {cell}.setInputType({input_type});\n        {cell}.setMaxLines(1);\n        {cell}.setPadding(doweDp(8), 0, doweDp(8), 0);\n        {cell}.setBackground({background});\n        {cell}.setText({initial});\n        {pin_cells}[{index}] = {cell};\n",
            text_size
        ));
        let write = bind_path
            .as_deref()
            .map(|path| format!("doweWrite(\"{}\", pinValue.toString());", escape_java(path)))
            .unwrap_or_default();
        output.push_str(&format!(
            "        {cell}.addTextChangedListener(new TextWatcher() {{\n            public void beforeTextChanged(CharSequence value, int start, int count, int after) {{}}\n            public void onTextChanged(CharSequence value, int start, int before, int count) {{}}\n            public void afterTextChanged(Editable value) {{ if ({pin_updating}[0]) return; String next = value.toString(); {sanitize} if (!next.equals(value.toString())) {{ value.replace(0, value.length(), next); return; }} if (next.length() > 1) {{ {pin_updating}[0] = true; int accepted = Math.min(next.length(), {pin_cells}.length - {}); for (int offset = 0; offset < accepted; offset++) {pin_cells}[{} + offset].setText(String.valueOf(next.charAt(offset))); {pin_updating}[0] = false; StringBuilder pinValue = new StringBuilder(); for (EditText pinCell : {pin_cells}) pinValue.append(pinCell.getText().toString()); {write} {pin_cells}[{} + accepted - 1].requestFocus(); return; }} StringBuilder pinValue = new StringBuilder(); for (EditText pinCell : {pin_cells}) pinValue.append(pinCell.getText().toString()); {write} if (!next.isEmpty() && {} + 1 < {pin_cells}.length) {pin_cells}[{} + 1].requestFocus(); }}\n        }});\n        {cell}.setOnKeyListener((focused, keyCode, event) -> {{ if (keyCode == KeyEvent.KEYCODE_DEL && event.getAction() == KeyEvent.ACTION_DOWN && {cell}.getText().length() == 0 && {} > 0) {{ {pin_cells}[{} - 1].requestFocus(); return true; }} return false; }});\n        doweAdd({row}, {cell}, 8, true);\n",
            index,
            index,
            index,
            index,
            index,
            index,
            index
        ));
    }
    output.push_str(&format!(
        "        {row}.addOnLayoutChangeListener((target, left, top, right, bottom, oldLeft, oldTop, oldRight, oldBottom) -> {{ int availableCellWidth = Math.max(doweDp(1), ((right - left) - doweDp(8) * Math.max(0, {pin_cells}.length - 1)) / Math.max(1, {pin_cells}.length)); int responsiveCellWidth = Math.min(doweDp({width}), availableCellWidth); for (EditText pinCell : {pin_cells}) {{ ViewGroup.LayoutParams params = pinCell.getLayoutParams(); if (params.width != responsiveCellWidth) {{ params.width = responsiveCellWidth; pinCell.setLayoutParams(params); }} }} }});\n"
    ));
    output.push_str(&format!("        doweAdd({view}, {row});\n"));
    if has_validation {
        output.push_str(&format!(
            "        DoweValidationBinding {view}Validation = doweValidation(\"{view}\", {view}, {pin_cells}[0], {pin_cells}[0], {}, {}, {}, () -> {{ StringBuilder validationValue = new StringBuilder(); for (EditText pinCell : {pin_cells}) validationValue.append(pinCell.getText().toString()); return validationValue.toString(); }}, false, {content}, {font});\n        for (int validationIndex = 0; validationIndex < {pin_cells}.length; validationIndex++) {{ if (validationIndex > 0) {view}Validation.addSurface({pin_cells}[validationIndex]); {view}Validation.watchText({pin_cells}[validationIndex]); }}\n",
            dev_nullable_string(props.help_text.as_deref()),
            dev_nullable_string(props.error_text.as_deref()),
            dev_validation_rules(&props.style.element, context)
        ));
    }
    if !has_validation
        && let Some(text) = props.error_text.as_deref().or(props.help_text.as_deref())
    {
        let color = if props.error_text.is_some() {
            java_color(ColorToken::Danger).to_string()
        } else {
            format!("doweAlpha({content}, 0.72f)")
        };
        let help = next_dev_view(counter);
        output.push_str(&format!(
            "        TextView {help} = doweText(\"{}\", {color}, 12f, 400, 0f, 1.2f, {font});\n        doweAdd({view}, {help}, 4, false);\n",
            escape_java(text)
        ));
    }
    apply_dev_android_style(&props.style.style, &view, false, output);
    if parent_horizontal && props.style.style.sizing.w.is_none() {
        output.push_str(&format!(
            "        {view}.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));\n"
        ));
    }
    output.push_str(&dev_add(parent, &view, parent_gap, parent_horizontal));
}
