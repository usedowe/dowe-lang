use crate::error::{DoweError, DoweResult};
use crate::model::{
    DoweType, DoweTypeField, EnvironmentConfig, EnvironmentVisibility, TranslationCatalog,
    ViewNode, ViewPage, ViewPlatform, ViewRoute, ViewTargetRoutes, WebOutput,
};
use crate::parser::source_ast::{
    SourceFile, SourceNode, SourceObjectEntry, SourceProp, SourceValue,
};
use crate::parser::source_i18n::validate_view_i18n_keys;
use crate::parser::source_imports::resolve_import;
use crate::parser::source_parser::parse_source_file;
use crate::parser::source_types::{TypeRegistry, validate_source_value_type};
use dowe_components::{
    BuiltinComponent, COMPONENT_REGISTRY, ComponentError, ComponentProp, PropScalar, PropValue,
    ResponsivePropEntry, ViewAction, ViewActionKind, ViewAssignAction, ViewNavigationAction,
    ViewRequestAction, ViewRequestMethod, ViewResetAction, ViewSection, ViewSignal,
    ViewSignalValue, VisibilityCondition, accordion_component_node, accordion_item_component,
    alert_dialog_component_node, arc_chart_component_node, area_chart_component_node,
    audio_component_node, avatar_component_node, avatar_group_component_node,
    avatar_group_item_component, badge_component_node, bar_chart_component_node,
    bar_component_node, candlestick_node, carousel_component_node, carousel_slide_component,
    chat_box_component_node, checkbox_component_node, children_node, chip_component_node,
    code_node, collapsible_component_node, color_component_node, combo_box_component_node,
    combo_option_component, command_component_node, command_group_component, compose_tree,
    container_component_node, countdown_component_node, csv_column_component,
    csv_field_component_node, date_component_node, date_range_component_node, divider_node,
    drag_drop_component_node, drag_group_component, drag_item_component, dropdown_component_node,
    dropzone_component_node, editor_component_node, empty_component_node, fab_action_component,
    fab_component_node, first_text, image_component_node, image_cropper_component_node, input_node,
    line_chart_component_node, map_component_node, map_marker_component, map_waypoint_component,
    marquee_component_node, modal_component_node, nav_menu_component_node, nav_menu_item_component,
    nav_menu_megamenu_component, nav_menu_submenu_component, navigation_action, node_child_groups,
    node_element_props, overlay_icon_component, overlay_item_component,
    password_field_component_node, phone_field_component_node, pie_chart_component_node,
    pin_field_component_node, radio_group_component_node, radio_option_component,
    record_component_node, rich_text_component_node, rich_text_mark_component,
    scaffold_component_node, select_node, select_option_component, side_nav_component_node,
    side_nav_header_component, side_nav_icon_component, side_nav_item_component,
    side_nav_submenu_component, sidebar_component_node, skeleton_component_node,
    slider_component_node, svg_component_node, svg_path_component, table_column_component,
    table_node, tabs_component_node, tabs_tab_component, text_component_node, text_node,
    textarea_component_node, theme_toggle_component_node, toast_component_node,
    toggle_component_node, toggle_group_component_node, toggle_group_item_component,
    tooltip_component_node, type_writer_component_node, type_writer_item_component,
    validate_view_tree, video_node,
};
use dowe_generator_web::{
    build_layout_chunk, build_page_chunk, build_translation_chunks, render_page_document,
    render_routed_page_body, router_js,
};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ParsedViews {
    pub web: WebOutput,
    pub desktop_web: WebOutput,
    pub routes: ViewTargetRoutes,
}

pub fn parse_views_file(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
    translations: &TranslationCatalog,
) -> DoweResult<ParsedViews> {
    let imports = view_imports(root, file)?;
    let declarations = view_declarations(file)?;
    let used = declarations
        .iter()
        .flat_map(used_components)
        .collect::<HashSet<_>>();
    for local in imports.keys() {
        if !used.contains(local) {
            return Err(DoweError::at_path(
                &file.path,
                format!("import `{local}` is not used by the route graph"),
            ));
        }
    }

    let mut context = RouteBuildContext {
        root,
        views_path: &file.path,
        imports,
        modules: HashMap::new(),
        chunks: Vec::new(),
        chunk_indexes: HashMap::new(),
        outputs: PlatformRouteOutputs::default(),
        environment,
    };

    for declaration in declarations {
        context.visit_route(&declaration, "/", Vec::new(), ViewPlatform::all().to_vec())?;
    }

    validate_navigation(&context.outputs.web.pages)?;
    validate_navigation(&context.outputs.desktop.pages)?;
    validate_navigation(&context.outputs.android.pages)?;
    validate_navigation(&context.outputs.ios.pages)?;
    let routes = ViewTargetRoutes {
        web: context.outputs.web.routes,
        desktop: context.outputs.desktop.routes,
        android: context.outputs.android.routes,
        ios: context.outputs.ios.routes,
    };
    validate_view_i18n_keys(&file.path, &routes, translations)?;
    let translation_chunks = build_translation_chunks(root, translations);
    let web = web_output_for(
        context.outputs.web.pages.clone(),
        &context.chunks,
        &translation_chunks,
        translations,
    );
    let desktop_web = web_output_for(
        context.outputs.desktop.pages.clone(),
        &context.chunks,
        &translation_chunks,
        translations,
    );
    Ok(ParsedViews {
        web,
        desktop_web,
        routes,
    })
}

pub fn validate_design_copilot_dowe(source: &str) -> DoweResult<ViewNode> {
    let path = Path::new("dowe-copilot.dowe");
    let file = parse_source_file(Path::new(""), path, source.to_string())?;
    let types = TypeRegistry::parse(&file.path, &file.nodes)?;
    let node = if file.nodes.len() == 1 && matches!(file.nodes[0].name.as_str(), "page" | "layout")
    {
        export_tree(
            &file.nodes[0],
            file.nodes[0].name == "layout",
            &EnvironmentConfig::default(),
            &types,
        )?
    } else {
        single_tree(path, lower_node_sequence(&file.nodes, false)?)?
    };
    validate_view_tree(&node).map_err(|error| DoweError::at_path(path, error.to_string()))?;
    Ok(node)
}

pub(crate) fn validate_view_source(
    file: &SourceFile,
    environment: &EnvironmentConfig,
) -> DoweResult<ViewNode> {
    let types = TypeRegistry::parse(&file.path, &file.nodes)?;
    let root_node = single_export(file)?;
    match root_node.name.as_str() {
        "layout" => export_tree(root_node, true, environment, &types),
        "page" => export_tree(root_node, false, environment, &types),
        _ => Err(node_error(
            root_node,
            "view modules must export a layout or page",
        )),
    }
}

#[derive(Clone)]
struct ViewImport {
    path: PathBuf,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ImportedViewKind {
    Layout,
    Page,
}

#[derive(Clone)]
struct ViewDeclaration {
    path: String,
    component: String,
    platforms: Option<Vec<ViewPlatform>>,
    children: Vec<ViewDeclaration>,
}

#[derive(Clone)]
struct ParsedViewModule {
    tree: ViewNode,
    source: String,
    path: PathBuf,
    kind: ImportedViewKind,
}

#[derive(Clone)]
struct RouteLayout {
    tree: ViewNode,
    chunk_id: String,
    js_path: String,
    css_path: String,
}

struct RoutePage {
    tree: ViewNode,
    path: PathBuf,
    chunk_id: String,
    js_path: String,
    css_path: String,
}

struct RouteBuildContext<'a> {
    root: &'a Path,
    views_path: &'a Path,
    imports: HashMap<String, ViewImport>,
    modules: HashMap<String, ParsedViewModule>,
    chunks: Vec<dowe_generator_web::GeneratedChunk>,
    chunk_indexes: HashMap<String, usize>,
    outputs: PlatformRouteOutputs,
    environment: &'a EnvironmentConfig,
}

#[derive(Default)]
struct PlatformRouteOutputs {
    web: PlatformRouteOutput,
    desktop: PlatformRouteOutput,
    android: PlatformRouteOutput,
    ios: PlatformRouteOutput,
}

#[derive(Default)]
struct PlatformRouteOutput {
    pages: Vec<ViewPage>,
    routes: Vec<ViewRoute>,
    seen_paths: HashSet<String>,
}

fn web_output_for(
    mut pages: Vec<ViewPage>,
    chunks: &[dowe_generator_web::GeneratedChunk],
    translation_chunks: &[dowe_generator_web::GeneratedTranslationChunk],
    translations: &TranslationCatalog,
) -> WebOutput {
    for page in &mut pages {
        page.html_document = render_page_document(page);
    }
    let needed_chunks = pages
        .iter()
        .flat_map(|page| {
            page.layout_chunk_ids
                .iter()
                .chain(std::iter::once(&page.page_chunk_id))
        })
        .cloned()
        .collect::<HashSet<_>>();
    let chunks = chunks
        .iter()
        .filter(|chunk| needed_chunks.contains(&chunk.id))
        .cloned()
        .collect::<Vec<_>>();
    let mut web = WebOutput {
        chunks,
        pages,
        translation_chunks: translation_chunks.to_vec(),
        default_locale: translations.default_locale.clone(),
        router_js: String::new(),
    };
    web.router_js = router_js(&web);
    web
}

impl PlatformRouteOutputs {
    fn add_page(
        &mut self,
        platform: ViewPlatform,
        page: ViewPage,
        route: ViewRoute,
        views_path: &Path,
    ) -> DoweResult<()> {
        let output = self.output_mut(platform);
        if !output.seen_paths.insert(page.route_path.clone()) {
            return Err(DoweError::at_path(
                views_path,
                format!(
                    "duplicate view path `{}` for platform `{}`",
                    page.route_path,
                    platform.as_str()
                ),
            ));
        }
        output.pages.push(page);
        output.routes.push(route);
        Ok(())
    }

    fn output_mut(&mut self, platform: ViewPlatform) -> &mut PlatformRouteOutput {
        match platform {
            ViewPlatform::Web => &mut self.web,
            ViewPlatform::Desktop => &mut self.desktop,
            ViewPlatform::Android => &mut self.android,
            ViewPlatform::Ios => &mut self.ios,
        }
    }
}

fn effective_platforms(
    declaration: &ViewDeclaration,
    parent_platforms: Vec<ViewPlatform>,
    views_path: &Path,
) -> DoweResult<Vec<ViewPlatform>> {
    let declared = declaration
        .platforms
        .clone()
        .unwrap_or_else(|| ViewPlatform::all().to_vec());
    let platforms = ViewPlatform::all()
        .iter()
        .copied()
        .filter(|platform| parent_platforms.contains(platform) && declared.contains(platform))
        .collect::<Vec<_>>();
    if platforms.is_empty() {
        return Err(DoweError::at_path(
            views_path,
            format!(
                "route path `{}` has no platforms in common with its parent",
                declaration.path
            ),
        ));
    }
    Ok(platforms)
}

impl RouteBuildContext<'_> {
    fn visit_route(
        &mut self,
        declaration: &ViewDeclaration,
        parent_path: &str,
        mut layouts: Vec<RouteLayout>,
        parent_platforms: Vec<ViewPlatform>,
    ) -> DoweResult<()> {
        let platforms = effective_platforms(declaration, parent_platforms, self.views_path)?;
        let route_path = normalize_route_path(parent_path, &declaration.path);
        if declaration.children.is_empty() {
            self.add_page_route(declaration, route_path, layouts, platforms)
        } else {
            let layout = self.layout_for(&declaration.component)?;
            layouts.push(layout);
            for child in &declaration.children {
                self.visit_route(child, &route_path, layouts.clone(), platforms.clone())?;
            }
            Ok(())
        }
    }

    fn add_page_route(
        &mut self,
        declaration: &ViewDeclaration,
        route_path: String,
        layouts: Vec<RouteLayout>,
        platforms: Vec<ViewPlatform>,
    ) -> DoweResult<()> {
        let page = self.page_for(&declaration.component)?;
        let layout_tree = combine_layout_stack(&layouts);
        let layout_chunk_ids = layouts
            .iter()
            .map(|layout| layout.chunk_id.clone())
            .collect::<Vec<_>>();
        let mut js_chunks = layouts
            .iter()
            .map(|layout| layout.js_path.clone())
            .collect::<Vec<_>>();
        let mut css_chunks = layouts
            .iter()
            .map(|layout| layout.css_path.clone())
            .collect::<Vec<_>>();
        js_chunks.push(page.js_path.clone());
        css_chunks.push(page.css_path.clone());
        let mut boundaries = layout_chunk_ids
            .iter()
            .map(|id| format!("layout:{id}"))
            .collect::<Vec<_>>();
        boundaries.push(format!("page:{}", page.chunk_id));
        let body_html =
            render_routed_page_body(&layout_tree, &page.tree, &layout_chunk_ids, &page.chunk_id);
        let layout_text = first_text(&layout_tree).unwrap_or_default();
        let page_text = first_text(&page.tree)
            .ok_or_else(|| DoweError::at_path(&page.path, "page must contain Text"))?;
        let id = route_id(&route_path);
        let composed_tree = compose_tree(&layout_tree, &page.tree);
        let sections = collect_sections(&page.path, &composed_tree)?;
        let navigation_actions = collect_navigation_actions(&composed_tree, &id);

        let view_page = ViewPage {
            id: id.clone(),
            route_path: route_path.clone(),
            source_path: page.path.clone(),
            layout_tree: layout_tree.clone(),
            page_tree: page.tree.clone(),
            body_html,
            html_document: String::new(),
            layout_text,
            page_text,
            layout_chunk_id: layout_chunk_ids.first().cloned().unwrap_or_default(),
            page_chunk_id: page.chunk_id.clone(),
            layout_chunk_ids,
            js_chunks,
            css_chunks,
            boundaries,
            sections: sections.clone(),
            navigation_actions: navigation_actions.clone(),
        };
        let view_route = ViewRoute {
            id,
            route_path,
            layout_tree,
            page_tree: page.tree,
            sections,
            navigation_actions,
        };
        for platform in platforms {
            self.outputs.add_page(
                platform,
                view_page.clone(),
                view_route.clone(),
                self.views_path,
            )?;
        }
        Ok(())
    }

    fn layout_for(&mut self, component: &str) -> DoweResult<RouteLayout> {
        let module = self.module_for(component, ImportedViewKind::Layout)?;
        let chunk = self.chunk_for(component, &module)?;
        Ok(RouteLayout {
            tree: module.tree,
            chunk_id: chunk.id,
            js_path: strip_web_prefix(&chunk.relative_path),
            css_path: strip_web_prefix(&chunk.css_relative_path),
        })
    }

    fn page_for(&mut self, component: &str) -> DoweResult<RoutePage> {
        let module = self.module_for(component, ImportedViewKind::Page)?;
        let chunk = self.chunk_for(component, &module)?;
        Ok(RoutePage {
            tree: module.tree,
            path: module.path,
            chunk_id: chunk.id,
            js_path: strip_web_prefix(&chunk.relative_path),
            css_path: strip_web_prefix(&chunk.css_relative_path),
        })
    }

    fn module_for(
        &mut self,
        component: &str,
        expected: ImportedViewKind,
    ) -> DoweResult<ParsedViewModule> {
        if let Some(module) = self.modules.get(component) {
            if module.kind != expected {
                return Err(DoweError::at_path(
                    self.views_path,
                    format!("component `{component}` is used in the wrong route position"),
                ));
            }
            return Ok(module.clone());
        }
        let import = self.imports.get(component).cloned().ok_or_else(|| {
            DoweError::at_path(
                self.views_path,
                format!("missing import for view component `{component}`"),
            )
        })?;
        let source = fs::read_to_string(&import.path)
            .map_err(|error| DoweError::at_path(&import.path, error.to_string()))?;
        let file = parse_source_file(self.root, &import.path, source)?;
        let types = TypeRegistry::parse(&file.path, &file.nodes)?;
        let root_node = single_export(&file)?;
        let kind = match root_node.name.as_str() {
            "layout" => ImportedViewKind::Layout,
            "page" => ImportedViewKind::Page,
            _ => {
                return Err(node_error(
                    root_node,
                    "view modules must export a layout or page",
                ));
            }
        };
        if kind != expected {
            return Err(DoweError::at_path(
                self.views_path,
                format!("component `{component}` is used in the wrong route position"),
            ));
        }
        let export_name = root_node
            .args
            .first()
            .and_then(SourceValue::as_required_string)
            .ok_or_else(|| node_error(root_node, "layout or page export must declare a name"))?;
        if export_name != component {
            return Err(node_error(
                root_node,
                format!("export `{export_name}` does not match import `{component}`"),
            ));
        }
        let tree = export_tree(
            root_node,
            kind == ImportedViewKind::Layout,
            self.environment,
            &types,
        )?;
        let module = ParsedViewModule {
            tree,
            source: file.source,
            path: file.path,
            kind,
        };
        self.modules.insert(component.to_string(), module.clone());
        Ok(module)
    }

    fn chunk_for(
        &mut self,
        component: &str,
        module: &ParsedViewModule,
    ) -> DoweResult<dowe_generator_web::GeneratedChunk> {
        if let Some(index) = self.chunk_indexes.get(component) {
            return Ok(self.chunks[*index].clone());
        }
        let chunk = match module.kind {
            ImportedViewKind::Layout => {
                build_layout_chunk(self.root, &module.path, &module.source, &module.tree)
            }
            ImportedViewKind::Page => {
                build_page_chunk(self.root, &module.path, &module.source, &module.tree)
            }
        };
        let index = self.chunks.len();
        self.chunks.push(chunk.clone());
        self.chunk_indexes.insert(component.to_string(), index);
        Ok(chunk)
    }
}

fn view_imports(root: &Path, file: &SourceFile) -> DoweResult<HashMap<String, ViewImport>> {
    let mut imports = HashMap::new();
    for import in &file.imports {
        let path = resolve_import(root, &file.path, import)?;
        if imports
            .insert(import.local.clone(), ViewImport { path })
            .is_some()
        {
            return Err(DoweError::at_path(
                &file.path,
                format!("duplicate import `{}`", import.local),
            ));
        }
    }
    Ok(imports)
}

fn view_declarations(file: &SourceFile) -> DoweResult<Vec<ViewDeclaration>> {
    let views = file
        .nodes
        .iter()
        .filter(|node| node.name == "views")
        .collect::<Vec<_>>();
    if views.len() != 1 {
        return Err(DoweError::at_path(
            &file.path,
            "`src/views.dowe` must declare one `views` block",
        ));
    }
    let declarations = views[0]
        .children
        .iter()
        .map(parse_route_node)
        .collect::<DoweResult<Vec<_>>>()?;
    if declarations.is_empty() {
        return Err(DoweError::at_path(
            &file.path,
            "views must declare at least one route",
        ));
    }
    Ok(declarations)
}

fn parse_route_node(node: &SourceNode) -> DoweResult<ViewDeclaration> {
    match node.name.as_str() {
        "route" => {
            reject_unknown_route_props(node, &["path", "layout", "platform"])?;
            Ok(ViewDeclaration {
                path: required_path_prop(node)?,
                component: required_prop_string(node, "layout")?,
                platforms: optional_platforms_prop(node)?,
                children: node
                    .children
                    .iter()
                    .map(parse_route_node)
                    .collect::<DoweResult<Vec<_>>>()?,
            })
        }
        "page" => {
            reject_unknown_route_props(node, &["path", "component", "platform"])?;
            Ok(ViewDeclaration {
                path: required_path_prop(node)?,
                component: required_prop_string(node, "component")?,
                platforms: optional_platforms_prop(node)?,
                children: Vec::new(),
            })
        }
        _ => Err(node_error(
            node,
            "route graph only accepts `route` and `page`",
        )),
    }
}

fn reject_unknown_route_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.contains(&prop.name.as_str()) {
            return Err(prop_error(
                prop,
                format!("`{}` does not support `{}`", node.name, prop.name),
            ));
        }
    }
    Ok(())
}

fn optional_platforms_prop(node: &SourceNode) -> DoweResult<Option<Vec<ViewPlatform>>> {
    let Some(prop) = node.prop("platform") else {
        return Ok(None);
    };
    let values = match &prop.value {
        SourceValue::String(value) => vec![platform_from_string(prop, value)?],
        SourceValue::Array(values) => {
            if values.is_empty() {
                return Err(prop_error(
                    prop,
                    "`platform` must include at least one value",
                ));
            }
            values
                .iter()
                .map(|value| match value {
                    SourceValue::String(value) => platform_from_string(prop, value),
                    _ => Err(quoted_static_string_error(prop)),
                })
                .collect::<DoweResult<Vec<_>>>()?
        }
        _ => return Err(quoted_static_string_error(prop)),
    };
    let mut seen = BTreeSet::new();
    let mut platforms = Vec::new();
    for platform in values {
        if !seen.insert(platform) {
            return Err(prop_error(
                prop,
                format!("duplicate platform `{}`", platform.as_str()),
            ));
        }
        platforms.push(platform);
    }
    Ok(Some(
        ViewPlatform::all()
            .iter()
            .copied()
            .filter(|platform| platforms.contains(platform))
            .collect(),
    ))
}

fn platform_from_string(prop: &SourceProp, value: &str) -> DoweResult<ViewPlatform> {
    ViewPlatform::from_name(value).ok_or_else(|| {
        prop_error(
            prop,
            format!(
                "`platform` must be one of \"web\", \"desktop\", \"android\" or \"ios\", got `{value}`"
            ),
        )
    })
}

fn export_tree(
    node: &SourceNode,
    allow_children: bool,
    environment: &EnvironmentConfig,
    types: &TypeRegistry,
) -> DoweResult<ViewNode> {
    let tree = lower_export_tree(node, allow_children, types)?;
    validate_view_tree(&tree).map_err(|error| node_error(node, error.to_string()))?;
    validate_reactive_view_tree(&node.location.path, &tree, environment)?;
    Ok(tree)
}

fn lower_export_tree(
    node: &SourceNode,
    allow_children: bool,
    types: &TypeRegistry,
) -> DoweResult<ViewNode> {
    let mut signals = Vec::new();
    let mut actions = Vec::new();
    let mut visual_nodes = Vec::new();
    let scope_name = node
        .args
        .first()
        .and_then(SourceValue::as_required_string)
        .unwrap_or_default();

    for child in &node.children {
        match child.name.as_str() {
            "signal" => signals.push(parse_signal(child, &node.name, &scope_name, types)?),
            "action" => actions.push(parse_action(child, &node.name, &scope_name)?),
            _ => visual_nodes.push(child.clone()),
        }
    }

    let tree = single_tree(
        &node.location.path,
        lower_node_sequence(&visual_nodes, allow_children)?,
    )?;
    if signals.is_empty() && actions.is_empty() {
        Ok(tree)
    } else {
        Ok(ViewNode::Scope {
            signals,
            actions,
            children: vec![tree],
        })
    }
}

fn parse_signal(
    node: &SourceNode,
    scope_kind: &str,
    scope_name: &str,
    types: &TypeRegistry,
) -> DoweResult<ViewSignal> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "`signal` must declare one name and no children",
        ));
    }
    let name = node.args[0]
        .as_required_string()
        .ok_or_else(|| node_error(node, "`signal` must declare a name"))?;
    let value = node
        .prop("value")
        .ok_or_else(|| node_error(node, "`signal` requires `value`"))?;
    let initial = signal_value(&value.value, node)?;
    let schema = optional_prop_string(node, "type")?
        .map(|name| {
            let schema = types.resolve(node, &name)?;
            validate_source_value_type(node, &value.value, &schema, "signal value")?;
            Ok::<ViewSignalValue, DoweError>(view_schema_value(&schema))
        })
        .transpose()?;
    Ok(ViewSignal {
        id: reactive_id("signal", scope_kind, scope_name, node, &name),
        name,
        initial,
        schema,
    })
}

fn view_schema_value(value: &DoweType) -> ViewSignalValue {
    match value {
        DoweType::Unknown | DoweType::Null => ViewSignalValue::Null,
        DoweType::Bool => ViewSignalValue::Bool(false),
        DoweType::Number => ViewSignalValue::Number("0".to_string()),
        DoweType::String => ViewSignalValue::String(String::new()),
        DoweType::Array(item) => ViewSignalValue::Array(vec![view_schema_value(item)]),
        DoweType::Object(fields) => ViewSignalValue::Object(
            fields
                .iter()
                .map(|field| (field.name.clone(), view_schema_field_value(field)))
                .collect(),
        ),
    }
}

fn view_schema_field_value(field: &DoweTypeField) -> ViewSignalValue {
    view_schema_value(&field.value)
}

fn signal_value(value: &SourceValue, node: &SourceNode) -> DoweResult<ViewSignalValue> {
    match value {
        SourceValue::Null => Ok(ViewSignalValue::Null),
        SourceValue::Boolean(value) => Ok(ViewSignalValue::Bool(*value)),
        SourceValue::Number(value) => Ok(ViewSignalValue::Number(value.clone())),
        SourceValue::String(value) => Ok(ViewSignalValue::String(value.clone())),
        SourceValue::Bareword(_) => Err(node_error(
            node,
            "`signal value` string literals must use double quotes",
        )),
        SourceValue::Array(values) => values
            .iter()
            .map(|value| signal_value(value, node))
            .collect::<DoweResult<Vec<_>>>()
            .map(ViewSignalValue::Array),
        SourceValue::Object(entries) => {
            let mut values = Vec::new();
            for entry in entries {
                match entry {
                    SourceObjectEntry::KeyValue { key, value } => {
                        values.push((key.clone(), signal_value(value, node)?));
                    }
                    SourceObjectEntry::Spread(_) => {
                        return Err(node_error(node, "`signal` value cannot use object spread"));
                    }
                }
            }
            Ok(ViewSignalValue::Object(values))
        }
    }
}

fn parse_action(node: &SourceNode, scope_kind: &str, scope_name: &str) -> DoweResult<ViewAction> {
    if node.args.len() != 1 {
        return Err(node_error(node, "`action` must declare one name"));
    }
    let name = node.args[0]
        .as_required_string()
        .ok_or_else(|| node_error(node, "`action` must declare a name"))?;
    if node.children.len() != 1 {
        return Err(node_error(node, "`action` must contain one operation"));
    }
    let operation = &node.children[0];
    let kind = match operation.name.as_str() {
        "request" => ViewActionKind::Request(parse_request_action(operation)?),
        "assign" => ViewActionKind::Assign(parse_assign_action(operation)?),
        "reset" => ViewActionKind::Reset(parse_reset_action(operation)?),
        _ => {
            return Err(node_error(
                operation,
                "`action` operations must be `request`, `assign` or `reset`",
            ));
        }
    };
    Ok(ViewAction {
        id: reactive_id("action", scope_kind, scope_name, node, &name),
        name,
        kind,
    })
}

fn parse_request_action(node: &SourceNode) -> DoweResult<ViewRequestAction> {
    if node.args.len() != 1 && node.args.len() != 2 {
        return Err(node_error(
            node,
            "`request` must use `request METHOD path` or `request METHOD route:\"/path\"`",
        ));
    }
    reject_request_unknown_props(node)?;
    let method_name = node.args[0]
        .as_required_string()
        .ok_or_else(|| node_error(node, "`request` method must be a name"))?;
    let method = ViewRequestMethod::from_name(&method_name).ok_or_else(|| {
        node_error(
            node,
            "`request` method must be GET, POST, PUT, PATCH or DELETE",
        )
    })?;
    let path = request_path(node)?;
    if !path.starts_with('/') {
        return Err(node_error(node, "`request` path must start with `/`"));
    }
    let result = request_result_blocks(node)?;
    let base_env = optional_env_ref_prop(node, "base")?.or_else(|| {
        if is_api_route(&path) {
            Some("BACKEND_URL".to_string())
        } else {
            None
        }
    });
    Ok(ViewRequestAction {
        method,
        path,
        base_env,
        body: optional_prop_string(node, "body")?,
        update: optional_prop_string(node, "update")?,
        reset: optional_prop_string(node, "reset")?,
        success_alert: result
            .success_alert
            .or(optional_prop_string(node, "successAlert")?),
        success_message: result
            .success_message
            .or(optional_static_string_prop(node, "successMessage")?),
        error_alert: result
            .error_alert
            .or(optional_prop_string(node, "errorAlert")?),
        error_message: result
            .error_message
            .or(optional_static_string_prop(node, "errorMessage")?),
        autoload: optional_prop_bool(node, "autoload")?.unwrap_or(false),
    })
}

fn reject_request_unknown_props(node: &SourceNode) -> DoweResult<()> {
    let allowed = [
        "base",
        "route",
        "path",
        "body",
        "update",
        "reset",
        "successAlert",
        "successMessage",
        "errorAlert",
        "errorMessage",
        "autoload",
    ];
    for prop in &node.props {
        if !allowed.contains(&prop.name.as_str()) {
            return Err(node_error(
                node,
                format!("`request` does not support `{}`", prop.name),
            ));
        }
    }
    Ok(())
}

fn request_path(node: &SourceNode) -> DoweResult<String> {
    let positional = match node.args.get(1) {
        Some(value) => Some(
            value
                .as_required_string()
                .ok_or_else(|| node_error(node, "`request` path must be a string"))?,
        ),
        None => None,
    };
    let route = optional_static_string_prop(node, "route")?;
    let path = optional_static_string_prop(node, "path")?;
    let count = usize::from(positional.is_some())
        + usize::from(route.is_some())
        + usize::from(path.is_some());
    if count == 0 {
        return Err(node_error(
            node,
            "`request` must declare a route with a positional path, `route`, or `path`",
        ));
    }
    if count > 1 {
        return Err(node_error(
            node,
            "`request` must declare only one route path",
        ));
    }
    Ok(positional.or(route).or(path).unwrap_or_default())
}

#[derive(Default)]
struct RequestResultBlocks {
    success_alert: Option<String>,
    success_message: Option<String>,
    error_alert: Option<String>,
    error_message: Option<String>,
}

fn request_result_blocks(node: &SourceNode) -> DoweResult<RequestResultBlocks> {
    let mut result = RequestResultBlocks::default();
    for child in &node.children {
        let outcome = parse_request_outcome(child)?;
        match child.name.as_str() {
            "onSuccess" => {
                if node.prop("successAlert").is_some()
                    || node.prop("successMessage").is_some()
                    || result.success_alert.is_some()
                    || result.success_message.is_some()
                {
                    return Err(node_error(
                        child,
                        "`onSuccess` cannot be combined with inline success props",
                    ));
                }
                result.success_alert = Some(outcome.target);
                result.success_message = Some(outcome.message);
            }
            "onError" => {
                if node.prop("errorAlert").is_some()
                    || node.prop("errorMessage").is_some()
                    || result.error_alert.is_some()
                    || result.error_message.is_some()
                {
                    return Err(node_error(
                        child,
                        "`onError` cannot be combined with inline error props",
                    ));
                }
                result.error_alert = Some(outcome.target);
                result.error_message = Some(outcome.message);
            }
            _ => {
                return Err(node_error(
                    child,
                    "`request` children must be `onSuccess` or `onError`",
                ));
            }
        }
    }
    Ok(result)
}

struct RequestOutcome {
    target: String,
    message: String,
}

fn parse_request_outcome(node: &SourceNode) -> DoweResult<RequestOutcome> {
    if !node.args.is_empty() || !node.children.is_empty() {
        return Err(node_error(
            node,
            "`onSuccess` and `onError` only accept props",
        ));
    }
    for prop in &node.props {
        if !matches!(prop.name.as_str(), "alert" | "message" | "target") {
            return Err(node_error(
                node,
                format!("`{}` is not valid in `{}`", prop.name, node.name),
            ));
        }
    }
    let message = optional_static_string_prop(node, "alert")?
        .or(optional_static_string_prop(node, "message")?)
        .ok_or_else(|| node_error(node, format!("`{}` must declare `alert`", node.name)))?;
    let target = optional_prop_string(node, "target")?.unwrap_or_else(|| "alert".to_string());
    Ok(RequestOutcome { target, message })
}

fn is_api_route(path: &str) -> bool {
    path == "/api" || path.starts_with("/api/")
}

fn optional_env_ref_prop(node: &SourceNode, name: &str) -> DoweResult<Option<String>> {
    node.prop(name)
        .map(|prop| match &prop.value {
            SourceValue::Bareword(value) => parse_env_ref(node, name, value),
            _ => Err(node_error(node, format!("`{name}` must be `env.NAME`"))),
        })
        .transpose()
}

fn parse_env_ref(node: &SourceNode, name: &str, value: &str) -> DoweResult<String> {
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() != 2 || parts[0] != "env" || parts[1].is_empty() {
        return Err(node_error(node, format!("`{name}` must be `env.NAME`")));
    }
    Ok(parts[1].to_string())
}

fn parse_assign_action(node: &SourceNode) -> DoweResult<ViewAssignAction> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(
            node,
            "`assign` must use `assign target source:value`",
        ));
    }
    Ok(ViewAssignAction {
        target: node.args[0]
            .as_required_string()
            .ok_or_else(|| node_error(node, "`assign` target must be a signal name"))?,
        source: required_prop_string(node, "source")?,
    })
}

fn parse_reset_action(node: &SourceNode) -> DoweResult<ViewResetAction> {
    if node.args.len() != 1 || !node.children.is_empty() {
        return Err(node_error(node, "`reset` must use `reset target`"));
    }
    Ok(ViewResetAction {
        target: node.args[0]
            .as_required_string()
            .ok_or_else(|| node_error(node, "`reset` target must be a signal name"))?,
    })
}

fn optional_prop_string(node: &SourceNode, name: &str) -> DoweResult<Option<String>> {
    node.prop(name)
        .map(|prop| {
            prop.value
                .as_required_string()
                .ok_or_else(|| node_error(node, format!("`{name}` must be a string")))
        })
        .transpose()
}

fn optional_static_string_prop(node: &SourceNode, name: &str) -> DoweResult<Option<String>> {
    node.prop(name)
        .map(|prop| match &prop.value {
            SourceValue::String(value) if !value.is_empty() => Ok(value.clone()),
            SourceValue::String(_) => Err(node_error(node, format!("`{name}` must be a string"))),
            _ => Err(quoted_static_string_error(prop)),
        })
        .transpose()
}

fn optional_prop_bool(node: &SourceNode, name: &str) -> DoweResult<Option<bool>> {
    node.prop(name)
        .map(|prop| match &prop.value {
            SourceValue::Boolean(value) => Ok(*value),
            _ => Err(node_error(node, format!("`{name}` must be a boolean"))),
        })
        .transpose()
}

fn lower_node_sequence(nodes: &[SourceNode], allow_children: bool) -> DoweResult<Vec<ViewNode>> {
    let mut output = Vec::new();
    let mut index = 0usize;
    while index < nodes.len() {
        let node = &nodes[index];
        match node.name.as_str() {
            "if" => {
                let else_node = nodes.get(index + 1).filter(|next| next.name == "else");
                output.extend(lower_if(node, else_node, allow_children)?);
                index += if else_node.is_some() { 2 } else { 1 };
            }
            "else" => {
                return Err(node_error(
                    node,
                    "`else` must follow an `if` at the same indentation level",
                ));
            }
            "each" => {
                output.push(lower_each(node, allow_children)?);
                index += 1;
            }
            _ => {
                output.push(lower_view_node(node, allow_children)?);
                index += 1;
            }
        }
    }
    Ok(output)
}

fn lower_if(
    node: &SourceNode,
    else_node: Option<&SourceNode>,
    allow_children: bool,
) -> DoweResult<Vec<ViewNode>> {
    if node.children.is_empty() {
        return Err(node_error(node, "`if` must contain view nodes"));
    }
    if !node.props.is_empty() || node.args.len() != 1 {
        return Err(node_error(node, "`if` must declare one condition"));
    }
    let condition = node.args[0].as_string_like().unwrap_or_default();
    match condition.as_str() {
        "true" => lower_node_sequence(&node.children, allow_children),
        "false" => else_node
            .map(|node| lower_node_sequence(&node.children, allow_children))
            .unwrap_or_else(|| Ok(Vec::new())),
        _ => Err(node_error(
            node,
            "condition cannot be resolved by the current Dowe data surface",
        )),
    }
}

fn lower_each(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    if node.args.len() != 3 {
        return Err(node_error(
            node,
            "`each` must use `each item in collection`",
        ));
    }
    let item = node.args[0]
        .as_required_string()
        .ok_or_else(|| node_error(node, "`each` item must be a name"))?;
    let separator = node.args[1]
        .as_required_string()
        .ok_or_else(|| node_error(node, "`each` must use `in`"))?;
    if separator != "in" {
        return Err(node_error(
            node,
            "`each` must use `each item in collection`",
        ));
    }
    let collection = node.args[2]
        .as_required_string()
        .ok_or_else(|| node_error(node, "`each` collection must be a signal name"))?;
    let key = required_prop_string(node, "key")?;
    if node.children.is_empty() {
        return Err(node_error(node, "`each` must contain view nodes"));
    }
    Ok(ViewNode::Each {
        item,
        collection,
        key,
        children: lower_node_sequence(&node.children, allow_children)?,
    })
}

fn lower_view_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    if node.name == "children" {
        return children_node(allow_children).map_err(|error| node_error(node, error.to_string()));
    }
    let component = COMPONENT_REGISTRY.get(&node.name).ok_or_else(|| {
        node_error(
            node,
            ComponentError::unknown_component(&node.name).to_string(),
        )
    })?;
    if component == BuiltinComponent::Code {
        return lower_code_node(node);
    }
    let props = component_props(node, component)?;
    match component {
        BuiltinComponent::Input => {
            reject_children(node)?;
            input_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Select => lower_select_node(node),
        BuiltinComponent::ComboBox => lower_combo_box_node(node),
        BuiltinComponent::CsvField => lower_csv_field_node(node),
        BuiltinComponent::DragDrop => lower_drag_drop_node(node),
        BuiltinComponent::Option => Err(node_error(
            node,
            ComponentError::invalid_prop_combination("Option can only be used inside Select")
                .to_string(),
        )),
        BuiltinComponent::ComboOption => Err(node_error(
            node,
            ComponentError::invalid_prop_combination(
                "comboOption can only be used inside ComboBox",
            )
            .to_string(),
        )),
        BuiltinComponent::CsvColumn => Err(node_error(
            node,
            ComponentError::invalid_prop_combination("csvColumn can only be used inside CsvField")
                .to_string(),
        )),
        BuiltinComponent::DragGroup => Err(node_error(
            node,
            ComponentError::invalid_prop_combination("dragGroup can only be used inside DragDrop")
                .to_string(),
        )),
        BuiltinComponent::DragItem => Err(node_error(
            node,
            ComponentError::invalid_prop_combination(
                "dragItem can only be used inside DragDrop or dragGroup",
            )
            .to_string(),
        )),
        BuiltinComponent::Code => unreachable!("Code lowers before scalar props"),
        BuiltinComponent::Video => {
            reject_children(node)?;
            video_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Audio => {
            reject_children(node)?;
            audio_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Image => {
            reject_children(node)?;
            image_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Candlestick => {
            reject_children(node)?;
            candlestick_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::ArcChart => {
            reject_children(node)?;
            arc_chart_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::AreaChart => {
            reject_children(node)?;
            area_chart_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::BarChart => {
            reject_children(node)?;
            bar_chart_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::LineChart => {
            reject_children(node)?;
            line_chart_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::PieChart => {
            reject_children(node)?;
            pie_chart_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Table => lower_table_node(node),
        BuiltinComponent::Tabs => lower_tabs_node(node, allow_children),
        BuiltinComponent::Tab => Err(node_error(
            node,
            ComponentError::invalid_prop_combination("tab can only be used inside Tabs")
                .to_string(),
        )),
        BuiltinComponent::NavMenu => lower_nav_menu_node(node, allow_children),
        BuiltinComponent::Divider => {
            reject_children(node)?;
            divider_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Alert => {
            reject_children(node)?;
            container_component_node(component, props, Vec::new(), allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Avatar => lower_avatar_node(node),
        BuiltinComponent::Badge => {
            let children = lower_node_sequence(&node.children, allow_children)?;
            badge_component_node(props, children, allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Chip => lower_chip_node(node),
        BuiltinComponent::Skeleton => {
            reject_children(node)?;
            skeleton_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Modal => lower_modal_node(node, allow_children),
        BuiltinComponent::AlertDialog => {
            reject_children(node)?;
            alert_dialog_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Tooltip => {
            let children = lower_node_sequence(&node.children, allow_children)?;
            tooltip_component_node(props, children, allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Toast => {
            reject_children(node)?;
            toast_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Dropdown => lower_dropdown_node(node, allow_children),
        BuiltinComponent::Command => lower_command_node(node),
        BuiltinComponent::AvatarGroup => lower_avatar_group_node(node),
        BuiltinComponent::ChatBox => {
            reject_children(node)?;
            chat_box_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Empty => {
            reject_children(node)?;
            empty_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Marquee => {
            let children = lower_node_sequence(&node.children, allow_children)?;
            marquee_component_node(props, children, allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::TypeWriter => lower_type_writer_node(node),
        BuiltinComponent::RichText => lower_rich_text_node(node),
        BuiltinComponent::Record => {
            reject_children(node)?;
            record_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::ToggleGroup => lower_toggle_group_node(node),
        BuiltinComponent::Collapsible => {
            let children = lower_node_sequence(&node.children, allow_children)?;
            collapsible_component_node(props, children, allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Countdown => {
            reject_children(node)?;
            countdown_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Map => lower_map_node(node),
        BuiltinComponent::Accordion => lower_accordion_node(node, allow_children),
        BuiltinComponent::Carousel => lower_carousel_node(node, allow_children),
        BuiltinComponent::Checkbox => {
            reject_children(node)?;
            checkbox_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Color => {
            reject_children(node)?;
            color_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Date => {
            reject_children(node)?;
            date_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::DateRange => {
            reject_children(node)?;
            date_range_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::RadioGroup => lower_radio_group_node(node),
        BuiltinComponent::Toggle => {
            reject_children(node)?;
            toggle_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Title | BuiltinComponent::Text => {
            reject_text_prop(node, component)?;
            let value = required_text_child(node, component)?;
            text_component_node(component, props, value)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Button => {
            reject_text_prop(node, component)?;
            let value = required_text_child(node, component)?;
            let children = vec![text_node(value).map_err(|error| component_error(node, error))?];
            container_component_node(component, props, children, allow_children)
                .map_err(|error| component_error(node, error))
        }
        BuiltinComponent::ToggleTheme => {
            reject_children(node)?;
            theme_toggle_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Fab => lower_fab_node(node),
        BuiltinComponent::FabAction => Err(node_error(
            node,
            ComponentError::invalid_prop_combination("fabAction can only be used inside Fab")
                .to_string(),
        )),
        BuiltinComponent::Slider => {
            reject_children(node)?;
            slider_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Dropzone => {
            reject_children(node)?;
            dropzone_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Editor => {
            reject_children(node)?;
            editor_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::ImageCropper => {
            reject_children(node)?;
            image_cropper_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::PasswordField => {
            reject_children(node)?;
            password_field_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::PhoneField => {
            reject_children(node)?;
            phone_field_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::PinField => {
            reject_children(node)?;
            pin_field_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Textarea => {
            reject_children(node)?;
            textarea_component_node(props).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Svg => lower_svg_node(node),
        BuiltinComponent::Path => Err(node_error(
            node,
            ComponentError::invalid_prop_combination("Path can only be used inside Svg")
                .to_string(),
        )),
        BuiltinComponent::AppBar | BuiltinComponent::Footer | BuiltinComponent::BottomBar => {
            lower_bar_node(node, component, allow_children)
        }
        BuiltinComponent::SideNav | BuiltinComponent::Sidebar => {
            lower_side_nav_node(node, component)
        }
        BuiltinComponent::Scaffold => lower_scaffold_node(node, allow_children),
        BuiltinComponent::Box
        | BuiltinComponent::Section
        | BuiltinComponent::Flex
        | BuiltinComponent::Grid
        | BuiltinComponent::Card
        | BuiltinComponent::Drawer => {
            let children = lower_node_sequence(&node.children, allow_children)?;
            container_component_node(component, props, children, allow_children)
                .map_err(|error| component_error(node, error))
        }
    }
}

fn lower_avatar_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Avatar)?;
    let mut icon = None;
    for child in &node.children {
        if child.name != "icon" {
            return Err(node_error(child, "Avatar only accepts an icon region"));
        }
        if icon.is_some() {
            return Err(node_error(child, "duplicate `icon` region in Avatar"));
        }
        icon = Some(lower_overlay_icon(child, BuiltinComponent::Avatar)?);
    }
    avatar_component_node(props, icon).map_err(|error| component_error(node, error))
}

fn lower_chip_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Chip)?;
    let mut start = None;
    let mut end = None;
    let mut labels = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "start" if start.is_none() => {
                start = Some(lower_overlay_icon(child, BuiltinComponent::Chip)?)
            }
            "start" => return Err(node_error(child, "duplicate `start` region in Chip")),
            "end" if end.is_none() => {
                end = Some(lower_overlay_icon(child, BuiltinComponent::Chip)?)
            }
            "end" => return Err(node_error(child, "duplicate `end` region in Chip")),
            _ => labels.push(text_child_line(child)?),
        }
    }
    if labels.len() != 1 {
        return Err(node_error(node, "Chip requires one direct text child"));
    }
    chip_component_node(props, &labels[0], start, end).map_err(|error| component_error(node, error))
}

fn lower_modal_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Modal)?;
    let mut header = None;
    let mut footer = None;
    let mut body_nodes = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "header" if header.is_none() => {
                header = Some(lower_region(child, "Modal header", allow_children)?)
            }
            "header" => return Err(node_error(child, "duplicate `header` region in Modal")),
            "footer" if footer.is_none() => {
                footer = Some(lower_region(child, "Modal footer", allow_children)?)
            }
            "footer" => return Err(node_error(child, "duplicate `footer` region in Modal")),
            _ => body_nodes.push(child.clone()),
        }
    }
    let body = lower_node_sequence(&body_nodes, allow_children)?;
    modal_component_node(
        props,
        header.unwrap_or_default(),
        body,
        footer.unwrap_or_default(),
        allow_children,
    )
    .map_err(|error| component_error(node, error))
}

fn lower_dropdown_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Dropdown)?;
    let mut trigger = None;
    let mut header = None;
    let mut footer = None;
    let mut entries = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "trigger" if trigger.is_none() => {
                trigger = Some(lower_region(child, "Dropdown trigger", allow_children)?)
            }
            "trigger" => return Err(node_error(child, "duplicate `trigger` region in Dropdown")),
            "header" if header.is_none() => {
                header = Some(lower_region(child, "Dropdown header", allow_children)?)
            }
            "header" => return Err(node_error(child, "duplicate `header` region in Dropdown")),
            "footer" if footer.is_none() => {
                footer = Some(lower_region(child, "Dropdown footer", allow_children)?)
            }
            "footer" => return Err(node_error(child, "duplicate `footer` region in Dropdown")),
            "item" => entries.push(dowe_components::OverlayEntry::Item(lower_overlay_item(
                child,
                BuiltinComponent::Dropdown,
            )?)),
            "divider" => {
                if !child.args.is_empty() || !child.props.is_empty() || !child.children.is_empty() {
                    return Err(node_error(
                        child,
                        "Dropdown divider cannot declare args, props or children",
                    ));
                }
                entries.push(dowe_components::OverlayEntry::Divider);
            }
            _ => {
                return Err(node_error(
                    child,
                    "Dropdown only accepts trigger, header, footer, item or divider entries",
                ));
            }
        }
    }
    dropdown_component_node(
        props,
        trigger.unwrap_or_default(),
        header.unwrap_or_default(),
        entries,
        footer.unwrap_or_default(),
        allow_children,
    )
    .map_err(|error| component_error(node, error))
}

fn lower_command_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Command)?;
    let mut entries = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "item" => entries.push(dowe_components::CommandEntry::Item(lower_overlay_item(
                child,
                BuiltinComponent::Command,
            )?)),
            "group" => entries.push(lower_command_group(child)?),
            _ => {
                return Err(node_error(
                    child,
                    "Command only accepts item or group entries",
                ));
            }
        }
    }
    command_component_node(props, entries).map_err(|error| component_error(node, error))
}

fn lower_avatar_group_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::AvatarGroup)?;
    let mut items = Vec::new();
    for child in &node.children {
        if child.name != "item" {
            return Err(node_error(child, "AvatarGroup only accepts item entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "AvatarGroup item cannot declare args"));
        }
        reject_children(child)?;
        items.push(
            avatar_group_item_component(avatar_group_item_props(child)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    avatar_group_component_node(props, items).map_err(|error| component_error(node, error))
}

fn lower_type_writer_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::TypeWriter)?;
    let mut items = Vec::new();
    for child in &node.children {
        if child.name != "item" {
            return Err(node_error(child, "TypeWriter only accepts item entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "TypeWriter item cannot declare args"));
        }
        reject_children(child)?;
        items.push(
            type_writer_item_component(type_writer_item_props(child)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    type_writer_component_node(props, items).map_err(|error| component_error(node, error))
}

fn lower_rich_text_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::RichText)?;
    let mut marks = Vec::new();
    for child in &node.children {
        if child.name != "mark" {
            return Err(node_error(child, "RichText only accepts mark entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "RichText mark cannot declare args"));
        }
        reject_children(child)?;
        marks.push(
            rich_text_mark_component(rich_text_mark_props(child)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    rich_text_component_node(props, marks).map_err(|error| component_error(node, error))
}

fn lower_toggle_group_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::ToggleGroup)?;
    let mut items = Vec::new();
    for child in &node.children {
        if child.name != "item" {
            return Err(node_error(child, "ToggleGroup only accepts item entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "ToggleGroup item cannot declare args"));
        }
        reject_children(child)?;
        items.push(
            toggle_group_item_component(toggle_group_item_props(child)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    toggle_group_component_node(props, items).map_err(|error| component_error(node, error))
}

fn lower_map_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Map)?;
    let mut markers = Vec::new();
    let mut waypoints = Vec::new();
    for child in &node.children {
        if !matches!(child.name.as_str(), "marker" | "waypoint") {
            return Err(node_error(
                child,
                "Map only accepts marker and waypoint entries",
            ));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "Map entries cannot declare args"));
        }
        reject_children(child)?;
        if child.name == "marker" {
            markers.push(
                map_marker_component(map_marker_props(child)?)
                    .map_err(|error| component_error(child, error))?,
            );
        } else {
            waypoints.push(
                map_waypoint_component(map_waypoint_props(child)?)
                    .map_err(|error| component_error(child, error))?,
            );
        }
    }
    map_component_node(props, markers, waypoints).map_err(|error| component_error(node, error))
}

fn lower_accordion_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Accordion)?;
    let mut items = Vec::new();
    for child in &node.children {
        if child.name != "item" {
            return Err(node_error(child, "Accordion only accepts item entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "Accordion item cannot declare args"));
        }
        let children = lower_node_sequence(&child.children, allow_children)?;
        items.push(
            accordion_item_component(accordion_item_props(child)?, children)
                .map_err(|error| component_error(child, error))?,
        );
    }
    accordion_component_node(props, items).map_err(|error| component_error(node, error))
}

fn lower_carousel_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Carousel)?;
    let mut slides = Vec::new();
    for child in &node.children {
        if child.name != "slide" {
            return Err(node_error(child, "Carousel only accepts slide entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "Carousel slide cannot declare args"));
        }
        let children = lower_node_sequence(&child.children, allow_children)?;
        slides.push(
            carousel_slide_component(carousel_slide_props(child)?, children)
                .map_err(|error| component_error(child, error))?,
        );
    }
    carousel_component_node(props, slides).map_err(|error| component_error(node, error))
}

fn lower_radio_group_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::RadioGroup)?;
    let mut options = Vec::new();
    for child in &node.children {
        if child.name != "item" {
            return Err(node_error(child, "RadioGroup only accepts item entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "RadioGroup item cannot declare args"));
        }
        reject_children(child)?;
        options.push(
            radio_option_component(radio_item_props(child)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    radio_group_component_node(props, options).map_err(|error| component_error(node, error))
}

fn lower_fab_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Fab)?;
    let mut actions = Vec::new();
    for child in &node.children {
        let component = COMPONENT_REGISTRY.get(&child.name).ok_or_else(|| {
            node_error(
                child,
                ComponentError::unknown_component(&child.name).to_string(),
            )
        })?;
        if component != BuiltinComponent::FabAction {
            return Err(node_error(child, "Fab only accepts fabAction entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "fabAction cannot declare args"));
        }
        reject_children(child)?;
        actions.push(
            fab_action_component(component_props(child, component)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    fab_component_node(props, actions).map_err(|error| component_error(node, error))
}

fn avatar_group_item_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(
                prop.name.as_str(),
                "src"
                    | "name"
                    | "alt"
                    | "href"
                    | "navigate"
                    | "history"
                    | "target"
                    | "externalMode"
                    | "onClick"
            ) {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::AvatarGroup, &prop.name)
                        .to_string(),
                ));
            }
            if prop.name != "onClick" && static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn type_writer_item_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if prop.name != "text" {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::TypeWriter, &prop.name)
                        .to_string(),
                ));
            }
            if static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn rich_text_mark_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(prop.name.as_str(), "text" | "style" | "scheme" | "color") {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::RichText, &prop.name)
                        .to_string(),
                ));
            }
            if static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn toggle_group_item_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(prop.name.as_str(), "id" | "label" | "icon") {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::ToggleGroup, &prop.name)
                        .to_string(),
                ));
            }
            if static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn map_marker_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(
                prop.name.as_str(),
                "id" | "lat" | "lng" | "label" | "popup" | "icon" | "onClick"
            ) {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Map, &prop.name).to_string(),
                ));
            }
            if prop.name != "onClick"
                && !matches!(prop.name.as_str(), "lat" | "lng")
                && static_value_has_bareword(&prop.value)
            {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn map_waypoint_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(prop.name.as_str(), "lat" | "lng") {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Map, &prop.name).to_string(),
                ));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn accordion_item_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(
                prop.name.as_str(),
                "id" | "label" | "disabled" | "defaultOpen"
            ) {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Accordion, &prop.name)
                        .to_string(),
                ));
            }
            if matches!(prop.name.as_str(), "id" | "label")
                && static_value_has_bareword(&prop.value)
            {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn carousel_slide_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if prop.name != "id" {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Carousel, &prop.name)
                        .to_string(),
                ));
            }
            if static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn radio_item_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(prop.name.as_str(), "value" | "label" | "disabled") {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::RadioGroup, &prop.name)
                        .to_string(),
                ));
            }
            if matches!(prop.name.as_str(), "value" | "label")
                && static_value_has_bareword(&prop.value)
            {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn lower_command_group(node: &SourceNode) -> DoweResult<dowe_components::CommandEntry> {
    if !node.args.is_empty() {
        return Err(node_error(node, "Command group cannot declare args"));
    }
    let mut icon = None;
    let mut items = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "icon" if icon.is_none() => {
                icon = Some(lower_overlay_icon(child, BuiltinComponent::Command)?)
            }
            "icon" => {
                return Err(node_error(
                    child,
                    "duplicate `icon` region in Command group",
                ));
            }
            "item" => items.push(lower_overlay_item(child, BuiltinComponent::Command)?),
            _ => {
                return Err(node_error(
                    child,
                    "Command group only accepts icon or item entries",
                ));
            }
        }
    }
    command_group_component(command_group_props(node)?, icon, items)
        .map_err(|error| component_error(node, error))
}

fn lower_overlay_item(
    node: &SourceNode,
    owner: BuiltinComponent,
) -> DoweResult<dowe_components::OverlayItemProps> {
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            format!("{} item cannot declare args", owner.as_str()),
        ));
    }
    let mut icon = None;
    for child in &node.children {
        if child.name != "icon" {
            return Err(node_error(
                child,
                format!("{} item only accepts an icon region", owner.as_str()),
            ));
        }
        if icon.is_some() {
            return Err(node_error(
                child,
                format!("duplicate `icon` region in {} item", owner.as_str()),
            ));
        }
        icon = Some(lower_overlay_icon(child, owner)?);
    }
    overlay_item_component(owner, overlay_item_props(node, owner)?, icon)
        .map_err(|error| component_error(node, error))
}

fn lower_overlay_icon(
    node: &SourceNode,
    owner: BuiltinComponent,
) -> DoweResult<dowe_components::SideNavIcon> {
    if !node.args.is_empty() || !node.props.is_empty() || node.children.len() != 1 {
        return Err(node_error(
            node,
            format!("{} icon requires exactly one Svg child", owner.as_str()),
        ));
    }
    let child = &node.children[0];
    if child.name != "Svg" {
        return Err(node_error(
            child,
            format!("{} icon requires exactly one Svg child", owner.as_str()),
        ));
    }
    overlay_icon_component(lower_svg_node(child)?, owner)
        .map_err(|error| component_error(node, error))
}

fn lower_region(node: &SourceNode, label: &str, allow_children: bool) -> DoweResult<Vec<ViewNode>> {
    if !node.args.is_empty() || !node.props.is_empty() {
        return Err(node_error(
            node,
            format!("{label} cannot declare args or props"),
        ));
    }
    lower_node_sequence(&node.children, allow_children)
}

fn overlay_item_props(
    node: &SourceNode,
    owner: BuiltinComponent,
) -> DoweResult<Vec<ComponentProp>> {
    let allowed = [
        "label",
        "description",
        "href",
        "navigate",
        "history",
        "target",
        "externalMode",
        "onClick",
        "disabled",
    ];
    node.props
        .iter()
        .map(|prop| {
            if !allowed.contains(&prop.name.as_str()) {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(owner, &prop.name).to_string(),
                ));
            }
            if prop.name != "onClick" && static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn command_group_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if prop.name != "label" {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Command, &prop.name).to_string(),
                ));
            }
            if static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn lower_tabs_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Tabs)?;
    let mut tabs = Vec::new();
    for child in &node.children {
        if child.name != "tab" {
            return Err(node_error(child, "Tabs only accepts tab entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "Tabs tab cannot declare args"));
        }
        let children = lower_node_sequence(&child.children, allow_children)?;
        tabs.push(
            tabs_tab_component(tabs_tab_props(child)?, children)
                .map_err(|error| component_error(child, error))?,
        );
    }
    tabs_component_node(props, tabs).map_err(|error| component_error(node, error))
}

fn tabs_tab_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(prop.name.as_str(), "id" | "label") {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Tab, &prop.name).to_string(),
                ));
            }
            if static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn lower_table_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Table)?;
    let mut columns = Vec::new();
    for child in &node.children {
        if child.name != "column" {
            return Err(node_error(child, "Table only accepts column entries"));
        }
        if !child.args.is_empty() {
            return Err(node_error(child, "Table column cannot declare args"));
        }
        reject_children(child)?;
        columns.push(
            table_column_component(table_column_props(child)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    table_node(props, columns).map_err(|error| component_error(node, error))
}

fn table_column_props(node: &SourceNode) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !matches!(prop.name.as_str(), "field" | "label" | "align" | "width") {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::Table, &prop.name).to_string(),
                ));
            }
            if static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn lower_nav_menu_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::NavMenu)?;
    let mut items = Vec::new();
    for child in &node.children {
        items.push(match child.name.as_str() {
            "item" => lower_nav_menu_item(child)?,
            "submenu" => lower_nav_menu_submenu(child)?,
            "megamenu" => lower_nav_menu_megamenu(child, allow_children)?,
            _ => {
                return Err(node_error(
                    child,
                    "NavMenu only accepts item, submenu or megamenu entries",
                ));
            }
        });
    }
    nav_menu_component_node(props, items).map_err(|error| component_error(node, error))
}

fn lower_nav_menu_item(node: &SourceNode) -> DoweResult<dowe_components::NavMenuItem> {
    if !node.args.is_empty() {
        return Err(node_error(node, "NavMenu entries cannot declare args"));
    }
    let icon = lower_nav_menu_icon_children(node)?;
    let props = nav_menu_entry_props(
        node,
        &[
            "label",
            "description",
            "href",
            "navigate",
            "target",
            "externalMode",
            "onClick",
        ],
    )?;
    nav_menu_item_component(props, icon).map_err(|error| component_error(node, error))
}

fn lower_nav_menu_submenu(node: &SourceNode) -> DoweResult<dowe_components::NavMenuItem> {
    if !node.args.is_empty() {
        return Err(node_error(node, "NavMenu submenu cannot declare args"));
    }
    let props = nav_menu_entry_props(node, &["label", "description"])?;
    let mut icon = None;
    let mut items = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "icon" if icon.is_none() => icon = Some(lower_nav_menu_icon(child)?),
            "icon" => {
                return Err(node_error(
                    child,
                    "duplicate `icon` block in NavMenu submenu",
                ));
            }
            "item" => {
                let item = lower_nav_menu_item(child)?;
                let dowe_components::NavMenuItem::Item(props) = item else {
                    unreachable!("NavMenu submenu item");
                };
                items.push(props);
            }
            _ => {
                return Err(node_error(
                    child,
                    "NavMenu submenu only accepts icon or item children",
                ));
            }
        }
    }
    nav_menu_submenu_component(props, icon, items).map_err(|error| component_error(node, error))
}

fn lower_nav_menu_megamenu(
    node: &SourceNode,
    allow_children: bool,
) -> DoweResult<dowe_components::NavMenuItem> {
    if !node.args.is_empty() {
        return Err(node_error(node, "NavMenu megamenu cannot declare args"));
    }
    let props = nav_menu_entry_props(node, &["label", "description"])?;
    let mut icon = None;
    let mut content = None;
    for child in &node.children {
        match child.name.as_str() {
            "icon" if icon.is_none() => icon = Some(lower_nav_menu_icon(child)?),
            "icon" => {
                return Err(node_error(
                    child,
                    "duplicate `icon` block in NavMenu megamenu",
                ));
            }
            "content" if content.is_none() => {
                if !child.args.is_empty() || !child.props.is_empty() {
                    return Err(node_error(
                        child,
                        "NavMenu megamenu content cannot declare args or props",
                    ));
                }
                content = Some(lower_node_sequence(&child.children, allow_children)?);
            }
            "content" => {
                return Err(node_error(
                    child,
                    "duplicate `content` region in NavMenu megamenu",
                ));
            }
            _ => {
                return Err(node_error(
                    child,
                    "NavMenu megamenu only accepts icon and content children",
                ));
            }
        }
    }
    nav_menu_megamenu_component(props, icon, content.unwrap_or_default(), allow_children)
        .map_err(|error| component_error(node, error))
}

fn lower_nav_menu_icon_children(
    node: &SourceNode,
) -> DoweResult<Option<dowe_components::SideNavIcon>> {
    let mut icon = None;
    for child in &node.children {
        if child.name != "icon" {
            return Err(node_error(
                child,
                "NavMenu entry only accepts an icon block",
            ));
        }
        if icon.is_some() {
            return Err(node_error(child, "duplicate `icon` block in NavMenu entry"));
        }
        icon = Some(lower_nav_menu_icon(child)?);
    }
    Ok(icon)
}

fn lower_nav_menu_icon(node: &SourceNode) -> DoweResult<dowe_components::SideNavIcon> {
    if !node.args.is_empty() || !node.props.is_empty() || node.children.len() != 1 {
        return Err(node_error(
            node,
            "NavMenu icon requires exactly one Svg child",
        ));
    }
    let child = &node.children[0];
    if child.name != "Svg" {
        return Err(node_error(
            child,
            "NavMenu icon requires exactly one Svg child",
        ));
    }
    side_nav_icon_component(lower_svg_node(child)?).map_err(|error| component_error(node, error))
}

fn nav_menu_entry_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| {
            if !allowed.contains(&prop.name.as_str()) {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(BuiltinComponent::NavMenu, &prop.name).to_string(),
                ));
            }
            if prop.name != "onClick" && static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn lower_side_nav_node(node: &SourceNode, component: BuiltinComponent) -> DoweResult<ViewNode> {
    let props = component_props(node, component)?;
    let mut items = Vec::new();
    for child in &node.children {
        items.push(match child.name.as_str() {
            "header" => lower_side_nav_entry(child, true, component)?,
            "item" => lower_side_nav_entry(child, false, component)?,
            "divider" => {
                if !child.args.is_empty() || !child.props.is_empty() || !child.children.is_empty() {
                    return Err(node_error(
                        child,
                        format!(
                            "{} divider cannot declare args, props or children",
                            component.as_str()
                        ),
                    ));
                }
                dowe_components::SideNavItem::Divider
            }
            "submenu" => lower_side_nav_submenu(child, component)?,
            _ => {
                return Err(node_error(
                    child,
                    format!(
                        "{} only accepts header, item, divider or submenu entries",
                        component.as_str()
                    ),
                ));
            }
        });
    }
    match component {
        BuiltinComponent::SideNav => {
            side_nav_component_node(props, items).map_err(|error| component_error(node, error))
        }
        BuiltinComponent::Sidebar => {
            sidebar_component_node(props, items).map_err(|error| component_error(node, error))
        }
        _ => unreachable!("navigation component"),
    }
}

fn lower_side_nav_entry(
    node: &SourceNode,
    header: bool,
    component: BuiltinComponent,
) -> DoweResult<dowe_components::SideNavItem> {
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            format!("{} entries cannot declare args", component.as_str()),
        ));
    }
    let icon = lower_side_nav_icon_children(node, component)?;
    let props = side_nav_entry_props(
        node,
        if header {
            &[
                "label",
                "description",
                "href",
                "navigate",
                "target",
                "externalMode",
                "onClick",
            ]
        } else {
            &[
                "label",
                "description",
                "status",
                "href",
                "navigate",
                "target",
                "externalMode",
                "onClick",
            ]
        },
        false,
        component,
    )?;
    if header {
        side_nav_header_component(props, icon).map_err(|error| component_error(node, error))
    } else {
        side_nav_item_component(props, icon).map_err(|error| component_error(node, error))
    }
}

fn lower_side_nav_submenu(
    node: &SourceNode,
    component: BuiltinComponent,
) -> DoweResult<dowe_components::SideNavItem> {
    if !node.args.is_empty() {
        return Err(node_error(
            node,
            format!("{} submenu cannot declare args", component.as_str()),
        ));
    }
    let open = optional_prop_bool(node, "open")?.unwrap_or(false);
    let props = side_nav_entry_props(node, &["label", "description", "status"], true, component)?;
    let mut icon = None;
    let mut items = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "icon" if icon.is_none() => icon = Some(lower_side_nav_icon(child, component)?),
            "icon" => {
                return Err(node_error(
                    child,
                    format!("duplicate `icon` block in {} submenu", component.as_str()),
                ));
            }
            "item" => {
                let item = lower_side_nav_entry(child, false, component)?;
                let dowe_components::SideNavItem::Item(props) = item else {
                    unreachable!("SideNav submenu item");
                };
                items.push(props);
            }
            _ => {
                return Err(node_error(
                    child,
                    format!(
                        "{} submenu only accepts icon or item children",
                        component.as_str()
                    ),
                ));
            }
        }
    }
    side_nav_submenu_component(props, icon, open, items)
        .map_err(|error| component_error(node, error))
}

fn lower_side_nav_icon_children(
    node: &SourceNode,
    component: BuiltinComponent,
) -> DoweResult<Option<dowe_components::SideNavIcon>> {
    let mut icon = None;
    for child in &node.children {
        if child.name != "icon" {
            return Err(node_error(
                child,
                format!("{} entry only accepts an icon block", component.as_str()),
            ));
        }
        if icon.is_some() {
            return Err(node_error(
                child,
                format!("duplicate `icon` block in {} entry", component.as_str()),
            ));
        }
        icon = Some(lower_side_nav_icon(child, component)?);
    }
    Ok(icon)
}

fn lower_side_nav_icon(
    node: &SourceNode,
    component: BuiltinComponent,
) -> DoweResult<dowe_components::SideNavIcon> {
    if !node.args.is_empty() || !node.props.is_empty() || node.children.len() != 1 {
        return Err(node_error(
            node,
            format!("{} icon requires exactly one Svg child", component.as_str()),
        ));
    }
    let child = &node.children[0];
    if child.name != "Svg" {
        return Err(node_error(
            child,
            format!("{} icon requires exactly one Svg child", component.as_str()),
        ));
    }
    side_nav_icon_component(lower_svg_node(child)?).map_err(|error| component_error(node, error))
}

fn side_nav_entry_props(
    node: &SourceNode,
    allowed: &[&str],
    skip_open: bool,
    component: BuiltinComponent,
) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .filter(|prop| !skip_open || prop.name != "open")
        .map(|prop| {
            if !allowed.contains(&prop.name.as_str()) {
                return Err(node_error(
                    node,
                    ComponentError::unknown_prop(component, &prop.name).to_string(),
                ));
            }
            if prop.name != "onClick" && static_value_has_bareword(&prop.value) {
                return Err(quoted_static_string_error(prop));
            }
            Ok(ComponentProp {
                name: prop.name.clone(),
                value: prop_value(prop)?,
            })
        })
        .collect()
}

fn lower_scaffold_node(node: &SourceNode, allow_children: bool) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Scaffold)?;
    let mut app_bar = None;
    let mut start = None;
    let mut main = None;
    let mut end = None;
    let mut bottom_bar = None;

    for child in &node.children {
        if !matches!(
            child.name.as_str(),
            "appBar" | "start" | "main" | "end" | "bottomBar"
        ) {
            return Err(node_error(
                child,
                "Scaffold only accepts appBar, start, main, end or bottomBar regions",
            ));
        }
        if !child.args.is_empty() || !child.props.is_empty() {
            return Err(node_error(
                child,
                "Scaffold regions cannot declare args or props",
            ));
        }
        let children = lower_node_sequence(&child.children, allow_children)?;
        match child.name.as_str() {
            "appBar" if app_bar.is_none() => app_bar = Some(children),
            "start" if start.is_none() => start = Some(children),
            "main" if main.is_none() => main = Some(children),
            "end" if end.is_none() => end = Some(children),
            "bottomBar" if bottom_bar.is_none() => bottom_bar = Some(children),
            name => {
                return Err(node_error(
                    child,
                    format!("duplicate `{name}` region in Scaffold"),
                ));
            }
        }
    }

    scaffold_component_node(
        props,
        app_bar.unwrap_or_default(),
        start.unwrap_or_default(),
        main.unwrap_or_default(),
        end.unwrap_or_default(),
        bottom_bar.unwrap_or_default(),
        allow_children,
    )
    .map_err(|error| component_error(node, error))
}

fn lower_bar_node(
    node: &SourceNode,
    component: BuiltinComponent,
    allow_children: bool,
) -> DoweResult<ViewNode> {
    let props = component_props(node, component)?;
    let mut start = None;
    let mut center = None;
    let mut end = None;

    for child in &node.children {
        if !matches!(child.name.as_str(), "start" | "center" | "end") {
            return Err(node_error(
                child,
                format!(
                    "{} only accepts start, center or end regions",
                    component.as_str()
                ),
            ));
        }
        if !child.args.is_empty() || !child.props.is_empty() {
            return Err(node_error(
                child,
                "bar regions cannot declare args or props",
            ));
        }
        let children = lower_node_sequence(&child.children, allow_children)?;
        match child.name.as_str() {
            "start" if start.is_none() => start = Some(children),
            "center" if center.is_none() => center = Some(children),
            "end" if end.is_none() => end = Some(children),
            name => {
                return Err(node_error(
                    child,
                    format!("duplicate `{name}` region in {}", component.as_str()),
                ));
            }
        }
    }

    bar_component_node(
        component,
        props,
        start.unwrap_or_default(),
        center.unwrap_or_default(),
        end.unwrap_or_default(),
        allow_children,
    )
    .map_err(|error| component_error(node, error))
}

fn lower_select_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Select)?;
    let mut options = Vec::new();
    for child in &node.children {
        let component = COMPONENT_REGISTRY.get(&child.name).ok_or_else(|| {
            node_error(
                child,
                ComponentError::unknown_component(&child.name).to_string(),
            )
        })?;
        if component != BuiltinComponent::Option {
            return Err(node_error(child, "Select can only contain Option children"));
        }
        reject_children(child)?;
        options.push(
            select_option_component(component_props(child, component)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    select_node(props, options).map_err(|error| component_error(node, error))
}

fn lower_combo_box_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::ComboBox)?;
    let mut options = Vec::new();
    for child in &node.children {
        let component = COMPONENT_REGISTRY.get(&child.name).ok_or_else(|| {
            node_error(
                child,
                ComponentError::unknown_component(&child.name).to_string(),
            )
        })?;
        if component != BuiltinComponent::ComboOption {
            return Err(node_error(
                child,
                "ComboBox can only contain comboOption children",
            ));
        }
        reject_children(child)?;
        options.push(
            combo_option_component(component_props(child, component)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    combo_box_component_node(props, options).map_err(|error| component_error(node, error))
}

fn lower_csv_field_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::CsvField)?;
    let mut columns = Vec::new();
    for child in &node.children {
        let component = COMPONENT_REGISTRY.get(&child.name).ok_or_else(|| {
            node_error(
                child,
                ComponentError::unknown_component(&child.name).to_string(),
            )
        })?;
        if component != BuiltinComponent::CsvColumn {
            return Err(node_error(
                child,
                "CsvField can only contain csvColumn children",
            ));
        }
        reject_children(child)?;
        columns.push(
            csv_column_component(component_props(child, component)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    csv_field_component_node(props, columns).map_err(|error| component_error(node, error))
}

fn lower_drag_drop_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::DragDrop)?;
    let mut items = Vec::new();
    let mut groups = Vec::new();
    for child in &node.children {
        let component = COMPONENT_REGISTRY.get(&child.name).ok_or_else(|| {
            node_error(
                child,
                ComponentError::unknown_component(&child.name).to_string(),
            )
        })?;
        match component {
            BuiltinComponent::DragItem => {
                reject_children(child)?;
                items.push(
                    drag_item_component(component_props(child, component)?)
                        .map_err(|error| component_error(child, error))?,
                );
            }
            BuiltinComponent::DragGroup => {
                groups.push(lower_drag_group_node(child)?);
            }
            _ => {
                return Err(node_error(
                    child,
                    "DragDrop can only contain dragItem or dragGroup children",
                ));
            }
        }
    }
    drag_drop_component_node(props, items, groups).map_err(|error| component_error(node, error))
}

fn lower_drag_group_node(node: &SourceNode) -> DoweResult<dowe_components::DragGroup> {
    let props = component_props(node, BuiltinComponent::DragGroup)?;
    let mut items = Vec::new();
    for child in &node.children {
        let component = COMPONENT_REGISTRY.get(&child.name).ok_or_else(|| {
            node_error(
                child,
                ComponentError::unknown_component(&child.name).to_string(),
            )
        })?;
        if component != BuiltinComponent::DragItem {
            return Err(node_error(
                child,
                "dragGroup can only contain dragItem children",
            ));
        }
        reject_children(child)?;
        items.push(
            drag_item_component(component_props(child, component)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    drag_group_component(props, items).map_err(|error| component_error(node, error))
}

fn lower_code_node(node: &SourceNode) -> DoweResult<ViewNode> {
    reject_children(node)?;
    let lines = code_lines(node)?;
    let props = node
        .props
        .iter()
        .filter(|prop| prop.name != "lines")
        .map(|prop| component_prop(BuiltinComponent::Code, prop))
        .collect::<DoweResult<Vec<_>>>()?;
    code_node(props, lines).map_err(|error| component_error(node, error))
}

fn code_lines(node: &SourceNode) -> DoweResult<Vec<String>> {
    let prop = node
        .prop("lines")
        .ok_or_else(|| node_error(node, "`Code` requires `lines`"))?;
    let SourceValue::Array(values) = &prop.value else {
        return Err(node_error(
            node,
            "`Code lines` must be a non-empty string array",
        ));
    };
    if values.is_empty() {
        return Err(node_error(
            node,
            "`Code lines` must be a non-empty string array",
        ));
    }
    values
        .iter()
        .map(|value| match value {
            SourceValue::String(value) => Ok(value.clone()),
            _ => Err(node_error(
                node,
                "`Code lines` must contain quoted static strings",
            )),
        })
        .collect()
}

fn lower_svg_node(node: &SourceNode) -> DoweResult<ViewNode> {
    let props = component_props(node, BuiltinComponent::Svg)?;
    let mut paths = Vec::new();
    for child in &node.children {
        let component = COMPONENT_REGISTRY.get(&child.name).ok_or_else(|| {
            node_error(
                child,
                ComponentError::unknown_component(&child.name).to_string(),
            )
        })?;
        if component != BuiltinComponent::Path {
            return Err(node_error(
                child,
                ComponentError::invalid_prop_combination("Svg only accepts Path children")
                    .to_string(),
            ));
        }
        reject_children(child)?;
        paths.push(
            svg_path_component(component_props(child, component)?)
                .map_err(|error| component_error(child, error))?,
        );
    }
    svg_component_node(props, paths).map_err(|error| component_error(node, error))
}

fn component_props(
    node: &SourceNode,
    component: BuiltinComponent,
) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| component_prop(component, prop))
        .collect()
}

fn component_prop(component: BuiltinComponent, prop: &SourceProp) -> DoweResult<ComponentProp> {
    validate_component_prop_source(component, prop)?;
    Ok(ComponentProp {
        name: prop.name.clone(),
        value: prop_value(prop)?,
    })
}

fn validate_component_prop_source(
    component: BuiltinComponent,
    prop: &SourceProp,
) -> DoweResult<()> {
    if matches!(
        component,
        BuiltinComponent::Drawer
            | BuiltinComponent::Modal
            | BuiltinComponent::AlertDialog
            | BuiltinComponent::Command
    ) && prop.name == "open"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("open", "signal bool path").to_string(),
        ));
    }
    if component == BuiltinComponent::Toast
        && prop.name == "source"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("source", "signal object path").to_string(),
        ));
    }
    if component == BuiltinComponent::AvatarGroup
        && prop.name == "items"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("items", "signal array path").to_string(),
        ));
    }
    if component == BuiltinComponent::ChatBox
        && prop.name == "messages"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("messages", "signal array path").to_string(),
        ));
    }
    if component == BuiltinComponent::ChatBox
        && matches!(
            prop.name.as_str(),
            "loading" | "sending" | "streaming" | "hasMore"
        )
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop(&prop.name, "signal bool path").to_string(),
        ));
    }
    if component == BuiltinComponent::DateRange
        && matches!(prop.name.as_str(), "start" | "end")
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop(&prop.name, "signal string path").to_string(),
        ));
    }
    if component == BuiltinComponent::ToggleGroup
        && prop.name == "value"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("value", "signal string path").to_string(),
        ));
    }
    if !is_known_component_prop(component, &prop.name)
        || allows_bare_component_reference(component, prop)
    {
        return Ok(());
    }
    if static_value_has_bareword(&prop.value) {
        Err(quoted_static_string_error(prop))
    } else {
        Ok(())
    }
}

fn allows_bare_component_reference(component: BuiltinComponent, prop: &SourceProp) -> bool {
    match (component, prop.name.as_str(), &prop.value) {
        (_, "show", SourceValue::Bareword(_))
            if !matches!(component, BuiltinComponent::Option | BuiltinComponent::Path) =>
        {
            true
        }
        (
            BuiltinComponent::Input
            | BuiltinComponent::Select
            | BuiltinComponent::Slider
            | BuiltinComponent::Checkbox
            | BuiltinComponent::Color
            | BuiltinComponent::Date
            | BuiltinComponent::RadioGroup
            | BuiltinComponent::Toggle
            | BuiltinComponent::ComboBox
            | BuiltinComponent::Editor
            | BuiltinComponent::ImageCropper
            | BuiltinComponent::PasswordField
            | BuiltinComponent::PhoneField
            | BuiltinComponent::PinField
            | BuiltinComponent::Textarea,
            "bind",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::DateRange, "start" | "end", SourceValue::Bareword(_))
        | (BuiltinComponent::ToggleGroup, "value", SourceValue::Bareword(_))
        | (BuiltinComponent::Candlestick, "data", SourceValue::Bareword(_))
        | (
            BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart,
            "data" | "series",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::Table, "data", SourceValue::Bareword(_))
        | (BuiltinComponent::AvatarGroup, "items", SourceValue::Bareword(_))
        | (
            BuiltinComponent::ChatBox,
            "messages" | "loading" | "sending" | "streaming" | "hasMore",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Button | BuiltinComponent::Avatar | BuiltinComponent::Empty,
            "onClick",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::ChatBox,
            "onSend" | "onLoadMore" | "onStop" | "onVoiceNote" | "onFileAttach" | "onCameraCapture",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Record,
            "onStart" | "onPause" | "onResume" | "onStop" | "onDiscard" | "onConfirm",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::ToggleGroup, "onChange", SourceValue::Bareword(_))
        | (BuiltinComponent::Countdown, "onComplete", SourceValue::Bareword(_))
        | (
            BuiltinComponent::Map,
            "onLocation" | "onLocationError" | "onRoute",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Fab | BuiltinComponent::FabAction,
            "onClick",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::Alert, "visible" | "onClose", SourceValue::Bareword(_))
        | (
            BuiltinComponent::Chip | BuiltinComponent::Modal | BuiltinComponent::AlertDialog,
            "onClose" | "onConfirm" | "onCancel",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Drawer
            | BuiltinComponent::Modal
            | BuiltinComponent::AlertDialog
            | BuiltinComponent::Command,
            "open",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::Toast, "source", SourceValue::Bareword(_)) => true,
        (BuiltinComponent::Alert, "message", SourceValue::Bareword(value)) => {
            is_dynamic_reference(value)
        }
        _ => false,
    }
}

fn is_known_component_prop(component: BuiltinComponent, name: &str) -> bool {
    let shared_style = !matches!(
        component,
        BuiltinComponent::Option
            | BuiltinComponent::FabAction
            | BuiltinComponent::ComboOption
            | BuiltinComponent::CsvColumn
            | BuiltinComponent::DragGroup
            | BuiltinComponent::DragItem
            | BuiltinComponent::Svg
            | BuiltinComponent::Path
    ) && matches!(
        name,
        "id" | "show"
            | "font"
            | "p"
            | "px"
            | "py"
            | "pl"
            | "pr"
            | "pt"
            | "pb"
            | "w"
            | "h"
            | "minW"
            | "minH"
            | "rounded"
            | "border"
    );
    shared_style
        || match component {
            BuiltinComponent::Box => {
                matches!(
                    name,
                    "bg" | "color" | "cover" | "overlay" | "animation" | "colSpan" | "rowSpan"
                )
            }
            BuiltinComponent::Section => {
                matches!(
                    name,
                    "bg" | "color"
                        | "background"
                        | "cover"
                        | "overlay"
                        | "animation"
                        | "colSpan"
                        | "rowSpan"
                )
            }
            BuiltinComponent::Flex => matches!(name, "justify" | "align" | "gap"),
            BuiltinComponent::Grid => {
                matches!(name, "columns" | "rows" | "justify" | "align" | "gap")
            }
            BuiltinComponent::Input | BuiltinComponent::Select => {
                matches!(
                    name,
                    "bind" | "variant" | "scheme" | "label" | "placeholder" | "labelFloating"
                )
            }
            BuiltinComponent::Code => {
                matches!(
                    name,
                    "lines" | "language" | "copyLabel" | "copiedLabel" | "variant" | "scheme"
                )
            }
            BuiltinComponent::Video => {
                matches!(
                    name,
                    "src" | "poster" | "autoplay" | "aspect" | "variant" | "scheme"
                )
            }
            BuiltinComponent::Audio => {
                matches!(
                    name,
                    "src" | "subtitle" | "avatarSrc" | "variant" | "scheme" | "color"
                )
            }
            BuiltinComponent::Image => matches!(
                name,
                "src"
                    | "alt"
                    | "aspect"
                    | "objectFit"
                    | "loading"
                    | "hideControls"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::Candlestick => {
                matches!(
                    name,
                    "data"
                        | "stream"
                        | "variant"
                        | "scheme"
                        | "upColor"
                        | "downColor"
                        | "emptyLabel"
                        | "maxPoints"
                )
            }
            BuiltinComponent::ArcChart => {
                matches!(
                    name,
                    "data"
                        | "variant"
                        | "scheme"
                        | "bg"
                        | "color"
                        | "size"
                        | "palette"
                        | "legendPosition"
                        | "emptyLabel"
                        | "loading"
                        | "hideLegend"
                        | "centerText"
                        | "centerValue"
                        | "thickness"
                        | "gap"
                        | "startAngle"
                        | "endAngle"
                        | "showInlineLabels"
                        | "hideValues"
                        | "showGlow"
                )
            }
            BuiltinComponent::AreaChart => {
                matches!(
                    name,
                    "data"
                        | "series"
                        | "variant"
                        | "scheme"
                        | "bg"
                        | "color"
                        | "size"
                        | "palette"
                        | "legendPosition"
                        | "emptyLabel"
                        | "loading"
                        | "hideLegend"
                        | "curve"
                        | "strokeWidth"
                        | "fillOpacity"
                        | "stacked"
                        | "hideLine"
                        | "showPoints"
                        | "hideGrid"
                        | "hideXAxis"
                        | "hideYAxis"
                        | "showGlow"
                )
            }
            BuiltinComponent::BarChart => {
                matches!(
                    name,
                    "data"
                        | "series"
                        | "variant"
                        | "scheme"
                        | "bg"
                        | "color"
                        | "size"
                        | "palette"
                        | "legendPosition"
                        | "emptyLabel"
                        | "loading"
                        | "hideLegend"
                        | "grouped"
                        | "stacked"
                        | "showValues"
                        | "barRadius"
                        | "hideGrid"
                        | "showGlow"
                )
            }
            BuiltinComponent::LineChart => {
                matches!(
                    name,
                    "data"
                        | "series"
                        | "variant"
                        | "scheme"
                        | "bg"
                        | "color"
                        | "size"
                        | "palette"
                        | "legendPosition"
                        | "emptyLabel"
                        | "loading"
                        | "hideLegend"
                        | "curve"
                        | "strokeWidth"
                        | "pointRadius"
                        | "hidePoints"
                        | "hideGrid"
                        | "hideXAxis"
                        | "hideYAxis"
                        | "showGradientFill"
                        | "showGlow"
                )
            }
            BuiltinComponent::PieChart => {
                matches!(
                    name,
                    "data"
                        | "variant"
                        | "scheme"
                        | "bg"
                        | "color"
                        | "size"
                        | "palette"
                        | "legendPosition"
                        | "emptyLabel"
                        | "loading"
                        | "hideLegend"
                        | "donut"
                        | "donutWidth"
                        | "centerLabel"
                        | "centerValue"
                        | "startAngle"
                        | "padAngle"
                        | "hideLabels"
                        | "hideValues"
                        | "hidePercentages"
                        | "showGlow"
                )
            }
            BuiltinComponent::Table => {
                matches!(
                    name,
                    "data"
                        | "variant"
                        | "scheme"
                        | "size"
                        | "striped"
                        | "bordered"
                        | "dividers"
                        | "emptyTitle"
                        | "emptyDescription"
                )
            }
            BuiltinComponent::Divider => matches!(name, "orientation" | "scheme"),
            BuiltinComponent::Option => matches!(name, "value" | "label" | "description"),
            BuiltinComponent::ComboBox => matches!(
                name,
                "bind"
                    | "value"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "label"
                    | "placeholder"
                    | "labelFloating"
                    | "searchPlaceholder"
                    | "emptyText"
                    | "loadingText"
                    | "loadingMoreText"
                    | "clearable"
                    | "disabled"
                    | "helpText"
                    | "errorText"
                    | "color"
            ),
            BuiltinComponent::ComboOption => matches!(
                name,
                "value" | "label" | "description" | "src" | "icon" | "disabled"
            ),
            BuiltinComponent::CsvField => matches!(
                name,
                "buttonText"
                    | "modalTitle"
                    | "instructions"
                    | "cancelText"
                    | "confirmText"
                    | "clearText"
                    | "previewTitle"
                    | "multiple"
                    | "showPreview"
                    | "previewRows"
                    | "previewPageSize"
                    | "errorText"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "color"
            ),
            BuiltinComponent::CsvColumn => matches!(name, "name" | "label"),
            BuiltinComponent::DragDrop => matches!(
                name,
                "emptyText"
                    | "direction"
                    | "allowGroupTransfer"
                    | "disabled"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "color"
            ),
            BuiltinComponent::DragGroup => matches!(name, "id" | "title"),
            BuiltinComponent::DragItem => {
                matches!(name, "id" | "label" | "description" | "disabled")
            }
            BuiltinComponent::Editor => matches!(
                name,
                "bind"
                    | "value"
                    | "placeholder"
                    | "label"
                    | "helpText"
                    | "errorText"
                    | "minHeight"
                    | "hideToolbar"
                    | "disabled"
                    | "readonly"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "color"
            ),
            BuiltinComponent::ImageCropper => matches!(
                name,
                "bind"
                    | "src"
                    | "alt"
                    | "accept"
                    | "placeholder"
                    | "label"
                    | "helpText"
                    | "errorText"
                    | "aspectRatio"
                    | "minWidth"
                    | "minHeight"
                    | "maxWidth"
                    | "maxHeight"
                    | "shape"
                    | "disabled"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "color"
            ),
            BuiltinComponent::PasswordField => matches!(
                name,
                "bind"
                    | "value"
                    | "placeholder"
                    | "label"
                    | "labelFloating"
                    | "helpText"
                    | "errorText"
                    | "hideStrength"
                    | "weakLabel"
                    | "mediumLabel"
                    | "strongLabel"
                    | "disabled"
                    | "readonly"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "color"
            ),
            BuiltinComponent::PhoneField => matches!(
                name,
                "bind"
                    | "value"
                    | "country"
                    | "dialCodeName"
                    | "placeholder"
                    | "label"
                    | "labelFloating"
                    | "searchPlaceholder"
                    | "emptyText"
                    | "loadingText"
                    | "priorityCountries"
                    | "disabled"
                    | "helpText"
                    | "errorText"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "color"
            ),
            BuiltinComponent::PinField => matches!(
                name,
                "bind"
                    | "value"
                    | "length"
                    | "type"
                    | "label"
                    | "helpText"
                    | "errorText"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "color"
            ),
            BuiltinComponent::Textarea => matches!(
                name,
                "bind"
                    | "value"
                    | "placeholder"
                    | "label"
                    | "labelFloating"
                    | "helpText"
                    | "errorText"
                    | "rows"
                    | "cols"
                    | "maxLength"
                    | "resize"
                    | "disabled"
                    | "readonly"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "color"
            ),
            BuiltinComponent::Button => matches!(
                name,
                "onClick"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "href"
                    | "navigate"
                    | "history"
                    | "target"
                    | "externalMode"
            ),
            BuiltinComponent::ToggleTheme => {
                matches!(
                    name,
                    "variant" | "scheme" | "size" | "lightLabel" | "darkLabel" | "color"
                )
            }
            BuiltinComponent::Fab => matches!(
                name,
                "position"
                    | "fixed"
                    | "offsetX"
                    | "offsetY"
                    | "icon"
                    | "label"
                    | "onClick"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "color"
            ),
            BuiltinComponent::FabAction => matches!(
                name,
                "label" | "icon" | "scheme" | "href" | "target" | "navigate" | "onClick" | "color"
            ),
            BuiltinComponent::Slider => matches!(
                name,
                "bind"
                    | "value"
                    | "min"
                    | "max"
                    | "step"
                    | "label"
                    | "name"
                    | "hideLabel"
                    | "scheme"
                    | "size"
                    | "color"
            ),
            BuiltinComponent::Dropzone => matches!(
                name,
                "accept"
                    | "multiple"
                    | "maxSize"
                    | "name"
                    | "label"
                    | "helpText"
                    | "errorText"
                    | "placeholder"
                    | "disabled"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "color"
            ),
            BuiltinComponent::Alert => {
                matches!(
                    name,
                    "type" | "message" | "visible" | "onClose" | "variant" | "scheme"
                )
            }
            BuiltinComponent::Card => {
                matches!(
                    name,
                    "variant"
                        | "scheme"
                        | "cover"
                        | "overlay"
                        | "animation"
                        | "colSpan"
                        | "rowSpan"
                )
            }
            BuiltinComponent::Svg => {
                matches!(name, "id" | "show" | "viewBox" | "color" | "w" | "h")
            }
            BuiltinComponent::Path => matches!(name, "d" | "fill"),
            BuiltinComponent::AppBar | BuiltinComponent::BottomBar => {
                matches!(
                    name,
                    "variant" | "scheme" | "bordered" | "blurred" | "boxed" | "floating"
                )
            }
            BuiltinComponent::Footer => {
                matches!(
                    name,
                    "variant" | "scheme" | "bordered" | "blurred" | "boxed"
                )
            }
            BuiltinComponent::SideNav => matches!(name, "variant" | "scheme" | "size" | "wide"),
            BuiltinComponent::Sidebar => {
                matches!(name, "variant" | "scheme" | "size" | "wide" | "color")
            }
            BuiltinComponent::NavMenu => {
                matches!(name, "variant" | "scheme" | "size" | "color")
            }
            BuiltinComponent::Scaffold => matches!(name, "boxed"),
            BuiltinComponent::Tabs => matches!(name, "variant" | "scheme" | "position"),
            BuiltinComponent::Tab => matches!(name, "id" | "label"),
            BuiltinComponent::Drawer => matches!(
                name,
                "open"
                    | "position"
                    | "variant"
                    | "scheme"
                    | "disableOverlayClose"
                    | "hideCloseButton"
            ),
            BuiltinComponent::Avatar => matches!(
                name,
                "src"
                    | "name"
                    | "alt"
                    | "href"
                    | "navigate"
                    | "history"
                    | "target"
                    | "externalMode"
                    | "onClick"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "status"
                    | "bordered"
                    | "color"
            ),
            BuiltinComponent::Badge => {
                matches!(name, "text" | "variant" | "scheme" | "position" | "color")
            }
            BuiltinComponent::Chip => {
                matches!(name, "onClose" | "variant" | "scheme" | "size" | "color")
            }
            BuiltinComponent::Skeleton => matches!(name, "variant" | "animation"),
            BuiltinComponent::Modal => matches!(
                name,
                "open"
                    | "onClose"
                    | "variant"
                    | "scheme"
                    | "disableOverlayClose"
                    | "hideCloseButton"
                    | "color"
            ),
            BuiltinComponent::AlertDialog => matches!(
                name,
                "open"
                    | "title"
                    | "description"
                    | "confirmText"
                    | "cancelText"
                    | "onConfirm"
                    | "onCancel"
                    | "variant"
                    | "scheme"
                    | "loading"
                    | "color"
            ),
            BuiltinComponent::Tooltip => {
                matches!(name, "label" | "position" | "variant" | "scheme" | "color")
            }
            BuiltinComponent::Toast => matches!(
                name,
                "source"
                    | "type"
                    | "title"
                    | "description"
                    | "position"
                    | "variant"
                    | "scheme"
                    | "showIcon"
                    | "color"
            ),
            BuiltinComponent::Dropdown => matches!(name, "scheme" | "color"),
            BuiltinComponent::Command => matches!(
                name,
                "open"
                    | "placeholder"
                    | "emptyText"
                    | "closeText"
                    | "navigateText"
                    | "selectText"
                    | "toggleText"
                    | "shortcut"
                    | "disableGlobalShortcut"
                    | "showFooter"
                    | "variant"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::AvatarGroup => matches!(
                name,
                "items"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "max"
                    | "autoFit"
                    | "inline"
                    | "bordered"
                    | "color"
            ),
            BuiltinComponent::ChatBox => matches!(
                name,
                "messages"
                    | "mode"
                    | "currentUserId"
                    | "userName"
                    | "userAvatar"
                    | "userStatus"
                    | "assistantName"
                    | "assistantAvatar"
                    | "showHeader"
                    | "placeholder"
                    | "showAttachments"
                    | "showVoiceNote"
                    | "showCamera"
                    | "loading"
                    | "sending"
                    | "streaming"
                    | "hasMore"
                    | "onSend"
                    | "onLoadMore"
                    | "onStop"
                    | "onVoiceNote"
                    | "onFileAttach"
                    | "onCameraCapture"
                    | "variant"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::Empty => matches!(
                name,
                "type"
                    | "title"
                    | "description"
                    | "href"
                    | "navigate"
                    | "history"
                    | "target"
                    | "externalMode"
                    | "onClick"
                    | "actionLabel"
                    | "variant"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::Marquee => matches!(
                name,
                "speed" | "pauseOnHover" | "reverse" | "orientation" | "fade" | "fadeColor" | "gap"
            ),
            BuiltinComponent::TypeWriter => matches!(
                name,
                "typeSpeed"
                    | "deleteSpeed"
                    | "afterTyped"
                    | "afterDeleted"
                    | "repeat"
                    | "bg"
                    | "color"
            ),
            BuiltinComponent::RichText => {
                matches!(
                    name,
                    "size" | "weight" | "spacing" | "bg" | "color" | "i18n"
                )
            }
            BuiltinComponent::Record => matches!(
                name,
                "name"
                    | "url"
                    | "disabled"
                    | "maxDuration"
                    | "onStart"
                    | "onPause"
                    | "onResume"
                    | "onStop"
                    | "onDiscard"
                    | "onConfirm"
                    | "variant"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::ToggleGroup => matches!(
                name,
                "value"
                    | "selected"
                    | "size"
                    | "wide"
                    | "vertical"
                    | "disabled"
                    | "ariaLabel"
                    | "onChange"
                    | "variant"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::Collapsible => matches!(
                name,
                "label" | "defaultOpen" | "disabled" | "variant" | "scheme" | "color"
            ),
            BuiltinComponent::Countdown => matches!(
                name,
                "target"
                    | "showDays"
                    | "showHours"
                    | "showMinutes"
                    | "showSeconds"
                    | "size"
                    | "daysLabel"
                    | "hoursLabel"
                    | "minutesLabel"
                    | "secondsLabel"
                    | "onComplete"
                    | "variant"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::Map => matches!(
                name,
                "centerLat"
                    | "centerLng"
                    | "zoom"
                    | "height"
                    | "width"
                    | "showControls"
                    | "showScale"
                    | "showLocationControl"
                    | "interactive"
                    | "routeStartLat"
                    | "routeStartLng"
                    | "routeEndLat"
                    | "routeEndLng"
                    | "onLocation"
                    | "onLocationError"
                    | "onRoute"
                    | "variant"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::Accordion => {
                matches!(name, "variant" | "scheme" | "multiple" | "color")
            }
            BuiltinComponent::Carousel => matches!(
                name,
                "autoplay"
                    | "autoplayInterval"
                    | "disableLoop"
                    | "hideControls"
                    | "hideIndicators"
                    | "showNavigation"
                    | "showCounter"
                    | "orientation"
                    | "scheme"
                    | "size"
                    | "indicatorType"
                    | "title"
                    | "slideWidth"
                    | "slideHeight"
                    | "slidesPerView"
                    | "gap"
                    | "color"
            ),
            BuiltinComponent::Checkbox => {
                matches!(
                    name,
                    "bind" | "checked" | "label" | "name" | "disabled" | "scheme" | "color"
                )
            }
            BuiltinComponent::Color => matches!(
                name,
                "bind"
                    | "value"
                    | "label"
                    | "placeholder"
                    | "labelFloating"
                    | "helpText"
                    | "errorText"
                    | "showHex"
                    | "showRgb"
                    | "showCmyk"
                    | "showOklch"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "color"
            ),
            BuiltinComponent::Date => matches!(
                name,
                "bind"
                    | "value"
                    | "label"
                    | "placeholder"
                    | "labelFloating"
                    | "helpText"
                    | "errorText"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "min"
                    | "max"
                    | "color"
            ),
            BuiltinComponent::DateRange => matches!(
                name,
                "start"
                    | "end"
                    | "startValue"
                    | "endValue"
                    | "label"
                    | "placeholder"
                    | "labelFloating"
                    | "helpText"
                    | "errorText"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "min"
                    | "max"
                    | "color"
            ),
            BuiltinComponent::RadioGroup => matches!(
                name,
                "bind" | "label" | "name" | "info" | "error" | "scheme" | "size" | "color"
            ),
            BuiltinComponent::Toggle => matches!(
                name,
                "bind"
                    | "checked"
                    | "label"
                    | "labelLeft"
                    | "labelRight"
                    | "name"
                    | "disabled"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::Title | BuiltinComponent::Text => {
                matches!(
                    name,
                    "size" | "weight" | "spacing" | "bg" | "color" | "i18n"
                )
            }
        }
}

fn static_value_has_bareword(value: &SourceValue) -> bool {
    match value {
        SourceValue::Bareword(_) => true,
        SourceValue::Object(entries) => entries.iter().any(|entry| match entry {
            SourceObjectEntry::KeyValue { value, .. } => static_value_has_bareword(value),
            SourceObjectEntry::Spread(_) => false,
        }),
        SourceValue::Array(values) => values.iter().any(static_value_has_bareword),
        SourceValue::String(_)
        | SourceValue::Number(_)
        | SourceValue::Boolean(_)
        | SourceValue::Null => false,
    }
}

fn prop_value(prop: &SourceProp) -> DoweResult<PropValue> {
    match &prop.value {
        SourceValue::String(value) | SourceValue::Bareword(value) => {
            Ok(PropValue::String(value.clone()))
        }
        SourceValue::Number(value) => Ok(PropValue::Number(value.clone())),
        SourceValue::Boolean(value) => Ok(PropValue::Boolean(*value)),
        SourceValue::Object(entries) => {
            Ok(PropValue::Responsive(responsive_entries(prop, entries)?))
        }
        SourceValue::Null | SourceValue::Array(_) => Err(DoweError::at_path(
            &prop.location.path,
            format!(
                "{}:{}: prop `{}` has unsupported value",
                prop.location.line, prop.location.column, prop.name
            ),
        )),
    }
}

fn responsive_entries(
    prop: &SourceProp,
    entries: &[SourceObjectEntry],
) -> DoweResult<Vec<ResponsivePropEntry>> {
    entries
        .iter()
        .map(|entry| match entry {
            SourceObjectEntry::KeyValue { key, value } => Ok(ResponsivePropEntry {
                breakpoint: key.clone(),
                value: prop_scalar(prop, value)?,
            }),
            SourceObjectEntry::Spread(_) => Err(DoweError::at_path(
                &prop.location.path,
                format!(
                    "{}:{}: prop `{}` cannot use object spread",
                    prop.location.line, prop.location.column, prop.name
                ),
            )),
        })
        .collect()
}

fn prop_scalar(prop: &SourceProp, value: &SourceValue) -> DoweResult<PropScalar> {
    match value {
        SourceValue::String(value) | SourceValue::Bareword(value) => {
            Ok(PropScalar::String(value.clone()))
        }
        SourceValue::Number(value) => Ok(PropScalar::Number(value.clone())),
        SourceValue::Boolean(value) => Ok(PropScalar::Boolean(*value)),
        SourceValue::Null | SourceValue::Array(_) | SourceValue::Object(_) => {
            Err(DoweError::at_path(
                &prop.location.path,
                format!(
                    "{}:{}: responsive prop `{}` has unsupported value",
                    prop.location.line, prop.location.column, prop.name
                ),
            ))
        }
    }
}

fn required_text_child(node: &SourceNode, component: BuiltinComponent) -> DoweResult<String> {
    text_child_value(node)?.ok_or_else(|| {
        node_error(
            node,
            format!("{} requires a text child", component.as_str()),
        )
    })
}

fn reject_text_prop(node: &SourceNode, component: BuiltinComponent) -> DoweResult<()> {
    if let Some(prop) = node.prop("text") {
        Err(prop_error(
            prop,
            ComponentError::unknown_prop(component, "text").to_string(),
        ))
    } else {
        Ok(())
    }
}

fn text_child_value(node: &SourceNode) -> DoweResult<Option<String>> {
    if node.children.is_empty() {
        return Ok(None);
    }
    if node.children.len() != 1 {
        return Err(node_error(node, "text components accept one text child"));
    }
    let child = &node.children[0];
    if !child.children.is_empty() || !child.props.is_empty() {
        return Err(node_error(child, "text child must be plain text"));
    }
    let mut parts = Vec::new();
    parts.push(text_token(&child.name));
    parts.extend(child.args.iter().map(SourceValue::to_source).map(|value| {
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            value[1..value.len() - 1].to_string()
        } else {
            value
        }
    }));
    Ok(Some(parts.join(" ")))
}

fn text_child_line(node: &SourceNode) -> DoweResult<String> {
    if !node.children.is_empty() || !node.props.is_empty() {
        return Err(node_error(node, "text child must be plain text"));
    }
    let mut parts = Vec::new();
    parts.push(text_token(&node.name));
    parts.extend(node.args.iter().map(SourceValue::to_source).map(|value| {
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            value[1..value.len() - 1].to_string()
        } else {
            value
        }
    }));
    Ok(parts.join(" "))
}

fn text_token(value: &str) -> String {
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn reject_children(node: &SourceNode) -> DoweResult<()> {
    if node.children.is_empty() {
        Ok(())
    } else {
        Err(node_error(
            node,
            "children are not valid for this component",
        ))
    }
}

fn single_export(file: &SourceFile) -> DoweResult<&SourceNode> {
    let exports = file
        .nodes
        .iter()
        .filter(|node| matches!(node.name.as_str(), "layout" | "page"))
        .collect::<Vec<_>>();
    if exports.len() != 1
        || file
            .nodes
            .iter()
            .any(|node| !matches!(node.name.as_str(), "type" | "layout" | "page"))
    {
        return Err(DoweError::at_path(
            &file.path,
            "view modules must declare one export",
        ));
    }
    Ok(exports[0])
}

fn single_tree(path: &Path, mut nodes: Vec<ViewNode>) -> DoweResult<ViewNode> {
    if nodes.len() != 1 {
        return Err(DoweError::at_path(
            path,
            "layout and page exports must contain one root view node",
        ));
    }
    Ok(nodes.remove(0))
}

fn required_prop_string(node: &SourceNode, name: &str) -> DoweResult<String> {
    node.prop(name)
        .and_then(|prop| prop.value.as_required_string())
        .ok_or_else(|| node_error(node, format!("missing `{name}`")))
}

fn required_path_prop(node: &SourceNode) -> DoweResult<String> {
    let prop = node
        .prop("path")
        .ok_or_else(|| node_error(node, "missing `path`"))?;
    match &prop.value {
        SourceValue::String(value) => Ok(value.clone()),
        _ => Err(quoted_static_string_error(prop)),
    }
}

fn used_components(declaration: &ViewDeclaration) -> Vec<String> {
    let mut used = vec![declaration.component.clone()];
    for child in &declaration.children {
        used.extend(used_components(child));
    }
    used
}

fn collect_sections(path: &Path, tree: &ViewNode) -> DoweResult<Vec<ViewSection>> {
    let mut sections = Vec::new();
    let mut seen = HashSet::new();
    collect_sections_from_node(path, tree, &mut sections, &mut seen)?;
    Ok(sections)
}

fn collect_sections_from_node(
    path: &Path,
    node: &ViewNode,
    sections: &mut Vec<ViewSection>,
    seen: &mut HashSet<String>,
) -> DoweResult<()> {
    if let Some(id) = node_element_props(node).and_then(|props| props.id.as_ref()) {
        if !seen.insert(id.clone()) {
            return Err(DoweError::at_path(
                path,
                format!("duplicate section id `{id}` in route"),
            ));
        }
        sections.push(ViewSection { id: id.clone() });
    }
    for group in node_child_groups(node) {
        for child in group {
            collect_sections_from_node(path, child, sections, seen)?;
        }
    }
    Ok(())
}

fn collect_navigation_actions(tree: &ViewNode, route_id: &str) -> Vec<ViewNavigationAction> {
    let mut actions = Vec::new();
    collect_navigation_actions_from_node(tree, route_id, &mut actions);
    actions
}

fn collect_navigation_actions_from_node(
    node: &ViewNode,
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    if let Some(action) = navigation_action(node) {
        actions.push(ViewNavigationAction {
            id: format!("nav-{}-{}", route_id, actions.len()),
            action: action.clone(),
        });
    }
    match node {
        ViewNode::SideNav { items, .. } | ViewNode::Sidebar { items, .. } => {
            collect_side_nav_navigation_actions(items, route_id, actions);
        }
        ViewNode::NavMenu { items, .. } => {
            collect_nav_menu_navigation_actions(items, route_id, actions);
        }
        ViewNode::Dropdown { entries, .. } => {
            collect_overlay_entry_navigation_actions(entries, route_id, actions);
        }
        ViewNode::Command { entries, .. } => {
            collect_command_entry_navigation_actions(entries, route_id, actions);
        }
        ViewNode::AvatarGroup { items, .. } => {
            collect_avatar_group_navigation_actions(items, route_id, actions);
        }
        ViewNode::Fab {
            actions: fab_actions,
            ..
        } => {
            for action in fab_actions {
                if let Some(navigation) = action.navigation.as_ref() {
                    actions.push(ViewNavigationAction {
                        id: format!("nav-{}-{}", route_id, actions.len()),
                        action: navigation.clone(),
                    });
                }
            }
        }
        _ => {}
    }
    for group in node_child_groups(node) {
        for child in group {
            collect_navigation_actions_from_node(child, route_id, actions);
        }
    }
}

fn collect_avatar_group_navigation_actions(
    items: &[dowe_components::AvatarGroupItem],
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    for item in items {
        if let Some(action) = item.navigation.as_ref() {
            actions.push(ViewNavigationAction {
                id: format!("nav-{}-{}", route_id, actions.len()),
                action: action.clone(),
            });
        }
    }
}

fn collect_side_nav_navigation_actions(
    items: &[dowe_components::SideNavItem],
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    for item in items {
        match item {
            dowe_components::SideNavItem::Header(props)
            | dowe_components::SideNavItem::Item(props) => {
                if let Some(action) = props.navigation.as_ref() {
                    actions.push(ViewNavigationAction {
                        id: format!("nav-{}-{}", route_id, actions.len()),
                        action: action.clone(),
                    });
                }
            }
            dowe_components::SideNavItem::Submenu { items, .. } => {
                for props in items {
                    if let Some(action) = props.navigation.as_ref() {
                        actions.push(ViewNavigationAction {
                            id: format!("nav-{}-{}", route_id, actions.len()),
                            action: action.clone(),
                        });
                    }
                }
            }
            dowe_components::SideNavItem::Divider => {}
        }
    }
}

fn collect_nav_menu_navigation_actions(
    items: &[dowe_components::NavMenuItem],
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    for item in items {
        match item {
            dowe_components::NavMenuItem::Item(props) => {
                collect_nav_menu_entry_navigation_action(props, route_id, actions);
            }
            dowe_components::NavMenuItem::Submenu { items, .. } => {
                for props in items {
                    collect_nav_menu_entry_navigation_action(props, route_id, actions);
                }
            }
            dowe_components::NavMenuItem::Megamenu { .. } => {}
        }
    }
}

fn collect_nav_menu_entry_navigation_action(
    props: &dowe_components::NavMenuItemProps,
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    if let Some(action) = props.navigation.as_ref() {
        actions.push(ViewNavigationAction {
            id: format!("nav-{}-{}", route_id, actions.len()),
            action: action.clone(),
        });
    }
}

fn collect_overlay_entry_navigation_actions(
    entries: &[dowe_components::OverlayEntry],
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    for entry in entries {
        if let dowe_components::OverlayEntry::Item(props) = entry {
            collect_overlay_item_navigation_action(props, route_id, actions);
        }
    }
}

fn collect_command_entry_navigation_actions(
    entries: &[dowe_components::CommandEntry],
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    for entry in entries {
        match entry {
            dowe_components::CommandEntry::Item(props) => {
                collect_overlay_item_navigation_action(props, route_id, actions)
            }
            dowe_components::CommandEntry::Group { items, .. } => {
                for props in items {
                    collect_overlay_item_navigation_action(props, route_id, actions);
                }
            }
        }
    }
}

fn collect_overlay_item_navigation_action(
    props: &dowe_components::OverlayItemProps,
    route_id: &str,
    actions: &mut Vec<ViewNavigationAction>,
) {
    if let Some(action) = props.navigation.as_ref() {
        actions.push(ViewNavigationAction {
            id: format!("nav-{}-{}", route_id, actions.len()),
            action: action.clone(),
        });
    }
}

fn validate_reactive_view_tree(
    path: &Path,
    tree: &ViewNode,
    environment: &EnvironmentConfig,
) -> DoweResult<()> {
    match tree {
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => {
            let signal_names = unique_names(
                path,
                signals.iter().map(|signal| signal.name.as_str()),
                "signal",
            )?;
            let action_names = unique_names(
                path,
                actions.iter().map(|action| action.name.as_str()),
                "action",
            )?;
            let signal_types = signals
                .iter()
                .map(|signal| {
                    (
                        signal.name.clone(),
                        signal
                            .schema
                            .clone()
                            .unwrap_or_else(|| signal.initial.clone()),
                    )
                })
                .collect::<HashMap<_, _>>();
            for action in actions {
                validate_action_references(path, action, &signal_names, environment)?;
            }
            let locals = HashMap::new();
            for child in children {
                validate_node_references(path, child, &signal_types, &action_names, &locals)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn unique_names<'a>(
    path: &Path,
    names: impl Iterator<Item = &'a str>,
    kind: &str,
) -> DoweResult<HashSet<String>> {
    let mut output = HashSet::new();
    for name in names {
        if !output.insert(name.to_string()) {
            return Err(DoweError::at_path(
                path,
                format!("duplicate {kind} `{name}`"),
            ));
        }
    }
    Ok(output)
}

fn validate_action_references(
    path: &Path,
    action: &ViewAction,
    signals: &HashSet<String>,
    environment: &EnvironmentConfig,
) -> DoweResult<()> {
    match &action.kind {
        ViewActionKind::Request(request) => {
            validate_request_base_env(path, environment, request.base_env.as_deref())?;
            validate_optional_body_name(path, signals, request.body.as_deref())?;
            validate_optional_signal_name(path, signals, request.update.as_deref(), "update")?;
            validate_optional_signal_name(path, signals, request.reset.as_deref(), "reset")?;
            validate_optional_signal_name(
                path,
                signals,
                request.success_alert.as_deref(),
                "successAlert",
            )?;
            validate_optional_signal_name(
                path,
                signals,
                request.error_alert.as_deref(),
                "errorAlert",
            )?;
        }
        ViewActionKind::Assign(assign) => {
            validate_signal_name(path, signals, &assign.target, "target")?;
            let source_root = path_root(&assign.source);
            if !signals.contains(source_root) && source_root != "item" {
                return Err(DoweError::at_path(
                    path,
                    format!("unknown assign source `{}`", assign.source),
                ));
            }
        }
        ViewActionKind::Reset(reset) => {
            validate_signal_name(path, signals, &reset.target, "target")?;
        }
    }
    Ok(())
}

fn validate_request_base_env(
    path: &Path,
    environment: &EnvironmentConfig,
    name: Option<&str>,
) -> DoweResult<()> {
    let Some(name) = name else {
        return Ok(());
    };
    let variable = environment.variable(name).ok_or_else(|| {
        DoweError::at_path(path, format!("unknown environment variable `{name}`"))
    })?;
    if variable.visibility != EnvironmentVisibility::Client {
        return Err(DoweError::at_path(
            path,
            format!("environment variable `{name}` is server-only and cannot be used from views"),
        ));
    }
    if let Some(value) = variable.resolved_value.as_deref()
        && !value.is_empty()
        && !valid_request_base_url(value)
    {
        return Err(DoweError::at_path(
            path,
            format!("environment variable `{name}` must resolve to an http or https URL"),
        ));
    }
    Ok(())
}

fn valid_request_base_url(value: &str) -> bool {
    if value.contains('?') || value.contains('#') || value.chars().any(char::is_whitespace) {
        return false;
    }
    let Some(rest) = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
    else {
        return false;
    };
    !rest.is_empty() && !rest.starts_with('/') && !rest.starts_with('?') && !rest.starts_with('#')
}

fn validate_node_references(
    path: &Path,
    node: &ViewNode,
    signals: &HashMap<String, ViewSignalValue>,
    actions: &HashSet<String>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
) -> DoweResult<()> {
    if let Some(props) = node_element_props(node) {
        if let Some(VisibilityCondition::Signal(show)) = props.show.as_ref() {
            validate_typed_path(
                path,
                signals,
                locals,
                show,
                "show",
                ViewPathExpectation::Bool,
            )?;
        }
        if let Some(binding) = props.bind.as_ref() {
            let expectation = match node {
                ViewNode::Checkbox { .. } | ViewNode::Toggle { .. } => ViewPathExpectation::Bool,
                ViewNode::Slider { .. } => ViewPathExpectation::Number,
                _ => ViewPathExpectation::String,
            };
            validate_typed_path(path, signals, locals, binding, "bind", expectation)?;
        }
        if let Some(action) = props.on_click.as_ref()
            && !actions.contains(action)
        {
            return Err(DoweError::at_path(
                path,
                format!("unknown action `{action}`"),
            ));
        }
    }

    match node {
        ViewNode::Scope { children, .. } => {
            for child in children {
                validate_node_references(path, child, signals, actions, locals)?;
            }
        }
        ViewNode::Each {
            item,
            collection,
            key,
            children,
        } => {
            let Some(collection_type) = signals.get(collection) else {
                return Err(DoweError::at_path(
                    path,
                    format!("unknown signal `{collection}` in `collection`"),
                ));
            };
            let ViewSignalValue::Array(items) = collection_type else {
                return Err(DoweError::at_path(
                    path,
                    format!("signal `{collection}` in `collection` must be an array"),
                ));
            };
            if path_root(key) != item {
                return Err(DoweError::at_path(
                    path,
                    format!("`each` key `{key}` must start with `{item}`"),
                ));
            }
            let mut scoped = locals.clone();
            scoped.insert(item.clone(), items.first().cloned());
            validate_typed_path(path, signals, &scoped, key, "key", ViewPathExpectation::Any)?;
            for child in children {
                validate_node_references(path, child, signals, actions, &scoped)?;
            }
        }
        ViewNode::Title { props, value } | ViewNode::Text { props, value } => {
            if props.i18n.is_some() && is_dynamic_reference(value) {
                return Err(DoweError::at_path(
                    path,
                    "`i18n` requires a static fallback text child",
                ));
            }
            if is_dynamic_reference(value) {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    value,
                    "text",
                    ViewPathExpectation::String,
                )?;
            }
        }
        ViewNode::Alert { props } => {
            if is_dynamic_reference(&props.message) {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    &props.message,
                    "message",
                    ViewPathExpectation::String,
                )?;
            }
            if let Some(visible) = props.visible.as_ref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    visible,
                    "visible",
                    ViewPathExpectation::Bool,
                )?;
            }
            if let Some(action) = props.on_close.as_ref()
                && !actions.contains(action)
            {
                return Err(DoweError::at_path(
                    path,
                    format!("unknown action `{action}`"),
                ));
            }
        }
        ViewNode::Candlestick { props } => {
            validate_candlestick_data(path, signals, &props.data)?;
        }
        ViewNode::ArcChart { props } => {
            validate_category_chart_common(path, signals, &props.common, "ArcChart")?;
        }
        ViewNode::PieChart { props } => {
            validate_category_chart_common(path, signals, &props.common, "PieChart")?;
        }
        ViewNode::BarChart { props } => {
            validate_category_or_series_chart_common(path, signals, &props.common, "BarChart")?;
        }
        ViewNode::AreaChart { props } => {
            validate_point_or_series_chart_common(path, signals, &props.common, "AreaChart")?;
        }
        ViewNode::LineChart { props } => {
            validate_point_or_series_chart_common(path, signals, &props.common, "LineChart")?;
        }
        ViewNode::Table { props } => {
            validate_table_data(path, signals, &props.data, &props.columns)?;
        }
        ViewNode::SideNav { items, .. } | ViewNode::Sidebar { items, .. } => {
            validate_side_nav_actions(path, items, actions)?;
        }
        ViewNode::NavMenu { items, .. } => {
            validate_nav_menu_actions(path, items, actions)?;
            for group in node_child_groups(node) {
                for child in group {
                    validate_node_references(path, child, signals, actions, locals)?;
                }
            }
        }
        ViewNode::Drawer { props, children } => {
            validate_typed_path(
                path,
                signals,
                locals,
                &props.open,
                "open",
                ViewPathExpectation::Bool,
            )?;
            for child in children {
                validate_node_references(path, child, signals, actions, locals)?;
            }
        }
        ViewNode::Chip { props, .. } => {
            validate_optional_action(path, actions, props.on_close.as_deref())?;
        }
        ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } => {
            validate_typed_path(
                path,
                signals,
                locals,
                &props.open,
                "open",
                ViewPathExpectation::Bool,
            )?;
            validate_optional_action(path, actions, props.on_close.as_deref())?;
            for child in header.iter().chain(body).chain(footer) {
                validate_node_references(path, child, signals, actions, locals)?;
            }
        }
        ViewNode::AlertDialog { props } => {
            validate_typed_path(
                path,
                signals,
                locals,
                &props.open,
                "open",
                ViewPathExpectation::Bool,
            )?;
            validate_optional_action(path, actions, props.on_confirm.as_deref())?;
            validate_optional_action(path, actions, props.on_cancel.as_deref())?;
        }
        ViewNode::Toast { props } => {
            if let Some(source) = props.source.as_ref() {
                validate_toast_source(path, signals, locals, source)?;
            }
        }
        ViewNode::Dropdown {
            trigger,
            header,
            entries,
            footer,
            ..
        } => {
            validate_overlay_entry_actions(path, entries, actions)?;
            for child in trigger.iter().chain(header).chain(footer) {
                validate_node_references(path, child, signals, actions, locals)?;
            }
        }
        ViewNode::Command { props, entries } => {
            if let Some(open) = props.open.as_ref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    open,
                    "open",
                    ViewPathExpectation::Bool,
                )?;
            }
            validate_command_entry_actions(path, entries, actions)?;
        }
        ViewNode::AvatarGroup { props, items } => {
            if let Some(source) = props.items.as_ref() {
                validate_avatar_group_items(path, signals, locals, source)?;
            }
            validate_avatar_group_actions(path, items, actions)?;
        }
        ViewNode::ChatBox { props } => {
            validate_chat_box_messages(path, signals, locals, &props.messages)?;
            validate_optional_typed_path(
                path,
                signals,
                locals,
                props.loading.as_deref(),
                "loading",
                ViewPathExpectation::Bool,
            )?;
            validate_optional_typed_path(
                path,
                signals,
                locals,
                props.sending.as_deref(),
                "sending",
                ViewPathExpectation::Bool,
            )?;
            validate_optional_typed_path(
                path,
                signals,
                locals,
                props.streaming.as_deref(),
                "streaming",
                ViewPathExpectation::Bool,
            )?;
            validate_optional_typed_path(
                path,
                signals,
                locals,
                props.has_more.as_deref(),
                "hasMore",
                ViewPathExpectation::Bool,
            )?;
            validate_optional_action(path, actions, props.on_send.as_deref())?;
            validate_optional_action(path, actions, props.on_load_more.as_deref())?;
            validate_optional_action(path, actions, props.on_stop.as_deref())?;
            validate_optional_action(path, actions, props.on_voice_note.as_deref())?;
            validate_optional_action(path, actions, props.on_file_attach.as_deref())?;
            validate_optional_action(path, actions, props.on_camera_capture.as_deref())?;
        }
        ViewNode::DateRange { props } => {
            if let Some(start) = props.start.as_ref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    start,
                    "start",
                    ViewPathExpectation::String,
                )?;
            }
            if let Some(end) = props.end.as_ref() {
                validate_typed_path(
                    path,
                    signals,
                    locals,
                    end,
                    "end",
                    ViewPathExpectation::String,
                )?;
            }
        }
        ViewNode::Fab {
            actions: fab_actions,
            ..
        } => {
            for action in fab_actions {
                validate_optional_action(path, actions, action.on_click.as_deref())?;
            }
        }
        _ => {
            for group in node_child_groups(node) {
                for child in group {
                    validate_node_references(path, child, signals, actions, locals)?;
                }
            }
        }
    }
    Ok(())
}

fn validate_optional_action(
    path: &Path,
    actions: &HashSet<String>,
    action: Option<&str>,
) -> DoweResult<()> {
    if let Some(action) = action
        && !actions.contains(action)
    {
        return Err(DoweError::at_path(
            path,
            format!("unknown action `{action}`"),
        ));
    }
    Ok(())
}

fn validate_optional_typed_path(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
    value: Option<&str>,
    label: &str,
    expectation: ViewPathExpectation,
) -> DoweResult<()> {
    if let Some(value) = value {
        validate_typed_path(path, signals, locals, value, label, expectation)?;
    }
    Ok(())
}

fn validate_avatar_group_items(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
    source: &str,
) -> DoweResult<()> {
    let Some(value) = signal_path_value(path, signals, locals, source, "items")? else {
        return Ok(());
    };
    let ViewSignalValue::Array(items) = value else {
        return Err(DoweError::at_path(
            path,
            format!("invalid signal path `{source}` in `items`: expected array"),
        ));
    };
    for item in items {
        let ViewSignalValue::Object(fields) = item else {
            return Err(DoweError::at_path(
                path,
                "AvatarGroup items must be objects",
            ));
        };
        for field in ["src", "name", "alt", "href", "onClick"] {
            if let Some(value) = object_field(&fields, field)
                && !matches!(value, ViewSignalValue::String(_))
            {
                return Err(DoweError::at_path(
                    path,
                    format!("AvatarGroup item field `{field}` must be string"),
                ));
            }
        }
    }
    Ok(())
}

fn validate_chat_box_messages(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
    source: &str,
) -> DoweResult<()> {
    let Some(value) = signal_path_value(path, signals, locals, source, "messages")? else {
        return Ok(());
    };
    let ViewSignalValue::Array(items) = value else {
        return Err(DoweError::at_path(
            path,
            format!("invalid signal path `{source}` in `messages`: expected array"),
        ));
    };
    for item in items {
        let ViewSignalValue::Object(fields) = item else {
            return Err(DoweError::at_path(path, "ChatBox messages must be objects"));
        };
        for field in [
            "id", "userId", "name", "avatar", "message", "text", "type", "status",
        ] {
            if let Some(value) = object_field(&fields, field)
                && !matches!(value, ViewSignalValue::String(_))
            {
                return Err(DoweError::at_path(
                    path,
                    format!("ChatBox message field `{field}` must be string"),
                ));
            }
        }
        for field in ["own", "isOwn", "streaming"] {
            if let Some(value) = object_field(&fields, field)
                && !matches!(value, ViewSignalValue::Bool(_))
            {
                return Err(DoweError::at_path(
                    path,
                    format!("ChatBox message field `{field}` must be bool"),
                ));
            }
        }
        if let Some(value) = object_field(&fields, "createdAt")
            && !matches!(
                value,
                ViewSignalValue::String(_) | ViewSignalValue::Number(_)
            )
        {
            return Err(DoweError::at_path(
                path,
                "ChatBox message field `createdAt` must be string or number",
            ));
        }
    }
    Ok(())
}

fn validate_avatar_group_actions(
    path: &Path,
    items: &[dowe_components::AvatarGroupItem],
    actions: &HashSet<String>,
) -> DoweResult<()> {
    for item in items {
        validate_optional_action(path, actions, item.on_click.as_deref())?;
    }
    Ok(())
}

fn validate_toast_source(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
    source: &str,
) -> DoweResult<()> {
    let Some(value) = signal_path_value(path, signals, locals, source, "source")? else {
        return Ok(());
    };
    let ViewSignalValue::Object(fields) = value else {
        return Err(DoweError::at_path(
            path,
            format!("invalid signal path `{source}` in `source`: expected object"),
        ));
    };
    let visible = object_field(&fields, "visible")
        .ok_or_else(|| DoweError::at_path(path, "Toast source must include `visible`"))?;
    if !matches!(visible, ViewSignalValue::Bool(_)) {
        return Err(DoweError::at_path(
            path,
            "Toast source field `visible` must be bool",
        ));
    }
    let message = object_field(&fields, "message")
        .ok_or_else(|| DoweError::at_path(path, "Toast source must include `message`"))?;
    if !matches!(message, ViewSignalValue::String(_)) {
        return Err(DoweError::at_path(
            path,
            "Toast source field `message` must be string",
        ));
    }
    for field in ["title", "type"] {
        if let Some(value) = object_field(&fields, field)
            && !matches!(value, ViewSignalValue::String(_))
        {
            return Err(DoweError::at_path(
                path,
                format!("Toast source field `{field}` must be string"),
            ));
        }
    }
    Ok(())
}

fn signal_path_value(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
    value: &str,
    label: &str,
) -> DoweResult<Option<ViewSignalValue>> {
    let root = path_root(value);
    let mut resolved = if let Some(value) = signals.get(root) {
        Some(value.clone())
    } else if let Some(value) = locals.get(root) {
        value.clone()
    } else {
        return Err(DoweError::at_path(
            path,
            format!("unknown signal path `{value}` in `{label}`"),
        ));
    };
    let Some(mut resolved_value) = resolved.take() else {
        return Ok(None);
    };
    for field in value.split('.').skip(1) {
        let ViewSignalValue::Object(fields) = resolved_value else {
            return Err(DoweError::at_path(
                path,
                format!("unknown signal path `{value}` in `{label}`"),
            ));
        };
        let Some((_, next)) = fields.into_iter().find(|(name, _)| name == field) else {
            return Err(DoweError::at_path(
                path,
                format!("unknown signal path `{value}` in `{label}`"),
            ));
        };
        resolved_value = next;
    }
    Ok(Some(resolved_value))
}

fn object_field<'a>(
    fields: &'a [(String, ViewSignalValue)],
    name: &str,
) -> Option<&'a ViewSignalValue> {
    fields
        .iter()
        .find_map(|(field, value)| (field == name).then_some(value))
}

fn validate_overlay_entry_actions(
    path: &Path,
    entries: &[dowe_components::OverlayEntry],
    actions: &HashSet<String>,
) -> DoweResult<()> {
    for entry in entries {
        if let dowe_components::OverlayEntry::Item(props) = entry {
            validate_overlay_item_action(path, props, actions)?;
        }
    }
    Ok(())
}

fn validate_command_entry_actions(
    path: &Path,
    entries: &[dowe_components::CommandEntry],
    actions: &HashSet<String>,
) -> DoweResult<()> {
    for entry in entries {
        match entry {
            dowe_components::CommandEntry::Item(props) => {
                validate_overlay_item_action(path, props, actions)?
            }
            dowe_components::CommandEntry::Group { items, .. } => {
                for props in items {
                    validate_overlay_item_action(path, props, actions)?;
                }
            }
        }
    }
    Ok(())
}

fn validate_overlay_item_action(
    path: &Path,
    props: &dowe_components::OverlayItemProps,
    actions: &HashSet<String>,
) -> DoweResult<()> {
    validate_optional_action(path, actions, props.on_click.as_deref())
}

fn validate_candlestick_data(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    data: &str,
) -> DoweResult<()> {
    let root = path_root(data);
    let Some(collection_type) = signals.get(root) else {
        return Err(DoweError::at_path(
            path,
            format!("unknown signal `{root}` in `data`"),
        ));
    };
    let ViewSignalValue::Array(items) = collection_type else {
        return Err(DoweError::at_path(
            path,
            format!("signal `{root}` in `data` must be an array"),
        ));
    };
    for item in items {
        validate_candlestick_item(path, item)?;
    }
    Ok(())
}

fn validate_category_chart_common(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    props: &dowe_components::ChartCommonProps,
    component: &str,
) -> DoweResult<()> {
    let Some(data) = props.data.as_deref() else {
        return Err(DoweError::at_path(
            path,
            format!("{component} requires `data`"),
        ));
    };
    validate_chart_data(
        path,
        signals,
        data,
        "data",
        component,
        ChartDataShape::Category,
    )
}

fn validate_category_or_series_chart_common(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    props: &dowe_components::ChartCommonProps,
    component: &str,
) -> DoweResult<()> {
    if let Some(data) = props.data.as_deref() {
        validate_chart_data(
            path,
            signals,
            data,
            "data",
            component,
            ChartDataShape::Category,
        )?;
    }
    if let Some(series) = props.series.as_deref() {
        validate_chart_data(
            path,
            signals,
            series,
            "series",
            component,
            ChartDataShape::CategorySeries,
        )?;
    }
    Ok(())
}

fn validate_point_or_series_chart_common(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    props: &dowe_components::ChartCommonProps,
    component: &str,
) -> DoweResult<()> {
    if let Some(data) = props.data.as_deref() {
        validate_chart_data(
            path,
            signals,
            data,
            "data",
            component,
            ChartDataShape::Point,
        )?;
    }
    if let Some(series) = props.series.as_deref() {
        validate_chart_data(
            path,
            signals,
            series,
            "series",
            component,
            ChartDataShape::PointSeries,
        )?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum ChartDataShape {
    Category,
    Point,
    CategorySeries,
    PointSeries,
}

fn validate_chart_data(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    source: &str,
    prop: &str,
    component: &str,
    shape: ChartDataShape,
) -> DoweResult<()> {
    let root = path_root(source);
    let Some(collection_type) = signals.get(root) else {
        return Err(DoweError::at_path(
            path,
            format!("unknown signal `{root}` in `{prop}`"),
        ));
    };
    let ViewSignalValue::Array(items) = collection_type else {
        return Err(DoweError::at_path(
            path,
            format!("signal `{root}` in `{prop}` must be an array"),
        ));
    };
    for item in items {
        match shape {
            ChartDataShape::Category => validate_chart_category_item(path, item, component)?,
            ChartDataShape::Point => validate_chart_point_item(path, item, component)?,
            ChartDataShape::CategorySeries => {
                validate_chart_series_item(path, item, component, ChartDataShape::Category)?
            }
            ChartDataShape::PointSeries => {
                validate_chart_series_item(path, item, component, ChartDataShape::Point)?
            }
        }
    }
    Ok(())
}

fn validate_chart_series_item(
    path: &Path,
    item: &ViewSignalValue,
    component: &str,
    shape: ChartDataShape,
) -> DoweResult<()> {
    let ViewSignalValue::Object(fields) = item else {
        return Err(DoweError::at_path(
            path,
            format!("{component} series items must be objects"),
        ));
    };
    let name = chart_field(path, fields, component, "series", "name")?;
    if !matches!(
        name,
        ViewSignalValue::String(_) | ViewSignalValue::Number(_)
    ) {
        return Err(DoweError::at_path(
            path,
            format!("{component} series field `name` must be string or number"),
        ));
    }
    if let Some((_, color)) = fields.iter().find(|(field, _)| field == "color") {
        validate_chart_color(path, color, component)?;
    }
    let data = chart_field(path, fields, component, "series", "data")?;
    let ViewSignalValue::Array(items) = data else {
        return Err(DoweError::at_path(
            path,
            format!("{component} series field `data` must be an array"),
        ));
    };
    for item in items {
        match shape {
            ChartDataShape::Category => validate_chart_category_item(path, item, component)?,
            ChartDataShape::Point => validate_chart_point_item(path, item, component)?,
            ChartDataShape::CategorySeries | ChartDataShape::PointSeries => {}
        }
    }
    Ok(())
}

fn validate_chart_category_item(
    path: &Path,
    item: &ViewSignalValue,
    component: &str,
) -> DoweResult<()> {
    let ViewSignalValue::Object(fields) = item else {
        return Err(DoweError::at_path(
            path,
            format!("{component} data items must be objects"),
        ));
    };
    let label = chart_field(path, fields, component, "data item", "label")?;
    if !matches!(
        label,
        ViewSignalValue::String(_) | ViewSignalValue::Number(_)
    ) {
        return Err(DoweError::at_path(
            path,
            format!("{component} data item field `label` must be string or number"),
        ));
    }
    let value = chart_number_field(path, fields, component, "data item", "value")?;
    if value < 0.0 {
        return Err(DoweError::at_path(
            path,
            format!("{component} data item field `value` must be non-negative"),
        ));
    }
    if let Some((_, max)) = fields.iter().find(|(field, _)| field == "max") {
        let ViewSignalValue::Number(value) = max else {
            return Err(DoweError::at_path(
                path,
                format!("{component} data item field `max` must be number"),
            ));
        };
        let parsed = value.parse::<f64>().map_err(|_| {
            DoweError::at_path(
                path,
                format!("{component} data item field `max` must be number"),
            )
        })?;
        if parsed <= 0.0 || !parsed.is_finite() {
            return Err(DoweError::at_path(
                path,
                format!("{component} data item field `max` must be positive"),
            ));
        }
    }
    if let Some((_, color)) = fields.iter().find(|(field, _)| field == "color") {
        validate_chart_color(path, color, component)?;
    }
    Ok(())
}

fn validate_chart_point_item(
    path: &Path,
    item: &ViewSignalValue,
    component: &str,
) -> DoweResult<()> {
    let ViewSignalValue::Object(fields) = item else {
        return Err(DoweError::at_path(
            path,
            format!("{component} data items must be objects"),
        ));
    };
    chart_number_field(path, fields, component, "data item", "x")?;
    chart_number_field(path, fields, component, "data item", "y")?;
    Ok(())
}

fn validate_chart_color(path: &Path, value: &ViewSignalValue, component: &str) -> DoweResult<()> {
    let ViewSignalValue::String(value) = value else {
        return Err(DoweError::at_path(
            path,
            format!("{component} color fields must be strings"),
        ));
    };
    if dowe_components::ColorToken::from_name(value).is_some() {
        Ok(())
    } else {
        Err(DoweError::at_path(
            path,
            format!("{component} color fields must be Dowe color tokens"),
        ))
    }
}

fn chart_field<'a>(
    path: &Path,
    fields: &'a [(String, ViewSignalValue)],
    component: &str,
    item_name: &str,
    name: &str,
) -> DoweResult<&'a ViewSignalValue> {
    fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, value)| value)
        .ok_or_else(|| {
            DoweError::at_path(
                path,
                format!("{component} {item_name} must include `{name}`"),
            )
        })
}

fn chart_number_field(
    path: &Path,
    fields: &[(String, ViewSignalValue)],
    component: &str,
    item_name: &str,
    name: &str,
) -> DoweResult<f64> {
    let ViewSignalValue::Number(value) = chart_field(path, fields, component, item_name, name)?
    else {
        return Err(DoweError::at_path(
            path,
            format!("{component} {item_name} field `{name}` must be number"),
        ));
    };
    let parsed = value.parse::<f64>().map_err(|_| {
        DoweError::at_path(
            path,
            format!("{component} {item_name} field `{name}` must be number"),
        )
    })?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(DoweError::at_path(
            path,
            format!("{component} {item_name} field `{name}` must be number"),
        ))
    }
}

fn validate_table_data(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    data: &str,
    columns: &[dowe_components::TableColumn],
) -> DoweResult<()> {
    let root = path_root(data);
    let Some(collection_type) = signals.get(root) else {
        return Err(DoweError::at_path(
            path,
            format!("unknown signal `{root}` in `data`"),
        ));
    };
    let ViewSignalValue::Array(items) = collection_type else {
        return Err(DoweError::at_path(
            path,
            format!("signal `{root}` in `data` must be an array"),
        ));
    };
    for item in items {
        validate_table_item(path, item, columns)?;
    }
    Ok(())
}

fn validate_table_item(
    path: &Path,
    item: &ViewSignalValue,
    columns: &[dowe_components::TableColumn],
) -> DoweResult<()> {
    for column in columns {
        let value = table_field_value(path, item, &column.field)?;
        if matches!(
            value,
            ViewSignalValue::Array(_) | ViewSignalValue::Object(_)
        ) {
            return Err(DoweError::at_path(
                path,
                format!(
                    "Table column field `{}` must resolve to string, number or bool",
                    column.field
                ),
            ));
        }
    }
    Ok(())
}

fn table_field_value<'a>(
    path: &Path,
    item: &'a ViewSignalValue,
    field: &str,
) -> DoweResult<&'a ViewSignalValue> {
    let mut current = item;
    for segment in field.split('.') {
        let ViewSignalValue::Object(fields) = current else {
            return Err(DoweError::at_path(
                path,
                format!("unknown Table column field `{field}`"),
            ));
        };
        let Some((_, next)) = fields.iter().find(|(name, _)| name == segment) else {
            return Err(DoweError::at_path(
                path,
                format!("unknown Table column field `{field}`"),
            ));
        };
        current = next;
    }
    Ok(current)
}

fn validate_candlestick_item(path: &Path, item: &ViewSignalValue) -> DoweResult<()> {
    let ViewSignalValue::Object(fields) = item else {
        return Err(DoweError::at_path(
            path,
            "Candlestick data items must be objects",
        ));
    };
    let time = candlestick_field(path, fields, "time")?;
    if !matches!(
        time,
        ViewSignalValue::String(_) | ViewSignalValue::Number(_)
    ) {
        return Err(DoweError::at_path(
            path,
            "Candlestick data item field `time` must be string or number",
        ));
    }
    let open = candlestick_number_field(path, fields, "open")?;
    let high = candlestick_number_field(path, fields, "high")?;
    let low = candlestick_number_field(path, fields, "low")?;
    let close = candlestick_number_field(path, fields, "close")?;
    if high < open.max(close) || low > open.min(close) {
        return Err(DoweError::at_path(
            path,
            "Candlestick data item violates OHLC bounds",
        ));
    }
    Ok(())
}

fn candlestick_field<'a>(
    path: &Path,
    fields: &'a [(String, ViewSignalValue)],
    name: &str,
) -> DoweResult<&'a ViewSignalValue> {
    fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, value)| value)
        .ok_or_else(|| {
            DoweError::at_path(path, format!("Candlestick data item must include `{name}`"))
        })
}

fn candlestick_number_field(
    path: &Path,
    fields: &[(String, ViewSignalValue)],
    name: &str,
) -> DoweResult<f64> {
    let ViewSignalValue::Number(value) = candlestick_field(path, fields, name)? else {
        return Err(DoweError::at_path(
            path,
            format!("Candlestick data item field `{name}` must be number"),
        ));
    };
    value.parse::<f64>().map_err(|_| {
        DoweError::at_path(
            path,
            format!("Candlestick data item field `{name}` must be number"),
        )
    })
}

fn validate_side_nav_actions(
    path: &Path,
    items: &[dowe_components::SideNavItem],
    actions: &HashSet<String>,
) -> DoweResult<()> {
    for item in items {
        match item {
            dowe_components::SideNavItem::Header(props)
            | dowe_components::SideNavItem::Item(props) => {
                validate_side_nav_item_action(path, props, actions)?;
            }
            dowe_components::SideNavItem::Submenu { props, items, .. } => {
                validate_side_nav_item_action(path, props, actions)?;
                for props in items {
                    validate_side_nav_item_action(path, props, actions)?;
                }
            }
            dowe_components::SideNavItem::Divider => {}
        }
    }
    Ok(())
}

fn validate_side_nav_item_action(
    path: &Path,
    props: &dowe_components::SideNavItemProps,
    actions: &HashSet<String>,
) -> DoweResult<()> {
    if let Some(action) = props.on_click.as_ref()
        && !actions.contains(action)
    {
        return Err(DoweError::at_path(
            path,
            format!("unknown action `{action}`"),
        ));
    }
    Ok(())
}

fn validate_nav_menu_actions(
    path: &Path,
    items: &[dowe_components::NavMenuItem],
    actions: &HashSet<String>,
) -> DoweResult<()> {
    for item in items {
        match item {
            dowe_components::NavMenuItem::Item(props) => {
                validate_nav_menu_item_action(path, props, actions)?;
            }
            dowe_components::NavMenuItem::Submenu { props, items } => {
                validate_nav_menu_item_action(path, props, actions)?;
                for props in items {
                    validate_nav_menu_item_action(path, props, actions)?;
                }
            }
            dowe_components::NavMenuItem::Megamenu { props, .. } => {
                validate_nav_menu_item_action(path, props, actions)?;
            }
        }
    }
    Ok(())
}

fn validate_nav_menu_item_action(
    path: &Path,
    props: &dowe_components::NavMenuItemProps,
    actions: &HashSet<String>,
) -> DoweResult<()> {
    if let Some(action) = props.on_click.as_ref()
        && !actions.contains(action)
    {
        return Err(DoweError::at_path(
            path,
            format!("unknown action `{action}`"),
        ));
    }
    Ok(())
}

fn validate_optional_signal_name(
    path: &Path,
    signals: &HashSet<String>,
    value: Option<&str>,
    label: &str,
) -> DoweResult<()> {
    if let Some(value) = value {
        validate_signal_name(path, signals, value, label)?;
    }
    Ok(())
}

fn validate_optional_body_name(
    path: &Path,
    signals: &HashSet<String>,
    value: Option<&str>,
) -> DoweResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    let root = path_root(value);
    if signals.contains(root) || root == "item" {
        Ok(())
    } else {
        Err(DoweError::at_path(
            path,
            format!("unknown request body `{value}`"),
        ))
    }
}

fn validate_signal_name(
    path: &Path,
    signals: &HashSet<String>,
    value: &str,
    label: &str,
) -> DoweResult<()> {
    if signals.contains(value) {
        Ok(())
    } else {
        Err(DoweError::at_path(
            path,
            format!("unknown signal `{value}` in `{label}`"),
        ))
    }
}

#[derive(Clone, Copy)]
enum ViewPathExpectation {
    Any,
    String,
    Bool,
    Number,
}

fn validate_typed_path(
    path: &Path,
    signals: &HashMap<String, ViewSignalValue>,
    locals: &HashMap<String, Option<ViewSignalValue>>,
    value: &str,
    label: &str,
    expectation: ViewPathExpectation,
) -> DoweResult<()> {
    let root = path_root(value);
    let mut resolved = if let Some(value) = signals.get(root) {
        Some(value.clone())
    } else if let Some(value) = locals.get(root) {
        value.clone()
    } else {
        return Err(DoweError::at_path(
            path,
            format!("unknown signal path `{value}` in `{label}`"),
        ));
    };
    let Some(mut resolved_value) = resolved.take() else {
        return Ok(());
    };
    for field in value.split('.').skip(1) {
        let ViewSignalValue::Object(fields) = resolved_value else {
            return Err(DoweError::at_path(
                path,
                format!("unknown signal path `{value}` in `{label}`"),
            ));
        };
        let Some((_, next)) = fields.into_iter().find(|(name, _)| name == field) else {
            return Err(DoweError::at_path(
                path,
                format!("unknown signal path `{value}` in `{label}`"),
            ));
        };
        resolved_value = next;
    }
    let valid = match expectation {
        ViewPathExpectation::Any => true,
        ViewPathExpectation::String => matches!(resolved_value, ViewSignalValue::String(_)),
        ViewPathExpectation::Bool => matches!(resolved_value, ViewSignalValue::Bool(_)),
        ViewPathExpectation::Number => matches!(resolved_value, ViewSignalValue::Number(_)),
    };
    if valid {
        Ok(())
    } else {
        let expected = match expectation {
            ViewPathExpectation::Any => unreachable!(),
            ViewPathExpectation::String => "string",
            ViewPathExpectation::Bool => "bool",
            ViewPathExpectation::Number => "number",
        };
        Err(DoweError::at_path(
            path,
            format!("invalid signal path `{value}` in `{label}`: expected {expected}"),
        ))
    }
}

fn path_root(value: &str) -> &str {
    value.split('.').next().unwrap_or(value)
}

fn reactive_id(
    namespace: &str,
    scope_kind: &str,
    scope_name: &str,
    node: &SourceNode,
    name: &str,
) -> String {
    let source = format!(
        "{scope_kind}:{scope_name}:{}:{}:{name}",
        node.location.line, node.location.column
    );
    let mut hash = 0xcbf29ce484222325u64;
    for byte in namespace.bytes().chain([0]).chain(source.bytes()) {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let alphabet = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut value = hash;
    let mut id = String::with_capacity(8);
    for index in 0..8 {
        let digit = (value % 36) as usize;
        id.push(alphabet[digit] as char);
        value /= 36;
        if value == 0 {
            value = hash.rotate_left((index + 1) as u32);
        }
    }
    id
}

#[cfg(test)]
mod tests {
    use super::validate_view_source;
    use crate::model::{
        EnvironmentConfig, EnvironmentValueSource, EnvironmentVariable, EnvironmentVisibility,
    };
    use crate::parser::source_parser::parse_source_file;
    use dowe_components::{
        AvatarStatus, Breakpoint, ButtonSize, CarouselIndicatorType, CarouselOrientation,
        ChartCurve, ChartLegendPosition, ChartPalette, ChartSize, ChatBoxMode, ColorFamily,
        ColorToken, CommandEntry, ComponentVariant, CountdownSize, DividerOrientation, EmptyKind,
        ImageAspect, ImageLoading, ImageObjectFit, MapMarkerIcon, MarqueeOrientation, MarqueeSpeed,
        NavigationAction, OverlayCornerPosition, OverlayEntry, OverlayPosition, RichTextMarkStyle,
        SectionBackground, SkeletonAnimation, SkeletonVariant, SvgPathFill, TableColumnAlign,
        TableSize, ToastKind, VideoAspect, ViewActionKind, ViewAnimation, ViewIcon, ViewNode,
        VisibilityCondition,
    };
    use std::path::Path;

    #[test]
    fn parses_request_route_blocks_and_api_base_default() {
        let tree = parse_page(
            r#"page blogsPage
  signal blogs value:[]
  signal alert value:{ type:"info" message:"" visible:false }
  action load
    request GET route:"/api/blogs" update:blogs autoload:true
      onError alert:"No se pudieron cargar los blogs"
  Box
    Text size:"md"
      Crear y editar entradas usando signals, Input bind, Button onClick y Store."#,
        )
        .expect("tree");
        let ViewNode::Scope {
            actions, children, ..
        } = tree
        else {
            panic!("scope");
        };
        let ViewActionKind::Request(request) = &actions[0].kind else {
            panic!("request");
        };

        assert_eq!(actions[0].name, "load");
        assert_eq!(request.path, "/api/blogs");
        assert_eq!(request.base_env.as_deref(), Some("BACKEND_URL"));
        assert_eq!(request.error_alert.as_deref(), Some("alert"));
        assert_eq!(
            request.error_message.as_deref(),
            Some("No se pudieron cargar los blogs")
        );
        assert!(request.autoload);
        assert!(matches!(
            &children[0],
            ViewNode::Box { children, .. }
                if matches!(&children[0], ViewNode::Text { value, .. }
                    if value == "Crear y editar entradas usando signals, Input bind, Button onClick y Store.")
        ));
    }

    #[test]
    fn parses_request_path_alias_and_success_block_target() {
        let tree = parse_page(
            r#"page blogsPage
  signal blogs value:[]
  signal feedback value:{ type:"info" message:"" visible:false }
  action create
    request POST path:"/api/blogs" update:blogs
      onSuccess target:feedback alert:"Blog creado"
  Box
    Text
      Blogs"#,
        )
        .expect("tree");
        let ViewNode::Scope { actions, .. } = tree else {
            panic!("scope");
        };
        let ViewActionKind::Request(request) = &actions[0].kind else {
            panic!("request");
        };

        assert_eq!(request.path, "/api/blogs");
        assert_eq!(request.success_alert.as_deref(), Some("feedback"));
        assert_eq!(request.success_message.as_deref(), Some("Blog creado"));
    }

    #[test]
    fn parses_advanced_form_components_and_structural_children() {
        let tree = parse_page(
            r#"page advancedPage
  signal form value:{ role:"editor" notes:"" password:"" phone:"" pin:"" bio:"" avatar:"" }
  Box
    ComboBox bind:form.role label:"Role" placeholder:"Choose" clearable:true
      comboOption value:"admin" label:"Admin" description:"Full access"
      comboOption value:"editor" label:"Editor"
    CsvField label:"Import" buttonText:"Upload CSV"
      csvColumn name:"email" label:"Email"
    DragDrop label:"Tasks" direction:"horizontal"
      dragGroup id:"todo" title:"Todo"
        dragItem id:"draft" label:"Draft" description:"Prepare"
    Editor bind:form.notes label:"Notes" placeholder:"Write notes" minHeight:180
    ImageCropper bind:form.avatar label:"Avatar" shape:"circle"
    PasswordField bind:form.password label:"Password" hideStrength:false
    PhoneField bind:form.phone label:"Phone" country:"US"
    PinField bind:form.pin label:"Code" length:6 type:"number"
    Textarea bind:form.bio label:"Bio" rows:4 maxLength:160"#,
        )
        .expect("tree");
        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Box { children, .. } = &children[0] else {
            panic!("box");
        };

        assert!(matches!(
            &children[0],
            ViewNode::ComboBox { props, options }
                if props.clearable
                    && props.style.element.bind.as_deref() == Some("form.role")
                    && options.len() == 2
                    && options[0].description.as_deref() == Some("Full access")
        ));
        assert!(matches!(
            &children[1],
            ViewNode::CsvField { props, columns }
                if props.button_text == "Upload CSV"
                    && columns.len() == 1
                    && columns[0].name == "email"
        ));
        assert!(matches!(
            &children[2],
            ViewNode::DragDrop { props, groups, .. }
                if props.direction.as_str() == "horizontal"
                    && groups.len() == 1
                    && groups[0].items[0].id == "draft"
        ));
        assert!(matches!(
            &children[3],
            ViewNode::Editor { props }
                if props.min_height == 180
                    && props.style.element.bind.as_deref() == Some("form.notes")
        ));
        assert!(matches!(
            &children[4],
            ViewNode::ImageCropper { props }
                if props.shape.as_str() == "circle"
                    && props.style.element.bind.as_deref() == Some("form.avatar")
        ));
        assert!(matches!(
            &children[5],
            ViewNode::PasswordField { props } if !props.hide_strength
        ));
        assert!(matches!(
            &children[6],
            ViewNode::PhoneField { props } if props.country.as_deref() == Some("US")
        ));
        assert!(matches!(
            &children[7],
            ViewNode::PinField { props }
                if props.length == 6 && props.kind.as_str() == "number"
        ));
        assert!(matches!(
            &children[8],
            ViewNode::Textarea { props } if props.rows == 4 && props.max_length == Some(160)
        ));
    }

    #[test]
    fn rejects_duplicate_request_path_forms() {
        let error = parse_page(
            r#"page blogsPage
  signal blogs value:[]
  action load
    request GET "/api/blogs" route:"/api/blogs" update:blogs
  Box
    Text
      Blogs"#,
        )
        .expect_err("error");

        assert!(error.to_string().contains("only one route path"));
    }

    #[test]
    fn rejects_text_prop_on_text_component() {
        let error = parse_page(
            r#"page blogsPage
  Text text:"Blogs""#,
        )
        .expect_err("error");

        assert!(error.to_string().contains("unknown prop `text`"));
    }

    #[test]
    fn rejects_unknown_and_incompatible_signal_paths() {
        let missing = parse_page(
            r#"page blogsPage
  signal blog value:{ title:"" visible:false }
  Box
    Input bind:blog.missing"#,
        )
        .expect_err("missing field");
        assert!(
            missing
                .to_string()
                .contains("unknown signal path `blog.missing`")
        );

        let incompatible = parse_page(
            r#"page blogsPage
  signal alert value:{ message:"" visible:false }
  Alert type:"info" message:alert.visible visible:alert.message"#,
        )
        .expect_err("incompatible field");
        assert!(
            incompatible
                .to_string()
                .contains("invalid signal path `alert.visible` in `message`: expected string")
        );
    }

    #[test]
    fn rejects_i18n_with_reactive_text_child() {
        let error = parse_page(
            r#"page profilePage
  signal profile value:{ title:"" }
  Text i18n:"profile.title"
    profile.title"#,
        )
        .expect_err("reactive fallback");

        assert!(
            error
                .to_string()
                .contains("`i18n` requires a static fallback text child")
        );
    }

    #[test]
    fn parses_show_visibility_conditions() {
        let tree = parse_page(
            r#"page readyPage
  signal isReady value:false
  signal rows value:[{ id:"1" ready:true }]
  Box show:{ xs:false md:true }
    Text show:isReady
      Ready
    each row in rows key:row.id
      Text show:row.ready
        Row"#,
        )
        .expect("tree");

        let ViewNode::Scope {
            signals, children, ..
        } = tree
        else {
            panic!("scope");
        };
        assert_eq!(signals[0].name, "isReady");

        let ViewNode::Box {
            props,
            children: box_children,
        } = &children[0]
        else {
            panic!("box");
        };
        match props.element.show.as_ref().expect("box show") {
            VisibilityCondition::Static(value) => {
                assert_eq!(value.entries.len(), 2);
                assert_eq!(value.entries[0].breakpoint, Breakpoint::Xs);
                assert!(!value.entries[0].value);
                assert_eq!(value.entries[1].breakpoint, Breakpoint::Md);
                assert!(value.entries[1].value);
            }
            VisibilityCondition::Signal(_) => panic!("static show"),
        }

        let ViewNode::Text { props, .. } = &box_children[0] else {
            panic!("text");
        };
        assert_eq!(
            props.style.element.show,
            Some(VisibilityCondition::Signal("isReady".to_string()))
        );

        let ViewNode::Each {
            children: row_children,
            ..
        } = &box_children[1]
        else {
            panic!("each");
        };
        let ViewNode::Text { props, .. } = &row_children[0] else {
            panic!("row text");
        };
        assert_eq!(
            props.style.element.show,
            Some(VisibilityCondition::Signal("row.ready".to_string()))
        );
    }

    #[test]
    fn parses_box_and_card_animation_props() {
        let tree = parse_page(
            r#"page motionPage
  Box animation:"fadeIn"
    Card animation:"slideUp"
      Text
        Motion"#,
        )
        .expect("tree");

        let ViewNode::Box { props, children } = tree else {
            panic!("box");
        };
        assert_eq!(props.animation, Some(ViewAnimation::FadeIn));

        let ViewNode::Card { props, .. } = &children[0] else {
            panic!("card");
        };
        assert_eq!(props.style.animation, Some(ViewAnimation::SlideUp));
    }

    #[test]
    fn parses_section_background_props() {
        let tree = parse_page(
            r#"page landingPage
  Section id:"hero" background:{ xs:"soft" md:"aurora" } color:"onBackground" p:8
    Text
      Hero"#,
        )
        .expect("tree");

        let ViewNode::Section { props, children } = tree else {
            panic!("section");
        };
        assert_eq!(props.element.id.as_deref(), Some("hero"));
        assert_eq!(
            props.background.expect("background").entries[1].value,
            SectionBackground::Aurora
        );
        assert!(props.text.is_some());
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn rejects_invalid_show_visibility_conditions() {
        let non_bool = parse_page(
            r#"page readyPage
  signal profile value:{ name:"" }
  Text show:profile.name
    Ready"#,
        )
        .expect_err("non bool");
        assert!(
            non_bool
                .to_string()
                .contains("invalid signal path `profile.name` in `show`: expected bool")
        );

        let responsive_string = parse_page(
            r#"page readyPage
  Text show:{ xs:"false" }
    Ready"#,
        )
        .expect_err("responsive string");
        assert!(
            responsive_string
                .to_string()
                .contains("invalid value for prop `show`: expected boolean")
        );
    }

    #[test]
    fn rejects_each_over_non_array_signal() {
        let error = parse_page(
            r#"page blogsPage
  signal blog value:{ id:"" title:"" }
  each item in blog key:item.id
    Text
      item.title"#,
        )
        .expect_err("collection type");

        assert!(error.to_string().contains("must be an array"));
    }

    #[test]
    fn validates_typed_signals_and_empty_typed_collections() {
        parse_page(
            r#"type BlogForm
  id?:string
  title:string
  content:string

type BlogItem
  id:string
  title:string

page blogsPage
  signal blog type:BlogForm value:{ id:null title:"" content:"" }
  signal blogs type:BlogItem[] value:[]
  Box
    Input bind:blog.title
    each item in blogs key:item.id
      Text
        item.title"#,
        )
        .expect("typed page");

        let error = parse_page(
            r#"type BlogItem
  id:string
  title:string

page blogsPage
  signal blogs type:BlogItem[] value:[]
  Box
    each item in blogs key:item.id
      Text
        item.missing"#,
        )
        .expect_err("missing typed field");

        assert!(
            error
                .to_string()
                .contains("unknown signal path `item.missing`")
        );
    }

    #[test]
    fn rejects_result_block_with_inline_result_props() {
        let error = parse_page(
            r#"page blogsPage
  signal blogs value:[]
  signal alert value:{ type:"info" message:"" visible:false }
  action create
    request POST route:"/api/blogs" update:blogs successAlert:alert
      onSuccess alert:"Blog creado"
  Box
    Text
      Blogs"#,
        )
        .expect_err("error");

        assert!(
            error
                .to_string()
                .contains("cannot be combined with inline success props")
        );
    }

    #[test]
    fn parses_svg_component_with_path_children() {
        let tree = parse_page(
            r#"page iconPage
  Svg viewBox:"0 0 24 24" color:"tertiary" w:8 h:8
    Path d:"M0 0h24v24H0z" fill:"none"
    Path fill:"currentColor" d:"M22 12c0-5.523-4.477-10-10-10S2 6.477 2 12s4.477 10 10 10s10-4.477 10-10"
    Path fill:"tertiary" d:"M1 1h2v2H1z""#,
        )
        .expect("tree");

        let ViewNode::Svg { props, paths } = tree else {
            panic!("svg");
        };
        assert_eq!(props.view_box.as_str(), "0 0 24 24");
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].fill, SvgPathFill::None);
        assert_eq!(paths[1].fill, SvgPathFill::CurrentColor);
        assert_eq!(paths[2].fill, SvgPathFill::Color(ColorToken::Tertiary));
    }

    #[test]
    fn parses_video_component_with_hls_source() {
        let tree = parse_page(
            r#"page videoPage
  Video src:"https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8" poster:"/images/video.jpg" autoplay:true aspect:"vertical" variant:"soft" scheme:"tertiary""#,
        )
        .expect("tree");

        let ViewNode::Video { props } = tree else {
            panic!("video");
        };
        assert_eq!(
            props.src,
            "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8"
        );
        assert_eq!(props.poster.as_deref(), Some("/images/video.jpg"));
        assert!(props.autoplay);
        assert_eq!(props.aspect, VideoAspect::Vertical);
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.style.color, Some(ColorFamily::Tertiary));
    }

    #[test]
    fn parses_candlestick_component_with_typed_data_and_stream() {
        let tree = parse_page(
            r#"type Candle
  time:string
  open:number
  high:number
  low:number
  close:number

page marketPage
  signal candles type:Candle[] value:[{ time:"2026-06-01T09:30:00Z" open:102 high:108 low:99 close:106 }]
  Candlestick data:candles stream:"/api/market/candles" variant:"soft" scheme:"surface" upColor:"success" downColor:"danger" emptyLabel:"Waiting" maxPoints:120"#,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Candlestick { props } = &children[0] else {
            panic!("candlestick");
        };
        assert_eq!(props.data, "candles");
        assert_eq!(props.stream.as_deref(), Some("/api/market/candles"));
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.style.color, Some(ColorFamily::Surface));
        assert_eq!(props.up_color, ColorToken::Success);
        assert_eq!(props.down_color, ColorToken::Danger);
        assert_eq!(props.empty_label, "Waiting");
        assert_eq!(props.max_points, 120);
    }

    #[test]
    fn rejects_invalid_candlestick_usage() {
        let missing = parse_page(
            r#"page marketPage
  Candlestick"#,
        )
        .expect_err("missing");
        assert!(
            missing
                .to_string()
                .contains("invalid value for prop `data`: expected signal array path")
        );

        let wrong_type = parse_page(
            r#"page marketPage
  signal candles value:{ time:"" }
  Candlestick data:candles"#,
        )
        .expect_err("wrong type");
        assert!(
            wrong_type
                .to_string()
                .contains("signal `candles` in `data` must be an array")
        );

        let missing_field = parse_page(
            r#"type Candle
  time:string
  open:number
  high:number
  low:number

page marketPage
  signal candles type:Candle[] value:[]
  Candlestick data:candles"#,
        )
        .expect_err("missing field");
        assert!(
            missing_field
                .to_string()
                .contains("Candlestick data item must include `close`")
        );

        let invalid_candle = parse_page(
            r#"page marketPage
  signal candles value:[{ time:"1" open:10 high:9 low:8 close:10 }]
  Candlestick data:candles"#,
        )
        .expect_err("invalid candle");
        assert!(
            invalid_candle
                .to_string()
                .contains("Candlestick data item violates OHLC bounds")
        );

        let stream = parse_page(
            r#"page marketPage
  signal candles value:[]
  Candlestick data:candles stream:"http://example.com/events""#,
        )
        .expect_err("stream");
        assert!(
            stream
                .to_string()
                .contains("invalid value for prop `stream`: expected absolute path or https URL")
        );

        let child = parse_page(
            r#"page marketPage
  signal candles value:[]
  Candlestick data:candles
    Text
      Invalid"#,
        )
        .expect_err("child");
        assert!(
            child
                .to_string()
                .contains("children are not valid for this component")
        );
    }

    #[test]
    fn parses_chart_components_with_typed_signals() {
        let tree = parse_page(
            r#"type ChartPoint
  x:number
  y:number

type ChartSlice
  label:string
  value:number

page chartPage
  signal points type:ChartPoint[] value:[{ x:1 y:12 }, { x:2 y:18 }]
  signal slices type:ChartSlice[] value:[{ label:"Docs" value:40 }, { label:"CLI" value:60 }]
  Box
    LineChart data:points curve:"smooth" palette:"ocean" size:"lg" showGradientFill:true
    AreaChart data:points legendPosition:"bottom" fillOpacity:0.42 showPoints:true
    BarChart data:slices grouped:true showValues:true
    ArcChart data:slices legendPosition:"right" thickness:18 showInlineLabels:true
    PieChart data:slices donut:true donutWidth:72"#,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Box {
            children: chart_children,
            ..
        } = &children[0]
        else {
            panic!("box");
        };
        let ViewNode::LineChart { props } = &chart_children[0] else {
            panic!("line chart");
        };
        assert_eq!(props.common.data.as_deref(), Some("points"));
        assert_eq!(props.common.palette, ChartPalette::Ocean);
        assert_eq!(props.common.size, ChartSize::Lg);
        assert_eq!(props.curve, ChartCurve::Smooth);
        assert!(props.show_gradient_fill);

        let ViewNode::AreaChart { props } = &chart_children[1] else {
            panic!("area chart");
        };
        assert_eq!(props.common.legend_position, ChartLegendPosition::Bottom);
        assert_eq!(props.fill_opacity, 42);
        assert!(props.show_points);

        let ViewNode::BarChart { props } = &chart_children[2] else {
            panic!("bar chart");
        };
        assert!(props.grouped);
        assert!(props.show_values);

        let ViewNode::ArcChart { props } = &chart_children[3] else {
            panic!("arc chart");
        };
        assert_eq!(props.common.legend_position, ChartLegendPosition::Right);
        assert_eq!(props.thickness, 18);
        assert!(props.show_inline_labels);

        let ViewNode::PieChart { props } = &chart_children[4] else {
            panic!("pie chart");
        };
        assert!(props.donut);
        assert_eq!(props.donut_width, 72);
    }

    #[test]
    fn rejects_invalid_chart_data_shape() {
        let error = parse_page(
            r#"page chartPage
  signal points value:[{ x:1 }]
  LineChart data:points"#,
        )
        .expect_err("line chart data");
        assert!(
            error
                .to_string()
                .contains("LineChart data item must include `y`")
        );
    }

    #[test]
    fn parses_table_component_with_typed_data_and_columns() {
        let tree = parse_page(
            r#"type UserRow
  name:string
  status:string

page usersPage
  signal users type:UserRow[] value:[{ name:"Ana" status:"active" }]
  Table data:users variant:"soft" scheme:"surface" size:"lg" striped:true bordered:true dividers:true emptyTitle:"No users" emptyDescription:"Invite users"
    column field:"name" label:"Name"
    column field:"status" label:"Status" align:"end" width:"8rem""#,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Table { props } = &children[0] else {
            panic!("table");
        };
        assert_eq!(props.data, "users");
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.style.color, Some(ColorFamily::Surface));
        assert_eq!(props.size, TableSize::Lg);
        assert!(props.striped);
        assert!(props.bordered);
        assert!(props.dividers);
        assert_eq!(props.empty_title, "No users");
        assert_eq!(props.empty_description, "Invite users");
        assert_eq!(props.columns.len(), 2);
        assert_eq!(props.columns[1].field, "status");
        assert_eq!(props.columns[1].align, TableColumnAlign::End);
        assert_eq!(props.columns[1].width.as_deref(), Some("8rem"));
    }

    #[test]
    fn rejects_invalid_table_usage() {
        let wrong_type = parse_page(
            r#"page usersPage
  signal users value:{ name:"Ana" }
  Table data:users
    column field:"name" label:"Name""#,
        )
        .expect_err("wrong type");
        assert!(
            wrong_type
                .to_string()
                .contains("signal `users` in `data` must be an array")
        );

        let color_prop = parse_page(
            r#"page usersPage
  signal users value:[{ name:"Ana" }]
  Table data:users color:"primary"
    column field:"name" label:"Name""#,
        )
        .expect_err("color prop");
        assert!(
            color_prop
                .to_string()
                .contains("unknown prop `color` on `Table`; use `scheme` for visual family")
        );

        let missing_field = parse_page(
            r#"type UserRow
  name:string

page usersPage
  signal users type:UserRow[] value:[]
  Table data:users
    column field:"status" label:"Status""#,
        )
        .expect_err("missing field");
        assert!(
            missing_field
                .to_string()
                .contains("unknown Table column field `status`")
        );
    }

    #[test]
    fn parses_divider_component_with_orientation_and_scheme() {
        let tree = parse_page(
            r#"page dividerPage
  Divider orientation:"vertical" scheme:"primary" h:24"#,
        )
        .expect("tree");

        let ViewNode::Divider { props } = tree else {
            panic!("divider");
        };
        assert_eq!(props.orientation, DividerOrientation::Vertical);
        assert_eq!(props.color, ColorFamily::Primary);
        assert!(props.style.sizing.h.is_some());
    }

    #[test]
    fn rejects_invalid_divider_usage() {
        let orientation = parse_page(
            r#"page dividerPage
  Divider orientation:"diagonal""#,
        )
        .expect_err("orientation");
        assert!(
            orientation
                .to_string()
                .contains("expected horizontal or vertical")
        );

        let child = parse_page(
            r#"page dividerPage
  Divider
    Text
      Invalid"#,
        )
        .expect_err("child");
        assert!(
            child
                .to_string()
                .contains("children are not valid for this component")
        );
    }

    #[test]
    fn rejects_invalid_video_usage() {
        let missing = parse_page(
            r#"page videoPage
  Video"#,
        )
        .expect_err("missing");
        assert!(missing.to_string().contains("invalid value for prop `src`"));

        let http = parse_page(
            r#"page videoPage
  Video src:"http://example.com/video.mp4""#,
        )
        .expect_err("http");
        assert!(http.to_string().contains("expected https URL"));

        let aspect = parse_page(
            r#"page videoPage
  Video src:"https://example.com/video.mp4" aspect:"wide""#,
        )
        .expect_err("aspect");
        assert!(
            aspect
                .to_string()
                .contains("expected horizontal, vertical or square")
        );

        let child = parse_page(
            r#"page videoPage
  Video src:"https://example.com/video.mp4"
    Text
      Invalid"#,
        )
        .expect_err("child");
        assert!(
            child
                .to_string()
                .contains("children are not valid for this component")
        );
    }

    #[test]
    fn parses_layout_bar_regions() {
        let tree = parse_page(
            r#"page barsPage
  AppBar variant:"soft" scheme:"surface" bordered:true blurred:true boxed:true floating:true
    start
      Text
        Menu
    center
      Text
        Brand
    end
      Button href:"/"
        Home"#,
        )
        .expect("tree");

        let ViewNode::AppBar {
            props,
            start,
            center,
            end,
        } = tree
        else {
            panic!("appbar");
        };

        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.style.color, Some(ColorFamily::Surface));
        assert!(props.bordered);
        assert!(props.blurred);
        assert!(props.boxed);
        assert!(props.floating);
        assert_eq!(start.len(), 1);
        assert_eq!(center.len(), 1);
        assert_eq!(end.len(), 1);
    }

    #[test]
    fn rejects_invalid_layout_bar_regions() {
        let duplicate = parse_page(
            r#"page barsPage
  AppBar
    start
      Text
        Menu
    start
      Text
        Brand"#,
        )
        .expect_err("duplicate");
        assert!(
            duplicate
                .to_string()
                .contains("duplicate `start` region in AppBar")
        );

        let direct_child = parse_page(
            r#"page barsPage
  AppBar
    Text
      Brand"#,
        )
        .expect_err("direct child");
        assert!(
            direct_child
                .to_string()
                .contains("AppBar only accepts start, center or end regions")
        );
    }

    #[test]
    fn parses_side_nav_entries_submenus_and_svg_icons() {
        let tree = parse_page(
            r#"page navPage
  SideNav variant:"soft" scheme:"surface" size:"lg" wide:true
    header label:"Workspace" description:"Account navigation"
    item label:"Home" href:"/"
      icon
        Svg viewBox:"0 0 24 24" w:5 h:5
          Path d:"M3 11l9-8 9 8v10H3z" fill:"currentColor"
    divider
    submenu label:"Content" status:"2" open:true
      item label:"Blogs" href:"/blogs" status:"12""#,
        )
        .expect("tree");

        let ViewNode::SideNav { props, items } = tree else {
            panic!("side nav");
        };
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.style.color, Some(ColorFamily::Surface));
        assert_eq!(props.size, dowe_components::SideNavSize::Lg);
        assert!(props.wide);
        assert!(matches!(
            &items[1],
            dowe_components::SideNavItem::Item(props) if props.icon.is_some()
        ));
        assert!(matches!(
            &items[3],
            dowe_components::SideNavItem::Submenu { open: true, items, .. } if items.len() == 1
        ));
    }

    #[test]
    fn parses_sidebar_as_side_navigation_surface() {
        let tree = parse_page(
            r#"page navPage
  Sidebar variant:"solid" scheme:"primary" size:"sm" wide:true
    header label:"Workspace"
    item label:"Home" href:"/"
    submenu label:"Content" open:true
      item label:"Blogs" href:"/blogs""#,
        )
        .expect("tree");

        let ViewNode::Sidebar { props, items } = tree else {
            panic!("sidebar");
        };
        assert_eq!(props.style.variant, Some(ComponentVariant::Solid));
        assert_eq!(props.style.color, Some(ColorFamily::Primary));
        assert_eq!(props.size, dowe_components::SideNavSize::Sm);
        assert!(props.wide);
        assert!(matches!(
            &items[2],
            dowe_components::SideNavItem::Submenu { open: true, items, .. } if items.len() == 1
        ));
    }

    #[test]
    fn parses_nav_menu_items_submenus_and_megamenu_content() {
        let tree = parse_page(
            r##"page navMenuPage
  NavMenu variant:"ghost" scheme:"muted" size:"lg"
    item label:"Home" href:"/"
    submenu label:"Docs"
      item label:"Guide" href:"/docs"
      item label:"Install" href:"#install"
    megamenu label:"Resources"
      content
        Card variant:"soft" scheme:"surface"
          Text
            Resource hub"##,
        )
        .expect("tree");

        let ViewNode::NavMenu { props, items } = tree else {
            panic!("nav menu");
        };
        assert_eq!(props.style.variant, Some(ComponentVariant::Ghost));
        assert_eq!(props.style.color, Some(ColorFamily::Muted));
        assert_eq!(props.size, dowe_components::SideNavSize::Lg);
        assert!(matches!(
            &items[1],
            dowe_components::NavMenuItem::Submenu { items, .. } if items.len() == 2
        ));
        assert!(matches!(
            &items[2],
            dowe_components::NavMenuItem::Megamenu { content, .. } if content.len() == 1
        ));
    }

    #[test]
    fn parses_scaffold_regions_with_required_main() {
        let tree = parse_page(
            r#"page shellPage
  Scaffold boxed:true
    appBar
      AppBar
        center
          Text
            Top
    start
      Sidebar
        item label:"Home" href:"/"
    main
      Text
        Main
    bottomBar
      BottomBar
        center
          Text
            Bottom"#,
        )
        .expect("tree");

        let ViewNode::Scaffold {
            props,
            app_bar,
            start,
            main,
            bottom_bar,
            ..
        } = tree
        else {
            panic!("scaffold");
        };
        assert!(props.boxed);
        assert_eq!(app_bar.len(), 1);
        assert_eq!(start.len(), 1);
        assert_eq!(main.len(), 1);
        assert_eq!(bottom_bar.len(), 1);

        let missing_main = parse_page(
            r#"page shellPage
  Scaffold
    start
      Text
        Side"#,
        )
        .expect_err("missing main");
        assert!(
            missing_main
                .to_string()
                .contains("Scaffold requires a main region with content")
        );
    }

    #[test]
    fn rejects_color_prop_for_nav_menu_and_sidebar() {
        let nav_menu = parse_page(
            r#"page navMenuPage
  NavMenu color:"primary"
    item label:"Home" href:"/""#,
        )
        .expect_err("nav menu color");
        assert!(
            nav_menu
                .to_string()
                .contains("unknown prop `color` on `NavMenu`; use `scheme`")
        );

        let sidebar = parse_page(
            r#"page navPage
  Sidebar color:"primary"
    item label:"Home" href:"/""#,
        )
        .expect_err("sidebar color");
        assert!(
            sidebar
                .to_string()
                .contains("unknown prop `color` on `Sidebar`; use `scheme`")
        );
    }

    #[test]
    fn rejects_invalid_side_nav_structure() {
        let icon = parse_page(
            r#"page navPage
  SideNav
    item label:"Home"
      icon
        Text
          Home"#,
        )
        .expect_err("icon");
        assert!(
            icon.to_string()
                .contains("SideNav icon requires exactly one Svg child")
        );

        let navigation = parse_page(
            r#"page navPage
  SideNav
    item label:"Home" href:"/" onClick:open"#,
        )
        .expect_err("navigation");
        assert!(
            navigation
                .to_string()
                .contains("`href` and `onClick` cannot be used on the same SideNav entry")
        );
    }

    #[test]
    fn parses_tabs_entries_variants_and_panel_children() {
        let tree = parse_page(
            r#"page tabsPage
  Tabs variant:"line" scheme:"primary" position:"start"
    tab id:"overview" label:"Overview"
      Text
        Overview content
    tab id:"details" label:"Details"
      Button
        Save"#,
        )
        .expect("tree");

        let ViewNode::Tabs { props, tabs } = tree else {
            panic!("tabs");
        };
        assert_eq!(props.variant, dowe_components::TabsVariant::Line);
        assert_eq!(props.color, ColorFamily::Primary);
        assert_eq!(props.position, dowe_components::TabsPosition::Start);
        assert_eq!(tabs.len(), 2);
        assert_eq!(tabs[0].id, "overview");
        assert_eq!(tabs[0].label, "Overview");
        assert_eq!(tabs[1].children.len(), 1);
    }

    #[test]
    fn rejects_invalid_tabs_structure() {
        let color = parse_page(
            r#"page tabsPage
  Tabs color:"primary"
    tab id:"overview" label:"Overview"
      Text
        Overview"#,
        )
        .expect_err("color");
        assert!(
            color
                .to_string()
                .contains("unknown prop `color` on `Tabs`; use `scheme` for visual family")
        );

        let duplicate = parse_page(
            r#"page tabsPage
  Tabs
    tab id:"overview" label:"Overview"
      Text
        Overview
    tab id:"overview" label:"Duplicate"
      Text
        Duplicate"#,
        )
        .expect_err("duplicate");
        assert!(
            duplicate
                .to_string()
                .contains("duplicate Tabs tab id `overview`")
        );

        let child = parse_page(
            r#"page tabsPage
  Tabs
    Text
      Overview"#,
        )
        .expect_err("child");
        assert!(child.to_string().contains("Tabs only accepts tab entries"));

        let outside = parse_page(
            r#"page tabsPage
  tab id:"overview" label:"Overview"
    Text
      Overview"#,
        )
        .expect_err("outside");
        assert!(
            outside
                .to_string()
                .contains("tab can only be used inside Tabs")
        );
    }

    #[test]
    fn parses_drawer_with_signal_open_and_responsive_show() {
        let tree = parse_page(
            r#"page navPage
  signal drawerOpen value:false
  Drawer open:drawerOpen position:"end" variant:"soft" scheme:"surface" show:{ xs:true md:false } disableOverlayClose:true hideCloseButton:true
    Text
      Navigation"#,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Drawer { props, children } = &children[0] else {
            panic!("drawer");
        };
        assert_eq!(props.open, "drawerOpen");
        assert_eq!(props.position, dowe_components::DrawerPosition::End);
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.style.color, Some(ColorFamily::Surface));
        assert!(props.disable_overlay_close);
        assert!(props.hide_close_button);
        assert!(props.style.element.show.is_some());
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn parses_display_and_overlay_view_components() {
        let tree = parse_page(
            r#"page overlayPage
  signal modalOpen value:false
  signal toast value:{ type:"success" title:"Saved" message:"Profile updated" visible:true }
  action close
    reset modalOpen
  Box
    Avatar name:"Ada" alt:"Ada Lovelace" scheme:"success" variant:"soft" size:"lg" status:"online" bordered:true
    Badge text:"3" scheme:"danger" position:"bottom-right"
      Avatar name:"Ada" alt:"Ada"
    Chip variant:"outlined" scheme:"info" size:"sm" onClose:close
      Filter
    Skeleton variant:"rounded" animation:"pulse"
    Modal open:modalOpen scheme:"surface" hideCloseButton:true
      header
        Title
          Settings
      Text
        Body
      footer
        Button onClick:close
          Close
    AlertDialog open:modalOpen title:"Delete?" description:"Cannot undo." confirmText:"Delete" cancelText:"Cancel" onConfirm:close onCancel:close scheme:"danger"
    Tooltip label:"More actions" position:"end" scheme:"muted"
      Text
        Hover
    Toast source:toast position:"top-right" showIcon:true
    Dropdown scheme:"surface"
      trigger
        Button
          Menu
      item label:"Profile" onClick:close
      divider
      item label:"Docs" href:"/docs" description:"Open docs"
    Command open:modalOpen placeholder:"Search" shortcut:"p" scheme:"muted"
      item label:"Back" history:"back"
      group label:"Admin"
        item label:"Users" onClick:close"#,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Box {
            children: box_children,
            ..
        } = &children[0]
        else {
            panic!("box");
        };
        assert_eq!(box_children.len(), 10);

        let ViewNode::Avatar { props, .. } = &box_children[0] else {
            panic!("avatar");
        };
        assert_eq!(props.style.color, Some(ColorFamily::Success));
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.size, ButtonSize::Lg);
        assert_eq!(props.status, Some(AvatarStatus::Online));
        assert!(props.bordered);

        let ViewNode::Badge {
            props,
            children: badge_children,
        } = &box_children[1]
        else {
            panic!("badge");
        };
        assert_eq!(props.position, OverlayCornerPosition::BottomRight);
        assert_eq!(badge_children.len(), 1);

        let ViewNode::Chip { props, value, .. } = &box_children[2] else {
            panic!("chip");
        };
        assert_eq!(value, "Filter");
        assert_eq!(props.on_close.as_deref(), Some("close"));

        let ViewNode::Skeleton { props } = &box_children[3] else {
            panic!("skeleton");
        };
        assert_eq!(props.variant, SkeletonVariant::Rounded);
        assert_eq!(props.animation, SkeletonAnimation::Pulse);

        let ViewNode::Modal {
            props,
            header,
            body,
            footer,
        } = &box_children[4]
        else {
            panic!("modal");
        };
        assert_eq!(props.open, "modalOpen");
        assert!(props.hide_close_button);
        assert_eq!(header.len(), 1);
        assert_eq!(body.len(), 1);
        assert_eq!(footer.len(), 1);

        let ViewNode::AlertDialog { props } = &box_children[5] else {
            panic!("dialog");
        };
        assert_eq!(props.on_confirm.as_deref(), Some("close"));
        assert_eq!(props.on_cancel.as_deref(), Some("close"));

        let ViewNode::Tooltip { props, children } = &box_children[6] else {
            panic!("tooltip");
        };
        assert_eq!(props.position, OverlayPosition::End);
        assert_eq!(children.len(), 1);

        let ViewNode::Toast { props } = &box_children[7] else {
            panic!("toast");
        };
        assert_eq!(props.source.as_deref(), Some("toast"));
        assert_eq!(props.kind, ToastKind::Info);
        assert_eq!(props.position, OverlayCornerPosition::TopRight);
        assert!(props.show_icon);

        let ViewNode::Dropdown { entries, .. } = &box_children[8] else {
            panic!("dropdown");
        };
        assert!(matches!(entries[0], OverlayEntry::Item(_)));
        assert!(matches!(entries[1], OverlayEntry::Divider));

        let ViewNode::Command { props, entries } = &box_children[9] else {
            panic!("command");
        };
        assert_eq!(props.open.as_deref(), Some("modalOpen"));
        assert_eq!(props.shortcut, "p");
        assert!(matches!(
            &entries[0],
            CommandEntry::Item(props)
                if matches!(props.navigation, Some(NavigationAction::Back))
        ));
        assert!(matches!(entries[1], CommandEntry::Group { .. }));
    }

    #[test]
    fn parses_display_chat_and_motion_components() {
        let tree = parse_page(
            r#"type Person
  src:string
  name:string
  alt:string

type ChatMessage
  id:string
  role:string
  userId:string
  text:string
  status:string

page displayPage
  signal people type:Person[] value:[{ src:"/ada.png" name:"Ada" alt:"Ada Lovelace" }]
  signal messages type:ChatMessage[] value:[{ id:"1" role:"assistant" userId:"assistant" text:"Hello" status:"sent" }]
  signal loading value:false
  action sendMessage
    reset loading
  Box
    AvatarGroup items:people size:"sm" max:3 autoFit:true inline:false bordered:true scheme:"primary" variant:"soft"
      item src:"/ada.png" name:"Ada" alt:"Ada Lovelace" onClick:sendMessage
      item name:"Grace" alt:"Grace Hopper" href:"/docs"
    ChatBox messages:messages mode:"conversation" currentUserId:"ada" userName:"Ada" userAvatar:"/ada.png" userStatus:"online" assistantName:"Dowe" assistantAvatar:"/dowe.png" showHeader:true placeholder:"Ask Dowe" showAttachments:true showVoiceNote:true showCamera:true loading:loading sending:loading streaming:loading hasMore:loading onSend:sendMessage onLoadMore:sendMessage onStop:sendMessage onVoiceNote:sendMessage onFileAttach:sendMessage onCameraCapture:sendMessage
    Empty type:"result" title:"Nothing found" description:"Try again" actionLabel:"Retry" onClick:sendMessage scheme:"info" variant:"soft"
    Marquee speed:"fast" pauseOnHover:true reverse:true orientation:"horizontal" fade:true fadeColor:"background" gap:4
      Text
        Moving
    TypeWriter typeSpeed:10 deleteSpeed:5 afterTyped:20 afterDeleted:10 repeat:false
      item text:"Hello"
      item text:"World""#,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Box {
            children: box_children,
            ..
        } = &children[0]
        else {
            panic!("box");
        };
        assert_eq!(box_children.len(), 5);

        let ViewNode::AvatarGroup { props, items } = &box_children[0] else {
            panic!("avatar group");
        };
        assert_eq!(props.items.as_deref(), Some("people"));
        assert_eq!(props.size, ButtonSize::Sm);
        assert_eq!(props.max, Some(3));
        assert!(props.auto_fit);
        assert!(!props.inline);
        assert!(props.bordered);
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].on_click.as_deref(), Some("sendMessage"));
        assert!(items[1].navigation.is_some());

        let ViewNode::ChatBox { props } = &box_children[1] else {
            panic!("chat box");
        };
        assert_eq!(props.messages, "messages");
        assert_eq!(props.mode, ChatBoxMode::Conversation);
        assert_eq!(props.current_user_id, "ada");
        assert_eq!(props.loading.as_deref(), Some("loading"));
        assert_eq!(props.on_send.as_deref(), Some("sendMessage"));
        assert!(props.show_attachments);
        assert!(props.show_voice_note);
        assert!(props.show_camera);

        let ViewNode::Empty { props } = &box_children[2] else {
            panic!("empty");
        };
        assert_eq!(props.kind, EmptyKind::Result);
        assert_eq!(props.title.as_deref(), Some("Nothing found"));
        assert_eq!(props.action_label, "Retry");

        let ViewNode::Marquee { props, children } = &box_children[3] else {
            panic!("marquee");
        };
        assert_eq!(props.speed, MarqueeSpeed::Fast);
        assert_eq!(props.orientation, MarqueeOrientation::Horizontal);
        assert!(props.pause_on_hover);
        assert!(props.reverse);
        assert!(props.fade);
        assert_eq!(props.fade_color, ColorToken::Background);
        assert_eq!(children.len(), 1);

        let ViewNode::TypeWriter { props, items } = &box_children[4] else {
            panic!("type writer");
        };
        assert_eq!(props.type_speed, 10);
        assert_eq!(props.delete_speed, 5);
        assert!(!props.repeat);
        assert_eq!(items.len(), 2);
        assert_eq!(items[1].text, "World");
    }

    #[test]
    fn parses_rich_control_map_components() {
        let tree = parse_page(
            r#"page componentsPage
  signal mode value:"map"
  action choose
    reset mode
  action done
    reset mode
  Box
    RichText size:"lg" weight:"bold"
      mark text:"Launch" style:"grad" scheme:"primary"
      mark text:"ready" style:"pill" scheme:"success"
    Record name:"voice" maxDuration:90 onStart:choose onConfirm:done variant:"soft" scheme:"primary"
    ToggleGroup value:mode selected:"map" size:"sm" wide:true ariaLabel:"Display mode" onChange:choose variant:"soft" scheme:"secondary"
      item id:"list" label:"List" icon:"search"
      item id:"map" label:"Map" icon:"settings"
    Collapsible label:"Details" defaultOpen:true scheme:"surface"
      Text
        Body
    Countdown target:"2030-01-01T00:00:00Z" size:"xl" showDays:true showHours:true showMinutes:true showSeconds:false onComplete:done scheme:"primary" variant:"outlined"
    Map centerLat:4.7109 centerLng:-74.0721 zoom:12 height:"360px" width:"100%" showScale:true showLocationControl:true routeStartLat:4.7109 routeStartLng:-74.0721 routeEndLat:4.65 routeEndLng:-74.09 onRoute:done scheme:"primary" variant:"soft"
      marker id:"office" lat:4.7109 lng:-74.0721 label:"Office" popup:"Main" icon:"start" onClick:choose
      waypoint lat:4.68 lng:-74.08"#,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Box {
            children: box_children,
            ..
        } = &children[0]
        else {
            panic!("box");
        };
        assert_eq!(box_children.len(), 6);

        let ViewNode::RichText { props, marks } = &box_children[0] else {
            panic!("rich text");
        };
        assert_eq!(
            props
                .size
                .as_ref()
                .map(|value| value.entries[0].value.as_str()),
            Some("lg")
        );
        assert_eq!(marks.len(), 2);
        assert_eq!(marks[0].style, RichTextMarkStyle::Grad);
        assert_eq!(marks[1].color, ColorFamily::Success);

        let ViewNode::Record { props } = &box_children[1] else {
            panic!("record");
        };
        assert_eq!(props.name, "voice");
        assert_eq!(props.max_duration, Some(90));
        assert_eq!(props.on_start.as_deref(), Some("choose"));
        assert_eq!(props.on_confirm.as_deref(), Some("done"));
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));

        let ViewNode::ToggleGroup { props, items } = &box_children[2] else {
            panic!("toggle group");
        };
        assert_eq!(props.value.as_deref(), Some("mode"));
        assert_eq!(props.selected, "map");
        assert_eq!(props.size, ButtonSize::Sm);
        assert!(props.wide);
        assert_eq!(props.on_change.as_deref(), Some("choose"));
        assert_eq!(items.len(), 2);
        assert_eq!(items[1].icon, Some(ViewIcon::Settings));

        let ViewNode::Collapsible { props, children } = &box_children[3] else {
            panic!("collapsible");
        };
        assert_eq!(props.label, "Details");
        assert!(props.default_open);
        assert_eq!(props.style.color, Some(ColorFamily::Surface));
        assert_eq!(children.len(), 1);

        let ViewNode::Countdown { props } = &box_children[4] else {
            panic!("countdown");
        };
        assert_eq!(props.target, "2030-01-01T00:00:00Z");
        assert_eq!(props.size, CountdownSize::Xl);
        assert!(!props.show_seconds);
        assert_eq!(props.on_complete.as_deref(), Some("done"));

        let ViewNode::Map {
            props,
            markers,
            waypoints,
        } = &box_children[5]
        else {
            panic!("map");
        };
        assert_eq!(props.center_lat, "4.7109");
        assert_eq!(props.center_lng, "-74.0721");
        assert_eq!(props.zoom, 12);
        assert!(props.show_scale);
        assert!(props.show_location_control);
        assert_eq!(props.on_route.as_deref(), Some("done"));
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].lng, "-74.0721");
        assert_eq!(markers[0].icon, MapMarkerIcon::Start);
        assert_eq!(markers[0].on_click.as_deref(), Some("choose"));
        assert_eq!(waypoints.len(), 1);
        assert_eq!(waypoints[0].lng, "-74.08");
    }

    #[test]
    fn parses_media_display_and_form_components() {
        let tree = parse_page(
            r##"page componentsPage
  signal accepted value:false
  signal themeColor value:"#3366ff"
  signal shipDate value:"2026-06-05"
  signal startDate value:"2026-06-01"
  signal endDate value:"2026-06-08"
  signal choice value:"basic"
  Box
    Audio src:"https://cdn.pixabay.com/audio/2022/04/25/audio_5d61b5204f.mp3" subtitle:"Preview" avatarSrc:"https://example.com/avatar.png" scheme:"primary" variant:"soft"
    Image src:"https://example.com/photo.jpg" alt:"Photo" aspect:"square" objectFit:"contain" loading:"eager" hideControls:true scheme:"secondary"
    Accordion multiple:true variant:"outlined" scheme:"surface"
      item id:"intro" label:"Intro" defaultOpen:true
        Text
          Intro body
      item id:"details" label:"Details" disabled:true
        Text
          Details body
    Carousel title:"Samples" autoplay:true autoplayInterval:4000 showCounter:true orientation:"horizontal" size:"sm" indicatorType:"dot" slidesPerView:2 gap:12 scheme:"info"
      slide id:"one"
        Text
          First
      slide id:"two"
        Text
          Second
    Checkbox bind:accepted checked:true label:"I accept" name:"accepted" scheme:"success"
    Color bind:themeColor value:"#3366ff" label:"Theme" showHex:true showRgb:true showCmyk:true showOklch:true scheme:"primary"
    Date bind:shipDate value:"2026-06-05" label:"Ship date" min:"2026-01-01" max:"2026-12-31" scheme:"warning"
    DateRange start:startDate end:endDate startValue:"2026-06-01" endValue:"2026-06-08" label:"Range" scheme:"danger"
    RadioGroup bind:choice label:"Plan" size:"lg" info:"Choose one" scheme:"muted"
      item value:"basic" label:"Basic"
      item value:"pro" label:"Pro" disabled:true
    Toggle bind:accepted checked:true label:"Enabled" labelLeft:"Off" labelRight:"On" name:"enabled" scheme:"secondary""##,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Box {
            children: box_children,
            ..
        } = &children[0]
        else {
            panic!("box");
        };
        assert_eq!(box_children.len(), 10);

        let ViewNode::Audio { props } = &box_children[0] else {
            panic!("audio");
        };
        assert_eq!(props.subtitle.as_deref(), Some("Preview"));
        assert_eq!(props.style.color, Some(ColorFamily::Primary));

        let ViewNode::Image { props } = &box_children[1] else {
            panic!("image");
        };
        assert_eq!(props.aspect, ImageAspect::Square);
        assert_eq!(props.object_fit, ImageObjectFit::Contain);
        assert_eq!(props.loading, ImageLoading::Eager);
        assert!(props.hide_controls);

        let ViewNode::Accordion { props, items } = &box_children[2] else {
            panic!("accordion");
        };
        assert!(props.multiple);
        assert_eq!(props.style.variant, Some(ComponentVariant::Outlined));
        assert_eq!(props.style.color, Some(ColorFamily::Surface));
        assert_eq!(items.len(), 2);
        assert!(items[0].default_open);
        assert!(items[1].disabled);

        let ViewNode::Carousel { props, slides } = &box_children[3] else {
            panic!("carousel");
        };
        assert!(props.autoplay);
        assert_eq!(props.autoplay_interval, 4000);
        assert_eq!(props.orientation, CarouselOrientation::Horizontal);
        assert_eq!(props.indicator_type, CarouselIndicatorType::Dot);
        assert_eq!(props.slides_per_view, 2);
        assert_eq!(props.gap, 12);
        assert_eq!(slides.len(), 2);

        let ViewNode::Checkbox { props } = &box_children[4] else {
            panic!("checkbox");
        };
        assert_eq!(props.style.element.bind.as_deref(), Some("accepted"));
        assert_eq!(props.style.label.as_deref(), Some("I accept"));
        assert!(props.checked);

        let ViewNode::Color { props } = &box_children[5] else {
            panic!("color");
        };
        assert_eq!(props.style.element.bind.as_deref(), Some("themeColor"));
        assert!(props.show_hex && props.show_rgb && props.show_cmyk && props.show_oklch);

        let ViewNode::Date { props } = &box_children[6] else {
            panic!("date");
        };
        assert_eq!(props.style.element.bind.as_deref(), Some("shipDate"));
        assert_eq!(props.value.as_deref(), Some("2026-06-05"));
        assert_eq!(props.min.as_deref(), Some("2026-01-01"));

        let ViewNode::DateRange { props } = &box_children[7] else {
            panic!("date range");
        };
        assert_eq!(props.start.as_deref(), Some("startDate"));
        assert_eq!(props.end.as_deref(), Some("endDate"));
        assert_eq!(props.start_value.as_deref(), Some("2026-06-01"));

        let ViewNode::RadioGroup { props, options } = &box_children[8] else {
            panic!("radio group");
        };
        assert_eq!(props.style.element.bind.as_deref(), Some("choice"));
        assert_eq!(props.size, ButtonSize::Lg);
        assert_eq!(options.len(), 2);
        assert!(options[1].disabled);

        let ViewNode::Toggle { props } = &box_children[9] else {
            panic!("toggle");
        };
        assert_eq!(props.style.element.bind.as_deref(), Some("accepted"));
        assert_eq!(props.label_left.as_deref(), Some("Off"));
        assert_eq!(props.label_right.as_deref(), Some("On"));
        assert!(props.checked);
    }

    #[test]
    fn parses_theme_fab_slider_and_dropzone_components() {
        let tree = parse_page(
            r##"page controlsPage
  signal volume value:40
  action create
    reset volume
  Box
    ToggleTheme variant:"soft" scheme:"secondary" size:"sm" lightLabel:"Light mode" darkLabel:"Dark mode"
    Fab position:"top-left" fixed:false offsetX:6 offsetY:8 icon:"settings" label:"Actions" variant:"soft" scheme:"primary" size:"lg" onClick:create
      fabAction label:"Docs" icon:"link" href:"#top" navigate:"replace" scheme:"info"
      fabAction label:"Create" icon:"plus" onClick:create scheme:"success"
    Slider bind:volume min:0 max:100 step:5 label:"Volume" name:"volume" hideLabel:false scheme:"warning" size:"lg"
    Dropzone accept:"image/*" multiple:false maxSize:2048 name:"images" label:"Images" helpText:"PNG only" placeholder:"Drop images" disabled:false variant:"outlined" scheme:"surface" size:"sm""##,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Box {
            children: box_children,
            ..
        } = &children[0]
        else {
            panic!("box");
        };
        assert_eq!(box_children.len(), 4);

        let ViewNode::ToggleTheme { props } = &box_children[0] else {
            panic!("theme toggle");
        };
        assert_eq!(props.light_label, "Light mode");
        assert_eq!(props.dark_label, "Dark mode");
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.style.color, Some(ColorFamily::Secondary));

        let ViewNode::Fab { props, actions } = &box_children[1] else {
            panic!("fab");
        };
        assert_eq!(props.position, OverlayCornerPosition::TopLeft);
        assert!(!props.fixed);
        assert_eq!(props.icon, ViewIcon::Settings);
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.style.color, Some(ColorFamily::Primary));
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].label, "Docs");
        assert_eq!(actions[0].icon, ViewIcon::Link);
        assert_eq!(actions[0].color, ColorFamily::Info);
        assert!(matches!(
            actions[0].navigation,
            Some(NavigationAction::Section { .. })
        ));
        assert_eq!(actions[1].on_click.as_deref(), Some("create"));
        assert_eq!(actions[1].color, ColorFamily::Success);

        let ViewNode::Slider { props } = &box_children[2] else {
            panic!("slider");
        };
        assert_eq!(props.style.element.bind.as_deref(), Some("volume"));
        assert_eq!(props.value, "0");
        assert_eq!(props.min, "0");
        assert_eq!(props.max, "100");
        assert_eq!(props.step.as_deref(), Some("5"));
        assert_eq!(props.style.label.as_deref(), Some("Volume"));
        assert_eq!(props.style.color, Some(ColorFamily::Warning));
        assert_eq!(props.size, ButtonSize::Lg);

        let ViewNode::Dropzone { props } = &box_children[3] else {
            panic!("dropzone");
        };
        assert_eq!(props.accept.as_deref(), Some("image/*"));
        assert!(!props.multiple);
        assert_eq!(props.max_size, Some(2048));
        assert_eq!(props.name.as_deref(), Some("images"));
        assert_eq!(props.help_text.as_deref(), Some("PNG only"));
        assert_eq!(props.style.placeholder.as_deref(), Some("Drop images"));
        assert_eq!(props.style.variant, Some(ComponentVariant::Outlined));
        assert_eq!(props.style.color, Some(ColorFamily::Surface));
        assert_eq!(props.size, ButtonSize::Sm);
    }

    #[test]
    fn rejects_color_prop_for_new_view_components() {
        for source in [
            r#"page componentsPage
  Audio src:"https://example.com/audio.mp3" color:"primary""#,
            r#"page componentsPage
  Accordion color:"primary"
    item id:"one" label:"One"
      Text
        Body"#,
            r#"page componentsPage
  Checkbox color:"primary""#,
            r#"page componentsPage
  RadioGroup color:"primary"
    item value:"one" label:"One""#,
            r#"page componentsPage
  ToggleTheme color:"primary""#,
            r#"page componentsPage
  Fab color:"primary""#,
            r#"page componentsPage
  Slider color:"primary""#,
            r#"page componentsPage
  Dropzone color:"primary""#,
            r#"page componentsPage
  RichText
    mark text:"Launch" color:"primary""#,
            r#"page componentsPage
  Record name:"voice" color:"primary""#,
            r#"page componentsPage
  ToggleGroup color:"primary"
    item id:"one" label:"One""#,
            r#"page componentsPage
  Collapsible label:"Details" color:"primary"
    Text
      Body"#,
            r#"page componentsPage
  Countdown target:"2030-01-01T00:00:00Z" color:"primary""#,
            r#"page componentsPage
  Map color:"primary""#,
        ] {
            let error = parse_page(source).expect_err("color prop");
            assert!(error.to_string().contains("use `scheme`"));
        }
    }

    #[test]
    fn rejects_color_prop_for_display_and_overlay_components() {
        let error = parse_page(
            r#"page overlayPage
  Avatar color:"primary""#,
        )
        .expect_err("color");

        assert!(
            error
                .to_string()
                .contains("unknown prop `color` on `Avatar`; use `scheme` for visual family")
        );
    }

    #[test]
    fn rejects_invalid_drawer_open_signal() {
        let missing = parse_page(
            r#"page navPage
  Drawer
    Text
      Navigation"#,
        )
        .expect_err("open");
        assert!(
            missing
                .to_string()
                .contains("invalid value for prop `open`: expected signal bool path")
        );

        let wrong_type = parse_page(
            r#"page navPage
  signal title value:"Navigation"
  Drawer open:title
    Text
      Navigation"#,
        )
        .expect_err("bool");
        assert!(
            wrong_type
                .to_string()
                .contains("invalid signal path `title` in `open`: expected bool")
        );

        let quoted = parse_page(
            r#"page navPage
  signal drawerOpen value:false
  Drawer open:"drawerOpen"
    Text
      Navigation"#,
        )
        .expect_err("bare signal path");
        assert!(
            quoted
                .to_string()
                .contains("invalid value for prop `open`: expected signal bool path")
        );
    }

    #[test]
    fn rejects_unquoted_static_component_prop_strings() {
        let fill_error = parse_page(
            r#"page iconPage
  Svg viewBox:"0 0 24 24"
    Path d:"M0 0h24v24H0z" fill:none"#,
        )
        .expect_err("fill error");
        assert!(
            fill_error
                .to_string()
                .contains("invalid value for prop `fill`: expected quoted static string literal")
        );

        let option_error = parse_page(
            r#"page formPage
  Select label:"Role"
    Option value:admin label:"Administrator""#,
        )
        .expect_err("option error");
        assert!(
            option_error
                .to_string()
                .contains("invalid value for prop `value`: expected quoted static string literal")
        );

        let variant_error = parse_page(
            r#"page visualPage
  Input variant:outlined scheme:primary"#,
        )
        .expect_err("variant error");
        assert!(
            variant_error.to_string().contains(
                "invalid value for prop `variant`: expected quoted static string literal"
            )
        );

        let color_error = parse_page(
            r#"page visualPage
  Svg viewBox:"0 0 24 24" color:tertiary
    Path d:"M0 0h24v24H0z""#,
        )
        .expect_err("color error");
        assert!(
            color_error
                .to_string()
                .contains("invalid value for prop `color`: expected quoted static string literal")
        );

        let message_error = parse_page(
            r#"page alertPage
  Alert type:"info" message:Saved"#,
        )
        .expect_err("message error");
        assert!(
            message_error.to_string().contains(
                "invalid value for prop `message`: expected quoted static string literal"
            )
        );
    }

    #[test]
    fn rejects_path_outside_svg() {
        let error = parse_page(
            r#"page iconPage
  Path d:"M0 0""#,
        )
        .expect_err("error");

        assert!(
            error
                .to_string()
                .contains("Path can only be used inside Svg")
        );
    }

    #[test]
    fn rejects_non_path_svg_children() {
        let error = parse_page(
            r#"page iconPage
  Svg viewBox:"0 0 24 24"
    Text
      Bad"#,
        )
        .expect_err("error");

        assert!(error.to_string().contains("Svg only accepts Path children"));
    }

    fn parse_page(source: &str) -> crate::error::DoweResult<ViewNode> {
        let path = Path::new("/project/src/pages/blogs.dowe");
        let file = parse_source_file(Path::new("/project"), path, source.to_string())?;
        validate_view_source(&file, &environment())
    }

    fn environment() -> EnvironmentConfig {
        EnvironmentConfig {
            variables: vec![EnvironmentVariable {
                name: "BACKEND_URL".to_string(),
                visibility: EnvironmentVisibility::Client,
                required: false,
                default_value: Some(String::new()),
                resolved_source: EnvironmentValueSource::Default,
                resolved_value: Some(String::new()),
            }],
        }
    }
}

fn is_dynamic_reference(value: &str) -> bool {
    value.contains('.')
        && value
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || value == '_' || value == '.')
}

fn validate_navigation(pages: &[ViewPage]) -> DoweResult<()> {
    let mut sections_by_path = HashMap::new();
    for page in pages {
        sections_by_path.insert(
            page.route_path.clone(),
            page.sections
                .iter()
                .map(|section| section.id.clone())
                .collect::<HashSet<_>>(),
        );
    }
    for page in pages {
        for action in &page.navigation_actions {
            validate_navigation_action(page, action, &sections_by_path)?;
        }
    }
    Ok(())
}

fn validate_navigation_action(
    page: &ViewPage,
    action: &ViewNavigationAction,
    sections_by_path: &HashMap<String, HashSet<String>>,
) -> DoweResult<()> {
    match &action.action {
        dowe_components::NavigationAction::Internal { path, fragment, .. } => {
            let Some(sections) = sections_by_path.get(path) else {
                return Err(DoweError::at_path(
                    &page.source_path,
                    format!("unknown navigation route `{path}`"),
                ));
            };
            if let Some(fragment) = fragment
                && !sections.contains(fragment)
            {
                return Err(DoweError::at_path(
                    &page.source_path,
                    format!("unknown section `#{fragment}` for route `{path}`"),
                ));
            }
        }
        dowe_components::NavigationAction::Section { fragment, .. } => {
            let sections = sections_by_path
                .get(&page.route_path)
                .expect("current route sections");
            if !sections.contains(fragment) {
                return Err(DoweError::at_path(
                    &page.source_path,
                    format!(
                        "unknown section `#{fragment}` for route `{}`",
                        page.route_path
                    ),
                ));
            }
        }
        dowe_components::NavigationAction::External { .. }
        | dowe_components::NavigationAction::Back => {}
    }
    Ok(())
}

fn normalize_route_path(parent: &str, child: &str) -> String {
    let raw = if child.starts_with('/') {
        child.to_string()
    } else if child.is_empty() {
        parent.to_string()
    } else if parent == "/" {
        format!("/{child}")
    } else {
        format!("{}/{}", parent.trim_end_matches('/'), child)
    };
    let parts = raw
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}

fn combine_layout_stack(layouts: &[RouteLayout]) -> ViewNode {
    let mut tree = ViewNode::Children;
    for layout in layouts.iter().rev() {
        tree = compose_tree(&layout.tree, &tree);
    }
    tree
}

fn strip_web_prefix(path: &Path) -> String {
    path.strip_prefix("web")
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn route_id(path: &str) -> String {
    let name = path
        .trim_matches('/')
        .replace(|value: char| !value.is_ascii_alphanumeric(), "-");
    if name.is_empty() {
        "index".to_string()
    } else {
        name
    }
}

fn node_error(node: &SourceNode, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &node.location.path,
        format!(
            "{}:{}: {}",
            node.location.line,
            node.location.column,
            message.as_ref()
        ),
    )
}

fn prop_error(prop: &SourceProp, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &prop.location.path,
        format!(
            "{}:{}: {}",
            prop.location.line,
            prop.location.column,
            message.as_ref()
        ),
    )
}

fn quoted_static_string_error(prop: &SourceProp) -> DoweError {
    prop_error(
        prop,
        ComponentError::invalid_prop(&prop.name, "quoted static string literal").to_string(),
    )
}

fn component_error(node: &SourceNode, error: ComponentError) -> DoweError {
    let message = error.to_string();
    if let Some(name) = first_backtick_value(&message)
        && let Some(prop) = node.prop(name)
    {
        return prop_error(prop, message);
    }
    node_error(node, message)
}

fn first_backtick_value(message: &str) -> Option<&str> {
    let (_, after_open) = message.split_once('`')?;
    let (value, _) = after_open.split_once('`')?;
    if value.is_empty() { None } else { Some(value) }
}
