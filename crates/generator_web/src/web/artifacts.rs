use dowe_components::{
    AccordionItem, AccordionProps, AlertDialogProps, AlertProps, Align, ArcChartProps,
    AreaChartProps, AudioProps, AvatarGroupItem, AvatarGroupProps, AvatarProps, BadgeProps,
    BannerProps, BarChartProps, BarProps, BottomBarTab, BoxPosition, BrandProps, Breakpoint,
    ButtonSize, CandlestickProps, CanvasBackground, CanvasProps, CarouselIndicatorType,
    CarouselOrientation, CarouselProps, CarouselSlide, CarouselVariant, ChartCommonProps,
    ChatBoxProps, CheckboxProps, ChipProps, CodeProps, CodeTemplateSegment, CollapsibleProps,
    ColorFamily, ColorProps, ColorToken, ComboBoxProps, ComboOption, CommandEntry, CommandProps,
    ComponentVariant, CountdownProps, CoverSource, CsvColumn, CsvFieldProps, DateProps,
    DateRangeProps, DesignConfig, DesignTheme, DeviceProps, DividerProps, DragDropProps, DragGroup,
    DragItem, DrawerProps, DropdownProps, DropzoneProps, EditorProps, ElementProps, EmptyKind,
    EmptyProps, FabAction, FabProps, FlexDirection, FontConfig, FontFamily, GapSize, GapValue,
    FORM_CONTROL_FLOATING_HEIGHT_INCREMENT, GridAlignment, GridProps,
    INPUT_HORIZONTAL_PADDING, IframeProps, ImageCropperProps, ImageProps, Justify,
    LayoutProps,
    LineChartProps, MapMarker, MapProps, MapWaypoint, MarqueeProps, ModalProps, NativeExternalMode,
    NavMenuItem, NavMenuItemProps, NavMenuProps, NavigationAction, NavigationOperation,
    OverlayEntry, OverlayItemProps, OverlayPaint, PasswordProps, PhoneProps,
    PieChartProps, PinKind, PinProps, RadioGroupProps, RadioOption, RailNavItem,
    RailNavItemProps, RailNavProps, RecordProps, ResponsiveValue, RichTextMark, ScaffoldProps,
    ScaleValue, SectionBackground, SelectOption, SelectOptionEach, ShadowSize, SideNavIcon,
    SideNavItem, SideNavItemProps, SideNavProps, SidebarProps, SizeValue, SkeletonProps,
    SliderProps, StyleProps, SvgLineCap, SvgLineJoin, SvgPath, SvgPathFill, SvgProps, TabItem,
    TableColumn, TableColumnAlign, TableProps, TabsProps, TabsVariant, TextProps, TextSize,
    TextSpacing, TextWeight, TextareaProps, ThemeSelectProps, ThemeToggleProps, ToastKind,
    ToastProps, ToggleGroupItem, ToggleGroupKind, ToggleGroupProps, ToggleProps, TooltipProps,
    TranslationCatalog, TypeWriterItem, TypeWriterProps, VariantProps, VideoProps, ViewAction,
    ViewActionKind, ViewAnimation, ViewAssignAction, ViewConstant, ViewFunctionStatement,
    ViewGesture, ViewIcon, ViewMetadata, ViewNavigationAction, ViewNode, ViewRequestAction,
    ViewResetAction, ViewSection,
    ViewSignal, ViewSignalValue, VisibilityCondition, WebTarget, collect_node_font_families,
    form_control_min_height, form_control_text_size, ordered_phone_countries, phone_countries, phone_country,
    phone_country_flag_icon,
    side_nav_memory_key, side_nav_submenu_arrow_icon, solar_control_icon, text_binding_path, text_spacing_em,
    text_typography, text_weight_number,
};
use dowe_minifier::minify_js;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebOutput {
    pub chunks: Vec<GeneratedChunk>,
    pub pages: Vec<ViewPage>,
    pub translation_chunks: Vec<GeneratedTranslationChunk>,
    pub default_locale: Option<String>,
    pub router_js: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedChunk {
    pub id: String,
    pub file_name: String,
    pub relative_path: PathBuf,
    pub css_file_name: String,
    pub css_relative_path: PathBuf,
    pub css_content: String,
    pub source_path: PathBuf,
    pub content: String,
    pub kind: ChunkKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedTranslationChunk {
    pub id: String,
    pub locale: String,
    pub relative_path: PathBuf,
    pub source_path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkKind {
    Layout,
    Page,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewPage {
    pub id: String,
    pub route_path: String,
    pub source_path: PathBuf,
    pub layout_tree: ViewNode,
    pub page_tree: ViewNode,
    pub body_html: String,
    pub html_document: String,
    pub layout_text: String,
    pub page_text: String,
    pub layout_chunk_id: String,
    pub page_chunk_id: String,
    pub layout_chunk_ids: Vec<String>,
    pub js_chunks: Vec<String>,
    pub css_chunks: Vec<String>,
    pub boundaries: Vec<String>,
    pub sections: Vec<ViewSection>,
    pub navigation_actions: Vec<ViewNavigationAction>,
    pub metadata: Vec<ViewMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebArtifact {
    pub relative_path: PathBuf,
    pub content: String,
    pub kind: WebArtifactKind,
    pub target: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebArtifactKind {
    Chunk,
    Css,
    Manifest,
    Html,
}

pub fn build_layout_chunk(
    root: &Path,
    source_path: &Path,
    source: &str,
    layout_tree: &ViewNode,
) -> GeneratedChunk {
    let id = short_id("layout", source);
    let expression = js_render_expression(layout_tree);
    let definition = page_definition_json(layout_tree);
    let content = minify_js(&format!(
        r#"export const chunkId="{id}";export const doweLayout={definition};export function render(children=""){{return {expression};}}"#
    ));
    let css_content = css_for_tree(layout_tree);

    GeneratedChunk {
        id: id.clone(),
        file_name: format!("{id}.js"),
        relative_path: PathBuf::from(format!("web/chunks/layouts/{id}.js")),
        css_file_name: format!("{id}.css"),
        css_relative_path: PathBuf::from(format!("web/chunks/layouts/{id}.css")),
        css_content,
        source_path: source_path
            .strip_prefix(root)
            .unwrap_or(source_path)
            .to_path_buf(),
        content,
        kind: ChunkKind::Layout,
    }
}

pub fn build_page_chunk(
    root: &Path,
    source_path: &Path,
    source: &str,
    page_tree: &ViewNode,
) -> GeneratedChunk {
    let id = short_id("page", source);
    let expression = js_render_expression(page_tree);
    let definition = page_definition_json(page_tree);
    let content = minify_js(&format!(
        r#"export const chunkId="{id}";export const dowePage={definition};export function render(){{return {expression};}}"#
    ));
    let css_content = css_for_tree(page_tree);

    GeneratedChunk {
        id: id.clone(),
        file_name: format!("{id}.js"),
        relative_path: PathBuf::from(format!("web/chunks/pages/{id}.js")),
        css_file_name: format!("{id}.css"),
        css_relative_path: PathBuf::from(format!("web/chunks/pages/{id}.css")),
        css_content,
        source_path: source_path
            .strip_prefix(root)
            .unwrap_or(source_path)
            .to_path_buf(),
        content,
        kind: ChunkKind::Page,
    }
}

pub fn build_translation_chunks(
    root: &Path,
    catalog: &TranslationCatalog,
) -> Vec<GeneratedTranslationChunk> {
    catalog
        .locales
        .iter()
        .map(|locale| {
            let values = locale
                .values
                .iter()
                .map(|value| {
                    format!(
                        r#""{}":"{}""#,
                        escape_json(&value.key),
                        escape_json(&value.value)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let source = format!("{}\n{values}", locale.locale);
            let id = short_id("i18n", &source);
            GeneratedTranslationChunk {
                id: id.clone(),
                locale: locale.locale.clone(),
                relative_path: PathBuf::from(format!("web/chunks/i18n/{id}.js")),
                source_path: locale
                    .source_path
                    .strip_prefix(root)
                    .unwrap_or(&locale.source_path)
                    .to_path_buf(),
                content: minify_js(&format!(
                    r#"export const locale="{}";export const translations={{{values}}};"#,
                    escape_js(&locale.locale)
                )),
            }
        })
        .collect()
}

pub fn render_page_body(layout_tree: &ViewNode, page_tree: &ViewNode) -> String {
    let page_html = render_html(page_tree, None);
    render_html(layout_tree, Some(&page_html))
}

pub fn render_routed_page_body(
    layout_tree: &ViewNode,
    page_tree: &ViewNode,
    layout_chunk_ids: &[String],
    page_chunk_id: &str,
) -> String {
    let page_html = format!(
        r#"<div data-dowe-boundary="page:{page_chunk_id}">{}</div>"#,
        render_html(page_tree, None)
    );
    let body = render_html(layout_tree, Some(&page_html));

    if let Some(layout_chunk_id) = layout_chunk_ids.first() {
        format!(r#"<div data-dowe-boundary="layout:{layout_chunk_id}">{body}</div>"#)
    } else {
        page_html
    }
}

pub fn render_page_document(page: &ViewPage) -> String {
    render_page_document_with_icons(page, None, None)
}

pub fn render_page_document_with_icons(
    page: &ViewPage,
    favicon: Option<&str>,
    apple_touch_icon: Option<&str>,
) -> String {
    let css_links = page
        .css_chunks
        .iter()
        .map(|path| format!(r#"<link rel="stylesheet" href="/{path}">"#))
        .collect::<String>();
    let chunk_scripts = page
        .js_chunks
        .iter()
        .map(|path| format!(r#"<script type="module" src="/{path}"></script>"#))
        .collect::<String>();
    let theme_script = theme_bootstrap_script();
    let icon_links = icon_links(favicon, apple_touch_icon);
    let metadata = metadata_head(&page.metadata);

    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=5, viewport-fit=cover, interactive-widget=resizes-content">{icon_links}{metadata}{theme_script}<link rel="stylesheet" href="/design.css">{css_links}<script type="module" src="/router.js"></script>{chunk_scripts}</head><body><div id="dowe-app" data-dowe-route="{}">{}</div></body></html>"#,
        escape_attr(&page.route_path),
        page.body_html
    )
}

fn metadata_head(metadata: &[ViewMetadata]) -> String {
    let mut head = String::new();
    let mut has_title = false;
    for entry in metadata {
        match entry.name.as_str() {
            "title" => {
                has_title = true;
                head.push_str(&format!(
                    r#"<title data-dowe-meta>{}</title>"#,
                    escape_html_text(&entry.content)
                ));
            }
            "canonical" => head.push_str(&format!(
                r#"<link data-dowe-meta rel="canonical" href="{}">"#,
                escape_attr(&entry.content)
            )),
            name if name.starts_with("og:") => head.push_str(&format!(
                r#"<meta data-dowe-meta property="{}" content="{}">"#,
                escape_attr(name),
                escape_attr(&entry.content)
            )),
            name => head.push_str(&format!(
                r#"<meta data-dowe-meta name="{}" content="{}">"#,
                escape_attr(name),
                escape_attr(&entry.content)
            )),
        }
    }
    if !has_title {
        head.insert_str(0, "<title>Dowe</title>");
    }
    head
}

fn escape_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn icon_links(favicon: Option<&str>, apple_touch_icon: Option<&str>) -> String {
    let mut links = match favicon {
        Some(path) => format!(
            r#"<link rel="icon" type="image/png" sizes="32x32" href="{}">"#,
            escape_attr(path)
        ),
        None => r#"<link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect width='64' height='64' rx='16' fill='%23111827'/%3E%3Cpath d='M18 16h12c12 0 20 6 20 16s-8 16-20 16H18zm10 8v16h3c7 0 11-3 11-8s-4-8-11-8z' fill='%23fff'/%3E%3C/svg%3E">"#.to_string(),
    };
    if let Some(path) = apple_touch_icon {
        links.push_str(&format!(
            r#"<link rel="apple-touch-icon" href="{}">"#,
            escape_attr(path)
        ));
    }
    links
}

fn theme_bootstrap_script() -> &'static str {
    r#"<script>!function(){document.documentElement.classList.add("dowe-entrance-pending");try{var k="theme-preference",t=localStorage.getItem(k);if(!t){t=window.matchMedia&&window.matchMedia("(prefers-color-scheme: dark)").matches?"dark":"light";localStorage.setItem(k,t)}if(t&&t!=="light")document.documentElement.setAttribute("data-dowe-theme",t);else document.documentElement.removeAttribute("data-dowe-theme")}catch(e){}}();</script>"#
}

pub fn web_artifacts(
    web: &WebOutput,
    font_config: &FontConfig,
    design_config: &DesignConfig,
) -> Vec<WebArtifact> {
    web_artifacts_for_target(web, font_config, design_config, Path::new(""), "web")
}

pub fn web_artifacts_for_target(
    web: &WebOutput,
    font_config: &FontConfig,
    design_config: &DesignConfig,
    prefix: &Path,
    target: &'static str,
) -> Vec<WebArtifact> {
    let mut artifacts = Vec::new();

    for chunk in &web.chunks {
        artifacts.push(WebArtifact {
            relative_path: prefixed_path(prefix, &chunk.relative_path),
            content: chunk.content.clone(),
            kind: WebArtifactKind::Chunk,
            target,
        });
        artifacts.push(WebArtifact {
            relative_path: prefixed_path(prefix, &chunk.css_relative_path),
            content: chunk.css_content.clone(),
            kind: WebArtifactKind::Css,
            target,
        });
    }

    for chunk in &web.translation_chunks {
        artifacts.push(WebArtifact {
            relative_path: prefixed_path(prefix, &chunk.relative_path),
            content: chunk.content.clone(),
            kind: WebArtifactKind::Chunk,
            target,
        });
    }

    artifacts.push(WebArtifact {
        relative_path: prefixed_path(prefix, Path::new("web/design.css")),
        content: design_css_for_web(web, font_config, design_config),
        kind: WebArtifactKind::Css,
        target,
    });

    artifacts.push(WebArtifact {
        relative_path: prefixed_path(prefix, Path::new("web/router.js")),
        content: web.router_js.clone(),
        kind: WebArtifactKind::Chunk,
        target,
    });

    artifacts.push(WebArtifact {
        relative_path: prefixed_path(prefix, Path::new("web/manifest.json")),
        content: manifest(web),
        kind: WebArtifactKind::Manifest,
        target,
    });

    if let Some(page) = web.pages.first() {
        artifacts.push(WebArtifact {
            relative_path: prefixed_path(prefix, Path::new("web/index.html")),
            content: static_html_document(&page.html_document, ""),
            kind: WebArtifactKind::Html,
            target,
        });
    }

    for page in &web.pages {
        artifacts.push(WebArtifact {
            relative_path: prefixed_path(
                prefix,
                Path::new(&format!("web/pages/{}.html", page_file_name(page))),
            ),
            content: static_html_document(&page.html_document, "../"),
            kind: WebArtifactKind::Html,
            target,
        });
    }

    artifacts
}

fn prefixed_path(prefix: &Path, path: &Path) -> PathBuf {
    if prefix.as_os_str().is_empty() {
        path.to_path_buf()
    } else {
        prefix.join(path)
    }
}

fn static_html_document(document: &str, asset_prefix: &str) -> String {
    let document = document
        .replace(
            r#"href="/design.css""#,
            &format!(r#"href="{asset_prefix}design.css""#),
        )
        .replace(
            r#"href="/chunks/"#,
            &format!(r#"href="{asset_prefix}chunks/"#),
        )
        .replace(
            r#"src="/router.js""#,
            &format!(r#"src="{asset_prefix}router.js""#),
        )
        .replace(
            r#"src="/chunks/"#,
            &format!(r#"src="{asset_prefix}chunks/"#),
        );
    rewrite_static_route_hrefs(&document, asset_prefix)
}

fn rewrite_static_route_hrefs(document: &str, asset_prefix: &str) -> String {
    let mut output = String::new();
    let mut rest = document;

    while let Some(index) = rest.find(r#" href="/"#) {
        output.push_str(&rest[..index]);
        let value_start = index + r#" href=""#.len();
        let Some(value_end_offset) = rest[value_start..].find('"') else {
            output.push_str(&rest[index..]);
            return output;
        };
        let value_end = value_start + value_end_offset;
        let href = &rest[value_start..value_end];
        output.push_str(r#" href=""#);
        output.push_str(&static_route_href(href, asset_prefix));
        output.push('"');
        rest = &rest[value_end + 1..];
    }

    output.push_str(rest);
    output
}

fn static_route_href(href: &str, asset_prefix: &str) -> String {
    let (path, fragment) = href.split_once('#').unwrap_or((href, ""));
    if let Some(asset) = path.strip_prefix("/assets/") {
        return format!("{asset_prefix}assets/{asset}");
    }
    if let Some(icon) = path.strip_prefix("/icons/") {
        return format!("{asset_prefix}icons/{icon}");
    }
    let file = if path == "/" {
        if asset_prefix.is_empty() {
            "index.html".to_string()
        } else {
            "../index.html".to_string()
        }
    } else {
        let name = format!("{}.html", path.trim_matches('/').replace('/', "-"));
        if asset_prefix.is_empty() {
            format!("pages/{name}")
        } else {
            name
        }
    };

    if fragment.is_empty() {
        file
    } else {
        format!("{file}#{fragment}")
    }
}

pub fn manifest(web: &WebOutput) -> String {
    let chunks = web
        .chunks
        .iter()
        .map(|chunk| {
            let kind = match chunk.kind {
                ChunkKind::Layout => "layout",
                ChunkKind::Page => "page",
            };

            format!(
                r#"{{"kind":"{kind}","id":"{}","file":"{}","source":"{}"}}"#,
                chunk.id,
                chunk.relative_path.display(),
                chunk.source_path.display()
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let routes = web
        .pages
        .iter()
        .map(|page| {
            let file_name = page_file_name(page);
            let layout_stack = page
                .layout_chunk_ids
                .iter()
                .map(|id| format!(r#""{id}""#))
                .collect::<Vec<_>>()
                .join(",");
            let js_chunks = page
                .js_chunks
                .iter()
                .map(|path| format!(r#""{path}""#))
                .collect::<Vec<_>>()
                .join(",");
            let css_chunks = page
                .css_chunks
                .iter()
                .map(|path| format!(r#""{path}""#))
                .collect::<Vec<_>>()
                .join(",");
            let boundaries = page
                .boundaries
                .iter()
                .map(|boundary| format!(r#""{boundary}""#))
                .collect::<Vec<_>>()
                .join(",");
            let sections = page
                .sections
                .iter()
                .map(|section| format!(r#""{}""#, escape_json(&section.id)))
                .collect::<Vec<_>>()
                .join(",");
            let navigation_actions = page
                .navigation_actions
                .iter()
                .map(navigation_action_json)
                .collect::<Vec<_>>()
                .join(",");
            let metadata = metadata_json(&page.metadata);
            format!(
                r#"{{"id":"{}","path":"{}","layoutChunk":"{}","pageChunk":"{}","layoutStack":[{layout_stack}],"jsChunks":[{js_chunks}],"cssChunks":[{css_chunks}],"boundaries":[{boundaries}],"sections":[{sections}],"navigationActions":[{navigation_actions}],"metadata":{metadata},"staticFile":"web/pages/{file_name}.html"}}"#,
                escape_json(&page.id),
                escape_json(&page.route_path),
                escape_json(&page.layout_chunk_id),
                escape_json(&page.page_chunk_id)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let translation_chunks = web
        .translation_chunks
        .iter()
        .map(|chunk| {
            format!(
                r#"{{"locale":"{}","id":"{}","file":"{}","source":"{}"}}"#,
                escape_json(&chunk.locale),
                escape_json(&chunk.id),
                chunk.relative_path.display(),
                chunk.source_path.display()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let default_locale = json_optional_string(web.default_locale.as_deref());

    format!(
        r#"{{"chunks":[{chunks}],"translationChunks":[{translation_chunks}],"defaultLocale":{default_locale},"routes":[{routes}],"history":{{"push":true,"replace":true,"back":true}},"externalPolicies":{{"web":["self","blank"],"desktop":["system","webview"],"android":["system","webview"],"ios":["system","webview"]}},"deepLinks":{{"scheme":"dowe-dev","routesFromManifest":true}}}}"#
    )
}

fn metadata_json(metadata: &[ViewMetadata]) -> String {
    let entries = metadata
        .iter()
        .map(|entry| {
            format!(
                r#"{{"name":"{}","content":"{}"}}"#,
                escape_json(&entry.name),
                escape_json(&entry.content)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{entries}]")
}

pub fn design_css() -> String {
    design_css_for_fonts(
        &BTreeSet::new(),
        &FontConfig::default(),
        &DesignConfig::default(),
    )
}

fn design_css_for_web(
    web: &WebOutput,
    font_config: &FontConfig,
    design_config: &DesignConfig,
) -> String {
    let mut fonts = BTreeSet::new();
    for page in &web.pages {
        collect_node_font_families(&page.layout_tree, &mut fonts);
        collect_node_font_families(&page.page_tree, &mut fonts);
    }
    design_css_for_fonts(&fonts, font_config, design_config)
}
