pub fn build_layout_chunk(
    root: &Path,
    source_path: &Path,
    source: &str,
    layout_tree: &ViewNode,
) -> GeneratedChunk {
    build_layout_chunk_with_inspector(root, source_path, source, layout_tree, None)
}

pub fn build_layout_chunk_with_inspector(
    root: &Path,
    source_path: &Path,
    source: &str,
    layout_tree: &ViewNode,
    inspector: Option<&ViewInspectorMap>,
) -> GeneratedChunk {
    let id = short_id("layout", source);
    let expression = js_render_expression_with_inspector(layout_tree, inspector);
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
        inspector: inspector.cloned(),
    }
}

pub fn build_page_chunk(
    root: &Path,
    source_path: &Path,
    source: &str,
    page_tree: &ViewNode,
) -> GeneratedChunk {
    build_page_chunk_with_inspector(root, source_path, source, page_tree, None)
}

pub fn build_page_chunk_with_inspector(
    root: &Path,
    source_path: &Path,
    source: &str,
    page_tree: &ViewNode,
    inspector: Option<&ViewInspectorMap>,
) -> GeneratedChunk {
    let id = short_id("page", source);
    let expression = js_render_expression_with_inspector(page_tree, inspector);
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
        inspector: inspector.cloned(),
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

