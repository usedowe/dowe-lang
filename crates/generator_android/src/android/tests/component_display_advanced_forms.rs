#[test]
fn generates_android_slider_with_block_width_and_bound_initial_value() {
    let output = generate_android(
        &[slider_signal_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    let dev = dev_java_source(&output);

    assert!(
        views
            .content
            .contains("DoweSliderField(value = state.text(\"volume\").toFloatOrNull() ?: 0f")
    );
    assert!(
        views
            .content
            .contains("Column(modifier = modifier.fillMaxWidth()")
    );
    assert!(views.content.contains("modifier = Modifier.fillMaxWidth()"));
    assert!(dev.content.contains("dowePutInitial(\"volume\", 40);"));
    assert!(dev.content.contains("SeekBar"));
    assert!(dev.content.contains(
        ".setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));"
    ));
    assert!(
        dev.content
            .contains("Double.parseDouble(doweTextValue(\"volume\", null))")
    );
    assert!(
        dev.content
            .contains("BoundValue = Math.max(0, Math.min(100,")
    );
    assert!(dev.content.contains(".setText(String.valueOf("));
}

#[test]
fn generates_compose_advanced_form_components() {
    let output = generate_android(
        &[advanced_form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    let dev = dev_java_source(&output);

    assert!(views.content.contains("private fun DoweComboBox("));
    assert!(
        views
            .content
            .contains("DoweComboBox(value = state.text(\"profile.role\")")
    );
    let combo_box = views
        .content
        .lines()
        .find(|line| line.contains("DoweComboBox(value = state.text(\"profile.role\")"))
        .expect("combo box call");
    assert!(combo_box.contains("minHeight = 40.dp"), "{combo_box}");
    assert!(combo_box.contains("fontSize = doweTextSize(viewportWidth, min = 12f, preferredBase = 11.2f, preferredViewport = 0.2f, max = 14f)"), "{combo_box}");
    assert!(
        views
            .content
            .contains("DoweComboOption(\"admin\", \"Admin\", \"Full access\"")
    );
    assert!(
        views
            .content
            .contains("searchPlaceholder = \"Search roles\"")
    );
    assert!(views.content.contains("DoweAnchoredPopover("));
    assert!(views.content.contains("clearable = true"));
    assert!(dev.content.contains("doweComboPopup("));
    assert!(dev.content.contains("doweWrite(\"profile.role\", \"\");"));
    assert!(views.content.contains("private data class DoweCsvColumn"));
    assert!(views.content.contains("DoweCsvField(label = \"Import\""));
    assert!(
        views
            .content
            .contains("DoweCsvColumn(\"email\", \"Email\")")
    );
    assert!(views.content.contains("private data class DoweDragGroup"));
    assert!(views.content.contains("DoweDragDrop(label = \"Tasks\""));
    assert!(
        views
            .content
            .contains("DoweDragItem(\"draft\", \"Draft\", \"Prepare\", false)")
    );
    assert!(
        views
            .content
            .contains("DoweEditorField(value = state.text(\"profile.notes\")")
    );
    assert!(
        views
            .content
            .contains("onSave = { actionScope.launch { state.run(\"save-editor\") } }")
    );
    assert!(
        views
            .content
            .contains("DoweImageCropper(value = state.text(\"profile.avatar\")")
    );
    assert!(
        views
            .content
            .contains("rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument())")
    );
    assert!(views.content.contains("doweCropBitmap("));
    assert!(
        views
            .content
            .contains("TextButton(onClick = { cropDialog = false })")
    );
    assert!(dev.content.contains("doweOpenImageCropperPicker("));
    assert!(dev.content.contains(
        "doweOpenImageCropperPicker(String key, String accept, String aspect, String shape"
    ));
    assert!(dev.content.contains("doweShowImageCropperEditor("));
    assert!(dev.content.contains(
        "doweShowImageCropperEditor(Bitmap source, String key, String aspect, String shape"
    ));
    assert!(dev.content.contains("doweCropImage("));
    assert!(dev.content.contains("int resizedHeight"));
    assert!(dev.content.contains("int resizedWidth"));
    assert!(dev.content.contains("doweBitmapDataUrl("));
    assert!(
        views
            .content
            .contains("DowePassword(value = state.text(\"profile.password\")")
    );
    let password_call = views
        .content
        .lines()
        .find(|line| line.contains("DowePassword(value = state.text(\"profile.password\")"))
        .expect("password call");
    assert!(
        password_call.contains("minHeight = 48.dp"),
        "{password_call}"
    );
    assert!(
        password_call.contains("showIcon = { DoweSvg("),
        "{password_call}"
    );
    assert!(
        password_call.contains("hideIcon = { DoweSvg("),
        "{password_call}"
    );
    assert!(
        password_call.contains("Modifier.width(20.dp).height(20.dp)"),
        "{password_call}"
    );
    assert!(password_call.contains("fontSize = doweTextSize(viewportWidth, min = 14f, preferredBase = 13.12f, preferredViewport = 0.25f, max = 16f)"), "{password_call}");
    assert!(views.content.contains("value.any { it.isLowerCase() }"));
    assert!(views.content.contains("Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(4.dp))"));
    assert!(views.content.contains("DoweDesign.danger"));
    assert!(views.content.contains("DoweDesign.warning"));
    assert!(views.content.contains("DoweDesign.success"));
    assert!(views.content.contains("PasswordVisualTransformation()"));
    let password_runtime = views
        .content
        .split("private fun DowePassword(")
        .nth(1)
        .expect("password runtime");
    assert!(password_runtime.contains("fontSize: TextUnit, lineHeight: TextUnit"));
    assert!(
        views
            .content
            .contains("contentDescription = if (visible) \"Hide password\" else \"Show password\"")
    );
    assert!(
        dev.content
            .contains("PasswordTransformationMethod.getInstance()")
    );
    assert!(dev.content.contains("final View[]"));
    let dev_password = dev
        .content
        .find("= doweFloatingInput(")
        .map(|start| &dev.content[start..(start + 320).min(dev.content.len())])
        .expect("dev password");
    assert!(dev_password.contains("setMinimumHeight(doweDp(48))"));
    assert!(dev.content.contains("DOWE_DANGER"));
    assert!(dev.content.contains("DOWE_WARNING"));
    assert!(dev.content.contains("DOWE_SUCCESS"));
    assert!(
        dev.content
            .contains("setContentDescription(\"Show password\")")
    );
    assert!(dev.content.contains("setContentDescription("));
    assert!(dev.content.contains("DoweSvgPathEntry"));
    assert!(
        views
            .content
            .contains("DowePhone(value = state.text(\"profile.phone\")")
    );
    let phone_call = views
        .content
        .lines()
        .find(|line| line.contains("DowePhone(value = state.text(\"profile.phone\")"))
        .expect("phone call");
    assert!(phone_call.contains("minHeight = 56.dp"), "{phone_call}");
    assert!(phone_call.contains("fontSize = doweTextSize(viewportWidth, min = 16f, preferredBase = 15.2f, preferredViewport = 0.3f, max = 18f)"), "{phone_call}");
    assert!(views.content.contains("countries = dowePhoneCountries"));
    assert!(
        views
            .content
            .contains("private fun dowePhoneCountries0(): List<DowePhoneCountry>")
    );
    assert!(
        views
            .content
            .contains("private val dowePhoneCountries: List<DowePhoneCountry> = buildList")
    );
    let phone = views
        .content
        .split("private fun DowePhone(")
        .nth(1)
        .expect("phone runtime");
    assert!(phone.contains("DoweAnchoredPopover"));
    assert!(phone.contains("searchPlaceholder"));
    assert!(phone.contains("DoweSvg(viewBox = selected.viewBox"));
    assert!(phone.contains("Modifier.size(24.dp).align(Alignment.CenterVertically)"));
    assert!(phone.contains("modifier = Modifier.align(Alignment.CenterVertically)"));
    assert!(phone.contains("minWidth = 280.dp, maxWidth = 384.dp, maxHeight = 380.dp"));
    assert!(phone.contains("Text(item.name, modifier = Modifier.weight(1f)"));
    assert!(phone.contains("Text(\"+${item.dialCode}\", fontWeight = FontWeight.Bold"));
    assert!(phone.contains("var localValue by remember(value)"));
    assert!(phone.contains("char.isDigit()"));
    assert!(phone.contains("keyboardType = KeyboardType.Number"));
    assert!(dev.content.contains("dowePhonePopup"));
    assert!(dev.content.contains("setIncludeFontPadding(false)"));
    assert!(
        dev.content
            .contains("Params.gravity = Gravity.CENTER_VERTICAL")
    );
    assert!(dev.content.contains("Params.leftMargin = doweDp(6)"));
    assert!(dev.content.contains(
        "Math.min(doweDp(384), getResources().getDisplayMetrics().widthPixels - doweDp(16))"
    ));
    assert!(dev.content.contains("nameParams.leftMargin = doweDp(10)"));
    assert!(dev.content.contains("setMinimumHeight(doweDp(56))"));
    assert!(
        dev.content
            .contains("android.text.method.DigitsKeyListener.getInstance(\"0123456789\")")
    );
    let phone_route = output
        .files
        .iter()
        .find(|file| {
            file.relative_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("DoweDevRoute"))
                && file.content.contains("dowePhonePopup")
        })
        .expect("phone route shard");
    assert!(phone_route.content.contains("runtime.dowePhoneFlag("));
    let phone_flag_shards = output
        .files
        .iter()
        .filter(|file| {
            file.relative_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("DoweDevPhoneFlags"))
        })
        .collect::<Vec<_>>();
    assert!(phone_flag_shards.len() > 2);
    assert!(
        phone_flag_shards
            .iter()
            .all(|file| file.content.len() < 512_000)
    );
    assert!(
        phone_flag_shards
            .iter()
            .filter(|file| file
                .relative_path
                .file_name()
                .is_some_and(|name| { name.to_string_lossy() != "DoweDevPhoneFlags.java" }))
            .all(|file| file
                .content
                .contains("int viewportWidth = runtime.viewportWidth;"))
    );
    assert!(
        views
            .content
            .contains("DowePin(value = state.text(\"profile.pin\")")
    );
    let pin = views
        .content
        .split("private fun DowePin(")
        .nth(1)
        .expect("pin field runtime");
    assert!(pin.contains("FocusRequester"));
    assert!(pin.contains("requestFocus()"));
    assert!(pin.contains("KeyEvent.KEYCODE_DEL"));
    assert!(pin.contains("fontSize: TextUnit"));
    assert!(pin.contains("lineHeight: TextUnit"));
    assert!(pin.contains(".height(doweControlHeight(size))"));
    assert!(pin.contains("BoxWithConstraints(modifier = Modifier.fillMaxWidth())"));
    assert!(pin.contains("val responsiveCellWidth = minOf(cellWidth"));
    assert!(pin.contains(".width(responsiveCellWidth)"));
    assert!(!pin.contains("horizontalScroll(rememberScrollState())"));
    assert!(
        pin.contains(
            "TextStyle(color = contentColor, fontSize = fontSize, lineHeight = lineHeight"
        )
    );
    assert!(dev.content.contains("PinCells = new EditText["));
    assert!(
        dev.content
            .contains("PinUpdating = new boolean[] { false }")
    );
    assert!(
        dev.content
            .contains("int accepted = Math.min(next.length()")
    );
    assert!(dev.content.contains("setOnKeyListener"));
    assert!(dev.content.contains("doweWrite(\"profile.pin\""));
    assert!(dev.content.contains("doweDp(44), doweDp(40)"));
    assert!(dev.content.contains("addOnLayoutChangeListener"));
    assert!(
        dev.content
            .contains("int availableCellWidth = Math.max(doweDp(1)")
    );
    assert!(
        dev.content
            .contains("Math.min(doweDp(44), availableCellWidth)")
    );
    assert!(
        dev.content
            .contains("setTextSize(doweFluidTextSize(14f, 13.12f, 0.25f, 16f))")
    );
    assert!(
        views
            .content
            .contains("DoweTextarea(value = state.text(\"profile.bio\")")
    );
    let textarea_call = views
        .content
        .lines()
        .find(|line| line.contains("DoweTextarea(value = state.text(\"profile.bio\")"))
        .expect("textarea call");
    assert!(textarea_call.contains("fontSize = doweTextSize(viewportWidth, min = 14f, preferredBase = 13.12f, preferredViewport = 0.25f, max = 16f)"), "{textarea_call}");
    let textarea = views
        .content
        .split("private fun DoweTextarea(")
        .nth(1)
        .expect("textarea runtime");
    assert!(textarea.contains("var focused by remember { mutableStateOf(false) }"));
    assert!(textarea.contains("fontSize: TextUnit"));
    assert!(
        textarea.contains(
            "TextStyle(color = contentColor, fontSize = fontSize, lineHeight = lineHeight)"
        )
    );
    assert!(
        textarea
            .contains("if (value.isEmpty() && placeholder.isNotEmpty() && (!floating || focused))")
    );
    assert!(textarea.contains("modifier = Modifier.align(Alignment.TopStart)"));
    assert!(
        dev.content
            .contains("setGravity(Gravity.TOP | Gravity.START)")
    );
    assert!(dev.content.contains("doweFloatingTextarea("));
}

#[test]
fn generates_unique_dev_pin_cell_arrays_for_multiple_fields() {
    let mut route = advanced_form_route();
    let duplicate = match &route.page_tree {
        ViewNode::Scope { children, .. } => match &children[0] {
            ViewNode::Box { children, .. } => children
                .iter()
                .find(|node| matches!(node, ViewNode::Pin { .. }))
                .cloned()
                .expect("pin field"),
            _ => panic!("advanced form container"),
        },
        _ => panic!("advanced form scope"),
    };
    match &mut route.page_tree {
        ViewNode::Scope { children, .. } => match &mut children[0] {
            ViewNode::Box { children, .. } => children.push(duplicate),
            _ => panic!("advanced form container"),
        },
        _ => panic!("advanced form scope"),
    }
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let dev = dev_java_source(&output);
    let arrays = dev
        .content
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("EditText[] ")?
                .split_once(" = new EditText[")
                .map(|(name, _)| name)
        })
        .collect::<Vec<_>>();

    assert_eq!(arrays.len(), 2);
    assert_ne!(arrays[0], arrays[1]);
    assert!(!arrays.contains(&"pinCells"));
}

fn slider_signal_route() -> ViewRoute {
    ViewRoute {
        id: "slider".to_string(),
        route_path: "/slider".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Scope {
            constants: Vec::new(),
            signals: vec![ViewSignal {
                id: "volume".to_string(),
                name: "volume".to_string(),
                storage_key: "volume".to_string(),
                scope: dowe_components::ViewSignalScope::Page,
                storage: dowe_components::ViewSignalStorage::None,
                initial: ViewSignalValue::Number("40".to_string()),
                schema: None,
            }],
            actions: Vec::new(),
            children: vec![ViewNode::Slider {
                props: SliderProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Warning),
                        label: Some("Volume".to_string()),
                        element: ElementProps {
                            bind: Some("volume".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    value: "0".to_string(),
                    min: "0".to_string(),
                    max: "100".to_string(),
                    step: Some("5".to_string()),
                    size: ButtonSize::Lg,
                    name: Some("volume".to_string()),
                    hide_label: false,
                },
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

