#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebOutput {
    pub chunks: Vec<Arc<GeneratedChunk>>,
    pub pages: Vec<Arc<ViewPage>>,
    pub translation_chunks: Vec<GeneratedTranslationChunk>,
    pub default_locale: Option<String>,
    pub router_js: String,
    pub render_report: dowe_components::RenderReport,
}

impl WebOutput {
    pub fn router_file_name(&self) -> String {
        if let Some(file_name) = self
            .pages
            .first()
            .map(|page| page.router_file_name.as_str())
            .filter(|file_name| !file_name.is_empty())
        {
            return file_name.to_string();
        }
        self.generated_router_file_name()
    }

    fn generated_router_file_name(&self) -> String {
        content_file_name("router", &self.router_js)
    }

    pub fn runtime_chunks(&self) -> Vec<GeneratedRuntimeChunk> {
        let paths = self
            .pages
            .iter()
            .flat_map(|page| page.runtime_chunks.iter())
            .collect::<BTreeSet<_>>();
        let mut chunks = Vec::new();
        if paths
            .iter()
            .any(|path| path.starts_with("chunks/runtime/styles-"))
        {
            chunks.push(styles_runtime_chunk());
        }
        if paths
            .iter()
            .any(|path| path.starts_with("chunks/runtime/controls-"))
        {
            chunks.push(controls_runtime_chunk());
        }
        if paths
            .iter()
            .any(|path| path.starts_with("chunks/runtime/media-"))
        {
            chunks.push(media_runtime_chunk());
        }
        if paths
            .iter()
            .any(|path| path.starts_with("chunks/runtime/visualization-"))
        {
            chunks.push(visualization_runtime_chunk());
        }
        chunks
    }

    pub fn design_file_name(&self) -> &str {
        self.pages
            .first()
            .map(|page| page.design_file_name.as_str())
            .filter(|file_name| !file_name.is_empty())
            .unwrap_or("design.css")
    }

    pub fn design_file_names(&self) -> BTreeSet<String> {
        let mut file_names = self
            .pages
            .iter()
            .map(|page| page.design_file_name.clone())
            .filter(|file_name| !file_name.is_empty())
            .collect::<BTreeSet<_>>();
        file_names.insert(self.design_file_name().to_string());
        file_names
    }

    pub fn has_design_file_name(&self, file_name: &str) -> bool {
        !file_name.is_empty()
            && (self.design_file_name() == file_name
                || self
                    .pages
                    .iter()
                    .any(|page| page.design_file_name == file_name))
    }
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
    pub inspector: Option<ViewInspectorMap>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedTranslationChunk {
    pub id: String,
    pub locale: String,
    pub relative_path: PathBuf,
    pub source_path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedRuntimeChunk {
    pub name: &'static str,
    pub file_name: String,
    pub relative_path: PathBuf,
    pub content: String,
}

impl GeneratedRuntimeChunk {
    fn new(name: &'static str, content: String) -> Self {
        let file_name = content_file_name(name, &content);
        Self {
            name,
            relative_path: Path::new("web/chunks/runtime").join(&file_name),
            file_name,
            content,
        }
    }

    pub fn browser_path(&self) -> String {
        self.relative_path
            .strip_prefix("web")
            .unwrap_or(&self.relative_path)
            .to_string_lossy()
            .trim_start_matches('/')
            .to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GeneratedDesignCssChunk {
    name: &'static str,
    relative_path: PathBuf,
    content: String,
}

impl GeneratedDesignCssChunk {
    fn new(name: &'static str, content: String) -> Self {
        let file_name = format!(
            "{name}-{}.css",
            short_id(&format!("design:{name}"), &content)
        );
        Self {
            name,
            relative_path: Path::new("web/chunks/design").join(file_name),
            content,
        }
    }

    fn browser_path(&self) -> String {
        self.relative_path
            .strip_prefix("web")
            .unwrap_or(&self.relative_path)
            .to_string_lossy()
            .trim_start_matches('/')
            .to_string()
    }
}

fn content_file_name(name: &str, content: &str) -> String {
    format!(
        "{name}-{}.js",
        short_id(&format!("runtime:{name}"), content)
    )
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
    pub runtime_chunks: Vec<String>,
    pub design_file_name: String,
    pub router_file_name: String,
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

pub struct WebArtifactUpdate {
    pub files: Vec<WebArtifact>,
    pub expected_paths: BTreeSet<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebArtifactKind {
    Chunk,
    Css,
    Manifest,
    Html,
}

