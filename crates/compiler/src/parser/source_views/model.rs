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
    components: HashMap<PathBuf, ParsedComponentModule>,
    component_stack: Vec<PathBuf>,
    chunks: Vec<dowe_generator_web::GeneratedChunk>,
    chunk_indexes: HashMap<String, usize>,
    outputs: PlatformRouteOutputs,
    environment: &'a EnvironmentConfig,
    design_config: &'a DesignConfig,
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
