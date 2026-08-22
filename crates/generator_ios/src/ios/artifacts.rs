use dowe_components::{
    AccordionItem, AccordionProps, AlertDialogProps, Align, ArcChartProps, AudioProps,
    AvatarGroupItem, AvatarGroupProps, AvatarProps, BadgeProps, BarPosition, BarProps, BorderWidth,
    BottomBarTab, BoxPosition, Breakpoint, ButtonSize, CanvasBackground, CarouselProps,
    CarouselSlide, ChartCommonProps, ChatBoxProps, CheckboxProps, ChipProps, CodeTemplateSegment,
    CodeToken, CodeTokenKind, CollapsibleProps, ColorFamily, ColorProps, ColorToken, ComboBoxProps,
    ComboOption, CommandEntry, CommandProps, ComponentVariant, CountdownProps, CoverSource,
    CsvColumn, DateProps, DateRangeProps, DesignConfig, DesignTheme, DividerOrientation,
    DividerProps, DragGroup, DragItem, DrawerProps, DropdownProps, DropzoneProps, ElementProps,
    EmptyProps, FabAction, FabProps, FlexDirection, FlexItem, FontConfig, FontFamily, FormValidationRuleKind,
    GapSize, GapValue, GridAlignment, GridProps, GridTracks, INPUT_HORIZONTAL_PADDING,
    INPUT_MIN_HEIGHT, INPUT_TEXT_SIZE, ImageProps, Justify, LayoutProps, MapMarker, MapProps,
    MapWaypoint, MarqueeProps, ModalProps, NavMenuItem, NavMenuItemProps, NavMenuProps,
    NavigationAction, OverlayCornerPosition, OverlayEntry, OverlayItemProps, OverlayPaint,
    PieChartProps, PositionProps, RadioGroupProps, RadioOption, RailNavItem, RailNavProps,
    RecordProps, ResponsiveValue, RichTextMark, RoundedSize, SIDE_NAV_SUBMENU_ARROW_PATH,
    ScaffoldProps, ScaleValue, SectionBackground, SelectOption, SelectOptionEach, ShadowSize,
    SideNavIcon, SideNavItem, SideNavItemProps, SideNavProps, SideNavSize, SidebarProps, SizeValue,
    SkeletonProps, SliderProps, StyleProps, SvgLineCap, SvgLineJoin, SvgPath, SvgPathFill,
    SvgProps, SvgViewBox, TabItem, TableColumn, TableColumnAlign, TableSize, TabsProps,
    TabsVariant, TextAlign, TextProps, TextSize, TextWeight, ThemeSelectProps, ThemeToggleProps,
    ToastProps, ToggleGroupItem, ToggleGroupKind, ToggleGroupProps, ToggleProps, TooltipProps,
    TranslationCatalog, TypeWriterItem, TypeWriterProps, VariantProps, ViewAction, ViewActionKind,
    ViewAnimation, ViewConstant, ViewForm, ViewFormFieldKind, ViewGesture, ViewNode,
    ViewRequestAction, ViewRoute, ViewSignal, ViewSignalValue, ViewTransition, VisibilityCondition,
    collect_route_font_families, collect_view_forms, compose_tree, empty_icon, fixed_box_nodes,
    fixed_fab_nodes, form_control_min_height, form_control_text_size, node_child_groups,
    node_element_props, phone_countries, phone_country_flag_icon, side_nav_memory_key,
    side_nav_submenu_arrow_icon, solar_control_icon, text_binding_path, text_spacing_em,
    text_typography, view_icon,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IosOutput {
    pub files: Vec<IosArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IosArtifact {
    pub relative_path: PathBuf,
    pub content: String,
    pub kind: IosArtifactKind,
    pub target: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IosArtifactKind {
    Entrypoint,
    GeneratedView,
    Routing,
    Layouts,
    Pages,
    Theme,
    Responsive,
    Manifest,
    Localization,
    DevEntrypoint,
}

pub fn generate_ios(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
) -> IosOutput {
    generate_ios_with_app_and_translations(
        routes,
        font_config,
        design_config,
        environment,
        &TranslationCatalog::default(),
        "Dowe Dev",
        "dev.dowe.generated",
    )
}

pub fn generate_ios_with_translations(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    translations: &TranslationCatalog,
) -> IosOutput {
    generate_ios_with_app_and_translations(
        routes,
        font_config,
        design_config,
        environment,
        translations,
        "Dowe Dev",
        "dev.dowe.generated",
    )
}

pub fn generate_ios_with_app_and_translations(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    translations: &TranslationCatalog,
    app_name: &str,
    app_bundle: &str,
) -> IosOutput {
    generate_ios_with_app_translations_and_icons(
        routes,
        font_config,
        design_config,
        environment,
        translations,
        app_name,
        app_bundle,
        false,
    )
}

pub fn generate_ios_with_app_translations_and_icons(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    translations: &TranslationCatalog,
    app_name: &str,
    app_bundle: &str,
    has_app_icon: bool,
) -> IosOutput {
    let font_families = font_config.effective_families(&collect_route_font_families(routes));
    let (layouts, route_layouts) = reusable_ios_layouts(routes);
    let mut files = vec![
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DoweIosApp.swift"),
            content: ios_app(),
            kind: IosArtifactKind::Entrypoint,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/GeneratedViews.swift"),
            content: generated_views_index(),
            kind: IosArtifactKind::GeneratedView,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DoweRouting.swift"),
            content: ios_routing(routes),
            kind: IosArtifactKind::Routing,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DoweLayouts.swift"),
            content: ios_layouts_index(),
            kind: IosArtifactKind::Layouts,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DowePages.swift"),
            content: generated_views(routes, font_config, &font_families, design_config),
            kind: IosArtifactKind::Pages,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DoweEnvironment.swift"),
            content: ios_environment(environment),
            kind: IosArtifactKind::GeneratedView,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DoweTheme.swift"),
            content: ios_theme(design_config),
            kind: IosArtifactKind::Theme,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/DoweResponsive.swift"),
            content: ios_responsive(),
            kind: IosArtifactKind::Responsive,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/Info.plist"),
            content: info_plist(
                &font_families,
                translations.default_locale.as_deref(),
                app_name,
                app_bundle,
                routes.iter().any(|route| {
                    ios_canvas_motion(&route.layout_tree) || ios_canvas_motion(&route.page_tree)
                }),
                routes.iter().any(|route| {
                    ios_video_playback(&route.layout_tree) || ios_video_playback(&route.page_tree)
                }),
                routes.iter().any(|route| {
                    ios_tree_has_camera(&route.layout_tree) || ios_tree_has_camera(&route.page_tree)
                }),
                routes.iter().any(|route| {
                    ios_tree_has_microphone(&route.layout_tree)
                        || ios_tree_has_microphone(&route.page_tree)
                }),
                has_app_icon,
            ),
            kind: IosArtifactKind::Manifest,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/dev/DoweIosDevHost.swift"),
            content: ios_dev_host(),
            kind: IosArtifactKind::DevEntrypoint,
            target: "ios",
        },
        IosArtifact {
            relative_path: PathBuf::from("apps/ios/dev/DoweIosViewModule.swift"),
            content: ios_dev_module_factory(),
            kind: IosArtifactKind::DevEntrypoint,
            target: "ios",
        },
    ];
    files.extend(ios_layout_artifacts(&layouts, font_config));
    files.extend(ios_route_artifacts(routes, font_config, &route_layouts));
    if ios_has_dynamic_icon(routes) {
        files.extend(ios_dynamic_icon_catalog_artifacts());
    }
    if routes
        .iter()
        .any(|route| ios_tree_has_phone(&route.layout_tree) || ios_tree_has_phone(&route.page_tree))
    {
        files.extend(ios_phone_catalog_artifacts());
    }
    files.extend(ios_translation_artifacts(translations));
    IosOutput { files }
}

fn ios_route_artifacts(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    route_layouts: &[Option<usize>],
) -> Vec<IosArtifact> {
    routes
        .iter()
        .zip(route_layouts)
        .map(|(route, layout_index)| IosArtifact {
            relative_path: PathBuf::from(format!(
                "apps/ios/DowePage{}.swift",
                swift_view_name(&route.route_path)
            )),
            content: generated_route_view(route, font_config, *layout_index),
            kind: IosArtifactKind::Pages,
            target: "ios",
        })
        .collect()
}

fn ios_translation_artifacts(catalog: &TranslationCatalog) -> Vec<IosArtifact> {
    catalog
        .locales
        .iter()
        .map(|locale| IosArtifact {
            relative_path: PathBuf::from(format!(
                "apps/ios/{}.lproj/Localizable.strings",
                locale.locale
            )),
            content: ios_localizable_strings(locale),
            kind: IosArtifactKind::Localization,
            target: "ios",
        })
        .collect()
}

fn ios_phone_catalog_artifacts() -> Vec<IosArtifact> {
    const SHARD_SIZE: usize = 24;

    let countries = phone_countries()
        .iter()
        .filter_map(|country| {
            let icon = phone_country_flag_icon(country.code)?;
            Some(format!(
                "        DowePhoneCountry(code: {}, name: {}, dialCode: {}, flag: DoweControlIcon(viewBox: {}, paths: {}))",
                swift_string_literal(country.code),
                swift_string_literal(country.name),
                swift_string_literal(country.dial),
                swift_svg_view_box(&icon.props.view_box),
                swift_svg_paths(&icon.paths)
            ))
        })
        .collect::<Vec<_>>();
    let mut files = countries
        .chunks(SHARD_SIZE)
        .enumerate()
        .map(|(index, countries)| IosArtifact {
            relative_path: PathBuf::from(format!(
                "apps/ios/DowePhoneCatalogShard{index}.swift"
            )),
            content: format!(
                "import SwiftUI\n\nenum DowePhoneCatalogShard{index} {{\n    static let countries: [DowePhoneCountry] = [\n{}\n    ]\n}}\n",
                countries.join(",\n")
            ),
            kind: IosArtifactKind::GeneratedView,
            target: "ios",
        })
        .collect::<Vec<_>>();
    let append_shards = files
        .iter()
        .enumerate()
        .map(|(index, _)| {
            format!("        countries.append(contentsOf: DowePhoneCatalogShard{index}.countries)")
        })
        .collect::<Vec<_>>()
        .join("\n");
    files.push(IosArtifact {
        relative_path: PathBuf::from("apps/ios/DowePhoneCatalog.swift"),
        content: format!(
            "import SwiftUI\n\nenum DowePhoneCatalog {{\n    static let countries: [DowePhoneCountry] = {{\n        var countries: [DowePhoneCountry] = []\n{append_shards}\n        return countries\n    }}()\n}}\n"
        ),
        kind: IosArtifactKind::GeneratedView,
        target: "ios",
    });
    files
}

fn ios_localizable_strings(locale: &dowe_components::TranslationLocale) -> String {
    locale
        .values
        .iter()
        .map(|value| {
            format!(
                "\"{}\" = \"{}\";",
                escape_swift(&value.key),
                escape_swift(&value.value)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn ios_environment(environment: &[(String, String)]) -> String {
    let mut values = environment
        .iter()
        .map(|(name, value)| format!("    static let {} = \"{}\"", name, escape_swift(value)))
        .collect::<Vec<_>>();
    if !environment.iter().any(|(name, _)| name == "BACKEND_URL") {
        values.push("    static let BACKEND_URL = \"\"".to_string());
    }
    let values = values.join("\n");
    format!(
        r#"import Foundation

enum DoweEnvironment {{
{values}
}}
"#
    )
}

fn generated_views_index() -> String {
    "import SwiftUI\n".to_string()
}

fn ios_routing(routes: &[ViewRoute]) -> String {
    let route_paths = routes
        .iter()
        .map(|route| format!("        \"{}\",", route.route_path))
        .collect::<Vec<_>>()
        .join("\n");
    let initial = routes_first_path(routes);
    let deep_links = routes
        .iter()
        .map(|route| {
            format!(
                "        \"dowe-dev://generated{}\",",
                if route.route_path == "/" {
                    "/"
                } else {
                    route.route_path.as_str()
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let sections = routes
        .iter()
        .map(|route| {
            let values = route
                .sections
                .iter()
                .map(|section| format!("\"{}\"", escape_swift(&section.id)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("        \"{}\": [{values}],", route.route_path)
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"import SwiftUI

enum DoweRoutes {{
    static let initialPath = "{initial}"
    static let paths = [
{route_paths}
    ]
    static let sections: [String: [String]] = [
{sections}
    ]
    static let deepLinks = [
{deep_links}
    ]
}}
"#
    )
}

fn routes_first_path(routes: &[ViewRoute]) -> &str {
    routes
        .first()
        .map(|route| route.route_path.as_str())
        .unwrap_or("/")
}

fn ios_layouts_index() -> String {
    "import SwiftUI\n".to_string()
}

fn ios_layout_artifacts(layouts: &[&ViewNode], font_config: &FontConfig) -> Vec<IosArtifact> {
    layouts
        .iter()
        .enumerate()
        .map(|(index, layout)| IosArtifact {
            relative_path: PathBuf::from(format!("apps/ios/DoweLayout{index}.swift")),
            content: ios_layout(index, layout, font_config),
            kind: IosArtifactKind::Layouts,
            target: "ios",
        })
        .collect()
}

fn ios_layout(index: usize, layout: &ViewNode, font_config: &FontConfig) -> String {
    let mut output = String::from("import SwiftUI\n\n");
    let sections = ios_layout_sections(layout);
    let expressions = sections
        .iter()
        .enumerate()
        .map(|(section_index, section)| {
            (
                swift_node_key(section.node),
                format!("layoutSection{section_index}()"),
            )
        })
        .collect();
    let context = SwiftReactiveContext::default()
        .with_children_expression("content")
        .with_node_expressions(expressions);
    output.push_str(&format!(
        r#"struct DoweLayout{index}<Content: View>: View {{
    let viewportWidth: CGFloat
    let viewportHeight: CGFloat
    let activePath: String
    @ObservedObject var state: DoweReactiveState
    let navigate: (String, String, String?) -> Void
    let goBack: () -> Void
    let openExternal: (String, String) -> Void
    let content: Content

    init(
        viewportWidth: CGFloat,
        viewportHeight: CGFloat,
        activePath: String,
        state: DoweReactiveState,
        navigate: @escaping (String, String, String?) -> Void,
        goBack: @escaping () -> Void,
        openExternal: @escaping (String, String) -> Void,
        @ViewBuilder content: () -> Content
    ) {{
        self.viewportWidth = viewportWidth
        self.viewportHeight = viewportHeight
        self.activePath = activePath
        self.state = state
        self.navigate = navigate
        self.goBack = goBack
        self.openExternal = openExternal
        self.content = content()
    }}

    var body: some View {{
        Group {{
"#
    ));
    render_swift_node_in_flow(
        layout,
        12,
        &mut output,
        NativeFlow::Block,
        None,
        font_config.default_family,
        &context,
    );
    output.push_str("        }\n    }\n\n");
    for (section_index, section) in sections.iter().enumerate() {
        output.push_str(&format!(
            "    @ViewBuilder\n    private func layoutSection{section_index}() -> some View {{\n"
        ));
        let section_context = section.scopes.iter().fold(
            context.without_node_expression(section.node),
            |context, scope| context.with_scope(scope.constants, scope.signals, scope.actions),
        );
        render_swift_node_in_flow(
            section.node,
            8,
            &mut output,
            section.flow,
            None,
            font_config.default_family,
            &section_context,
        );
        output.push_str("    }\n\n");
    }
    output.push_str("}\n");
    output
}

fn ios_theme(design_config: &DesignConfig) -> String {
    swift_theme_module(design_config)
}

fn ios_responsive() -> String {
    r#"import SwiftUI

enum DoweResponsiveModule {
    static let generated = true
}
"#
    .to_string()
}

fn ios_app() -> String {
    r#"import SwiftUI

@main
struct DoweIosApp: App {
    var body: some Scene {
        WindowGroup {
            DoweRootView()
        }
    }
}
"#
    .to_string()
}

fn ios_dev_host() -> String {
    r#"import Darwin
import Foundation
import SwiftUI
import UIKit
@main
struct DoweIosDevHostApp: App {
    var body: some Scene {
        WindowGroup {
            DoweIosDevModuleHost()
                .ignoresSafeArea()
        }
    }
}

struct DoweIosDevModuleHost: UIViewControllerRepresentable {
    func makeCoordinator() -> DoweIosDevModuleCoordinator {
        DoweIosDevModuleCoordinator()
    }

    func makeUIViewController(context: Context) -> UIViewController {
        let controller = UIViewController()
        controller.view.backgroundColor = .systemBackground
        context.coordinator.start(controller)
        return controller
    }

    func updateUIViewController(_ controller: UIViewController, context: Context) {
    }
}

final class DoweIosDevModuleCoordinator: NSObject {
    private let endpointKey = "dowe.hmr.endpoint"
    private let activeVersionKey = "dowe.hmr.version"
    private weak var container: UIViewController?
    private var activeController: UIViewController?
    private var activeVersion = ""
    private var activeRoute = "/"
    private var attemptedVersion = ""
    private var moduleEndpoint: String?
    private var handles: [UnsafeMutableRawPointer] = []
    private var waitingView: UIView?
    private var timer: Timer?
    private var loading = false

    func start(_ controller: UIViewController) {
        container = controller
        showWaitingState(in: controller)
        moduleEndpoint = resolveEndpoint()
        restoreCachedModule()
        poll()
        timer = Timer.scheduledTimer(withTimeInterval: 0.3, repeats: true) { [weak self] _ in
            self?.poll()
        }
    }

    private func resolveEndpoint() -> String? {
        let arguments = ProcessInfo.processInfo.arguments
        if let index = arguments.firstIndex(of: "--dowe-dev-server"), arguments.indices.contains(index + 1) {
            let value = arguments[index + 1]
            if !value.isEmpty {
                UserDefaults.standard.set(value, forKey: endpointKey)
                return value
            }
        }
        return UserDefaults.standard.string(forKey: endpointKey)
    }

    private func showWaitingState(in controller: UIViewController) {
        let spinner = UIActivityIndicatorView(style: .large)
        spinner.startAnimating()

        let title = UILabel()
        title.font = .preferredFont(forTextStyle: .headline)
        title.text = "Preparing Dowe app"
        title.textAlignment = .center
        title.textColor = .label

        let detail = UILabel()
        detail.font = .preferredFont(forTextStyle: .subheadline)
        detail.text = "The first iOS build can take a few minutes."
        detail.textAlignment = .center
        detail.textColor = .secondaryLabel
        detail.numberOfLines = 0

        let stack = UIStackView(arrangedSubviews: [spinner, title, detail])
        stack.axis = .vertical
        stack.alignment = .center
        stack.spacing = 12
        stack.translatesAutoresizingMaskIntoConstraints = false
        controller.view.addSubview(stack)
        NSLayoutConstraint.activate([
            stack.centerXAnchor.constraint(equalTo: controller.view.centerXAnchor),
            stack.centerYAnchor.constraint(equalTo: controller.view.centerYAnchor),
            stack.leadingAnchor.constraint(greaterThanOrEqualTo: controller.view.layoutMarginsGuide.leadingAnchor),
            stack.trailingAnchor.constraint(lessThanOrEqualTo: controller.view.layoutMarginsGuide.trailingAnchor),
        ])
        waitingView = stack
    }

    private func moduleFile(version: String) -> URL? {
        guard let root = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first else {
            return nil
        }
        let directory = root.appendingPathComponent("DoweModules", isDirectory: true)
        do {
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
            return directory.appendingPathComponent("dowe-module-\(version).dylib")
        } catch {
            return nil
        }
    }

    private func restoreCachedModule() {
        guard
            let version = UserDefaults.standard.string(forKey: activeVersionKey),
            !version.isEmpty,
            let file = moduleFile(version: version),
            FileManager.default.fileExists(atPath: file.path)
        else {
            return
        }
        _ = apply(file, version: version)
    }

    private func poll() {
        persistCurrentPath()
        guard !loading, let endpoint = moduleEndpoint, let url = URL(string: endpoint + "/_dowe/dev/modules/manifest.json?dowe_hmr=\(UUID().uuidString)") else {
            return
        }
        var request = URLRequest(url: url)
        request.cachePolicy = .reloadIgnoringLocalCacheData
        request.setValue("no-cache", forHTTPHeaderField: "Cache-Control")
        loading = true
        URLSession.shared.dataTask(with: request) { [weak self] data, _, _ in
            guard let self else { return }
            defer { self.loading = false }
            guard
                let data,
                let value = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                let targets = value["targets"] as? [String: Any],
                let ios = targets["ios"] as? [String: Any],
                let version = ios["version"] as? String,
                let path = ios["path"] as? String,
                version != self.activeVersion,
                let moduleUrl = URL(string: endpoint + path + "?dowe_hmr=\(version)")
            else {
                return
            }
            var moduleRequest = URLRequest(url: moduleUrl)
            moduleRequest.cachePolicy = .reloadIgnoringLocalCacheData
            moduleRequest.setValue("no-cache", forHTTPHeaderField: "Cache-Control")
            URLSession.shared.dataTask(with: moduleRequest) { [weak self] data, _, _ in
                guard let self, let data, let file = self.moduleFile(version: version) else { return }
                do {
                    try data.write(to: file, options: .atomic)
                    DispatchQueue.main.async {
                        _ = self.apply(file, version: version)
                    }
                } catch {
                }
            }.resume()
        }.resume()
    }

    @discardableResult
    private func apply(_ file: URL, version: String) -> Bool {
        guard let container else { return false }
        let handle = dlopen(file.path, RTLD_NOW | RTLD_LOCAL)
        guard let handle, let symbol = dlsym(handle, "dowe_create_root_view_controller") else {
            if let handle { dlclose(handle) }
            return false
        }
        typealias Factory = @convention(c) (UnsafePointer<CChar>?) -> UnsafeMutableRawPointer
        let factory = unsafeBitCast(symbol, to: Factory.self)
        let path = currentPath()
        let pointer = path.withCString { factory($0) }
        let next = Unmanaged<UIViewController>.fromOpaque(pointer).takeRetainedValue()
        activeController?.willMove(toParent: nil)
        activeController?.view.removeFromSuperview()
        activeController?.removeFromParent()
        container.addChild(next)
        next.view.frame = container.view.bounds
        next.view.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        container.view.addSubview(next.view)
        next.didMove(toParent: container)
        waitingView?.removeFromSuperview()
        waitingView = nil
        activeController = next
        activeVersion = version
        activeRoute = path
        UserDefaults.standard.set(version, forKey: activeVersionKey)
        handles.append(handle)
        attemptedVersion = version
        return true
    }

    private func currentPath() -> String {
        if
            let activeController,
            activeController.responds(to: NSSelectorFromString("doweCurrentPath")),
            let value = activeController.value(forKey: "doweCurrentPath") as? String
        {
            return value
        }
        return activeRoute
    }

    private func persistCurrentPath() {
        guard activeController != nil else { return }
        activeRoute = currentPath()
    }
}
"#
    .to_string()
}

fn ios_dev_module_factory() -> String {
    r#"import SwiftUI
import UIKit

final class DoweIosDevRouteTracker {
    var path: String

    init(path: String) {
        self.path = path
    }
}

@objc(DoweIosDevModuleController___DOWE_IOS_SOURCE_REVISION__)
final class DoweIosDevModuleController: UIHostingController<AnyView> {
    let routeTracker: DoweIosDevRouteTracker

    @objc dynamic var doweCurrentPath: String {
        routeTracker.path
    }

    init(path: String) {
        let tracker = DoweIosDevRouteTracker(path: path)
        routeTracker = tracker
        let root = DoweRootView(initialPath: path) { next in
            tracker.path = next
        }
        super.init(rootView: AnyView(root))
    }

    @MainActor required dynamic init?(coder: NSCoder) {
        nil
    }
}

@_cdecl("dowe_create_root_view_controller")
public func doweCreateRootViewController(_ path: UnsafePointer<CChar>?) -> UnsafeMutableRawPointer {
    let initialPath = path.map { String(cString: $0) } ?? DoweRoutes.initialPath
    let resolved = DoweRoutes.paths.contains(initialPath) ? initialPath : DoweRoutes.initialPath
    let controller = DoweIosDevModuleController(path: resolved)
    return Unmanaged.passRetained(controller).toOpaque()
}
"#
    .to_string()
}

fn info_plist(
    font_families: &BTreeSet<FontFamily>,
    default_locale: Option<&str>,
    app_name: &str,
    app_bundle: &str,
    uses_motion: bool,
    uses_video: bool,
    uses_camera: bool,
    uses_microphone: bool,
    has_app_icon: bool,
) -> String {
    let fonts = font_families
        .iter()
        .filter(|font| font.catalog_entry().package_assets)
        .flat_map(|font| {
            font.catalog_entry()
                .weights
                .iter()
                .map(|weight| format!("        <string>Fonts/{}.ttf</string>", weight.asset_stem))
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n");
    let motion = if uses_motion {
        "    <key>NSMotionUsageDescription</key>\n    <string>Use device motion to control interactive Canvas scenes.</string>\n"
    } else {
        ""
    };
    let background_playback = if uses_video {
        "    <key>UIBackgroundModes</key>\n    <array>\n        <string>audio</string>\n    </array>\n"
    } else {
        ""
    };
    let camera_usage = if uses_camera {
        "    <key>NSCameraUsageDescription</key>\n    <string>Use the camera to capture a photo.</string>\n"
    } else {
        ""
    };
    let microphone_usage = if uses_microphone {
        "    <key>NSMicrophoneUsageDescription</key>\n    <string>Use the microphone to record audio.</string>\n"
    } else {
        ""
    };
    let app_icon = if has_app_icon {
        r#"    <key>CFBundleIconName</key>
    <string>AppIcon</string>
    <key>CFBundleIcons</key>
    <dict>
        <key>CFBundlePrimaryIcon</key>
        <dict>
            <key>CFBundleIconFiles</key>
            <array>
                <string>AppIcon60x60</string>
            </array>
            <key>CFBundleIconName</key>
            <string>AppIcon</string>
        </dict>
    </dict>
    <key>CFBundleIcons~ipad</key>
    <dict>
        <key>CFBundlePrimaryIcon</key>
        <dict>
            <key>CFBundleIconFiles</key>
            <array>
                <string>AppIcon60x60</string>
                <string>AppIcon76x76</string>
            </array>
            <key>CFBundleIconName</key>
            <string>AppIcon</string>
        </dict>
    </dict>
"#
    } else {
        ""
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>{}</string>
    <key>CFBundleDisplayName</key>
    <string>{}</string>
    <key>CFBundleIdentifier</key>
    <string>{}</string>
    <key>CFBundleExecutable</key>
    <string>DoweIosApp</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
{app_icon}    <key>NSAppTransportSecurity</key>
    <dict>
        <key>NSAllowsLocalNetworking</key>
        <true/>
    </dict>
{motion}{background_playback}{camera_usage}{microphone_usage}    <key>UILaunchScreen</key>
    <dict/>
    <key>UIAppFonts</key>
    <array>
{fonts}
    </array>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>{}</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>dowe-dev</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
"#,
        default_locale.unwrap_or("en"),
        escape_xml(app_name),
        escape_xml(app_bundle),
        escape_xml(app_name),
        escape_xml(app_bundle)
    )
}

fn ios_canvas_motion(node: &ViewNode) -> bool {
    if matches!(node, ViewNode::Canvas { props } if props.on_motion.is_some()) {
        return true;
    }
    node_child_groups(node)
        .into_iter()
        .flatten()
        .any(ios_canvas_motion)
}

fn ios_video_playback(node: &ViewNode) -> bool {
    if matches!(node, ViewNode::Video { .. }) {
        return true;
    }
    node_child_groups(node)
        .into_iter()
        .flatten()
        .any(ios_video_playback)
}

fn ios_tree_has_camera(node: &ViewNode) -> bool {
    if matches!(node, ViewNode::Camera { .. }) {
        return true;
    }
    node_child_groups(node)
        .into_iter()
        .flatten()
        .any(ios_tree_has_camera)
}

fn ios_tree_has_microphone(node: &ViewNode) -> bool {
    if matches!(node, ViewNode::Microphone { .. }) {
        return true;
    }
    node_child_groups(node)
        .into_iter()
        .flatten()
        .any(ios_tree_has_microphone)
}

fn ios_tree_has_phone(node: &ViewNode) -> bool {
    if matches!(node, ViewNode::Phone { .. }) {
        return true;
    }
    node_child_groups(node)
        .into_iter()
        .flatten()
        .any(ios_tree_has_phone)
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
