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

fn overlay_parity_route() -> ViewRoute {
    ViewRoute {
        id: "overlay-parity".to_string(),
        route_path: "/overlay-parity".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps::default(),
            children: vec![
                ViewNode::Modal {
                    props: ModalProps {
                        style: VariantProps {
                            variant: Some(ComponentVariant::Outlined),
                            color: Some(ColorFamily::Warning),
                            ..Default::default()
                        },
                        open: "modal01".to_string(),
                        on_close: None,
                        disable_overlay_close: false,
                        hide_close_button: false,
                    },
                    header: vec![text("Settings")],
                    body: vec![text("Body")],
                    footer: Vec::new(),
                },
                ViewNode::AlertDialog {
                    props: AlertDialogProps {
                        style: VariantProps {
                            variant: Some(ComponentVariant::Soft),
                            color: Some(ColorFamily::Warning),
                            ..Default::default()
                        },
                        open: "alert01".to_string(),
                        title: "Archive?".to_string(),
                        description: "Archive this project.".to_string(),
                        confirm_text: "Archive".to_string(),
                        cancel_text: "Cancel".to_string(),
                        on_confirm: None,
                        on_cancel: None,
                        loading: false,
                    },
                },
            ],
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
    assert!(views.contains("hasStart: true, hasEnd: true"));
    assert!(views.contains("DoweSvgView(viewBox:"));
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
        "backgroundColor: DoweDesign.surface, contentColor: DoweDesign.surfaceText, borderColor: nil, confirmBackgroundColor: DoweDesign.danger, confirmContentColor: DoweDesign.dangerText"
    ));
    assert!(views.contains("DoweTooltip(label: \"More actions\", position: \"end\""));
    assert!(!views.contains(".onTapGesture { open.toggle() }"));
    assert!(!views.contains("private var popoverPoint"));
    assert!(views.contains("DoweToast(visible: true, title: \"Saved\""));
    assert!(views.contains(
        "position: \"top-right\", backgroundColor: DoweDesign.surface, contentColor: DoweDesign.surfaceText, borderColor: Optional(DoweDesign.warning)"
    ));
    assert!(views.contains("struct DoweToastOverlayPresenter<Content: View>: UIViewRepresentable"));
    let toast_presenter_start = views
        .find("struct DoweToastOverlayPresenter<Content: View>: UIViewRepresentable")
        .expect("toast overlay presenter");
    let toast_presenter_end = views[toast_presenter_start..]
        .find("struct DoweModal<")
        .map(|offset| toast_presenter_start + offset)
        .expect("modal after toast overlay presenter");
    let toast_presenter_output = &views[toast_presenter_start..toast_presenter_end];
    assert!(toast_presenter_output.contains("UIHostingController<Content>"));
    assert!(toast_presenter_output.contains("private var containerView: UIView?"));
    assert!(toast_presenter_output.contains("context.coordinator.scheduleShow(from: uiView)"));
    assert!(toast_presenter_output.contains("private var presentationRevision = 0"));
    assert!(toast_presenter_output.contains("private var showScheduled = false"));
    assert!(toast_presenter_output.contains("private var isDismissing = false"));
    assert!(toast_presenter_output.contains("guard !showScheduled else"));
    assert!(toast_presenter_output.contains(
        "if containerView?.superview != nil {\n                showScheduled = false\n                isDismissing = false"
    ));
    assert_eq!(
        toast_presenter_output
            .matches("presentationRevision += 1")
            .count(),
        2
    );
    assert!(toast_presenter_output.contains("guard revision == self.presentationRevision else"));
    assert!(toast_presenter_output.contains(
        "self.showScheduled = false\n                guard self.parent.isPresented else"
    ));
    let toast_dismiss_start = toast_presenter_output
        .find("func dismiss(immediate: Bool = false)")
        .expect("toast dismissal");
    let toast_dismiss_output = &toast_presenter_output[toast_dismiss_start..];
    let dismiss_idempotence_guard = toast_dismiss_output
        .find("guard immediate || !isDismissing else")
        .expect("toast dismissal idempotence guard");
    let dismiss_revision = toast_dismiss_output
        .find("presentationRevision += 1")
        .expect("toast dismissal revision");
    assert!(dismiss_idempotence_guard < dismiss_revision);
    assert!(
        toast_dismiss_output
            .contains("guard immediate || showScheduled || containerView?.superview != nil else")
    );
    assert!(toast_dismiss_output.contains(
        "guard revision == self.presentationRevision, !self.parent.isPresented, self.isDismissing else"
    ));
    assert!(toast_dismiss_output.contains("self.isDismissing = false"));
    assert!(toast_dismiss_output.contains("container.removeFromSuperview()"));
    assert!(toast_presenter_output.contains("height: UIView.layoutFittingExpandedSize.height"));
    assert!(
        toast_presenter_output
            .contains("container.bounds = CGRect(origin: .zero, size: frame.size)")
    );
    assert!(
        toast_presenter_output.contains("container.center = CGPoint(x: frame.midX, y: frame.midY)")
    );
    let toast_measurement = toast_presenter_output
        .find("let measured = controller.sizeThatFits(")
        .expect("toast measurement");
    let toast_mount = toast_presenter_output
        .find("window.addSubview(container)")
        .expect("toast compact container mount");
    assert!(toast_measurement < toast_mount);
    assert!(!toast_presenter_output.contains("DoweToastOverlayHostView"));
    assert!(!toast_presenter_output.contains("point(inside"));
    assert!(!toast_presenter_output.contains("interactionFrame"));
    assert!(!toast_presenter_output.contains("backdrop"));
    assert!(!toast_presenter_output.contains("UIControl"));
    assert!(views.contains(
        "DoweToastOverlayPresenter(isPresented: visible && !dismissed, position: position)"
    ));
    assert!(views.contains("DoweOverlayCloseIcon(color: DoweDesign.mutedText)"));
    assert!(views.contains(".accessibilityLabel(\"Close toast\")"));
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
    assert!(
        !views[popover_start..popover_end].contains(".presentationCompactAdaptation(.popover)")
    );
    assert!(views.contains("DoweCommand(open: state.bool(\"modal01\")"));
}

#[test]
fn generates_ios_overlay_surface_action_and_close_parity() {
    let output = generate_ios(
        &[overlay_parity_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);
    assert!(views.contains(
        "backgroundColor: DoweDesign.surface, contentColor: DoweDesign.surfaceText, borderColor: Optional(DoweDesign.warning)"
    ));
    assert!(views.contains(
        "backgroundColor: DoweDesign.surface, contentColor: DoweDesign.surfaceText, borderColor: nil"
    ));
    assert!(views.contains(
        "confirmBackgroundColor: DoweDesign.warning, confirmContentColor: DoweDesign.warningText"
    ));
    assert!(views.contains("struct DoweOverlayCloseIcon: View"));
    assert!(views.contains("DoweOverlayCloseIcon(color: DoweDesign.mutedText)"));
    assert!(views.contains("let modalWidth = geometry.size.width * 0.95"));
    assert!(views.contains(".frame(maxWidth: modalWidth, alignment: .leading)"));
    assert!(views.contains(".frame(width: CGFloat(28), height: CGFloat(28))"));
    assert!(views.contains(".frame(width: CGFloat(18), height: CGFloat(18))"));
    assert!(views.contains(".accessibilityLabel(\"Close modal\")"));
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
fn generates_swiftui_dynamic_icon_catalog_with_module_visibility() {
    let dynamic_icon = icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String("@icon-binding:platform.icon".to_string()),
    }])
    .expect("dynamic icon");
    let output = generate_ios(
        &[ViewRoute {
            id: "dynamic-icon".to_string(),
            route_path: "/dynamic-icon".to_string(),
            layout_tree: ViewNode::Children,
            page_tree: dynamic_icon,
            sections: Vec::new(),
            navigation_actions: Vec::new(),
        }],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let pages = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.swift"))
        .expect("pages");
    let catalog = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweDynamicIconCatalog.swift"))
        .expect("dynamic icon catalog");
    let shards = output
        .files
        .iter()
        .filter(|file| {
            file.relative_path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.starts_with("DoweDynamicIconCatalogShard"))
        })
        .collect::<Vec<_>>();
    let views = output
        .files
        .iter()
        .find(|file| {
            file.relative_path
                .ends_with("DowePageDynamicIconView.swift")
        })
        .expect("dynamic icon view");

    assert!(!pages.content.contains("DoweDynamicIconCatalog"));
    assert!(
        catalog
            .content
            .contains("let DoweDynamicIconCatalog: [String: String]")
    );
    assert!(catalog.content.contains("catalog.reserveCapacity("));
    assert!(
        catalog
            .content
            .contains("DoweDynamicIconCatalogShard0.entries")
    );
    assert!(shards.len() > 2);
    assert!(shards.iter().all(|file| file.content.len() < 640_000));
    assert!(views.content.contains("DoweDynamicIconCatalog[state.text("));
}

