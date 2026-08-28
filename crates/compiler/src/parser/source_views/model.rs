#[derive(Clone)]
struct ViewImport {
    path: PathBuf,
}

#[derive(Clone)]
struct ImportedViewStore {
    name: String,
    storage_key: String,
    storage: ViewSignalStorage,
    initial: ViewSignalValue,
    schema: Option<ViewSignalValue>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ImportedViewKind {
    Layout,
    Page,
}

#[derive(Clone)]
struct ParsedComponentModule {
    name: String,
    children: Vec<SourceNode>,
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
    name: String,
    tree: ViewNode,
    inspector: Option<dowe_generator_web::ViewInspectorMap>,
    inspector_usages: HashMap<PathBuf, Vec<dowe_generator_web::ViewInspectorLocation>>,
    metadata: Vec<ViewMetadata>,
    source: String,
    path: PathBuf,
    kind: ImportedViewKind,
}

#[derive(Clone)]
struct CachedViewModule {
    module: Arc<ParsedViewModule>,
    chunk: Arc<dowe_generator_web::GeneratedChunk>,
}

#[derive(Clone, Default)]
pub(crate) struct ViewModuleCache {
    entries: HashMap<PathBuf, CachedViewModule>,
    hits: usize,
    misses: usize,
}

impl ViewModuleCache {
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn remove(&mut self, path: &Path) {
        self.entries.remove(path);
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn hits(&self) -> usize {
        self.hits
    }

    pub(crate) fn misses(&self) -> usize {
        self.misses
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PreviousViewOutputs<'a> {
    pub web: &'a WebOutput,
    pub desktop_web: &'a WebOutput,
    pub routes: &'a ViewTargetRoutes,
}

#[derive(Clone)]
struct RouteLayout {
    tree: ViewNode,
    inspector: Option<dowe_generator_web::ViewInspectorMap>,
    metadata: Vec<ViewMetadata>,
    chunk_id: String,
    js_path: String,
    css_path: String,
}

struct RoutePage {
    tree: ViewNode,
    inspector: Option<dowe_generator_web::ViewInspectorMap>,
    metadata: Vec<ViewMetadata>,
    path: PathBuf,
    chunk_id: String,
    js_path: String,
    css_path: String,
}

struct RouteBuildContext<'a> {
    root: &'a Path,
    views_path: &'a Path,
    imports: HashMap<String, ViewImport>,
    modules: HashMap<String, Arc<ParsedViewModule>>,
    module_chunks: HashMap<PathBuf, Arc<dowe_generator_web::GeneratedChunk>>,
    module_cache: Option<&'a mut ViewModuleCache>,
    components: HashMap<PathBuf, ParsedComponentModule>,
    inspector_usages: HashMap<PathBuf, Vec<dowe_generator_web::ViewInspectorLocation>>,
    component_stack: Vec<PathBuf>,
    chunks: Vec<Arc<dowe_generator_web::GeneratedChunk>>,
    chunk_indexes: HashMap<String, usize>,
    outputs: PlatformRouteOutputs,
    environment: &'a EnvironmentConfig,
    design_config: &'a DesignConfig,
    selected_platforms: &'a [ViewPlatform],
    dev_inspector: bool,
    previous: Option<PreviousViewOutputs<'a>>,
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
    pages: Vec<Arc<ViewPage>>,
    routes: Vec<ViewRoute>,
    seen_paths: HashSet<String>,
}

fn web_output_for(
    mut pages: Vec<Arc<ViewPage>>,
    chunks: &[Arc<dowe_generator_web::GeneratedChunk>],
    translation_chunks: &[dowe_generator_web::GeneratedTranslationChunk],
    translations: &TranslationCatalog,
    previous: Option<&WebOutput>,
) -> WebOutput {
    for page in &mut pages {
        let reused = previous.is_some_and(|previous| {
            previous
                .pages
                .iter()
                .any(|candidate| Arc::ptr_eq(candidate, page))
        });
        if !reused {
            let page = Arc::make_mut(page);
            page.runtime_chunks = dowe_generator_web::runtime_chunks_for_page(page)
                .iter()
                .map(dowe_generator_web::GeneratedRuntimeChunk::browser_path)
                .collect();
        }
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
        render_report: dowe_components::RenderReport::new(dowe_components::RenderTarget::Web, Vec::new()),
    };
    web.render_report = dowe_components::RenderReport::from_routes(
        dowe_components::RenderTarget::Web,
        web.pages
            .iter()
            .map(|page| dowe_components::RouteRenderReport {
                route_path: page.route_path.clone(),
                accepted: Vec::new(),
                lowered: Vec::new(),
                present: Vec::new(),
                consumed: {
                    let mut entries = dowe_generator_web::consumed_props_for_tree(&page.layout_tree);
                    entries.extend(dowe_generator_web::consumed_props_for_tree(&page.page_tree));
                    entries
                },
                emitted: Vec::new(),
            })
            .collect(),
    );
    if previous.is_none() {
        web.router_js = router_js(&web);
        let router_file_name = web.router_file_name();
        for page in &mut web.pages {
            let page = Arc::make_mut(page);
            page.router_file_name.clone_from(&router_file_name);
            page.html_document = render_page_document(page);
        }
    }
    web
}

impl PlatformRouteOutputs {
    fn add_page(
        &mut self,
        platform: ViewPlatform,
        page: Arc<ViewPage>,
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
