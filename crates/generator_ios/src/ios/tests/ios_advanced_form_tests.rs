#[test]
fn generates_swiftui_advanced_form_components() {
    let output = generate_ios(
        &[advanced_form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let phone_page = output
        .files
        .iter()
        .find(|file| file.content.contains("DowePhone(value:"))
        .expect("phone page");
    let phone_catalogs = output
        .files
        .iter()
        .filter(|file| {
            file.relative_path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.starts_with("DowePhoneCatalog"))
        })
        .collect::<Vec<_>>();
    let views = swift_content(&output);

    assert!(views.contains("struct DoweComboBox: View"));
    assert!(views.contains("DoweComboBox(value: state.binding(\"profile.role\")"));
    let combo_box = views
        .lines()
        .find(|line| line.contains("DoweComboBox(value: state.binding(\"profile.role\")"))
        .expect("combo box call");
    assert!(combo_box.contains("minHeight: CGFloat(40)"), "{combo_box}");
    assert!(combo_box.contains("fontSize: doweTextSize(viewportWidth, min: CGFloat(12), preferredBase: CGFloat(11.2), preferredViewport: CGFloat(0.2), max: CGFloat(14))"), "{combo_box}");
    assert!(views.contains("DoweComboOption(value: \"admin\", label: \"Admin\""));
    assert!(views.contains("DoweComboAnchorPresenter"));
    assert!(
        views.contains(
            "DoweAnchoredPopoverPresenter(isPresented: isPresented, minWidth: CGFloat(280)"
        )
    );
    assert!(views.contains("loadingText: \"Loading\""));
    assert!(views.contains("disabled: false"));
    assert!(views.contains("struct DoweCsvColumn: Identifiable"));
    assert!(views.contains("DoweCsvField(label: \"Import\""));
    assert!(views.contains("DoweCsvColumn(name: \"email\", label: \"Email\")"));
    assert!(views.contains("struct DoweDragGroup: Identifiable"));
    assert!(views.contains("DoweDragDrop(label: \"Tasks\""));
    assert!(views.contains("DoweDragItem(id: \"draft\", label: \"Draft\""));
    let editor_call = views
        .lines()
        .find(|line| line.contains("DoweEditorField(value: state.binding(\"profile.notes\")"))
        .expect("editor call");
    assert!(editor_call.find("language:").unwrap() < editor_call.find("initialValue:").unwrap());
    assert!(views.contains("onSave: { state.run(\"save-editor\") }"));
    assert!(views.contains("DoweImageCropper(value: state.binding(\"profile.avatar\")"));
    assert!(views.contains("fileImporter(isPresented: $pickerPresented"));
    assert!(views.contains("doweCropImage("));
    assert!(views.contains("return \"data:\\(jpeg ? \"image/jpeg\" : \"image/png\")"));
    assert!(views.contains("Button(\"Apply\")"));
    assert!(views.contains(
        "context.stroke(path, with: .color(.white.opacity(0.65)), lineWidth: CGFloat(1))"
    ));
    assert!(!views.contains("context.stroke(Path { path in"));
    assert!(views.contains("DowePassword(value: state.binding(\"profile.password\")"));
    let password_call = views
        .lines()
        .find(|line| line.contains("DowePassword(value: state.binding(\"profile.password\")"))
        .expect("password call");
    assert!(
        password_call.contains("minHeight: CGFloat(48)"),
        "{password_call}"
    );
    assert!(
        password_call.contains("showIcon: DoweControlIcon("),
        "{password_call}"
    );
    assert!(
        password_call.contains("hideIcon: DoweControlIcon("),
        "{password_call}"
    );
    assert!(password_call.contains("fontSize: doweTextSize(viewportWidth, min: CGFloat(14), preferredBase: CGFloat(13.12), preferredViewport: CGFloat(0.25), max: CGFloat(16))"), "{password_call}");
    assert!(views.contains("private var strengthColor: Color"));
    assert!(views.contains("DoweDesign.danger"));
    assert!(views.contains("DoweDesign.warning"));
    assert!(views.contains("DoweDesign.success"));
    assert!(
        views.contains(".frame(maxWidth: .infinity, minHeight: CGFloat(4), maxHeight: CGFloat(4))")
    );
    let password = views
        .split("struct DowePassword: View")
        .nth(1)
        .expect("password runtime");
    assert!(password.contains("private var visiblePlaceholder: String"));
    assert!(password.contains("TextField(visiblePlaceholder, text: textBinding)"));
    assert!(password.contains("SecureField(visiblePlaceholder, text: textBinding)"));
    assert!(password.contains("DoweSvgView("));
    assert!(password.contains(".frame(width: CGFloat(32), height: CGFloat(32))"));
    assert!(
        password.contains(".accessibilityLabel(visible ? \"Hide password\" : \"Show password\")")
    );
    assert!(password.contains("if validationError != nil"));
    assert!(views.contains("DowePhone(value: state.binding(\"profile.phone\")"));
    let phone_call = views
        .lines()
        .find(|line| line.contains("DowePhone(value: state.binding(\"profile.phone\")"))
        .expect("phone call");
    assert!(
        phone_call.contains("minHeight: CGFloat(56)"),
        "{phone_call}"
    );
    assert!(phone_call.contains("fontSize: doweTextSize(viewportWidth, min: CGFloat(16), preferredBase: CGFloat(15.2), preferredViewport: CGFloat(0.3), max: CGFloat(18))"), "{phone_call}");
    assert!(
        phone_page
            .content
            .contains("countries: DowePhoneCatalog.countries")
    );
    assert!(!phone_page.content.contains("DowePhoneCountry(code:"));
    assert!(phone_catalogs.len() > 2);
    assert!(
        phone_catalogs
            .iter()
            .all(|file| file.content.len() < 128_000)
    );
    assert!(views.contains("DowePhoneCountry(code: \"US\""));
    let phone = views
        .split("struct DowePhone: View")
        .nth(1)
        .expect("phone runtime")
        .split("struct DowePin: View")
        .next()
        .expect("phone body");
    assert!(views.contains("struct DowePhoneCountryAnchorPresenter: View"));
    assert!(views.contains("struct DowePhoneCountryPopover: View"));
    assert!(phone.contains("DowePhoneCountryAnchorPresenter("));
    let phone_anchor = views
        .split("struct DowePhoneCountryAnchorPresenter: View")
        .nth(1)
        .expect("phone country anchor runtime")
        .split("struct DowePhoneCountryPopover: View")
        .next()
        .expect("phone country anchor body");
    assert!(phone_anchor.contains("DoweAnchoredPopoverPresenter("));
    assert!(phone_anchor.contains("minWidth: CGFloat(280)"));
    assert!(phone_anchor.contains("maxWidth: CGFloat(384)"));
    assert!(views.contains("DoweDesign.surfaceText.opacity(0.07)"));
    assert!(phone.contains("filter { $0.isNumber }"));
    assert!(phone.contains(".keyboardType(.numberPad)"));
    assert!(phone.contains("DoweSvgView(viewBox: selectedCountry.flag.viewBox"));
    assert!(!phone.contains(".sheet(isPresented:"));
    assert!(!phone.contains("NavigationStack"));
    assert!(!phone.contains("List(filteredCountries)"));
    assert!(!phone.contains(".searchable(text:"));
    assert!(views.contains("DowePin(value: state.binding(\"profile.pin\")"));
    let pin = views
        .split("struct DowePin: View")
        .nth(1)
        .expect("pin field runtime");
    assert!(pin.contains("@FocusState private var focusedCell: Int?"));
    assert!(pin.contains(".focused($focusedCell, equals: index)"));
    assert!(
        pin.contains("let cellWidth: CGFloat = size == \"sm\" ? 40 : (size == \"lg\" ? 52 : 44)")
    );
    assert!(
        pin.contains("let cellHeight: CGFloat = size == \"sm\" ? 32 : (size == \"lg\" ? 48 : 40)")
    );
    assert!(pin.contains(".font(.system(size: fontSize, weight: .bold))"));
    assert!(
        pin.contains(
            "nextFocus = !nextCells[index].isEmpty && index + 1 < length ? index + 1 : nil"
        )
    );
    assert!(pin.contains("DispatchQueue.main.async"));
    assert!(pin.contains("SecureField(\"\", text: binding(for: index))"));
    assert!(views.contains("DoweTextarea(value: state.binding(\"profile.bio\")"));
    let textarea_call = views
        .lines()
        .find(|line| line.contains("DoweTextarea(value: state.binding(\"profile.bio\")"))
        .expect("textarea call");
    assert!(textarea_call.contains("fontSize: doweTextSize(viewportWidth, min: CGFloat(14), preferredBase: CGFloat(13.12), preferredViewport: CGFloat(0.25), max: CGFloat(16))"), "{textarea_call}");
    let textarea = views
        .split("struct DoweTextarea: View")
        .nth(1)
        .expect("textarea runtime");
    assert!(textarea.contains("@FocusState private var focused: Bool"));
    assert!(textarea.contains("let fontSize: CGFloat"));
    assert!(textarea.contains(".font(.system(size: fontSize))"));
    assert!(textarea.contains("private var visiblePlaceholder: Bool"));
    assert!(textarea.contains("!floating || focused"));
    assert!(textarea.contains("if visiblePlaceholder"));
    assert!(textarea.contains(".focused($focused)"));
}
#[test]
fn draw_ios_preserves_selection_and_gesture_contracts() {
    let runtime = super::swift_runtime_canvas();
    for expected in [
        "@StateObject private var selection = DoweCanvasSelection()",
        "let renderedCommands = commands",
        "selectedId: selectedId",
        "selectionPath(command)",
        "private var drawingPointer: ObjectIdentifier?",
        "private var drawingMode: String?",
        "private var nextLayerSequence = 1",
        "guard drawingPointer == pointer else { return }",
        "if kind != \"down\" || logical.inside",
        "points.append([\"x\": point.x, \"y\": point.y])",
        "segmentDistance(point, start:",
        "case \"text\":",
        "layer ?? [:]",
    ] {
        assert!(runtime.contains(expected), "missing Draw contract: {expected}");
    }
}
include!("draw_interactions.rs");
