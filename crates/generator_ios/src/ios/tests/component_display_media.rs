#[test]
fn emits_large_theme_catalog_once_outside_route_view_expressions() {
    let mut design = DesignConfig::default();
    let base_theme = design.themes[0].clone();
    design.themes = (1..=18)
        .map(|index| {
            let mut theme = base_theme.clone();
            theme.name = if index == 1 {
                "light".to_string()
            } else {
                format!("palette-{index}")
            };
            theme
        })
        .collect();
    let themes = design
        .themes
        .iter()
        .map(|theme| theme.name.clone())
        .collect::<Vec<_>>();
    let select = ViewNode::SelectTheme {
        props: ThemeSelectProps {
            style: Default::default(),
            label: "Theme".to_string(),
            placeholder: "Choose a theme".to_string(),
            themes,
            default_theme: "light".to_string(),
        },
    };
    let mut themed_route = route();
    themed_route.page_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![ViewNode::Flex {
            props: Default::default(),
            children: vec![select.clone(), select],
        }],
    };

    let output = generate_ios(&[themed_route], &FontConfig::default(), &design, &[]);
    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("generated route page");
    let theme = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweTheme.swift"))
        .expect("generated theme module");

    assert_eq!(
        page.content
            .matches("DoweThemeModule.selectOptions")
            .count(),
        2
    );
    assert!(!page.content.contains("DoweSelectOption(value:"));
    assert_eq!(
        page.content
            .matches("helpText: nil, errorText: nil, validationRules: []")
            .count(),
        2
    );
    assert!(
        page.content
            .contains("private func routeBranch0() -> some View")
    );
    assert!(
        page.content
            .contains("private func routeBranch2() -> some View")
    );
    assert!(
        theme
            .content
            .contains("static let selectOptions: [DoweSelectOption] = [")
    );
    assert!(theme.content.contains(
        "DoweSelectOption(value: \"palette-18\", label: \"Palette 18\", description: nil)"
    ));
}

#[test]
fn keeps_inherited_font_children_inside_their_swiftui_expression() {
    let mut inherited_route = route();
    inherited_route.page_tree = ViewNode::Box {
        props: StyleProps {
            font: Some(ResponsiveValue::scalar(dowe_components::FontFamily::Inter)),
            ..Default::default()
        },
        children: vec![text("One"), text("Two"), text("Three"), text("Four")],
    };

    let output = generate_ios(
        &[inherited_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("generated route page");

    assert!(!page.content.contains("private func routeBranch"));
    assert!(page.content.contains("xs: .inter"));
}

#[test]
fn generates_floating_input_icons_with_active_visibility() {
    let output = generate_ios(
        &[form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    let input = views
        .lines()
        .find(|line| {
            line.contains(
                r#"DoweInputField(value: nil, label: "Name", placeholder: "Full name", floating: true"#,
            )
        })
        .expect("floating input");

    assert!(input.contains("startIcon: DoweControlIcon("));
    assert!(input.contains("endIcon: DoweControlIcon("));
    assert!(input.contains(r#"data: "M4 12h16""#));
    assert!(input.contains(r#"data: "M12 4v16""#));
    assert!(views.contains("private var iconsVisible: Bool {\n        !floating || active\n    }"));
    assert!(views.contains("if let startIcon, iconsVisible {"));
    assert!(views.contains("if let endIcon, iconsVisible {"));
    assert!(
        views.contains(".padding(.leading, active && startIcon != nil ? CGFloat(32) : CGFloat(0))")
    );
}

#[test]
fn generates_swiftui_media_display_form_components() {
    let output = generate_ios(
        &[media_display_form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweAudioView: View"));
    assert!(views.contains("DoweAudioView(source:"));
    assert!(views.contains("@State private var player: AVPlayer"));
    assert!(views.contains("ForEach(0..<50"));
    assert!(views.contains("DragGesture(minimumDistance: 0)"));
    assert!(views.contains("private func doweAudioTime"));
    assert!(views.contains("playIcon: DoweVideoIcon"));
    assert!(views.contains("private let doweAudioWaveform: [CGFloat]"));
    assert!(views.contains("private struct DoweAudioControlButton: View"));
    assert!(views.contains(".animation(.easeInOut(duration: 0.3), value: currentTime)"));
    assert!(views.contains("struct DoweCoverImage: View"));
    let cover_runtime = views
        .split("struct DoweCoverImage: View")
        .nth(1)
        .expect("cover runtime")
        .split("private func doweImageURL")
        .next()
        .expect("cover runtime boundary");
    assert!(cover_runtime.contains("GeometryReader { proxy in"));
    assert!(cover_runtime.contains(".frame(width: proxy.size.width, height: proxy.size.height)"));
    assert!(views.contains("struct DoweImageView: View"));
    let image_runtime = views
        .split("struct DoweImageView: View")
        .nth(1)
        .expect("image runtime")
        .split("private func doweImageAspect")
        .next()
        .expect("image aspect helper");
    assert!(image_runtime.contains("DoweImageAspectLayout(ratio: doweImageAspect(aspect))"));
    assert!(image_runtime.contains("struct DoweImageAspectLayout: Layout"));
    assert!(
        image_runtime
            .contains("return CGSize(width: resolvedWidth, height: resolvedWidth / resolvedRatio)")
    );
    assert!(image_runtime.contains("proposal: ProposedViewSize(bounds.size)"));
    assert!(image_runtime.contains(".clipped()"));
    assert!(image_runtime.contains(".accessibilityAddTraits(.isImage)"));
    assert!(image_runtime.contains(".accessibilityLabel(Text(alt))"));
    assert!(image_runtime.contains(".accessibilityHidden(alt.isEmpty)"));
    assert!(!image_runtime.contains("if !hideControls"));
    assert!(!image_runtime.contains(".background(backgroundColor.opacity(0.72))"));
    assert!(!image_runtime.contains(".aspectRatio(doweImageAspect(aspect)"));
    assert!(views.contains("DoweAccordionView(multiple:"));
    assert!(views.contains("defaultOpenIds: [\"intro\"]"));
    assert!(views.contains("{ openIds, toggleItem in"));
    assert!(views.contains("open: openIds.contains(\"intro\")"));
    assert!(views.contains(
        "@ViewBuilder let content: (Set<String>, @escaping (String) -> Void) -> Content"
    ));
    assert!(views.contains("arrowIcon: {"));
    assert!(views.matches("m19.704 12l-8.491-8.727a.75.75").count() >= 2);
    assert!(!views.contains("__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__"));
    assert!(views.contains(".rotationEffect(open ? .degrees(90) : .degrees(0))"));
    assert!(views.contains("Text(label)\n                        .font(.system(size: CGFloat(15), weight: .bold))\n                        .foregroundStyle(contentColor)"));
    assert!(
        views.contains("variant == \"ghost\" || variant == \"line\" ? CGFloat(0) : CGFloat(8)")
    );
    assert!(
        views.contains("variant == \"ghost\" || variant == \"line\" ? CGFloat(0) : CGFloat(4)")
    );
    assert!(views.contains(".frame(maxWidth: .infinity, alignment: .leading)"));
    assert!(views.contains("borderStyle: \"separator\""));
    assert!(views.contains("if borderStyle == \"separator\""));
    assert!(views.contains("radius: CGFloat(0), action: { toggleItem(\"intro\") }"));
    assert!(!views.contains("radius: if variant == \"ghost\""));
    assert!(!views.contains("Button(playing ? \"Pause\" : \"Play\")"));
    assert!(!views.contains("Text(open ? \"^\" : \"v\")"));
    assert!(views.contains("DoweCarouselView(variant: \"snapping\""));
    assert!(views.contains("ScrollView(.horizontal"));
    assert!(views.contains("showsIndicators: false"));
    assert!(views.contains("if showNavigation"));
    assert!(views.contains(".disabled(disableLoop && currentIndex == 0)"));
    assert!(views.contains("containerRelativeFrame(.horizontal"));
    assert!(views.contains("carouselHorizontalOffset"));
    assert!(views.contains(".scrollPosition(id: $scrollId)"));
    assert!(views.contains(".onChange(of: scrollId) { _, value in"));
    assert!(!views.contains(".onChange(of: scrollId) { value in"));
    assert!(views.contains(".scrollTransition(.interactive, axis: .horizontal)"));
    assert!(views.contains("rotation3DEffect"));
    assert!(views.contains("nonisolated private func carouselRotation(_ phase: Double) -> Double"));
    assert!(views.contains("nonisolated private func carouselScale(_ phase: Double) -> CGFloat"));
    assert!(views.contains("nonisolated private func carouselTilt(_ phase: Double) -> Double"));
    assert!(views.contains("nonisolated private func carouselOffset(_ phase: Double) -> CGFloat"));
    assert!(views.contains("nonisolated private func carouselOpacity(_ phase: Double) -> Double"));
    let carousel_runtime = views
        .split("struct DoweCarouselView<Content: View>: View")
        .nth(1)
        .expect("carousel runtime")
        .split("struct DoweCarouselSlideView<Content: View>: View")
        .next()
        .expect("carousel body");
    assert!(!carousel_runtime.contains("ScrollViewReader"));
    assert!(!carousel_runtime.contains("proxy.scrollTo"));
    assert!(carousel_runtime.contains("withAnimation { scrollId = slideIds[next] }"));
    for variant in [
        "coverFlow",
        "stories",
        "smartStack",
        "cardStack",
        "flipbook",
        "masonry",
        "rtl",
        "controls",
        "dots",
        "thumbnails",
    ] {
        assert!(views.contains(variant));
    }
    assert!(views.contains("DoweCheckboxView(checked:"));
    assert!(views.contains("DoweColorField(value:"));
    assert!(views.contains("DoweDateField(value:"));
    assert!(views.contains("DoweDateRangeField(startValue:"));
    assert!(views.contains("DoweColorField(value:") && views.contains("fontSize: doweTextSize(viewportWidth, min: CGFloat(12), preferredBase: CGFloat(11.2), preferredViewport: CGFloat(0.2), max: CGFloat(14))"));
    assert!(views.contains("DoweDateField(value:") && views.contains("fontSize: doweTextSize(viewportWidth, min: CGFloat(16), preferredBase: CGFloat(15.2), preferredViewport: CGFloat(0.3), max: CGFloat(18))"));
    assert!(views.contains("let fontSize: CGFloat"));
    assert!(views.contains("case \"sm\":\n        return CGFloat(32)"));
    assert!(
        views.contains("minHeight: doweControlHeight(size) + (floating ? CGFloat(8) : CGFloat(0))")
    );
    assert!(views.contains("DoweDateCalendar("));
    assert!(views.contains("DoweAnchoredPopoverPresenter("));
    assert!(views.contains("DoweRadioGroupView(value:"));
    assert!(views.contains("orientation: \"horizontal\""));
    assert!(views.contains("DoweToggleView(checked:"));
    assert!(views.contains("struct DoweSliderView: View"));
    assert!(views.contains("Image(systemName: \"checkmark\")"));
    assert!(views.contains("private struct DoweColorPickerPanel: View"));
    assert!(views.contains("doweColorFromHsv(hue, saturation, brightness)"));
    assert!(views.contains("doweColorCmykText(doweColorRgb(value))"));
    assert!(views.contains("doweColorOklchText(doweColorRgb(value))"));
    assert!(
        views
            .contains("DoweAnchoredPopoverPresenter(isPresented: expanded, minWidth: CGFloat(300)")
    );
    assert!(views.contains("trigger\n                    .allowsHitTesting(false)"));
    assert!(views.contains(".padding(.leading, doweControlSwatchSize(size) + CGFloat(10))"));
    assert!(
        views.contains("Button(action: { expanded.toggle() }) {\n                    Color.clear")
    );
    assert!(views.contains(".zIndex(expanded ? 1000 : 0)"));
    assert!(!views.contains("TextField(\"Start\", text: startValue)"));
    assert!(views.contains("DoweRadioOptionView(value:"));
    assert!(views.contains(".tint(accentColor)"));
    assert!(views.contains("func boolBinding(_ path: String) -> Binding<Bool>"));
    assert!(!views.contains("DoweSimpleField"));
}

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
    assert!(views.contains("DoweEditorField(value: state.binding(\"profile.notes\")"));
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

