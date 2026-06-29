use dowe_components::{
    AccordionItem, AccordionProps, Align, AlertDialogProps, AudioProps, AvatarGroupItem,
    AvatarGroupProps, AvatarProps, BadgeProps, BarProps, BorderWidth, Breakpoint, ButtonSize,
    CarouselProps, CarouselSlide, ChartCommonProps, ChatBoxProps, CheckboxProps, ChipProps,
    CodeToken, CodeTokenKind, ColorFamily, ColorProps, ColorToken, ComboBoxProps, ComboOption,
    CommandEntry, CommandProps, CollapsibleProps, ComponentVariant, CountdownProps, CoverSource,
    CsvColumn, DateProps, DateRangeProps, DesignConfig, DesignTheme,
    DividerOrientation, DividerProps, DragGroup, DragItem, DrawerProps, DropzoneProps, DropdownProps, ElementProps, EmptyProps,
    FabAction, FabProps, FontConfig, FontFamily, GapSize, GapValue, GridAlignment, GridProps,
    GridTracks, INPUT_HORIZONTAL_PADDING, INPUT_MIN_HEIGHT, INPUT_TEXT_SIZE, ImageProps, Justify,
    LayoutProps, MapMarker, MapProps, MapWaypoint, MarqueeProps, ModalProps, NavMenuItem, NavMenuItemProps, NavMenuProps,
    NavigationAction, OverlayEntry, OverlayCornerPosition, OverlayItemProps, OverlayPaint,
    RadioGroupProps, RadioOption, RecordProps, ResponsiveValue, RichTextMark, RoundedSize, ScaleValue, ScaffoldProps,
    SectionBackground, SelectOption, SideNavIcon, SideNavItem, SideNavItemProps, SideNavProps,
    SideNavSize, SkeletonProps, SizeValue, SliderProps, StyleProps, SvgPath, SvgPathFill,
    SvgViewBox, TabItem, TableColumn, TableColumnAlign, TableSize, TabsProps, TabsVariant,
    TextProps, TextSize, TextWeight, ThemeToggleProps, ToastProps, ToggleGroupItem,
    ToggleGroupProps, ToggleProps, TooltipProps,
    TranslationCatalog, TypeWriterItem, TypeWriterProps, VariantProps, ViewAction,
    ViewActionKind, ViewAnimation, ViewIcon, ViewNode, ViewRequestAction, ViewRoute, ViewSignal,
    ViewSignalValue, VisibilityCondition, collect_route_font_families, compose_tree,
    node_element_props, text_spacing_em, text_typography,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

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
            content: ios_layouts(&layouts, font_config),
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
            ),
            kind: IosArtifactKind::Manifest,
            target: "ios",
        },
    ];
    files.extend(ios_route_artifacts(routes, font_config, &route_layouts));
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

fn ios_layouts(layouts: &[&ViewNode], font_config: &FontConfig) -> String {
    let mut output = String::from("import SwiftUI\n\n");
    for (index, layout) in layouts.iter().enumerate() {
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
    let activePath: String
    @ObservedObject var state: DoweReactiveState
    let navigate: (String, String, String?) -> Void
    let goBack: () -> Void
    let openExternal: (String, String) -> Void
    let content: Content

    init(
        viewportWidth: CGFloat,
        activePath: String,
        state: DoweReactiveState,
        navigate: @escaping (String, String, String?) -> Void,
        goBack: @escaping () -> Void,
        openExternal: @escaping (String, String) -> Void,
        @ViewBuilder content: () -> Content
    ) {{
        self.viewportWidth = viewportWidth
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
            render_swift_node_in_flow(
                section.node,
                8,
                &mut output,
                section.flow,
                None,
                font_config.default_family,
                &context.without_node_expression(section.node),
            );
            output.push_str("    }\n\n");
        }
        output.push_str("}\n\n");
    }
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

fn info_plist(
    font_families: &BTreeSet<FontFamily>,
    default_locale: Option<&str>,
    app_name: &str,
    app_bundle: &str,
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
    <key>NSAppTransportSecurity</key>
    <dict>
        <key>NSAllowsLocalNetworking</key>
        <true/>
    </dict>
    <key>UILaunchScreen</key>
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

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
