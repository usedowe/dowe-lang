use super::{
    IosOutput, generate_ios, generate_ios_with_app_and_translations, generate_ios_with_translations,
};
use dowe_components::{
    AccordionItem, AccordionProps, AlertDialogProps, AudioProps, AvatarGroupItem,
    AvatarGroupProps, AvatarProps, AvatarStatus, BadgeProps, BarProps, Breakpoint, ButtonSize,
    CarouselIndicatorType, CarouselOrientation, CarouselProps, CarouselSlide, ChatBoxMode,
    ChatBoxProps, CheckboxProps, ChipProps, ColorFamily, ColorProps, ColorToken, CommandEntry,
    CollapsibleProps, CommandProps, ComponentProp, ComponentVariant, CountdownProps,
    CountdownSize, CoverSource, DateProps, DateRangeProps, DesignConfig, DividerOrientation,
    DividerProps, DrawerPosition, DrawerProps, DropdownProps, ElementProps, EmptyKind,
    EmptyProps, FontConfig, GapSize, GapValue, GridProps, GridTracks, ImageAspect, ImageLoading,
    ImageObjectFit, ImageProps, MapMarker, MapMarkerIcon, MapProps, MapWaypoint,
    MarqueeOrientation, MarqueeProps, MarqueeSpeed, ModalProps, NavMenuItem, NavMenuItemProps,
    NavMenuProps, NavigationAction, NavigationOperation, OverlayCornerPosition, OverlayEntry,
    OverlayItemProps, OverlayPaint, OverlayPosition, PropValue, RadioGroupProps, RadioOption,
    RecordProps, ResponsiveEntry, ResponsiveValue, RichTextMark, RichTextMarkStyle, RoundedSize,
    ScaleValue, ScaffoldProps, SectionBackground, SelectOption, SideNavItem, SideNavItemProps,
    SideNavProps, SideNavSize, SkeletonAnimation, SkeletonProps, SkeletonVariant, StyleProps,
    SvgPath, SvgPathFill, SvgProps, SvgViewBox, TabItem, TabsPosition, TabsProps, TabsVariant,
    TextProps, TextSize, TextWeight, ToastKind, ToastProps, ToggleGroupItem, ToggleGroupProps,
    ToggleProps, TooltipProps, TranslationCatalog, TranslationLocale, TranslationValue,
    TypeWriterItem, TypeWriterProps, VariantProps, ViewAnimation, ViewNode, ViewRoute,
    ViewSection, ViewSignal, ViewSignalValue, VisibilityCondition,
};
use std::path::PathBuf;

fn swift_content(output: &IosOutput) -> String {
    output
        .files
        .iter()
        .filter(|file| file.relative_path.extension().and_then(|value| value.to_str()) == Some("swift"))
        .map(|file| file.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn generates_swiftui_box_and_text() {
    let output = generate_ios(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(
        views
            .contains("VStack(alignment: .leading, spacing: 0)")
    );
    assert!(views.contains("AnyView("));
    assert!(views.contains("routeSection0()"));
    assert!(views.contains("private func routeSection0() -> some View"));
    assert!(views.contains("private let activePath = \"/login\""));
    assert!(!views.contains("        let activePath ="));
    assert!(!views.contains("VStack(alignment: .leading) {"));
    assert!(
        views
            .contains(".frame(maxWidth: .infinity, alignment: .leading)")
    );
    assert!(views.contains(".background(DoweDesign.primary)"));
    assert!(views.contains("Text(\"Layout\")"));
    assert!(views.contains("Text(\"Login\")"));

    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");
    assert!(plist.content.contains("CFBundleExecutable"));
    assert!(plist.content.contains("DoweIosApp"));
    assert!(plist.content.contains("CFBundleURLSchemes"));
    assert!(plist.content.contains("dowe-dev"));
    assert!(plist.content.contains("UILaunchScreen"));
    assert!(plist.content.contains("NSAllowsLocalNetworking"));
    assert!(plist.content.contains("UIAppFonts"));
    assert!(plist.content.contains("Fonts/inter-regular.ttf"));
}

#[test]
fn generates_shared_swiftui_layout_once_for_multiple_routes() {
    let mut first = route();
    first.layout_tree = ViewNode::Box {
        props: Default::default(),
        children: vec![
            ViewNode::Box {
                props: Default::default(),
                children: vec![text("Layout")],
            },
            ViewNode::Children,
        ],
    };
    let mut second = first.clone();
    second.route_path = "/signup".to_string();
    second.page_tree = ViewNode::Text {
        props: Default::default(),
        value: "Signup".to_string(),
    };

    let output = generate_ios(
        &[first, second],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let layouts = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweLayouts.swift"))
        .expect("layouts");
    let login = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("login");
    let signup = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageSignupView.swift"))
        .expect("signup");

    assert_eq!(
        layouts
            .content
            .matches("struct DoweLayout0<Content: View>")
            .count(),
        1
    );
    assert_eq!(layouts.content.matches("Text(\"Layout\")").count(), 1);
    assert!(layouts.content.contains("layoutSection0()"));
    assert!(
        layouts
            .content
            .contains("private func layoutSection0() -> some View")
    );
    assert!(login.content.contains("DoweLayout0("));
    assert!(signup.content.contains("DoweLayout0("));
    assert!(!login.content.contains("Text(\"Layout\")"));
    assert!(!signup.content.contains("Text(\"Layout\")"));
    assert!(login.content.contains("Text(\"Login\")"));
    assert!(signup.content.contains("Text(\"Signup\")"));
}

#[test]
fn keeps_contextual_swiftui_layout_composed_with_its_page() {
    let mut contextual = route();
    contextual.layout_tree = ViewNode::Scope {
        signals: vec![ViewSignal {
            id: "layout.message".to_string(),
            name: "message".to_string(),
            initial: ViewSignalValue::String("Layout message".to_string()),
            schema: None,
        }],
        actions: Vec::new(),
        children: vec![ViewNode::Box {
            props: Default::default(),
            children: vec![ViewNode::Children],
        }],
    };
    contextual.page_tree = ViewNode::Text {
        props: Default::default(),
        value: "message".to_string(),
    };

    let output = generate_ios(
        &[contextual],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let layouts = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DoweLayouts.swift"))
        .expect("layouts");
    let login = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePageLoginView.swift"))
        .expect("login");

    assert!(!layouts.content.contains("struct DoweLayout0<"));
    assert!(!login.content.contains("DoweLayout0("));
    assert!(login.content.contains("state.text(\"layout.message\")"));
}

#[test]
fn generates_ios_app_metadata() {
    let output = generate_ios_with_app_and_translations(
        &[route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
        &TranslationCatalog::default(),
        "Clinic Desk",
        "com.example.clinic",
    );
    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");

    assert!(plist.content.contains("<string>Clinic Desk</string>"));
    assert!(plist.content.contains("<string>com.example.clinic</string>"));
    assert!(plist.content.contains("<key>CFBundleName</key>"));
}

#[test]
fn generates_swiftui_section_backgrounds() {
    let output = generate_ios(
        &[section_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("enum DoweSectionBackground"));
    assert!(
        views
            .contains("DoweSectionBackgroundView(background: background)")
    );
    assert!(views.contains("doweResponsive(viewportWidth, xs: DoweSectionBackground.aurora, md: DoweSectionBackground.ocean)"));
    assert!(views.contains("LinearGradient(colors: [DoweDesign.softPrimary, DoweDesign.softSecondary, DoweDesign.softTertiary]"));
    assert!(views.contains("DoweCoverImage(source:"));
    assert!(views.contains("https://example.com/hero.jpg"));
    assert!(
        views
            .contains("DoweOverlay.color(Color.black.opacity(0.35))")
    );
    assert!(views.contains("DoweOverlayView(overlay: overlay)"));
}

#[test]
fn generates_native_ios_translation_resources() {
    let mut localized_route = route();
    localized_route.page_tree = ViewNode::Title {
        props: TextProps {
            i18n: Some("home.hero.title".to_string()),
            ..Default::default()
        },
        value: "Dowe builds systems.".to_string(),
    };
    let output = generate_ios_with_translations(
        &[localized_route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
        &translations(),
    );
    let views = swift_content(&output);
    assert!(
        views
            .contains(r#"String(localized: "home.hero.title")"#)
    );
    let english = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("en.lproj/Localizable.strings"))
        .expect("english");
    assert!(english.content.contains("Dowe builds systems."));
    let spanish = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("es.lproj/Localizable.strings"))
        .expect("spanish");
    assert!(spanish.content.contains("Dowe construye sistemas."));
    let plist = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("Info.plist"))
        .expect("plist");
    assert!(plist.content.contains("CFBundleDevelopmentRegion"));
    assert!(plist.content.contains("<string>en</string>"));
}

#[test]
fn generates_swiftui_code_with_copy_and_theme_tokens() {
    let output = generate_ios(
        &[code_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweCodeView: View"));
    assert!(
        views
            .contains("UIPasteboard.general.string = source")
    );
    assert!(views.contains("DoweCodeView(source: \"page docsPage\\n  Card variant:\\\"soft\\\" p:4 show:true\\n    Text\\n      Documentation\""));
    assert!(views.contains("DoweDesign.primary"));
    assert!(views.contains("DoweDesign.info"));
    assert!(views.contains("DoweDesign.tertiary"));
    assert!(views.contains("DoweDesign.success"));
    assert!(views.contains("DoweDesign.warning"));
    assert!(views.contains("DoweDesign.danger"));
}

#[test]
fn generates_swiftui_video_with_native_hls_player() {
    let output = generate_ios(
        &[video_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("import AVKit"));
    assert!(views.contains("struct DoweVideoView: View"));
    assert!(views.contains("VideoPlayer(player: player)"));
    assert!(
        views
            .contains("AVPlayer(url: URL(string: source)!)")
    );
    assert!(
        views
            .contains("https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8")
    );
    assert!(views.contains("poster: \"/images/video.jpg\""));
    assert!(views.contains("aspect: \"vertical\""));
}

#[test]
fn generates_swiftui_candlestick_with_canvas_and_stream() {
    let output = generate_ios(
        &[candlestick_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweCandlestickView: View"));
    assert!(views.contains("Canvas { context, size in"));
    assert!(views.contains("URLSession.shared.bytes(from: url)"));
    assert!(
        views
            .contains("state.upsertCandles(dataPath, payload: payload, maxPoints: maxPoints)")
    );
    assert!(views.contains(
        "DoweCandlestickView(state: state, dataPath: \"candles\", stream: \"/api/candles\""
    ));
    assert!(views.contains("emptyLabel: \"Market closed\""));
}

#[test]
fn generates_swiftui_table_with_columns_and_scheme() {
    let output = generate_ios(
        &[table_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweTableView: View"));
    assert!(
        views
            .contains("DoweTableView(state: state, dataPath: \"users\"")
    );
    assert!(views.contains(
        "DoweTableColumn(field: \"status\", label: \"Status\", align: .end, width: \"8rem\")"
    ));
    assert!(views.contains("size: .lg"));
    assert!(
        views
            .contains("striped: true, bordered: true, dividers: true")
    );
    assert!(views.contains("emptyTitle: \"No users\""));
    assert!(
        views
            .contains("backgroundColor: DoweDesign.surface")
    );
    assert!(views.contains("contentColor: DoweDesign.onSurface"));
    assert!(views.contains("state.rows(dataPath)"));
}

#[test]
fn generates_swiftui_divider_with_native_shape() {
    let output = generate_ios(
        &[divider_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("Rectangle()"));
    assert!(views.contains(".fill(DoweDesign.primary)"));
    assert!(views.contains(".frame(width: CGFloat(1))"));
    assert!(views.contains(".frame(maxHeight: .infinity)"));
}

#[test]
fn generates_swiftui_responsive_runtime_values() {
    let output = generate_ios(
        &[responsive_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("GeometryReader { geometry in"));
    assert!(views.contains("let viewportWidth: CGFloat"));
    assert!(views.contains(
        ".padding(doweResponsive(viewportWidth, xs: CGFloat(16), md: CGFloat(32)) ?? CGFloat(0))"
    ));
    assert!(
        views
            .contains(".padding(doweResponsive(viewportWidth, md: CGFloat(32)) ?? CGFloat(0))")
    );
    assert!(views.contains(
            ".font(doweFont(.inter, size: doweResponsive(viewportWidth, md: doweTextSize(viewportWidth, min: CGFloat(16), preferredBase: CGFloat(15.2), preferredViewport: CGFloat(0.3), max: CGFloat(18))) ?? doweTextSize(viewportWidth, min: CGFloat(14), preferredBase: CGFloat(13.12), preferredViewport: CGFloat(0.25), max: CGFloat(16))))"
        ));
    assert!(views.contains(
            ".fontWeight(doweResponsive(viewportWidth, xs: Font.Weight.ultraLight, md: Font.Weight.thin, lg: Font.Weight.black) ?? Font.Weight.regular)"
        ));
    assert!(!views.contains(".padding(16)\n"));
}

#[test]
fn generates_swiftui_show_visibility_conditions() {
    let output = generate_ios(
        &[show_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(
        views
            .contains("if doweResponsive(viewportWidth, xs: false, md: true) ?? true {")
    );
    assert!(views.contains("if state.bool(\"ready01\") {"));
    assert!(
        views
            .contains("if state.bool(\"item.ready\", item: row.value) {")
    );
}

#[test]
fn generates_swiftui_layout_bars() {
    let output = generate_ios(
        &[bar_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("ZStack {"));
    assert!(
        views
            .contains("HStack(alignment: .center, spacing: 0)")
    );
    assert!(views.contains("Text(\"Brand\")"));
    assert!(views.contains(".background(DoweDesign.surface)"));
    assert!(
        views
            .contains(".foregroundStyle(DoweDesign.onSurface)")
    );
    assert!(
        views
            .contains(".clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusBox))")
    );
    assert!(views.contains(
            ".overlay(RoundedRectangle(cornerRadius: DoweDesign.radiusBox).stroke(DoweDesign.muted, lineWidth: CGFloat(1)))"
        ));
    assert!(
        !views
            .contains(".overlay(Rectangle().fill(DoweDesign.muted).frame(height: CGFloat(1))")
    );
    assert!(!views.contains(
            ".overlay(RoundedRectangle(cornerRadius: CGFloat(0)).stroke(DoweDesign.muted, lineWidth: CGFloat(1)))"
        ));
    assert!(views.contains(".padding(.horizontal, CGFloat(16))"));
    assert!(
        views
            .contains(".frame(maxWidth: CGFloat(1152), alignment: .center)")
    );
}

#[test]
fn generates_swiftui_nonfloating_bar_without_divider() {
    let output = generate_ios(
        &[appbar_divider_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(
        !views
            .contains(".overlay(Rectangle().fill(DoweDesign.muted).frame(height: CGFloat(1))")
    );
    assert!(!views.contains(
            ".overlay(RoundedRectangle(cornerRadius: CGFloat(0)).stroke(DoweDesign.muted, lineWidth: CGFloat(1)))"
        ));
}

#[test]
fn generates_swiftui_side_nav() {
    let output = generate_ios(
        &[side_nav_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(
        views
            .contains("struct DoweSideNavRow<Content: View>: View")
    );
    assert!(views.contains("DoweSideNavSubmenu(open: true)"));
    assert!(
        views
            .contains("withAnimation(.easeInOut(duration: 0.18))")
    );
    assert!(
        views
            .contains(".transition(.opacity.combined(with: .move(edge: .top)))")
    );
    assert!(views.contains("active: activePath == \"/bars\""));
    assert!(views.contains("Text(\"Workspace\")"));
    assert!(views.contains("Text(\"Blogs\")"));
}

#[test]
fn generates_swiftui_navigation_shell_components() {
    let output = generate_ios(
        &[navigation_shell_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("DoweNavMenu(gap:"));
    assert!(views.contains("DoweNavMenuItem(active: activePath == \"/\""));
    assert!(views.contains("DoweNavMenuItem(active: openIndex == 1"));
    assert!(views.contains("HStack(alignment: .top"));
    assert!(views.contains("Text(\"Resource hub\")"));
    assert!(views.contains("Text(\"Side Home\")"));
}

#[test]
fn generates_swiftui_tabs() {
    let output = generate_ios(
        &[tabs_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweTabs<Content: View>: View"));
    assert!(views.contains("DoweTabs(items: [DoweTabItem(id: \"overview\", label: \"Overview\"), DoweTabItem(id: \"details\", label: \"Details\")], initialId: \"overview\""));
    assert!(views.contains("position: \"start\", variant: \"line\""));
    assert!(views.contains("backgroundColor: Color.clear"));
    assert!(views.contains("accentColor: DoweDesign.primary"));
    assert!(views.contains("if activeTab == \"overview\""));
    assert!(views.contains("Text(\"Overview content\")"));
}

#[test]
fn generates_swiftui_drawer() {
    let output = generate_ios(
        &[drawer_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(
        views
            .contains("struct DoweDrawer<Content: View>: View")
    );
    assert!(
        views
            .contains("struct DoweDrawerPresenter<Content: View>: UIViewRepresentable")
    );
    assert!(views.contains("window.addSubview(controller.view)"));
    assert!(views.contains("DoweDrawer(open: state.bool(\"drawer01\"), close: { state.write(\"drawer01\", value: false) }, position: \"end\""));
    assert!(views.contains("radius: CGFloat(0)"));
    assert!(
        views
            .contains("disableOverlayClose: true, hideCloseButton: false")
    );
    assert!(
        views
            .contains("return CGSize(width: CGFloat(320), height: CGFloat(0))")
    );
    assert!(
        views
            .contains("private var panelShape: UnevenRoundedRectangle")
    );
    assert!(views.contains("return UnevenRoundedRectangle(topLeadingRadius: radius, bottomLeadingRadius: radius, bottomTrailingRadius: CGFloat(0), topTrailingRadius: CGFloat(0))"));
    let rounded_style = StyleProps {
        rounded: Some(ResponsiveValue::scalar(RoundedSize::Lg)),
        ..Default::default()
    };
    assert_eq!(
        super::swift_drawer_radius(&rounded_style),
        "doweResponsive(viewportWidth, xs: CGFloat(12)) ?? CGFloat(0)"
    );
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
    assert!(views.contains("DoweChip(text: \"Filter\", size: \"sm\""));
    assert!(views.contains("DoweSkeleton(variant: \"rounded\", animation: \"pulse\")"));
    assert!(views.contains("private let pathBuilder: @Sendable (CGRect) -> Path"));
    assert!(views.contains("DoweModal(open: state.bool(\"modal01\")"));
    assert!(views.contains("DoweAlertDialog(open: state.bool(\"modal01\")"));
    assert!(views.contains("DoweTooltip(label: \"More actions\", position: \"end\""));
    assert!(views.contains("DoweToast(visible: true, title: \"Saved\""));
    assert!(views.contains("DoweDropdown(backgroundColor: DoweDesign.surface"));
    assert!(views.contains("DoweCommand(open: state.bool(\"modal01\")"));
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
    assert!(views.contains("DoweAvatarGroup(items: doweAvatarGroupItems(state.rows(\"people\")"));
    assert!(views.contains("DoweChatBox(state: state, messagesPath: \"messages\""));
    assert!(views.contains("DoweEmpty(kind: \"result\""));
    assert!(views.contains("DoweMarquee(speed: \"fast\""));
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

    assert!(
        views
            .contains(".background(DoweDesign.background.ignoresSafeArea())")
    );
    assert!(views.contains("struct DoweSafeAreaReporter: UIViewRepresentable"));
    assert!(views.contains("final class DoweSafeAreaReportingView: UIView"));
    assert!(views.contains("@State private var safeAreaInsets = EdgeInsets()"));
    assert!(views.contains("DoweSafeAreaReporter { insets in"));
    assert!(views.contains("routeContent(currentEntry, viewportWidth: doweSafeAreaWidth(geometry, safeAreaInsets))"));
    assert!(views.contains(
        ".frame(width: doweSafeAreaWidth(geometry, safeAreaInsets), height: doweSafeAreaHeight(geometry, safeAreaInsets), alignment: .topLeading)"
    ));
    assert!(views.contains(
        ".frame(width: doweSafeAreaWidth(geometry, safeAreaInsets), height: doweSafeAreaHeight(geometry, safeAreaInsets), alignment: .topLeading)\n                .clipped()\n                .offset(x: safeAreaInsets.leading, y: safeAreaInsets.top)"
    ));
    assert!(views.contains(".offset(x: safeAreaInsets.leading, y: safeAreaInsets.top)"));
    assert!(views.contains("        .ignoresSafeArea()\n        .frame(maxWidth: .infinity"));
    assert!(views.contains("func doweSafeAreaWidth(_ geometry: GeometryProxy, _ insets: EdgeInsets) -> CGFloat"));
    assert!(views.contains("func doweSafeAreaHeight(_ geometry: GeometryProxy, _ insets: EdgeInsets) -> CGFloat"));
    assert!(views.contains("func doweInsetsEqual(_ lhs: EdgeInsets, _ rhs: EdgeInsets) -> Bool"));
    assert!(views.contains(
        "private func routeContent(_ entry: DoweRouteEntry, viewportWidth: CGFloat) -> some View"
    ));
    assert!(views.contains("LoginView(viewportWidth: viewportWidth, activeFragment: entry.fragment"));
    assert!(
        views
            .contains(".frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)")
    );
    assert!(!views.contains("safeAreaInsets:"));
    assert!(!views.contains("let safeAreaInsets"));
    assert!(!views.contains(".padding(.top, safeAreaInsets.top)"));
    assert!(!views.contains(".padding(.bottom, safeAreaInsets.bottom)"));
    assert!(views.contains(
            "        .background(DoweDesign.background)\n        .foregroundStyle(DoweDesign.onBackground)"
        ));
    assert!(
        !views
            .contains(".ignoresSafeArea()\n        .foregroundStyle")
    );
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
            "LazyVGrid(columns: doweGridColumns(doweResponsive(viewportWidth, xs: 1, md: 2) ?? 1, spacing: doweResponsive(viewportWidth, xs: CGFloat(16))),"
        ));
    assert!(
        views
            .contains("DoweInputField(value: nil, label: nil, placeholder: \"\", floating: false")
    );
    assert!(
        views
            .contains("minHeight: CGFloat(40), horizontalPadding: CGFloat(12)")
    );
    assert!(views.contains(
            "backgroundColor: Color.clear, contentColor: DoweDesign.secondary, borderColor: Optional(DoweDesign.muted)"
        ));
    assert!(
        views
            .contains(".foregroundStyle(DoweDesign.onSoftMuted)")
    );
    assert!(views.contains(".background(DoweDesign.surface)"));
    assert!(
        views
            .contains(".foregroundStyle(DoweDesign.onSurface)")
    );
    assert!(
        views
            .contains(".stroke(DoweDesign.surface, lineWidth: CGFloat(1))")
    );
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
    assert!(views.contains(
        r#"DoweInputField(value: nil, label: "Name", placeholder: "Full name", floating: true"#
    ));
    assert!(views.contains("let value: Binding<String>?"));
    assert!(
        views
            .contains("@State private var localText = \"\"")
    );
    assert!(
        views
            .contains("private var visiblePlaceholder: String")
    );
    assert!(
        views
            .contains("TextField(visiblePlaceholder, text: textBinding)")
    );
    assert!(views.contains("struct DoweSelectField: View"));
    assert!(views.contains("struct DoweSelectPopover: View"));
    assert!(
        views
            .contains("struct DoweSelectAnchorPresenter: UIViewRepresentable")
    );
    assert!(
        views
            .contains("UIHostingController<DoweSelectPopover>")
    );
    assert!(views.contains("UIView.animate(withDuration: 0.16"));
    assert!(views.contains("UIView.animate(withDuration: 0.14"));
    assert!(
        views
            .contains("anchor.convert(anchor.bounds, to: window)")
    );
    assert!(views.contains("window.addSubview(controller.view)"));
    assert!(
        views
            .contains("let contentHeight = CGFloat(8) + parent.options.reduce(CGFloat(0))")
    );
    assert!(
        views
            .contains("option.description == nil ? CGFloat(40) : CGFloat(58)")
    );
    assert!(
        views
            .contains("let height = min(maxHeight, max(CGFloat(44), contentHeight))")
    );
    assert!(!views.contains("max(size.height, estimated)"));
    assert!(
        views
            .contains("@State private var expanded = false")
    );
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
    assert!(
        views
            .contains("DoweSelectArrow(color: contentColor)")
    );
    assert!(
        views
            .contains("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4")
    );
    assert!(
        views
            .contains("if selectedOption != nil || !floating || expanded")
    );
    assert!(views.contains("Text(description).font(.caption)"));
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
    assert!(views.contains("DoweAccordionView(multiple:"));
    assert!(views.contains("DoweCarouselView(autoplay:"));
    assert!(views.contains("DoweCheckboxView(checked:"));
    assert!(views.contains("DoweColorField(value:"));
    assert!(views.contains("DoweDateField(value:"));
    assert!(views.contains("DoweDateRangeField(startValue:"));
    assert!(views.contains("DoweRadioGroupView(value:"));
    assert!(views.contains("DoweToggleView(checked:"));
    assert!(views.contains("Image(systemName: \"checkmark\")"));
    assert!(views.contains("doweColorFromHex(value.wrappedValue"));
    assert!(views.contains("DoweInputField(value: value"));
    assert!(views.contains("TextField(\"Start\", text: startValue)"));
    assert!(views.contains("Circle()"));
    assert!(views.contains(".tint(accentColor)"));
    assert!(views.contains("func boolBinding(_ path: String) -> Binding<Bool>"));
    assert!(!views.contains("DoweSimpleField"));
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
    assert!(
        views
            .contains("@State private var rootEntry = DoweRouteEntry")
    );
    assert!(
        views
            .contains("@State private var navigationPath: [DoweRouteEntry] = []")
    );
    assert!(
        views
            .contains("routeContent(currentEntry, viewportWidth: doweSafeAreaWidth(geometry, safeAreaInsets))")
    );
    assert!(
        views
            .contains(".simultaneousGesture(backSwipeGesture)")
    );
    assert!(!views.contains("NavigationStack(path: $navigationPath)"));
    assert!(!views.contains(".navigationDestination(for: DoweRouteEntry.self)"));
    assert!(!views.contains(".toolbar(.hidden, for: .navigationBar)"));
    assert!(!views.contains(".navigationBarHidden(true)"));
    assert!(
        views
            .contains("private var backSwipeGesture: some Gesture")
    );
    assert!(views.contains("navigationPath.append(destination)"));
    assert!(views.contains("navigationPath.removeLast()"));
    assert!(views.contains(
        "private func navigate(_ operation: String, _ target: String, _ fragment: String?)"
    ));
    assert!(
        views
            .contains(r#"{ navigate("push", "/signup", "join") }"#)
    );
    assert!(
        views
            .contains(r#"{ navigate("replace", "", "hero") }"#)
    );
    assert!(views.contains("{ goBack() }"));
    assert!(
        views
            .contains(r#"navigate("replace", path, url.fragment)"#)
    );
    assert!(views.contains("ScrollViewReader { proxy in"));
    assert!(views.contains("doweScroll(proxy, activeFragment)"));
    assert!(
        views
            .contains(".onChange(of: activeFragment) { _, value in doweScroll(proxy, value) }")
    );
    assert!(
        !views
            .contains(".onChange(of: activeFragment) { value in doweScroll(proxy, value) }")
    );
    assert!(views.contains(".id(\"hero\")"));
    assert!(
        routing
            .content
            .contains("static let sections: [String: [String]]")
    );
    assert!(routing.content.contains(r#""/signup": ["join"]"#));
}

#[test]
fn generates_swiftui_svg_views() {
    let output = generate_ios(
        &[svg_route()],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = swift_content(&output);

    assert!(views.contains("struct DoweSvgView: View"));
    assert!(views.contains("DoweSvgViewBox(minX: CGFloat(0), minY: CGFloat(0), width: CGFloat(24), height: CGFloat(24))"));
    assert!(views.contains("DoweSvgFill.currentColor"));
    assert!(views.contains(
        "doweResponsive(viewportWidth, xs: DoweDesign.tertiary) ?? DoweDesign.onBackground"
    ));
    assert!(views.contains("DoweSvgPathParser(data)"));
    assert!(views.contains("c.253.847.1 1.895-.62 2.618a.75.75"));
    assert!(
        views
            .contains("if characters[index] == \"-\" || characters[index] == \"+\"")
    );
    assert!(views.contains(
            ".frame(width: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))))"
        ));
    assert!(views.contains(
            ".frame(maxWidth: doweMaxSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))))"
        ));
    assert!(views.contains(
            ".frame(height: doweFixedSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))))"
        ));
    assert!(views.contains(
            ".frame(maxHeight: doweMaxSize(doweResponsive(viewportWidth, xs: DoweSize.fixed(CGFloat(32)))))"
        ));
    assert!(!views.contains(", maxWidth: doweMaxSize("));
    assert!(!views.contains(", maxHeight: doweMaxSize("));
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
    assert!(
        views
            .contains(".modifier(DoweAnimationModifier(preset: .fadeIn))")
    );
    assert!(
        views
            .contains(".modifier(DoweAnimationModifier(preset: .slideUp))")
    );
    assert!(
        views
            .contains(".animation(.easeOut(duration: 0.22), value: active)")
    );
}

fn route() -> ViewRoute {
    ViewRoute {
        id: "login".to_string(),
        route_path: "/login".to_string(),
        layout_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![text("Layout"), ViewNode::Children],
        },
        page_tree: ViewNode::Card {
            props: Default::default(),
            children: vec![text("Login")],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn section_route() -> ViewRoute {
    ViewRoute {
        id: "sections".to_string(),
        route_path: "/sections".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Section {
                    props: StyleProps {
                        text: Some(ResponsiveValue::scalar(ColorToken::OnBackground)),
                        background: Some(ResponsiveValue::ordered(vec![
                            ResponsiveEntry {
                                breakpoint: Breakpoint::Xs,
                                value: SectionBackground::Aurora,
                            },
                            ResponsiveEntry {
                                breakpoint: Breakpoint::Md,
                                value: SectionBackground::Ocean,
                            },
                        ])),
                        ..Default::default()
                    },
                    children: vec![text("Hero")],
                },
                ViewNode::Section {
                    props: StyleProps {
                        cover: Some(ResponsiveValue::scalar(CoverSource(
                            "https://example.com/hero.jpg".to_string(),
                        ))),
                        overlay: Some(ResponsiveValue::scalar(OverlayPaint::BlackOpacity(
                            "0.35".to_string(),
                        ))),
                        ..Default::default()
                    },
                    children: vec![text("Covered")],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn bar_route() -> ViewRoute {
    ViewRoute {
        id: "bars".to_string(),
        route_path: "/bars".to_string(),
        layout_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::AppBar {
                    props: bar_props(true),
                    start: vec![text("Menu")],
                    center: vec![text("Brand")],
                    end: vec![text("Account")],
                },
                ViewNode::Children,
                ViewNode::Footer {
                    props: bar_props(false),
                    start: vec![text("Footer")],
                    center: Vec::new(),
                    end: vec![text("Legal")],
                },
            ],
        },
        page_tree: ViewNode::BottomBar {
            props: bar_props(false),
            start: vec![text("Home")],
            center: vec![text("Search")],
            end: vec![text("Profile")],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn appbar_divider_route() -> ViewRoute {
    ViewRoute {
        id: "appbar-divider".to_string(),
        route_path: "/appbar-divider".to_string(),
        layout_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::AppBar {
                    props: bar_props(false),
                    start: vec![text("Menu")],
                    center: vec![text("Brand")],
                    end: vec![text("Account")],
                },
                ViewNode::Children,
            ],
        },
        page_tree: text("Page"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn side_nav_route() -> ViewRoute {
    ViewRoute {
        id: "side-nav".to_string(),
        route_path: "/bars".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::SideNav {
            props: SideNavProps {
                style: VariantProps {
                    variant: Some(ComponentVariant::Soft),
                    color: Some(ColorFamily::Surface),
                    ..Default::default()
                },
                size: SideNavSize::Md,
                wide: true,
            },
            items: vec![
                SideNavItem::Header(SideNavItemProps {
                    label: "Workspace".to_string(),
                    description: None,
                    status: None,
                    icon: None,
                    on_click: None,
                    navigation: None,
                }),
                SideNavItem::Submenu {
                    props: SideNavItemProps {
                        label: "Content".to_string(),
                        description: None,
                        status: None,
                        icon: None,
                        on_click: None,
                        navigation: None,
                    },
                    open: true,
                    items: vec![side_nav_item("Blogs", "/bars")],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn navigation_shell_route() -> ViewRoute {
    ViewRoute {
        id: "shell".to_string(),
        route_path: "/".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Scaffold {
            props: ScaffoldProps {
                style: StyleProps::default(),
                boxed: true,
            },
            app_bar: vec![ViewNode::NavMenu {
                props: NavMenuProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Ghost),
                        color: Some(ColorFamily::Muted),
                        ..Default::default()
                    },
                    size: SideNavSize::Md,
                },
                items: vec![
                    NavMenuItem::Item(NavMenuItemProps {
                        label: "Home".to_string(),
                        description: None,
                        icon: None,
                        on_click: None,
                        navigation: Some(NavigationAction::Internal {
                            path: "/".to_string(),
                            fragment: None,
                            operation: NavigationOperation::Push,
                        }),
                    }),
                    NavMenuItem::Submenu {
                        props: NavMenuItemProps {
                            label: "Docs".to_string(),
                            description: None,
                            icon: None,
                            on_click: None,
                            navigation: None,
                        },
                        items: vec![NavMenuItemProps {
                            label: "Guide".to_string(),
                            description: Some("Start here".to_string()),
                            icon: None,
                            on_click: None,
                            navigation: Some(NavigationAction::Internal {
                                path: "/docs".to_string(),
                                fragment: None,
                                operation: NavigationOperation::Push,
                            }),
                        }],
                    },
                    NavMenuItem::Megamenu {
                        props: NavMenuItemProps {
                            label: "Resources".to_string(),
                            description: None,
                            icon: None,
                            on_click: None,
                            navigation: None,
                        },
                        content: vec![text("Resource hub")],
                    },
                ],
            }],
            start: vec![ViewNode::Sidebar {
                props: SideNavProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    size: SideNavSize::Md,
                    wide: true,
                },
                items: vec![SideNavItem::Item(SideNavItemProps {
                    label: "Side Home".to_string(),
                    description: None,
                    status: None,
                    icon: None,
                    on_click: None,
                    navigation: Some(NavigationAction::Internal {
                        path: "/".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                })],
            }],
            main: vec![text("Main content")],
            end: Vec::new(),
            bottom_bar: vec![text("Bottom")],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn tabs_route() -> ViewRoute {
    ViewRoute {
        id: "tabs".to_string(),
        route_path: "/tabs".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Tabs {
            props: TabsProps {
                style: StyleProps::default(),
                variant: TabsVariant::Line,
                color: ColorFamily::Primary,
                position: TabsPosition::Start,
            },
            tabs: vec![
                TabItem {
                    id: "overview".to_string(),
                    label: "Overview".to_string(),
                    children: vec![text("Overview content")],
                },
                TabItem {
                    id: "details".to_string(),
                    label: "Details".to_string(),
                    children: vec![text("Details content")],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn drawer_route() -> ViewRoute {
    ViewRoute {
        id: "drawer".to_string(),
        route_path: "/drawer".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Scope {
            signals: vec![ViewSignal {
                id: "drawer01".to_string(),
                name: "drawerOpen".to_string(),
                initial: ViewSignalValue::Bool(false),
                schema: None,
            }],
            actions: Vec::new(),
            children: vec![ViewNode::Drawer {
                props: DrawerProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    open: "drawerOpen".to_string(),
                    position: DrawerPosition::End,
                    disable_overlay_close: true,
                    hide_close_button: false,
                },
                children: vec![text("Navigation")],
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn side_nav_item(label: &str, path: &str) -> SideNavItemProps {
    SideNavItemProps {
        label: label.to_string(),
        description: None,
        status: None,
        icon: None,
        on_click: None,
        navigation: Some(NavigationAction::Internal {
            path: path.to_string(),
            fragment: None,
            operation: NavigationOperation::Push,
        }),
    }
}

fn responsive_route() -> ViewRoute {
    ViewRoute {
        id: "login".to_string(),
        route_path: "/login".to_string(),
        layout_tree: ViewNode::Box {
            props: StyleProps {
                spacing: dowe_components::SpacingProps {
                    p: Some(responsive_scale(&[
                        (Breakpoint::Xs, 4),
                        (Breakpoint::Md, 8),
                    ])),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![ViewNode::Children],
        },
        page_tree: ViewNode::Box {
            props: StyleProps {
                spacing: dowe_components::SpacingProps {
                    p: Some(responsive_scale(&[(Breakpoint::Md, 8)])),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![ViewNode::Text {
                props: TextProps {
                    size: Some(ResponsiveValue::ordered(vec![ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: TextSize::Lg,
                    }])),
                    weight: Some(ResponsiveValue::ordered(vec![
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Xs,
                            value: TextWeight::Thin,
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Md,
                            value: TextWeight::Extralight,
                        },
                        ResponsiveEntry {
                            breakpoint: Breakpoint::Lg,
                            value: TextWeight::Black,
                        },
                    ])),
                    ..Default::default()
                },
                value: "Login".to_string(),
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn show_route() -> ViewRoute {
    ViewRoute {
        id: "ready".to_string(),
        route_path: "/ready".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Scope {
            signals: vec![
                ViewSignal {
                    id: "ready01".to_string(),
                    name: "isReady".to_string(),
                    initial: ViewSignalValue::Bool(false),
                    schema: None,
                },
                ViewSignal {
                    id: "rows01".to_string(),
                    name: "rows".to_string(),
                    initial: ViewSignalValue::Array(vec![ViewSignalValue::Object(vec![
                        ("id".to_string(), ViewSignalValue::String("1".to_string())),
                        ("ready".to_string(), ViewSignalValue::Bool(true)),
                    ])]),
                    schema: None,
                },
            ],
            actions: Vec::new(),
            children: vec![
                ViewNode::Box {
                    props: StyleProps {
                        element: ElementProps {
                            show: Some(VisibilityCondition::Static(responsive_bool(&[
                                (Breakpoint::Xs, false),
                                (Breakpoint::Md, true),
                            ]))),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    children: vec![ViewNode::Text {
                        props: TextProps {
                            style: StyleProps {
                                element: ElementProps {
                                    show: Some(VisibilityCondition::Signal("isReady".to_string())),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: "Ready".to_string(),
                    }],
                },
                ViewNode::Each {
                    item: "row".to_string(),
                    collection: "rows".to_string(),
                    key: "row.id".to_string(),
                    children: vec![ViewNode::Text {
                        props: TextProps {
                            style: StyleProps {
                                element: ElementProps {
                                    show: Some(VisibilityCondition::Signal(
                                        "row.ready".to_string(),
                                    )),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: "Row".to_string(),
                    }],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn index_route_with_navigation() -> ViewRoute {
    ViewRoute {
        id: "index".to_string(),
        route_path: "/".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps {
                element: ElementProps {
                    id: Some("hero".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![
                ViewNode::Button {
                    props: VariantProps {
                        navigation: Some(NavigationAction::Internal {
                            path: "/signup".to_string(),
                            fragment: Some("join".to_string()),
                            operation: NavigationOperation::Push,
                        }),
                        ..Default::default()
                    },
                    children: vec![text("Signup")],
                },
                ViewNode::Button {
                    props: VariantProps {
                        navigation: Some(NavigationAction::Section {
                            fragment: "hero".to_string(),
                            operation: NavigationOperation::Replace,
                        }),
                        ..Default::default()
                    },
                    children: vec![text("Hero")],
                },
                ViewNode::Button {
                    props: VariantProps {
                        navigation: Some(NavigationAction::Back),
                        ..Default::default()
                    },
                    children: vec![text("Back")],
                },
            ],
        },
        sections: vec![ViewSection {
            id: "hero".to_string(),
        }],
        navigation_actions: Vec::new(),
    }
}

fn signup_route() -> ViewRoute {
    ViewRoute {
        id: "signup".to_string(),
        route_path: "/signup".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: text("Signup"),
        sections: vec![ViewSection {
            id: "join".to_string(),
        }],
        navigation_actions: Vec::new(),
    }
}

fn parity_route() -> ViewRoute {
    ViewRoute {
        id: "parity".to_string(),
        route_path: "/parity".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Grid {
            props: GridProps {
                columns: Some(ResponsiveValue::ordered(vec![
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Xs,
                        value: GridTracks::Count(1),
                    },
                    ResponsiveEntry {
                        breakpoint: Breakpoint::Md,
                        value: GridTracks::Count(2),
                    },
                ])),
                gap: Some(ResponsiveValue::scalar(GapValue::Single(GapSize::Scale(
                    ScaleValue::from_half_steps(8),
                )))),
                ..Default::default()
            },
            children: vec![
                ViewNode::Input {
                    props: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Secondary),
                        ..Default::default()
                    },
                },
                ViewNode::Card {
                    props: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Muted),
                        ..Default::default()
                    },
                    children: vec![text("Card")],
                },
                ViewNode::Card {
                    props: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    children: vec![text("Surface")],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn form_route() -> ViewRoute {
    ViewRoute {
        id: "form".to_string(),
        route_path: "/form".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Input {
                    props: VariantProps {
                        label: Some("Name".to_string()),
                        placeholder: Some("Full name".to_string()),
                        label_floating: true,
                        variant: Some(ComponentVariant::Outlined),
                        ..Default::default()
                    },
                },
                ViewNode::Select {
                    props: VariantProps {
                        label: Some("Role".to_string()),
                        placeholder: Some("Choose role".to_string()),
                        label_floating: true,
                        variant: Some(ComponentVariant::Outlined),
                        ..Default::default()
                    },
                    options: vec![SelectOption {
                        value: "admin".to_string(),
                        label: "Admin".to_string(),
                        description: Some("Manages users".to_string()),
                    }],
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn media_display_form_route() -> ViewRoute {
    ViewRoute {
        id: "components".to_string(),
        route_path: "/components".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Audio {
                    props: AudioProps {
                        style: VariantProps {
                            variant: Some(ComponentVariant::Soft),
                            color: Some(ColorFamily::Primary),
                            ..Default::default()
                        },
                        src: "https://cdn.pixabay.com/audio/2022/04/25/audio_5d61b5204f.mp3"
                            .to_string(),
                        subtitle: Some("Preview".to_string()),
                        avatar_src: None,
                    },
                },
                ViewNode::Image {
                    props: ImageProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Secondary),
                            ..Default::default()
                        },
                        src: "https://example.com/photo.jpg".to_string(),
                        alt: "Photo".to_string(),
                        aspect: ImageAspect::Square,
                        object_fit: ImageObjectFit::Cover,
                        loading: ImageLoading::Lazy,
                        hide_controls: true,
                    },
                },
                ViewNode::Accordion {
                    props: AccordionProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Surface),
                            ..Default::default()
                        },
                        multiple: true,
                    },
                    items: vec![AccordionItem {
                        id: "intro".to_string(),
                        label: "Intro".to_string(),
                        disabled: false,
                        default_open: true,
                        children: vec![text("Intro body")],
                    }],
                },
                ViewNode::Carousel {
                    props: CarouselProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Info),
                            ..Default::default()
                        },
                        autoplay: false,
                        autoplay_interval: 3000,
                        disable_loop: false,
                        hide_controls: false,
                        hide_indicators: false,
                        show_navigation: true,
                        show_counter: true,
                        orientation: CarouselOrientation::Horizontal,
                        size: ButtonSize::Md,
                        indicator_type: CarouselIndicatorType::Bar,
                        title: Some("Samples".to_string()),
                        slide_width: None,
                        slide_height: None,
                        slides_per_view: 1,
                        gap: 8,
                    },
                    slides: vec![CarouselSlide {
                        id: "one".to_string(),
                        children: vec![text("First")],
                    }],
                },
                ViewNode::Checkbox {
                    props: CheckboxProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Success),
                            label: Some("Accept".to_string()),
                            element: ElementProps {
                                bind: Some("accepted".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        checked: true,
                        disabled: false,
                        name: Some("accepted".to_string()),
                    },
                },
                ViewNode::Color {
                    props: ColorProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Primary),
                            label: Some("Theme".to_string()),
                            element: ElementProps {
                                bind: Some("themeColor".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: "#3366ff".to_string(),
                        size: ButtonSize::Md,
                        name: None,
                        help_text: None,
                        error_text: None,
                        show_hex: true,
                        show_rgb: false,
                        show_cmyk: false,
                        show_oklch: false,
                    },
                },
                ViewNode::Date {
                    props: DateProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Warning),
                            label: Some("Ship date".to_string()),
                            element: ElementProps {
                                bind: Some("shipDate".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: Some("2026-06-05".to_string()),
                        size: ButtonSize::Md,
                        name: None,
                        help_text: None,
                        error_text: None,
                        min: None,
                        max: None,
                    },
                },
                ViewNode::DateRange {
                    props: DateRangeProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Danger),
                            label: Some("Range".to_string()),
                            ..Default::default()
                        },
                        start: Some("startDate".to_string()),
                        end: Some("endDate".to_string()),
                        start_value: Some("2026-06-01".to_string()),
                        end_value: Some("2026-06-08".to_string()),
                        size: ButtonSize::Md,
                        name: None,
                        help_text: None,
                        error_text: None,
                        min: None,
                        max: None,
                    },
                },
                ViewNode::RadioGroup {
                    props: RadioGroupProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Muted),
                            label: Some("Plan".to_string()),
                            element: ElementProps {
                                bind: Some("choice".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        size: ButtonSize::Md,
                        name: Some("plan".to_string()),
                        info: None,
                        error: None,
                    },
                    options: vec![RadioOption {
                        value: "basic".to_string(),
                        label: "Basic".to_string(),
                        disabled: false,
                    }],
                },
                ViewNode::Toggle {
                    props: ToggleProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Secondary),
                            label: Some("Enabled".to_string()),
                            element: ElementProps {
                                bind: Some("accepted".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        checked: true,
                        disabled: false,
                        name: None,
                        label_left: Some("Off".to_string()),
                        label_right: Some("On".to_string()),
                    },
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn svg_route() -> ViewRoute {
    ViewRoute {
        id: "svg".to_string(),
        route_path: "/svg".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: svg_tree(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn motion_route() -> ViewRoute {
    ViewRoute {
        id: "motion".to_string(),
        route_path: "/motion".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps {
                animation: Some(ViewAnimation::FadeIn),
                ..Default::default()
            },
            children: vec![ViewNode::Card {
                props: VariantProps {
                    style: StyleProps {
                        animation: Some(ViewAnimation::SlideUp),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                children: vec![text("Motion")],
            }],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn code_route() -> ViewRoute {
    ViewRoute {
        id: "code".to_string(),
        route_path: "/code".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::code_node(
            vec![
                ComponentProp {
                    name: "language".to_string(),
                    value: PropValue::String("dowe".to_string()),
                },
                ComponentProp {
                    name: "scheme".to_string(),
                    value: PropValue::String("surface".to_string()),
                },
            ],
            vec![
                "page docsPage".to_string(),
                "  Card variant:\"soft\" p:4 show:true".to_string(),
                "    Text".to_string(),
                "      Documentation".to_string(),
            ],
        )
        .expect("code"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn video_route() -> ViewRoute {
    ViewRoute {
        id: "video".to_string(),
        route_path: "/video".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::video_node(vec![
            ComponentProp {
                name: "src".to_string(),
                value: PropValue::String(
                    "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8".to_string(),
                ),
            },
            ComponentProp {
                name: "poster".to_string(),
                value: PropValue::String("/images/video.jpg".to_string()),
            },
            ComponentProp {
                name: "aspect".to_string(),
                value: PropValue::String("vertical".to_string()),
            },
            ComponentProp {
                name: "scheme".to_string(),
                value: PropValue::String("surface".to_string()),
            },
        ])
        .expect("video"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn candlestick_route() -> ViewRoute {
    ViewRoute {
        id: "market".to_string(),
        route_path: "/market".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::candlestick_node(vec![
            ComponentProp {
                name: "data".to_string(),
                value: PropValue::String("candles".to_string()),
            },
            ComponentProp {
                name: "stream".to_string(),
                value: PropValue::String("/api/candles".to_string()),
            },
            ComponentProp {
                name: "variant".to_string(),
                value: PropValue::String("soft".to_string()),
            },
            ComponentProp {
                name: "scheme".to_string(),
                value: PropValue::String("surface".to_string()),
            },
            ComponentProp {
                name: "emptyLabel".to_string(),
                value: PropValue::String("Market closed".to_string()),
            },
        ])
        .expect("candlestick"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn table_route() -> ViewRoute {
    ViewRoute {
        id: "users".to_string(),
        route_path: "/users".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: dowe_components::table_node(
            vec![
                ComponentProp {
                    name: "data".to_string(),
                    value: PropValue::String("users".to_string()),
                },
                ComponentProp {
                    name: "variant".to_string(),
                    value: PropValue::String("soft".to_string()),
                },
                ComponentProp {
                    name: "scheme".to_string(),
                    value: PropValue::String("surface".to_string()),
                },
                ComponentProp {
                    name: "size".to_string(),
                    value: PropValue::String("lg".to_string()),
                },
                ComponentProp {
                    name: "striped".to_string(),
                    value: PropValue::Boolean(true),
                },
                ComponentProp {
                    name: "bordered".to_string(),
                    value: PropValue::Boolean(true),
                },
                ComponentProp {
                    name: "emptyTitle".to_string(),
                    value: PropValue::String("No users".to_string()),
                },
            ],
            vec![
                dowe_components::table_column_component(vec![
                    ComponentProp {
                        name: "field".to_string(),
                        value: PropValue::String("name".to_string()),
                    },
                    ComponentProp {
                        name: "label".to_string(),
                        value: PropValue::String("Name".to_string()),
                    },
                ])
                .expect("name column"),
                dowe_components::table_column_component(vec![
                    ComponentProp {
                        name: "field".to_string(),
                        value: PropValue::String("status".to_string()),
                    },
                    ComponentProp {
                        name: "label".to_string(),
                        value: PropValue::String("Status".to_string()),
                    },
                    ComponentProp {
                        name: "align".to_string(),
                        value: PropValue::String("end".to_string()),
                    },
                    ComponentProp {
                        name: "width".to_string(),
                        value: PropValue::String("8rem".to_string()),
                    },
                ])
                .expect("status column"),
            ],
        )
        .expect("table"),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn divider_route() -> ViewRoute {
    ViewRoute {
        id: "divider".to_string(),
        route_path: "/divider".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Divider {
            props: DividerProps {
                style: StyleProps::default(),
                orientation: DividerOrientation::Vertical,
                color: ColorFamily::Primary,
            },
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn svg_tree() -> ViewNode {
    ViewNode::Svg {
        props: SvgProps {
            style: StyleProps {
                text: Some(ResponsiveValue::scalar(
                    dowe_components::ColorToken::Tertiary,
                )),
                sizing: dowe_components::SizingProps {
                    w: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        ScaleValue::from_half_steps(16),
                    ))),
                    h: Some(ResponsiveValue::scalar(dowe_components::SizeValue::Scale(
                        ScaleValue::from_half_steps(16),
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            },
            view_box: SvgViewBox {
                min_x: "0".to_string(),
                min_y: "0".to_string(),
                width: "24".to_string(),
                height: "24".to_string(),
            },
        },
        paths: vec![
            SvgPath {
                data: "M0 0h24v24H0z".to_string(),
                fill: SvgPathFill::None,
            },
            SvgPath {
                data: "M7.5 8.744c.253.847.1 1.895-.62 2.618a.75.75 0 0 1 0 1.5".to_string(),
                fill: SvgPathFill::CurrentColor,
            },
        ],
    }
}

fn display_overlay_route() -> ViewRoute {
    ViewRoute {
        id: "overlay".to_string(),
        route_path: "/overlay".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: display_overlay_tree(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn display_overlay_tree() -> ViewNode {
    ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::Avatar {
                props: AvatarProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Success),
                        ..Default::default()
                    },
                    src: None,
                    name: Some("Ada".to_string()),
                    alt: "Ada Lovelace".to_string(),
                    size: ButtonSize::Lg,
                    status: Some(AvatarStatus::Online),
                    bordered: true,
                },
                icon: None,
            },
            ViewNode::Badge {
                props: BadgeProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Danger),
                        ..Default::default()
                    },
                    text: "3".to_string(),
                    position: OverlayCornerPosition::BottomRight,
                },
                children: vec![text("Inbox")],
            },
            ViewNode::Chip {
                props: ChipProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Info),
                        size: Some(ButtonSize::Sm),
                        ..Default::default()
                    },
                    on_close: Some("close".to_string()),
                },
                value: "Filter".to_string(),
                start: None,
                end: None,
            },
            ViewNode::Skeleton {
                props: SkeletonProps {
                    style: StyleProps::default(),
                    variant: SkeletonVariant::Rounded,
                    animation: SkeletonAnimation::Pulse,
                },
            },
            ViewNode::Modal {
                props: ModalProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    open: "modal01".to_string(),
                    on_close: Some("close".to_string()),
                    disable_overlay_close: false,
                    hide_close_button: false,
                },
                header: vec![text("Settings")],
                body: vec![text("Body")],
                footer: vec![text("Footer")],
            },
            ViewNode::AlertDialog {
                props: AlertDialogProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Danger),
                        ..Default::default()
                    },
                    open: "modal01".to_string(),
                    title: "Delete?".to_string(),
                    description: "Cannot undo.".to_string(),
                    confirm_text: "Delete".to_string(),
                    cancel_text: "Cancel".to_string(),
                    on_confirm: Some("confirm".to_string()),
                    on_cancel: Some("close".to_string()),
                    loading: false,
                },
            },
            ViewNode::Tooltip {
                props: TooltipProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Muted),
                        ..Default::default()
                    },
                    label: "More actions".to_string(),
                    position: OverlayPosition::End,
                },
                children: vec![text("Hover")],
            },
            ViewNode::Toast {
                props: ToastProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Success),
                        ..Default::default()
                    },
                    source: None,
                    kind: ToastKind::Success,
                    title: Some("Saved".to_string()),
                    description: "Profile updated".to_string(),
                    position: OverlayCornerPosition::TopRight,
                    show_icon: true,
                },
            },
            ViewNode::Dropdown {
                props: DropdownProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                },
                trigger: vec![text("Menu")],
                header: Vec::new(),
                entries: vec![OverlayEntry::Item(OverlayItemProps {
                    label: "Profile".to_string(),
                    description: None,
                    icon: None,
                    on_click: Some("profile".to_string()),
                    navigation: None,
                    disabled: false,
                })],
                footer: Vec::new(),
            },
            ViewNode::Command {
                props: CommandProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Muted),
                        ..Default::default()
                    },
                    open: Some("modal01".to_string()),
                    placeholder: "Search".to_string(),
                    empty_text: "No results".to_string(),
                    close_text: "to close".to_string(),
                    navigate_text: "Navigate".to_string(),
                    select_text: "Select".to_string(),
                    toggle_text: "Toggle".to_string(),
                    shortcut: "p".to_string(),
                    disable_global_shortcut: false,
                    show_footer: true,
                },
                entries: vec![CommandEntry::Item(OverlayItemProps {
                    label: "Home".to_string(),
                    description: None,
                    icon: None,
                    on_click: None,
                    navigation: Some(NavigationAction::Internal {
                        path: "/".to_string(),
                        fragment: None,
                        operation: NavigationOperation::Push,
                    }),
                    disabled: false,
                })],
            },
        ],
    }
}

fn display_chat_motion_route() -> ViewRoute {
    ViewRoute {
        id: "display".to_string(),
        route_path: "/display".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: display_chat_motion_tree(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn rich_control_map_route() -> ViewRoute {
    ViewRoute {
        id: "richControls".to_string(),
        route_path: "/rich-controls".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: rich_control_map_tree(),
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn display_chat_motion_tree() -> ViewNode {
    ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::AvatarGroup {
                props: AvatarGroupProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Primary),
                        variant: Some(ComponentVariant::Soft),
                        ..Default::default()
                    },
                    items: Some("people".to_string()),
                    size: ButtonSize::Sm,
                    max: Some(3),
                    auto_fit: true,
                    inline: false,
                    bordered: true,
                },
                items: vec![
                    AvatarGroupItem {
                        src: Some("/ada.png".to_string()),
                        name: Some("Ada".to_string()),
                        alt: Some("Ada Lovelace".to_string()),
                        on_click: None,
                        navigation: None,
                    },
                    AvatarGroupItem {
                        src: None,
                        name: Some("Grace".to_string()),
                        alt: Some("Grace Hopper".to_string()),
                        on_click: None,
                        navigation: None,
                    },
                ],
            },
            ViewNode::ChatBox {
                props: ChatBoxProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Surface),
                        variant: Some(ComponentVariant::Soft),
                        ..Default::default()
                    },
                    messages: "messages".to_string(),
                    mode: ChatBoxMode::Conversation,
                    current_user_id: "ada".to_string(),
                    user_name: "Ada".to_string(),
                    user_avatar: Some("/ada.png".to_string()),
                    user_status: "online".to_string(),
                    assistant_name: "Dowe".to_string(),
                    assistant_avatar: Some("/dowe.png".to_string()),
                    show_header: true,
                    placeholder: "Ask Dowe".to_string(),
                    show_attachments: true,
                    show_voice_note: true,
                    show_camera: true,
                    loading: Some("loading".to_string()),
                    sending: Some("sending".to_string()),
                    streaming: Some("streaming".to_string()),
                    has_more: Some("hasMore".to_string()),
                    on_send: None,
                    on_load_more: None,
                    on_stop: None,
                    on_voice_note: None,
                    on_file_attach: None,
                    on_camera_capture: None,
                },
            },
            ViewNode::Empty {
                props: EmptyProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Info),
                        variant: Some(ComponentVariant::Soft),
                        ..Default::default()
                    },
                    kind: EmptyKind::Result,
                    title: Some("Nothing found".to_string()),
                    description: Some("Try again".to_string()),
                    action_label: "Retry".to_string(),
                },
            },
            ViewNode::Marquee {
                props: MarqueeProps {
                    style: StyleProps::default(),
                    speed: MarqueeSpeed::Fast,
                    pause_on_hover: true,
                    reverse: true,
                    orientation: MarqueeOrientation::Horizontal,
                    fade: true,
                    fade_color: ColorToken::Background,
                    gap: ScaleValue::from_half_steps(8),
                },
                children: vec![text("Moving")],
            },
            ViewNode::TypeWriter {
                props: TypeWriterProps {
                    style: StyleProps::default(),
                    type_speed: 10,
                    delete_speed: 5,
                    after_typed: 20,
                    after_deleted: 10,
                    repeat: false,
                },
                items: vec![
                    TypeWriterItem {
                        text: "Hello".to_string(),
                    },
                    TypeWriterItem {
                        text: "World".to_string(),
                    },
                ],
            },
        ],
    }
}

fn rich_control_map_tree() -> ViewNode {
    ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::RichText {
                props: TextProps::default(),
                marks: vec![
                    RichTextMark {
                        text: "Launch".to_string(),
                        style: RichTextMarkStyle::Grad,
                        color: ColorFamily::Primary,
                    },
                    RichTextMark {
                        text: "ready".to_string(),
                        style: RichTextMarkStyle::Pill,
                        color: ColorFamily::Success,
                    },
                ],
            },
            ViewNode::Record {
                props: RecordProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Primary),
                        ..Default::default()
                    },
                    name: "voice".to_string(),
                    url: None,
                    disabled: false,
                    max_duration: Some(90),
                    on_start: None,
                    on_pause: None,
                    on_resume: None,
                    on_stop: None,
                    on_discard: None,
                    on_confirm: None,
                },
            },
            ViewNode::ToggleGroup {
                props: ToggleGroupProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Secondary),
                        ..Default::default()
                    },
                    value: Some("mode".to_string()),
                    selected: "map".to_string(),
                    size: ButtonSize::Sm,
                    wide: true,
                    vertical: false,
                    disabled: false,
                    aria_label: Some("Display mode".to_string()),
                    on_change: None,
                },
                items: vec![
                    ToggleGroupItem {
                        id: "list".to_string(),
                        label: "List".to_string(),
                        icon: None,
                    },
                    ToggleGroupItem {
                        id: "map".to_string(),
                        label: "Map".to_string(),
                        icon: None,
                    },
                ],
            },
            ViewNode::Collapsible {
                props: CollapsibleProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    label: "Details".to_string(),
                    default_open: true,
                    disabled: false,
                },
                children: vec![text("Nested content")],
            },
            ViewNode::Countdown {
                props: CountdownProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Primary),
                        ..Default::default()
                    },
                    target: "2030-01-01T00:00:00Z".to_string(),
                    show_days: true,
                    show_hours: true,
                    show_minutes: true,
                    show_seconds: true,
                    size: CountdownSize::Md,
                    days_label: "Days".to_string(),
                    hours_label: "Hours".to_string(),
                    minutes_label: "Minutes".to_string(),
                    seconds_label: "Seconds".to_string(),
                    on_complete: None,
                },
            },
            ViewNode::Map {
                props: MapProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    center_lat: "4.7109".to_string(),
                    center_lng: "-74.0721".to_string(),
                    zoom: 12,
                    height: "360px".to_string(),
                    width: "100%".to_string(),
                    show_controls: true,
                    show_scale: true,
                    show_location_control: true,
                    interactive: true,
                    route_start_lat: Some("4.7109".to_string()),
                    route_start_lng: Some("-74.0721".to_string()),
                    route_end_lat: Some("4.6500".to_string()),
                    route_end_lng: Some("-74.0900".to_string()),
                    on_location: None,
                    on_location_error: None,
                    on_route: None,
                },
                markers: vec![MapMarker {
                    id: "office".to_string(),
                    lat: "4.7109".to_string(),
                    lng: "-74.0721".to_string(),
                    label: Some("Office".to_string()),
                    popup: Some("Main office".to_string()),
                    icon: MapMarkerIcon::Start,
                    on_click: None,
                }],
                waypoints: vec![MapWaypoint {
                    lat: "4.6800".to_string(),
                    lng: "-74.0800".to_string(),
                }],
            },
        ],
    }
}

fn text(value: &str) -> ViewNode {
    ViewNode::Text {
        props: Default::default(),
        value: value.to_string(),
    }
}

fn translations() -> TranslationCatalog {
    TranslationCatalog {
        default_locale: Some("en".to_string()),
        locales: vec![
            TranslationLocale {
                locale: "en".to_string(),
                source_path: PathBuf::from("src/i18n/en.dowe"),
                values: vec![TranslationValue {
                    key: "home.hero.title".to_string(),
                    value: "Dowe builds systems.".to_string(),
                }],
            },
            TranslationLocale {
                locale: "es".to_string(),
                source_path: PathBuf::from("src/i18n/es.dowe"),
                values: vec![TranslationValue {
                    key: "home.hero.title".to_string(),
                    value: "Dowe construye sistemas.".to_string(),
                }],
            },
        ],
    }
}

fn bar_props(floating: bool) -> BarProps {
    BarProps {
        style: VariantProps {
            variant: Some(ComponentVariant::Solid),
            color: Some(ColorFamily::Surface),
            ..Default::default()
        },
        bordered: true,
        blurred: true,
        boxed: true,
        floating,
    }
}

fn responsive_scale(entries: &[(Breakpoint, u16)]) -> ResponsiveValue<ScaleValue> {
    ResponsiveValue::ordered(
        entries
            .iter()
            .map(|(breakpoint, value)| ResponsiveEntry {
                breakpoint: *breakpoint,
                value: ScaleValue::from_half_steps(value * 2),
            })
            .collect(),
    )
}

fn responsive_bool(entries: &[(Breakpoint, bool)]) -> ResponsiveValue<bool> {
    ResponsiveValue::ordered(
        entries
            .iter()
            .map(|(breakpoint, value)| ResponsiveEntry {
                breakpoint: *breakpoint,
                value: *value,
            })
            .collect(),
    )
}
