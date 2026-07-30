fn dropzone_picker_route() -> ViewRoute {
    ViewRoute {
        id: "dropzone-picker".to_string(),
        route_path: "/dropzone-picker".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Dropzone {
            props: DropzoneProps {
                style: VariantProps {
                    label: Some("Assets".to_string()),
                    placeholder: Some("Choose files".to_string()),
                    variant: Some(ComponentVariant::Outlined),
                    color: Some(ColorFamily::Primary),
                    ..Default::default()
                },
                accept: Some("image/*".to_string()),
                multiple: true,
                max_size: Some(4096),
                size: ButtonSize::Md,
                name: Some("assets".to_string()),
                help_text: Some("Images only".to_string()),
                error_text: None,
                disabled: false,
            },
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

#[test]
fn generates_native_dropzone_file_picker_hooks() {
    let output = generate_ios(
        &[dropzone_picker_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains(".fileImporter("));
    assert!(views.contains("doweDropzoneFileTypes(accept)"));
    assert!(views.contains("allowsMultipleSelection: multiple"));
    assert!(views.contains("resourceValues(forKeys: [.nameKey, .fileSizeKey])"));
    assert!(views.contains("Int64(4096)"));
}

#[test]
fn generates_swiftui_display_overlay_components() {
    let output = generate_ios(
        &[display_overlay_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweAvatar<Icon: View>: View"));
    assert!(views.contains("DoweAvatar(source: nil, name: \"Ada\""));
    assert!(views.contains("DoweBadge(text: \"3\", position: \"bottom-right\""));
    assert!(views.contains(".alignmentGuide(.leading) { dimensions in"));
    assert!(views.contains("dimensions[HorizontalAlignment.center]"));
    assert!(views.contains(".alignmentGuide(.bottom) { dimensions in"));
    assert!(views.contains("dimensions[VerticalAlignment.center]"));
    assert!(views.contains("DoweChip(text: \"Filter\", size: \"sm\""));
    assert!(views.contains("DoweSkeleton(variant: \"rounded\", animation: \"pulse\")"));
    assert!(views.contains("private let pathBuilder: @Sendable (CGRect) -> Path"));
    assert!(views.contains("struct DoweWindowOverlayPresenter: UIViewRepresentable"));
    assert!(views.contains("DoweModal(open: state.bool(\"modal01\")"));
    let modal_start = views
        .find("DoweModal(open: state.bool(\"modal01\")")
        .expect("modal output");
    let modal_end = views[modal_start..]
        .find("DoweAlertDialog(open:")
        .map(|offset| modal_start + offset)
        .expect("alert dialog after modal");
    let modal_output = &views[modal_start..modal_end];
    assert!(modal_output.contains("} content: {"));
    assert!(!modal_output.contains("} content: { close in"));
    assert!(views.contains("DoweAlertDialog(open: state.bool(\"modal01\")"));
    assert!(views.contains(
        "backgroundColor: DoweDesign.surface, contentColor: DoweDesign.onSurface, dangerColor: DoweDesign.danger"
    ));
    assert!(views.contains("DoweTooltip(label: \"More actions\", position: \"end\""));
    assert!(!views.contains(".onTapGesture { open.toggle() }"));
    assert!(!views.contains("private var popoverPoint"));
    assert!(views.contains("DoweToast(visible: true, title: \"Saved\""));
    assert!(views.contains("DoweDropdown(backgroundColor: DoweDesign.surface"));
    let dropdown_start = views
        .find("DoweDropdown(backgroundColor: DoweDesign.surface")
        .expect("dropdown output");
    let dropdown_end = views[dropdown_start..]
        .find("DoweCommand(open:")
        .map(|offset| dropdown_start + offset)
        .expect("command after dropdown");
    let dropdown_output = &views[dropdown_start..dropdown_end];
    assert!(dropdown_output.contains("} content: { close in"));
    assert!(views.contains("action: { close(); navigate(\"push\", \"/docs\", nil) }"));
    assert!(views.contains(".allowsHitTesting(false)"));
    assert!(
        views.contains("struct DoweAnchoredPopoverPresenter<Content: View>: UIViewRepresentable")
    );
    assert!(views.contains("@MainActor final class Coordinator: NSObject"));
    assert!(views.contains("struct DoweDropdownPopover<Content: View>: View"));
    assert!(views.contains("DoweAnchoredPopoverPresenter("));
    assert!(views.contains(
        "ZStack {\n            trigger\n                .allowsHitTesting(false)\n        }\n        .background(\n            DoweAnchoredPopoverPresenter("
    ));
    assert!(views.contains(
        "            }\n        )\n        .overlay {\n            Button(action: { open.toggle() })"
    ));
    assert!(!views.contains(
        ".buttonStyle(.plain)\n            .background(\n                DoweAnchoredPopoverPresenter("
    ));
    assert!(views.contains("UIHostingController<Content>"));
    assert!(views.contains("private var containerView: UIView?"));
    assert!(views.contains("private var scrollView: UIScrollView?"));
    assert!(views.contains("context.coordinator.scheduleShow(from: uiView)"));
    let anchored_presenter_start = views
        .find("struct DoweAnchoredPopoverPresenter<Content: View>: UIViewRepresentable")
        .expect("anchored popover presenter");
    let anchored_presenter_end = views[anchored_presenter_start..]
        .find("struct DoweWindowOverlayPresenter: UIViewRepresentable")
        .map(|offset| anchored_presenter_start + offset)
        .expect("window overlay presenter after anchored popover presenter");
    let anchored_presenter_output = &views[anchored_presenter_start..anchored_presenter_end];
    let first_measurement = anchored_presenter_output
        .find("let measured = layout(for: anchor, controller: controller, in: window)")
        .expect("anchored popover measurement");
    let first_mount = anchored_presenter_output
        .find("window.addSubview(container)")
        .expect("anchored popover window mount");
    assert!(first_measurement < first_mount);
    assert!(
        !views[anchored_presenter_start..anchored_presenter_end]
            .contains("context.coordinator.show(from: uiView)")
    );
    assert!(
        !views[anchored_presenter_start..anchored_presenter_end]
            .contains("anchor.layoutIfNeeded()\n                DispatchQueue.main.async")
    );
    assert_eq!(
        views[anchored_presenter_start..anchored_presenter_end]
            .matches("presentationRevision += 1")
            .count(),
        2
    );
    assert!(views.contains("private var presentationRevision = 0"));
    assert!(views.contains("let revision = presentationRevision"));
    assert!(
        views.contains("guard revision == self.presentationRevision, self.parent.isPresented else")
    );
    assert!(views.contains("private weak var trackedAnchor: UIView?"));
    assert!(views.contains("private var anchorDisplayLink: CADisplayLink?"));
    assert!(
        views.contains("CADisplayLink(target: self, selector: #selector(refreshAnchorPosition))")
    );
    assert!(views.contains("displayLink.add(to: .main, forMode: .common)"));
    assert!(views.contains("@objc private func refreshAnchorPosition()"));
    assert!(views.contains("applyTrackedLayout(for: anchor, in: window, resetScroll: false)"));
    assert!(views.contains("anchorDisplayLink?.invalidate()"));
    assert!(views.contains("container.bounds = CGRect(origin: .zero, size: frame.size)"));
    assert!(views.contains("container.center = CGPoint(x: frame.midX, y: frame.midY)"));
    assert!(!views.contains("container.frame = measured.frame"));
    assert!(views.contains("window.layoutIfNeeded()"));
    assert!(views.contains("anchor.superview?.layoutIfNeeded()"));
    assert!(views.contains("anchor.layoutIfNeeded()"));
    assert!(views.contains("anchor.convert(anchor.bounds, to: window)"));
    assert!(!views.contains("let triggerSize: CGSize?"));
    assert!(views.contains("controller.sizeThatFits(in:"));
    assert!(views.contains("height: UIView.layoutFittingExpandedSize.height"));
    assert!(views.contains(
        "scroller.contentSize = CGSize(width: trackedWidth, height: trackedContentHeight)"
    ));
    assert!(views.contains("scroller.frame = container.bounds"));
    assert!(views.contains("container.layer.shadowOpacity = Float(0.12)"));
    assert!(views.contains("container.layer.shadowRadius = CGFloat(16)"));
    assert!(
        views.contains(
            "container.layer.shadowOffset = CGSize(width: CGFloat(0), height: CGFloat(8))"
        )
    );
    assert!(views.contains(
        "container.layer.shadowPath = UIBezierPath(roundedRect: container.bounds, cornerRadius: DoweDesign.radius).cgPath"
    ));
    assert!(views.contains("let below = anchorFrame.maxY + CGFloat(4)"));
    assert!(views.contains("anchorFrame.minY - height - CGFloat(4)"));
    assert!(views.contains("controller.view.frame = CGRect(x: CGFloat(0), y: CGFloat(0), width: trackedWidth, height: trackedContentHeight)"));
    assert!(views.contains("maxHeight: CGFloat(260)"));
    let popover_start = views
        .find("struct DoweDropdownPopover<Content: View>: View")
        .expect("dropdown popover runtime");
    let popover_end = views[popover_start..]
        .find("struct DoweOverlayItem<")
        .map(|offset| popover_start + offset)
        .expect("overlay item after dropdown popover");
    assert!(!views[popover_start..popover_end].contains("ScrollView"));
    assert!(!views[popover_start..popover_end].contains(".shadow("));
    assert!(!views.contains(".presentationCompactAdaptation(.popover)"));
    assert!(views.contains("DoweCommand(open: state.bool(\"modal01\")"));
}

#[test]
fn generates_swiftui_solar_icon_paints() {
    let fill = swift_svg_fill(SvgPathFill::Fill {
        color: Some(ColorToken::Secondary),
        opacity: 128,
        even_odd: true,
    });
    assert!(fill.contains("DoweSvgFill.fill(.some(DoweDesign.secondary)"));
    let stroke = swift_svg_fill(SvgPathFill::Stroke {
        color: Some(ColorToken::Tertiary),
        opacity: 255,
        width: 150,
        line_cap: SvgLineCap::Round,
        line_join: SvgLineJoin::Round,
    });
    assert!(stroke.contains("DoweSvgFill.stroke(.some(DoweDesign.tertiary)"));
    assert!(swift_runtime_svg_runtime().contains("StrokeStyle(lineWidth: width"));
}

#[test]
fn generates_swiftui_svg_logo_literal_paints() {
    let fill = swift_svg_fill(SvgPathFill::LiteralFill {
        red: 36,
        green: 41,
        blue: 47,
        opacity: 255,
        even_odd: false,
    });
    assert!(fill.contains("Color(red: 0.141, green: 0.161, blue: 0.184)"));
}

#[test]
fn generates_swiftui_svg_logo_paths() {
    let logo = icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String("svg-logos:github-icon".to_string()),
    }])
    .expect("SVG logo");
    let output = generate_ios(
        &[ViewRoute {
            id: "logo".to_string(),
            route_path: "/logo".to_string(),
            layout_tree: ViewNode::Children,
            page_tree: logo,
            sections: Vec::new(),
            navigation_actions: Vec::new(),
        }],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("DoweSvgFill.fill(.some(Color(red:"));
}

#[test]
fn generates_swiftui_display_chat_and_motion_components() {
    let output = generate_ios(
        &[display_chat_motion_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweAvatarGroup: View"));
    assert!(views.contains("return Array(items.prefix(max(1, maxCount)))"));
    assert!(!views.contains("maxCount - 1"));
    assert!(views.contains(
        "UIImage(named: source.trimmingCharacters(in: CharacterSet(charactersIn: \"/\")))"
    ));
    assert!(views.contains("DoweAvatarGroup(items: doweAvatarGroupItems(state.rows(\"people\")"));
    assert!(views.contains("DoweChatBox(state: state, messagesPath: \"messages\""));
    assert!(views.contains("DoweEmpty(kind: \"result\""));
    assert!(views.contains("DoweMarquee(speed: \"fast\""));
    assert!(views.contains(
        "withAnimation(.linear(duration: marqueeDuration).repeatForever(autoreverses: false))"
    ));
    assert!(!views.contains("Task.sleep(nanoseconds: 16_000_000)"));
    assert!(views.contains("DoweTypeWriter(texts: [\"Hello\", \"World\"]"));
}

#[test]
fn generates_swiftui_rich_control_map_components() {
    let output = generate_ios(
        &[rich_control_map_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweRichText: View"));
    assert!(views.contains("DoweRichText(marks: [DoweRichTextMark(text: \"Launch\""));
    assert!(views.contains("], font: .inter, fontSize:"));
    assert!(!views.contains("DoweRichText(marks: [DoweRichTextMark(text: \"Launch\", style: \"grad\", color: DoweDesign.primary), DoweRichTextMark(text: \"ready\", style: \"pill\", color: DoweDesign.success)], font: doweFont("));
    assert!(views.contains("DoweRecord(name: \"voice\""));
    assert!(views.contains("DoweToggleGroup(value: state.binding(\"mode\""));
    assert!(views.contains("DowePagination(value: state.binding(\"page\""));
    assert!(views.contains(
        "pageCount: max(1, min(25, (max(0, Int(state.text(\"total\")) ?? 0) + 59) / 60))"
    ));
    assert!(views.contains("previousIcon: {"));
    assert!(views.contains("nextIcon: {"));
    assert!(views.contains("DoweCollapsible(label: \"Details\""));
    assert!(views.contains("DoweCountdown(target: \"2030-01-01T00:00:00Z\""));
    assert!(views.contains("DoweMap(centerLat: \"4.7109\", centerLng: \"-74.0721\""));
    assert!(views.contains("DoweMapMarker(id: \"office\""));
}

#[test]
fn generates_full_scene_background_without_unsafe_content() {
    let output = generate_ios(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains(".background(DoweDesign.background.ignoresSafeArea())"));
    assert!(views.contains("struct DoweSafeAreaReporter: UIViewRepresentable"));
    assert!(views.contains("final class DoweSafeAreaReportingView: UIView"));
    assert!(views.contains("@State private var safeAreaInsets = EdgeInsets()"));
    assert!(views.contains("DoweSafeAreaReporter { insets in"));
    assert!(views.contains(
        "routeContent(currentEntry, viewportWidth: doweSafeAreaWidth(geometry, safeAreaInsets), viewportHeight: doweSafeAreaHeight(geometry, safeAreaInsets))"
    ));
    assert!(views.contains(
        ".frame(width: doweSafeAreaWidth(geometry, safeAreaInsets), height: doweSafeAreaHeight(geometry, safeAreaInsets), alignment: .topLeading)"
    ));
    assert!(views.contains(
        ".frame(width: doweSafeAreaWidth(geometry, safeAreaInsets), height: doweSafeAreaHeight(geometry, safeAreaInsets), alignment: .topLeading)\n                .clipped()\n                .offset(x: safeAreaInsets.leading, y: safeAreaInsets.top)"
    ));
    assert!(views.contains(".offset(x: safeAreaInsets.leading, y: safeAreaInsets.top)"));
    assert!(views.contains("        .ignoresSafeArea()\n        .frame(maxWidth: .infinity"));
    assert!(views.contains(
        "func doweSafeAreaWidth(_ geometry: GeometryProxy, _ insets: EdgeInsets) -> CGFloat"
    ));
    assert!(views.contains(
        "func doweSafeAreaHeight(_ geometry: GeometryProxy, _ insets: EdgeInsets) -> CGFloat"
    ));
    assert!(views.contains("func doweInsetsEqual(_ lhs: EdgeInsets, _ rhs: EdgeInsets) -> Bool"));
    assert!(views.contains(
        "private func routeContent(_ entry: DoweRouteEntry, viewportWidth: CGFloat, viewportHeight: CGFloat) -> some View"
    ));
    assert!(
        views.contains("LoginView(viewportWidth: viewportWidth, viewportHeight: viewportHeight, activeFragment: entry.fragment")
    );
    assert!(
        views.contains(".frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)")
    );
    assert!(!views.contains(
        "LoginView(viewportWidth: viewportWidth, viewportHeight: viewportHeight, activeFragment: entry.fragment, safeAreaInsets:"
    ));
    assert!(!views.contains("let safeAreaInsets ="));
    assert!(!views.contains(".padding(.top, safeAreaInsets.top)"));
    assert!(!views.contains(".padding(.bottom, safeAreaInsets.bottom)"));
    assert!(views.contains(
            "        .background(DoweDesign.background)\n        .foregroundStyle(DoweDesign.onBackground)"
        ));
    assert!(!views.contains(".ignoresSafeArea()\n        .foregroundStyle"));
}

#[test]
fn generates_portable_grid_controls_and_variant_colors() {
    let output = generate_ios(
        &[parity_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains(
            "DoweGridLayout(columns: doweResponsive(viewportWidth, xs: 1, md: 2) ?? 1, rowGap: doweResponsive(viewportWidth, xs: CGFloat(16)), columnGap: doweResponsive(viewportWidth, xs: CGFloat(24)), justify: nil, align: nil) {"
        ));
    assert!(views.contains("struct DoweGridLayout: Layout"));
    assert!(
        views.contains("DoweInputField(value: nil, label: nil, placeholder: \"\", floating: false")
    );
    assert!(views.contains("minHeight: CGFloat(40), horizontalPadding: CGFloat(12)"));
    assert!(views.contains(
            "backgroundColor: Color.clear, contentColor: DoweDesign.secondary, borderColor: Optional(DoweDesign.muted)"
        ));
    assert!(views.contains(".foregroundStyle(DoweDesign.onSoftMuted)"));
    assert!(views.contains(".background(DoweDesign.surface)"));
    assert!(views.contains(".foregroundStyle(DoweDesign.onSurface)"));
    assert!(views.contains(".stroke(DoweDesign.surface, lineWidth: CGFloat(1))"));
    assert!(views.contains("HStack(spacing: 8)"));
    assert!(views.contains("DoweSvgView(viewBox: DoweSvgViewBox(minX: CGFloat(0), minY: CGFloat(0), width: CGFloat(24), height: CGFloat(24)), color: DoweDesign.primary"));
    assert!(!views.contains("doweButtonHorizontalPadding(\"md\")"));
    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageParityView.swift"))
        .expect("parity page");
    let action = page
        .content
        .find("Text(verbatim: \"Action\")")
        .expect("button label");
    let button_tail = &page.content[action..];
    let frame = button_tail
        .find(".frame(maxWidth: .infinity, alignment: .center)")
        .expect("grid button frame");
    let background = button_tail
        .find(".background(Color.clear)")
        .expect("outlined button background");
    assert!(frame < background);
    assert!(button_tail.contains(".foregroundStyle(DoweDesign.primary)"));
}

#[test]
fn preserves_static_button_variants_with_reactive_scheme_on_ios() {
    let mut route = route();
    route.layout_tree = ViewNode::Children;
    route.page_tree = ViewNode::Scope {
        constants: Vec::new(),
        signals: vec![ViewSignal {
            id: "scheme01".to_string(),
            name: "schemeChoice".to_string(),
            storage_key: "schemeChoice".to_string(),
            scope: dowe_components::ViewSignalScope::Page,
            storage: dowe_components::ViewSignalStorage::None,
            initial: ViewSignalValue::String("primary".to_string()),
            schema: None,
        }],
        actions: Vec::new(),
        children: [
            ComponentVariant::Solid,
            ComponentVariant::Soft,
            ComponentVariant::Outlined,
            ComponentVariant::Ghost,
        ]
        .into_iter()
        .map(|variant| ViewNode::Button {
            props: VariantProps {
                variant: Some(variant),
                reactive: ReactiveVariantProps {
                    scheme: Some("schemeChoice".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Action")],
        })
        .collect(),
    };
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let generated = output
        .files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<String>();

    assert!(generated.contains("doweButtonContainer(\"solid\", state.text("));
    assert!(generated.contains("doweButtonContainer(\"soft\", state.text("));
    assert!(generated.contains("doweButtonContainer(\"outlined\", state.text("));
    assert!(generated.contains("doweButtonContainer(\"ghost\", state.text("));
    assert!(generated.contains("lineWidth: \"outlined\" == \"outlined\" ? CGFloat(1)"));
}

#[test]
fn generates_swiftui_ghost_card_without_variant_border() {
    let output = generate_ios(
        &[ViewRoute {
            id: "cards".to_string(),
            route_path: "/cards".to_string(),
            layout_tree: ViewNode::Children,
            page_tree: ViewNode::Box {
                props: Default::default(),
                children: vec![
                    ViewNode::Card {
                        props: VariantProps {
                            variant: Some(ComponentVariant::Outlined),
                            color: Some(ColorFamily::Info),
                            ..Default::default()
                        },
                        children: vec![text("Outlined")],
                    },
                    ViewNode::Card {
                        props: VariantProps {
                            variant: Some(ComponentVariant::Ghost),
                            color: Some(ColorFamily::Warning),
                            ..Default::default()
                        },
                        children: vec![text("Ghost")],
                    },
                ],
            },
            sections: Vec::new(),
            navigation_actions: Vec::new(),
        }],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let page = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageCardsView.swift"))
        .expect("cards page");
    let outlined = page
        .content
        .find("Text(verbatim: \"Outlined\")")
        .expect("outlined card");
    let ghost = page
        .content
        .find("Text(verbatim: \"Ghost\")")
        .expect("ghost card");
    let outlined_card = &page.content[outlined..ghost];
    let ghost_card = &page.content[ghost..];

    assert!(outlined_card.contains(".stroke(DoweDesign.info, lineWidth: CGFloat(1))"));
    assert!(ghost_card.contains(".background(Color.clear)"));
    assert!(ghost_card.contains(".foregroundStyle(DoweDesign.warning)"));
    assert!(!ghost_card.contains(".stroke("));
}

#[test]
fn generates_labeled_input_and_select_fields() {
    let output = generate_ios(
        &[form_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweInputField: View"));
    assert!(views.contains("let startIcon: DoweControlIcon?\n"));
    assert!(views.contains("let endIcon: DoweControlIcon?\n"));
    let input = views
        .lines()
        .find(|line| line.contains("DoweInputField(value: nil"))
        .expect("generated input field");
    assert!(input.contains("borderWidth: CGFloat(1)"), "{input}");
    assert!(views.contains(
        r#"DoweInputField(value: nil, label: "Name", placeholder: "Full name", floating: true"#
    ));
    assert!(views.contains("let value: Binding<String>?"));
    assert!(views.contains("@State private var localText = \"\""));
    assert!(views.contains("private var visiblePlaceholder: String"));
    assert!(views.contains("TextField(visiblePlaceholder, text: textBinding)"));
    assert!(views.contains("struct DoweSelectField: View"));
    assert!(views.contains("struct DoweSelectPopover: View"));
    assert!(views.contains("struct DoweSelectAnchorPresenter: View"));
    assert!(
        views.contains("struct DoweAnchoredPopoverPresenter<Content: View>: UIViewRepresentable")
    );
    assert!(views.contains("UIHostingController<Content>"));
    assert!(views.contains("UIView.animate(withDuration: 0.16"));
    assert!(views.contains("UIView.animate(withDuration: 0.14"));
    assert!(views.contains("anchor.convert(anchor.bounds, to: window)"));
    assert!(views.contains("window.addSubview(container)"));
    assert!(views.contains("container.addSubview(scroller)"));
    assert!(views.contains("scroller.addSubview(controller.view)"));
    assert!(views.contains("private var preferredHeight: CGFloat"));
    assert!(views.contains("CGFloat(8) + options.reduce(CGFloat(0))"));
    assert!(views.contains("option.description == nil ? CGFloat(40) : CGFloat(58)"));
    assert!(views.contains("let measuredHeight = parent.preferredHeight ?? measuredSize.height"));
    assert!(views.contains("let contentHeight = max(CGFloat(44), measuredHeight)"));
    assert!(views.contains("let height = min(heightLimit, contentHeight)"));
    let select_popover_start = views
        .find("struct DoweSelectPopover: View")
        .expect("select popover runtime");
    let select_popover_end = views[select_popover_start..]
        .find("struct DoweSelectArrow: View")
        .map(|offset| select_popover_start + offset)
        .expect("select arrow after select popover");
    assert!(!views[select_popover_start..select_popover_end].contains(".shadow("));
    assert!(!views.contains("max(size.height, estimated)"));
    assert!(views.contains("@State private var expanded = false"));
    assert!(views.contains("DoweSelectAnchorPresenter("));
    assert!(views.contains(".contentShape(Rectangle())"));
    assert!(views.contains(".zIndex(expanded ? 1000 : 0)"));
    assert!(!views.contains("Menu {"));
    assert!(!views.contains("Picker("));
    assert!(!views.contains("DoweSelectPortalOverlay"));
    assert!(views.contains(
        r#"DoweSelectField(value: nil, label: "Role", placeholder: "Choose role", floating: true"#
    ));
    assert!(views.contains(
        r#"DoweSelectOption(value: "admin", label: "Admin", description: "Manages users")"#
    ));
    assert!(views.contains("DoweSelectArrow(color: contentColor)"));
    assert!(views.contains("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4"));
    assert!(views.contains("if selectedOption != nil || !floating || expanded"));
    assert!(views.contains("Text(description).font(.caption)"));
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
    assert!(views.contains("DoweCarouselView(variant: \"snapping\""));
    assert!(views.contains("ScrollView(.horizontal"));
    assert!(views.contains("showsIndicators: false"));
    assert!(views.contains(".scrollPosition(id: $scrollId)"));
    assert!(views.contains(".onChange(of: scrollId) { _, value in"));
    assert!(!views.contains(".onChange(of: scrollId) { value in"));
    assert!(views.contains(".scrollTransition(.interactive, axis: .horizontal)"));
    assert!(views.contains("rotation3DEffect"));
    assert!(views.contains(
        "nonisolated private func carouselRotation(_ phase: Double) -> Double"
    ));
    assert!(views.contains(
        "nonisolated private func carouselScale(_ phase: Double) -> CGFloat"
    ));
    assert!(views.contains(
        "nonisolated private func carouselTilt(_ phase: Double) -> Double"
    ));
    assert!(views.contains(
        "nonisolated private func carouselOffset(_ phase: Double) -> CGFloat"
    ));
    assert!(views.contains(
        "nonisolated private func carouselOpacity(_ phase: Double) -> Double"
    ));
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
    assert!(views.contains("DoweDateCalendar("));
    assert!(views.contains("DoweAnchoredPopoverPresenter("));
    assert!(views.contains("DoweRadioGroupView(value:"));
    assert!(views.contains("orientation: \"horizontal\""));
    assert!(views.contains("DoweToggleView(checked:"));
    assert!(views.contains("struct DoweSliderView: View"));
    assert!(views.contains("Image(systemName: \"checkmark\")"));
    assert!(views.contains("doweColorFromHex(value.wrappedValue"));
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
        .find(|file| file.content.contains("DowePhoneField(value:"))
        .expect("phone field page");
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
    assert!(views.contains("DoweSelectOption(value: \"admin\", label: \"Admin\""));
    assert!(views.contains("struct DoweCsvColumn: Identifiable"));
    assert!(views.contains("DoweCsvField(label: \"Import\""));
    assert!(views.contains("DoweCsvColumn(name: \"email\", label: \"Email\")"));
    assert!(views.contains("struct DoweDragGroup: Identifiable"));
    assert!(views.contains("DoweDragDrop(label: \"Tasks\""));
    assert!(views.contains("DoweDragItem(id: \"draft\", label: \"Draft\""));
    assert!(views.contains("DoweEditorField(value: state.binding(\"profile.notes\")"));
    assert!(views.contains("DoweImageCropper(value: state.binding(\"profile.avatar\")"));
    assert!(views.contains("DowePasswordField(value: state.binding(\"profile.password\")"));
    assert!(views.contains("private var strengthColor: Color"));
    assert!(views.contains("DoweDesign.danger"));
    assert!(views.contains("DoweDesign.warning"));
    assert!(views.contains("DoweDesign.success"));
    assert!(
        views.contains(".frame(maxWidth: .infinity, minHeight: CGFloat(4), maxHeight: CGFloat(4))")
    );
    let password_field = views
        .split("struct DowePasswordField: View")
        .nth(1)
        .expect("password field runtime");
    assert!(password_field.contains("private var visiblePlaceholder: String"));
    assert!(password_field.contains("TextField(visiblePlaceholder, text: textBinding)"));
    assert!(password_field.contains("SecureField(visiblePlaceholder, text: textBinding)"));
    assert!(views.contains("DowePhoneField(value: state.binding(\"profile.phone\")"));
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
        .split("struct DowePhoneField: View")
        .nth(1)
        .expect("phone field runtime")
        .split("struct DowePinField: View")
        .next()
        .expect("phone field body");
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
    assert!(phone.contains("filter { $0.isNumber }"));
    assert!(phone.contains(".keyboardType(.numberPad)"));
    assert!(phone.contains("DoweSvgView(viewBox: selectedCountry.flag.viewBox"));
    assert!(!phone.contains(".sheet(isPresented:"));
    assert!(!phone.contains("NavigationStack"));
    assert!(!phone.contains("List(filteredCountries)"));
    assert!(!phone.contains(".searchable(text:"));
    assert!(views.contains("DowePinField(value: state.binding(\"profile.pin\")"));
    let pin = views
        .split("struct DowePinField: View")
        .nth(1)
        .expect("pin field runtime");
    assert!(pin.contains("@FocusState private var focusedCell: Int?"));
    assert!(pin.contains(".focused($focusedCell, equals: index)"));
    assert!(pin.contains(".onChange(of: cellValue(at: index))"));
    assert!(pin.contains("focusedCell = index + 1"));
    assert!(pin.contains("SecureField(\"\", text: binding(for: index))"));
    assert!(views.contains("DoweTextarea(value: state.binding(\"profile.bio\")"));
    let textarea = views
        .split("struct DoweTextarea: View")
        .nth(1)
        .expect("textarea runtime");
    assert!(textarea.contains("@FocusState private var focused: Bool"));
    assert!(textarea.contains("private var visiblePlaceholder: Bool"));
    assert!(textarea.contains("!floating || focused"));
    assert!(textarea.contains("if visiblePlaceholder"));
    assert!(textarea.contains(".focused($focused)"));
}

#[test]
fn generates_fragment_aware_native_history_and_deep_links() {
    let output = generate_ios(
        &[index_route_with_navigation(), signup_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    let routing = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweRouting.swift"))
        .expect("routing");

    assert!(views.contains("struct DoweRouteEntry: Hashable"));
    assert!(views.contains("@State private var rootEntry: DoweRouteEntry"));
    assert!(views.contains("_rootEntry = State(initialValue: DoweRouteEntry"));
    assert!(views.contains("@State private var navigationPath: [DoweRouteEntry] = []"));
    assert!(views.contains(
        "routeContent(currentEntry, viewportWidth: doweSafeAreaWidth(geometry, safeAreaInsets), viewportHeight: doweSafeAreaHeight(geometry, safeAreaInsets))"
    ));
    assert!(views.contains(".simultaneousGesture(backSwipeGesture)"));
    assert!(!views.contains("NavigationStack(path: $navigationPath)"));
    assert!(!views.contains(".navigationDestination(for: DoweRouteEntry.self)"));
    assert!(!views.contains(".toolbar(.hidden, for: .navigationBar)"));
    assert!(!views.contains(".navigationBarHidden(true)"));
    assert!(views.contains("private var backSwipeGesture: some Gesture"));
    assert!(views.contains("navigationPath.append(destination)"));
    assert!(views.contains("navigationPath.removeLast()"));
    assert!(views.contains(
        "private func navigate(_ operation: String, _ target: String, _ fragment: String?)"
    ));
    assert!(views.contains(r#"{ navigate("push", "/signup", "join") }"#));
    assert!(views.contains(r#"{ navigate("replace", "", "hero") }"#));
    assert!(views.contains("{ goBack() }"));
    assert!(views.contains(r#"navigate("replace", path, url.fragment)"#));
    assert!(views.contains("ScrollViewReader { proxy in"));
    assert!(views.contains("doweScroll(proxy, activeFragment)"));
    assert!(views.contains("proxy.scrollTo(\"__dowe_page_top\", anchor: .top)"));
    assert!(views.contains("routeSection0()\n                    .id(\"__dowe_page_top\")"));
    assert!(
        views.contains(".onChange(of: activeFragment) { _, value in doweScroll(proxy, value) }")
    );
    assert!(!views.contains(".onChange(of: activeFragment) { value in doweScroll(proxy, value) }"));
    assert!(views.contains(".id(\"hero\")"));
    assert!(
        routing
            .content
            .contains("static let sections: [String: [String]]")
    );
    assert!(routing.content.contains(r#""/signup": ["join"]"#));
}

fn advanced_form_route() -> ViewRoute {
    ViewRoute {
        id: "advanced".to_string(),
        route_path: "/advanced".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: advanced_form_tree(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn advanced_form_tree() -> ViewNode {
    ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::ComboBox {
                props: ComboBoxProps {
                    style: bound_style("profile.role", "Role", "Choose role"),
                    value: Some("editor".to_string()),
                    search_placeholder: "Search roles".to_string(),
                    empty_text: "No roles".to_string(),
                    loading_text: "Loading".to_string(),
                    loading_more_text: "Loading more".to_string(),
                    clearable: true,
                    disabled: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
                options: vec![ComboOption {
                    value: "admin".to_string(),
                    label: "Admin".to_string(),
                    description: Some("Full access".to_string()),
                    src: None,
                    icon: None,
                    disabled: false,
                }],
            },
            ViewNode::CsvField {
                props: CsvFieldProps {
                    style: advanced_style("Import", None, ComponentVariant::Outlined),
                    button_text: "Upload CSV".to_string(),
                    modal_title: "Review import".to_string(),
                    instructions: "Columns are checked".to_string(),
                    cancel_text: "Cancel".to_string(),
                    confirm_text: "Import".to_string(),
                    clear_text: "Clear".to_string(),
                    preview_title: "Preview".to_string(),
                    multiple: false,
                    show_preview: true,
                    preview_rows: 3,
                    preview_page_size: 10,
                    error_text: None,
                },
                columns: vec![CsvColumn {
                    name: "email".to_string(),
                    label: Some("Email".to_string()),
                }],
            },
            ViewNode::DragDrop {
                props: DragDropProps {
                    style: advanced_style("Tasks", None, ComponentVariant::Soft),
                    empty_text: "No tasks".to_string(),
                    direction: DragDropDirection::Horizontal,
                    allow_group_transfer: true,
                    disabled: false,
                    size: ButtonSize::Md,
                },
                items: Vec::new(),
                groups: vec![DragGroup {
                    id: "todo".to_string(),
                    title: Some("Todo".to_string()),
                    items: vec![DragItem {
                        id: "draft".to_string(),
                        label: Some("Draft".to_string()),
                        description: Some("Prepare".to_string()),
                        disabled: false,
                    }],
                }],
            },
            ViewNode::Editor {
                props: EditorProps {
                    style: bound_style("profile.notes", "Notes", "Write notes"),
                    value: None,
                    min_height: 180,
                    hide_toolbar: false,
                    disabled: false,
                    readonly: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::ImageCropper {
                props: ImageCropperProps {
                    style: bound_style("profile.avatar", "Avatar", "Upload avatar"),
                    src: None,
                    alt: "Avatar".to_string(),
                    accept: "image/*".to_string(),
                    aspect_ratio: None,
                    min_width: 128,
                    min_height: 128,
                    max_width: None,
                    max_height: None,
                    shape: ImageCropperShape::Circle,
                    disabled: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::PasswordField {
                props: PasswordFieldProps {
                    style: bound_style("profile.password", "Password", "Create password"),
                    value: None,
                    hide_strength: false,
                    weak_label: "Weak".to_string(),
                    medium_label: "Medium".to_string(),
                    strong_label: "Strong".to_string(),
                    disabled: false,
                    readonly: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::PhoneField {
                props: PhoneFieldProps {
                    style: bound_style("profile.phone", "Phone", "Phone number"),
                    value: None,
                    country: Some("US".to_string()),
                    dial_code_name: "dialCode".to_string(),
                    search_placeholder: "Search countries".to_string(),
                    empty_text: "No countries".to_string(),
                    loading_text: "Loading".to_string(),
                    priority_countries: vec!["US".to_string()],
                    disabled: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::PinField {
                props: PinFieldProps {
                    style: bound_style("profile.pin", "Code", ""),
                    value: None,
                    length: 6,
                    kind: PinFieldKind::Number,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
            ViewNode::Textarea {
                props: TextareaProps {
                    style: bound_style("profile.bio", "Bio", "Short bio"),
                    value: None,
                    rows: 4,
                    cols: None,
                    max_length: Some(160),
                    resize: true,
                    disabled: false,
                    readonly: false,
                    name: None,
                    help_text: None,
                    error_text: None,
                },
            },
        ],
    }
}

fn bound_style(bind: &str, label: &str, placeholder: &str) -> VariantProps {
    let mut style = advanced_style(label, Some(placeholder), ComponentVariant::Outlined);
    style.element.bind = Some(bind.to_string());
    style.label_floating = true;
    style
}

fn advanced_style(
    label: &str,
    placeholder: Option<&str>,
    variant: ComponentVariant,
) -> VariantProps {
    VariantProps {
        label: Some(label.to_string()),
        placeholder: placeholder.map(str::to_string),
        variant: Some(variant),
        color: Some(ColorFamily::Primary),
        ..Default::default()
    }
}

#[test]
fn generates_swiftui_svg_views() {
    let output = generate_ios(
        &[svg_route(), runtime_svg_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweSvgView: View"));
    assert!(views.contains(
        "DoweRuntimeSvgView(payload: state.json(\"iconData01\")"
    ));
    assert!(views.contains("private enum DoweRuntimeSvgParser"));
    assert!(views.contains("DoweSvgViewBox(minX: CGFloat(0), minY: CGFloat(0), width: CGFloat(24), height: CGFloat(24))"));
    assert!(views.contains("DoweSvgFill.currentColor"));
    assert!(views.contains(
        "doweResponsive(viewportWidth, xs: DoweDesign.tertiary) ?? DoweDesign.onBackground"
    ));
    assert!(views.contains("private final class DoweSvgPathCache: @unchecked Sendable"));
    assert!(views.contains("DoweSvgPathCache.shared.path(for: data)"));
    assert!(views.contains("storage.countLimit = 2048"));
    assert!(views.contains("transform: CGAffineTransform(a: 2, b: 0, c: 0, d: 2, tx: 4, ty: 6)"));
    assert!(views.contains("private final class DoweSvgImporter: NSObject, XMLParserDelegate"));
    assert!(views.contains("case \"parse.svg\": return DoweSvgImporter.convert(text(\"value\"))"));
    assert!(views.contains("c.253.847.1 1.895-.62 2.618a.75.75"));
    assert!(views.contains("if characters[index] == \"-\" || characters[index] == \"+\""));
    assert!(views.contains(
            ".frame(width: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))))"
        ));
    assert!(views.contains(
            ".frame(maxWidth: doweMaxSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))))"
        ));
    assert!(views.contains(
            ".frame(height: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32))), viewportHeight: viewportHeight))"
        ));
    assert!(views.contains(
            ".frame(maxHeight: doweMaxSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))))"
        ));
    assert!(!views.contains(", maxWidth: doweMaxSize("));
    assert!(!views.contains(", maxHeight: doweMaxSize("));
}

#[test]
fn generates_animated_svg_spinner_views() {
    let spinner = icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String("svg-spinners:3-dots-bounce".to_string()),
    }])
    .expect("spinner");
    let route = ViewRoute {
        id: "spinner".to_string(),
        route_path: "/spinner".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: spinner,
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("animated: true"));
    assert!(views.contains("@Environment(\\.accessibilityReduceMotion)"));
    assert!(views.contains("TimelineView(.animation)"));
}

#[test]
fn generates_loading_button_with_animated_spinner_and_disabled_state() {
    let route = ViewRoute {
        id: "loading-button".to_string(),
        route_path: "/loading-button".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Button {
            props: VariantProps {
                loading_icon: Some(svg_spinner_control_icon("3-dots-move").expect("button spinner")),
                reactive: ReactiveVariantProps {
                    loading: Some("saving".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![text("Save")],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains(".disabled(state.bool(\"saving\", fallback: true))"));
    assert!(views.contains("if state.bool(\"saving\", fallback: true)"));
    assert!(views.contains("animated: true"));
}

#[test]
fn generates_full_hit_targets_for_icon_and_text_buttons() {
    let route = ViewRoute {
        id: "button-hit-targets".to_string(),
        route_path: "/button-hit-targets".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Button {
                    props: VariantProps {
                        style: StyleProps {
                            sizing: SizingProps {
                                w: Some(ResponsiveValue::scalar(SizeValue::Scale(
                                    ScaleValue::from_half_steps(20),
                                ))),
                                h: Some(ResponsiveValue::scalar(SizeValue::Scale(
                                    ScaleValue::from_half_steps(20),
                                ))),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        icon_start: Some(
                            solar_control_icon("settings").expect("settings icon"),
                        ),
                        icon_only: true,
                        label: Some("Open settings".to_string()),
                        navigation: Some(NavigationAction::Internal {
                            path: "/settings".to_string(),
                            fragment: None,
                            operation: NavigationOperation::Push,
                        }),
                        ..Default::default()
                    },
                    children: Vec::new(),
                },
                ViewNode::Button {
                    props: VariantProps {
                        style: StyleProps {
                            spacing: SpacingProps {
                                px: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(8))),
                                py: Some(ResponsiveValue::scalar(ScaleValue::from_half_steps(5))),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        navigation: Some(NavigationAction::Internal {
                            path: "/save".to_string(),
                            fragment: None,
                            operation: NavigationOperation::Push,
                        }),
                        ..Default::default()
                    },
                    children: vec![text("Save")],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    let icon_start = views
        .find("Button(action: { navigate(\"push\", \"/settings\", nil) })")
        .expect("icon button");
    let text_start = views
        .find("Button(action: { navigate(\"push\", \"/save\", nil) })")
        .expect("text button");
    let icon_output = &views[icon_start..text_start];
    let text_output = &views[text_start..];
    let icon_width = icon_output.find(".frame(width:").expect("icon width");
    let icon_height = icon_output.find(".frame(height:").expect("icon height");
    let icon_hit_target = icon_output
        .find(".contentShape(Rectangle())")
        .expect("icon hit target");
    let icon_background = icon_output.find(".background(").expect("icon background");
    assert!(icon_width < icon_hit_target);
    assert!(icon_height < icon_hit_target);
    assert!(icon_hit_target < icon_background);
    assert!(icon_output.contains(".accessibilityLabel(Text(\"Open settings\"))"));
    let text_padding = text_output
        .find(".padding(EdgeInsets(")
        .expect("text button padding");
    let text_hit_target = text_output
        .find(".contentShape(Rectangle())")
        .expect("text button hit target");
    let text_background = text_output.find(".background(").expect("text button background");
    assert!(text_padding < text_hit_target);
    assert!(text_hit_target < text_background);
}

#[test]
fn generates_swiftui_viewport_minus_height() {
    let route = ViewRoute {
        id: "viewport".to_string(),
        route_path: "/viewport".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps {
                sizing: dowe_components::SizingProps {
                    h: Some(ResponsiveValue::scalar(
                        dowe_components::SizeValue::ViewportMinus(ScaleValue::from_half_steps(32)),
                    )),
                    min_h: Some(ResponsiveValue::scalar(
                        dowe_components::SizeValue::ViewportMinus(ScaleValue::from_half_steps(40)),
                    )),
                    max_w: Some(ResponsiveValue::scalar(
                        dowe_components::SizeValue::Scale(ScaleValue::from_half_steps(128)),
                    )),
                    max_h: Some(ResponsiveValue::scalar(
                        dowe_components::SizeValue::ViewportMinus(ScaleValue::from_half_steps(48)),
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_ios(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("routeContent(currentEntry, viewportWidth: doweSafeAreaWidth(geometry, safeAreaInsets), viewportHeight: doweSafeAreaHeight(geometry, safeAreaInsets))"));
    assert!(views.contains("DoweSize.viewportMinus(CGFloat(64))"));
    assert!(views.contains("DoweSize.viewportMinus(CGFloat(80))"));
    assert!(views.contains(
        ".frame(height: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.viewportMinus(CGFloat(64))), viewportHeight: viewportHeight))"
    ));
    assert!(views.contains(
        ".frame(minHeight: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.viewportMinus(CGFloat(80))), viewportHeight: viewportHeight))"
    ));
    assert!(views.contains(
        ".frame(maxWidth: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(256)))))"
    ));
    assert!(views.contains(
        ".frame(maxHeight: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.viewportMinus(CGFloat(96))), viewportHeight: viewportHeight))"
    ));
}

#[test]
fn generates_swiftui_view_motion() {
    let output = generate_ios(
        &[motion_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("enum DoweAnimationPreset"));
    assert!(views.contains(".modifier(DoweAnimationModifier(preset: .fadeIn))"));
    assert!(views.contains(".modifier(DoweAnimationModifier(preset: .slideUp))"));
    assert!(views.contains(".animation(.easeOut(duration: 0.22), value: active)"));
}
