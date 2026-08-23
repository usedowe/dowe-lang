pub fn web_artifact_update(
    web: &WebOutput,
    previous: Option<&WebOutput>,
    design_css: String,
) -> WebArtifactUpdate {
    let mut files = Vec::new();
    let mut expected_paths = BTreeSet::new();
    let previous_chunks = previous
        .into_iter()
        .flat_map(|output| output.chunks.iter())
        .map(|chunk| (chunk.relative_path.clone(), chunk))
        .collect::<BTreeMap<_, _>>();
    for chunk in &web.chunks {
        expected_paths.insert(chunk.relative_path.clone());
        expected_paths.insert(chunk.css_relative_path.clone());
        if previous_chunks
            .get(&chunk.relative_path)
            .is_none_or(|previous| previous.content != chunk.content)
        {
            files.push(WebArtifact {
                relative_path: chunk.relative_path.clone(),
                content: chunk.content.clone(),
                kind: WebArtifactKind::Chunk,
                target: "web",
            });
        }
        if previous_chunks
            .get(&chunk.relative_path)
            .is_none_or(|previous| previous.css_content != chunk.css_content)
        {
            files.push(WebArtifact {
                relative_path: chunk.css_relative_path.clone(),
                content: chunk.css_content.clone(),
                kind: WebArtifactKind::Css,
                target: "web",
            });
        }
    }

    let previous_translations = previous
        .into_iter()
        .flat_map(|output| output.translation_chunks.iter())
        .map(|chunk| (chunk.relative_path.clone(), chunk))
        .collect::<BTreeMap<_, _>>();
    for chunk in &web.translation_chunks {
        expected_paths.insert(chunk.relative_path.clone());
        if previous_translations
            .get(&chunk.relative_path)
            .is_none_or(|previous| previous.content != chunk.content)
        {
            files.push(WebArtifact {
                relative_path: chunk.relative_path.clone(),
                content: chunk.content.clone(),
                kind: WebArtifactKind::Chunk,
                target: "web",
            });
        }
    }

    let previous_runtime = previous
        .map(WebOutput::runtime_chunks)
        .unwrap_or_default()
        .into_iter()
        .map(|chunk| (chunk.relative_path, chunk.content))
        .collect::<BTreeMap<_, _>>();
    for chunk in web.runtime_chunks() {
        expected_paths.insert(chunk.relative_path.clone());
        if previous_runtime
            .get(&chunk.relative_path)
            .is_none_or(|content| content != &chunk.content)
        {
            files.push(WebArtifact {
                relative_path: chunk.relative_path,
                content: chunk.content,
                kind: WebArtifactKind::Chunk,
                target: "web",
            });
        }
    }

    let previous_design_chunks = previous
        .map(design_css_chunks_for_web)
        .unwrap_or_default()
        .into_iter()
        .map(|chunk| (chunk.relative_path, chunk.content))
        .collect::<BTreeMap<_, _>>();
    for chunk in design_css_chunks_for_web(web) {
        expected_paths.insert(chunk.relative_path.clone());
        if previous_design_chunks
            .get(&chunk.relative_path)
            .is_none_or(|content| content != &chunk.content)
        {
            files.push(WebArtifact {
                relative_path: chunk.relative_path,
                content: chunk.content,
                kind: WebArtifactKind::Css,
                target: "web",
            });
        }
    }

    let previous_design_file_names = previous
        .map(WebOutput::design_file_names)
        .unwrap_or_default();
    for file_name in web.design_file_names() {
        let design_path = Path::new("web").join(&file_name);
        expected_paths.insert(design_path.clone());
        if !previous_design_file_names.contains(&file_name) {
            files.push(WebArtifact {
                relative_path: design_path,
                content: design_css.clone(),
                kind: WebArtifactKind::Css,
                target: "web",
            });
        }
    }

    let router_file_name = prepared_router_file_name(web);
    let router_path = Path::new("web").join(router_file_name);
    expected_paths.insert(router_path.clone());
    if previous.is_none_or(|output| {
        prepared_router_file_name(output) != router_file_name || output.router_js != web.router_js
    }) {
        files.push(WebArtifact {
            relative_path: router_path,
            content: web.router_js.clone(),
            kind: WebArtifactKind::Chunk,
            target: "web",
        });
    }

    let manifest_path = PathBuf::from("web/manifest.json");
    expected_paths.insert(manifest_path.clone());
    files.push(WebArtifact {
        relative_path: manifest_path,
        content: manifest(web),
        kind: WebArtifactKind::Manifest,
        target: "web",
    });

    let previous_pages = previous
        .into_iter()
        .flat_map(|output| output.pages.iter())
        .map(|page| (page.route_path.as_str(), page))
        .collect::<BTreeMap<_, _>>();
    if let Some(page) = web.pages.first() {
        let index_path = PathBuf::from("web/index.html");
        expected_paths.insert(index_path.clone());
        if previous_pages
            .get(page.route_path.as_str())
            .is_none_or(|previous| previous.html_document != page.html_document)
        {
            files.push(WebArtifact {
                relative_path: index_path,
                content: static_html_document(&page.html_document, ""),
                kind: WebArtifactKind::Html,
                target: "web",
            });
        }
    }
    for page in &web.pages {
        let path = PathBuf::from(format!("web/pages/{}.html", page_file_name(page)));
        expected_paths.insert(path.clone());
        if previous_pages
            .get(page.route_path.as_str())
            .is_none_or(|previous| previous.html_document != page.html_document)
        {
            files.push(WebArtifact {
                relative_path: path,
                content: static_html_document(&page.html_document, "../"),
                kind: WebArtifactKind::Html,
                target: "web",
            });
        }
    }

    WebArtifactUpdate {
        files,
        expected_paths,
    }
}

fn prepared_router_file_name(web: &WebOutput) -> &str {
    web.pages
        .first()
        .map(|page| page.router_file_name.as_str())
        .filter(|file_name| !file_name.is_empty())
        .unwrap_or("router.js")
}

