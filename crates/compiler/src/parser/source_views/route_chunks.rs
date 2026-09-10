impl RouteBuildContext<'_> {
    fn chunk_for(
        &mut self,
        component: &str,
        module: &ParsedViewModule,
    ) -> DoweResult<Arc<dowe_generator_web::GeneratedChunk>> {
        if let Some(index) = self.chunk_indexes.get(component) {
            return Ok(self.chunks[*index].clone());
        }
        let chunk = self
            .module_chunks
            .get(&module.path)
            .cloned()
            .unwrap_or_else(|| {
                Arc::new(generated_chunk_for_module(
                    self.root,
                    self.dev_inspector,
                    module,
                ))
            });
        let index = self.chunks.len();
        self.chunks.push(chunk.clone());
        self.chunk_indexes.insert(component.to_string(), index);
        Ok(chunk)
    }
}

fn generated_chunk_for_module(
    root: &Path,
    dev_inspector: bool,
    module: &ParsedViewModule,
) -> dowe_generator_web::GeneratedChunk {
    match module.kind {
        ImportedViewKind::Layout if dev_inspector => {
            dowe_generator_web::build_layout_chunk_with_inspector(
                root,
                &module.path,
                &module.source,
                &module.tree,
                module.inspector.as_ref(),
            )
        }
        ImportedViewKind::Layout => {
            dowe_generator_web::build_layout_chunk(root, &module.path, &module.source, &module.tree)
        }
        ImportedViewKind::Page if dev_inspector => {
            dowe_generator_web::build_page_chunk_with_inspector(
                root,
                &module.path,
                &module.source,
                &module.tree,
                module.inspector.as_ref(),
            )
        }
        ImportedViewKind::Page => {
            dowe_generator_web::build_page_chunk(root, &module.path, &module.source, &module.tree)
        }
    }
}
