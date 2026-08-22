fn apply_component_visibility(
    children: Vec<SourceNode>,
    usage: &SourceNode,
) -> DoweResult<Vec<SourceNode>> {
    let Some(show) = usage.props.iter().find(|prop| prop.name == "show") else {
        return Ok(children);
    };
    if !matches!(
        show.value,
        SourceValue::Bareword(_) | SourceValue::Object(_)
    ) {
        return Err(node_error(
            usage,
            "component `show` must be a boolean Signal path or a supported condition",
        ));
    }
    Ok(children
        .into_iter()
        .map(|mut child| {
            child.props.retain(|prop| prop.name != "show");
            child.props.push(SourceProp {
                name: "show".to_string(),
                value: show.value.clone(),
                location: show.location.clone(),
            });
            child
        })
        .collect())
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
        let selected_platforms = platforms
            .iter()
            .copied()
            .filter(|platform| self.selected_platforms.contains(platform))
            .collect::<Vec<_>>();
        if selected_platforms.is_empty() {
            return Ok(());
        }
        let route_path = normalize_route_path(parent_path, &declaration.path);
        if declaration.children.is_empty() {
            self.add_page_route(declaration, route_path, layouts, selected_platforms)
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
        let metadata = compose_route_metadata(&layouts, &page.metadata);
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
        let reused = platforms
            .iter()
            .map(|platform| {
                self.reusable_route(
                    *platform,
                    &route_path,
                    &page.path,
                    &page.chunk_id,
                    &layout_chunk_ids,
                    &js_chunks,
                    &css_chunks,
                    &metadata,
                )
            })
            .collect::<Vec<_>>();
        if reused.iter().all(Option::is_some) {
            for (platform, reused) in platforms.into_iter().zip(reused) {
                let (page, route) = reused.expect("reusable route");
                self.outputs
                    .add_page(platform, page, route, self.views_path)?;
            }
            return Ok(());
        }
        let layout_inspector = self
            .dev_inspector
            .then(|| dowe_generator_web::ViewInspectorMap {
                nodes: layouts
                    .iter()
                    .flat_map(|layout| {
                        layout
                            .inspector
                            .as_ref()
                            .into_iter()
                            .flat_map(|map| map.nodes.clone())
                    })
                    .collect(),
            });
        let body_html = if self.dev_inspector {
            dowe_generator_web::render_routed_page_body_with_inspector(
                &layout_tree,
                &page.tree,
                &layout_chunk_ids,
                &page.chunk_id,
                layout_inspector.as_ref(),
                page.inspector.as_ref(),
            )
        } else {
            dowe_generator_web::render_routed_page_body(
                &layout_tree,
                &page.tree,
                &layout_chunk_ids,
                &page.chunk_id,
            )
        };
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
            runtime_chunks: Vec::new(),
            design_file_name: "design.css".to_string(),
            router_file_name: String::new(),
            boundaries,
            sections: sections.clone(),
            navigation_actions: navigation_actions.clone(),
            metadata,
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
            let mut platform_page = view_page.clone();
            if platform != ViewPlatform::Web {
                platform_page.metadata.clear();
            }
            self.outputs.add_page(
                platform,
                Arc::new(platform_page),
                view_route.clone(),
                self.views_path,
            )?;
        }
        Ok(())
    }

    fn reusable_route(
        &self,
        platform: ViewPlatform,
        route_path: &str,
        source_path: &Path,
        page_chunk_id: &str,
        layout_chunk_ids: &[String],
        js_chunks: &[String],
        css_chunks: &[String],
        metadata: &[ViewMetadata],
    ) -> Option<(Arc<ViewPage>, ViewRoute)> {
        let previous = self.previous?;
        let (pages, routes) = match platform {
            ViewPlatform::Web => (&previous.web.pages, &previous.routes.web),
            ViewPlatform::Desktop => (&previous.desktop_web.pages, &previous.routes.desktop),
            ViewPlatform::Android | ViewPlatform::Ios => return None,
        };
        let page = pages.iter().find(|page| page.route_path == route_path)?;
        let route = routes.iter().find(|route| route.route_path == route_path)?;
        let metadata_matches = platform != ViewPlatform::Web || page.metadata == metadata;
        let css_matches = page
            .css_chunks
            .iter()
            .filter(|path| !path.starts_with("chunks/design/"))
            .eq(css_chunks.iter());
        (page.source_path == source_path
            && page.page_chunk_id == page_chunk_id
            && page.layout_chunk_ids == layout_chunk_ids
            && page.js_chunks == js_chunks
            && css_matches
            && metadata_matches)
            .then(|| (Arc::clone(page), route.clone()))
    }

    fn layout_for(&mut self, component: &str) -> DoweResult<RouteLayout> {
        let module = self.module_for(component, ImportedViewKind::Layout)?;
        let chunk = self.chunk_for(component, module.as_ref())?;
        Ok(RouteLayout {
            tree: module.tree.clone(),
            inspector: module.inspector.clone(),
            metadata: module.metadata.clone(),
            chunk_id: chunk.id.clone(),
            js_path: strip_web_prefix(&chunk.relative_path),
            css_path: strip_web_prefix(&chunk.css_relative_path),
        })
    }

    fn page_for(&mut self, component: &str) -> DoweResult<RoutePage> {
        let module = self.module_for(component, ImportedViewKind::Page)?;
        let chunk = self.chunk_for(component, module.as_ref())?;
        Ok(RoutePage {
            tree: module.tree.clone(),
            inspector: module.inspector.clone(),
            metadata: module.metadata.clone(),
            path: module.path.clone(),
            chunk_id: chunk.id.clone(),
            js_path: strip_web_prefix(&chunk.relative_path),
            css_path: strip_web_prefix(&chunk.css_relative_path),
        })
    }

    fn module_for(
        &mut self,
        component: &str,
        expected: ImportedViewKind,
    ) -> DoweResult<Arc<ParsedViewModule>> {
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
        let cached = self.module_cache.as_deref_mut().and_then(|cache| {
            let cached = cache.entries.get(&import.path).cloned();
            if cached.is_some() {
                cache.hits += 1;
            }
            cached
        });
        if let Some(cached) = cached {
            if cached.module.name != component || cached.module.kind != expected {
                return Err(DoweError::at_path(
                    self.views_path,
                    format!("component `{component}` is used in the wrong route position"),
                ));
            }
            for (path, usages) in &cached.module.inspector_usages {
                self.inspector_usages
                    .entry(path.clone())
                    .or_default()
                    .extend(usages.clone());
            }
            self.module_chunks.insert(import.path.clone(), cached.chunk);
            self.modules
                .insert(component.to_string(), cached.module.clone());
            return Ok(cached.module);
        }
        if let Some(cache) = self.module_cache.as_deref_mut() {
            cache.misses += 1;
        }
        let source = fs::read_to_string(&import.path)
            .map_err(|error| DoweError::at_path(&import.path, error.to_string()))?;
        let file = parse_source_file(self.root, &import.path, source)?;
        let module_imports = view_imports(self.root, &file)?;
        let stores = view_store_imports(self.root, &file)?;
        let types = TypeRegistry::parse_file(self.root, &file)?;
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
        let usage_lengths = self
            .inspector_usages
            .iter()
            .map(|(path, usages)| (path.clone(), usages.len()))
            .collect::<HashMap<_, _>>();
        let expanded_root = self.expand_export_node(root_node, &module_imports)?;
        validate_view_theme_references(&expanded_root, self.design_config)?;
        if kind != expected {
            return Err(DoweError::at_path(
                self.views_path,
                format!("component `{component}` is used in the wrong route position"),
            ));
        }
        let export_name = expanded_root
            .args
            .first()
            .and_then(SourceValue::as_required_string)
            .ok_or_else(|| {
                node_error(&expanded_root, "layout or page export must declare a name")
            })?;
        if export_name != component {
            return Err(node_error(
                &expanded_root,
                format!("export `{export_name}` does not match import `{component}`"),
            ));
        }
        let mut tree = export_tree_with_stores(
            &expanded_root,
            kind == ImportedViewKind::Layout,
            self.environment,
            &types,
            &stores,
        )?;
        apply_design_defaults_to_tree(&mut tree, &self.design_config.defaults);
        apply_theme_catalog_to_tree(&mut tree, self.design_config);
        let metadata = parse_view_metadata(&expanded_root)?;
        let inspector = self
            .dev_inspector
            .then(|| build_view_inspector_map(&expanded_root, &tree, &self.inspector_usages));
        let inspector_usages = self
            .inspector_usages
            .iter()
            .filter_map(|(path, usages)| {
                let offset = usage_lengths.get(path).copied().unwrap_or_default();
                (offset < usages.len()).then(|| (path.clone(), usages[offset..].to_vec()))
            })
            .collect();
        let module = Arc::new(ParsedViewModule {
            name: export_name.to_string(),
            tree,
            inspector,
            inspector_usages,
            metadata,
            source: file.source,
            path: file.path,
            kind,
        });
        let chunk = Arc::new(generated_chunk_for_module(
            self.root,
            self.dev_inspector,
            &module,
        ));
        self.module_chunks
            .insert(import.path.clone(), chunk.clone());
        if let Some(cache) = self.module_cache.as_deref_mut() {
            cache.entries.insert(
                import.path,
                CachedViewModule {
                    module: module.clone(),
                    chunk,
                },
            );
        }
        self.modules.insert(component.to_string(), module.clone());
        Ok(module)
    }

    fn expand_export_node(
        &mut self,
        node: &SourceNode,
        imports: &HashMap<String, ViewImport>,
    ) -> DoweResult<SourceNode> {
        let mut used = HashSet::new();
        let mut expanded = node.clone();
        expanded.children = self.expand_node_children(imports, &node.children, &mut used)?;
        reject_unused_imports(&node.location.path, imports, &used)?;
        Ok(expanded)
    }

    fn expand_node_children(
        &mut self,
        imports: &HashMap<String, ViewImport>,
        nodes: &[SourceNode],
        used: &mut HashSet<String>,
    ) -> DoweResult<Vec<SourceNode>> {
        let mut expanded = Vec::new();
        for node in nodes {
            if COMPONENT_REGISTRY.get(&node.name).is_none() && node.name != "Pagination" {
                if let Some(import) = imports.get(&node.name) {
                    reject_component_usage_shape(node)?;
                    used.insert(node.name.clone());
                    if self.dev_inspector {
                        self.inspector_usages
                            .entry(import.path.clone())
                            .or_default()
                            .push(dowe_generator_web::ViewInspectorLocation {
                                path: node.location.relative_path.to_string_lossy().to_string(),
                                line: node.location.line,
                                column: node.location.column,
                            });
                    }
                    let children = self.component_children(&node.name, &import.path, node)?;
                    expanded.extend(apply_component_visibility(children, node)?);
                    continue;
                }
            }
            let mut child = node.clone();
            child.children = self.expand_node_children(imports, &node.children, used)?;
            expanded.push(child);
        }
        Ok(expanded)
    }

    fn component_children(
        &mut self,
        component: &str,
        path: &Path,
        usage: &SourceNode,
    ) -> DoweResult<Vec<SourceNode>> {
        let normalized = path.to_path_buf();
        if let Some(module) = self.components.get(&normalized) {
            if module.name != component {
                return Err(node_error(
                    usage,
                    format!(
                        "export `{}` does not match import `{component}`",
                        module.name
                    ),
                ));
            }
            return Ok(module.children.clone());
        }
        if self.component_stack.contains(&normalized) {
            return Err(node_error(
                usage,
                format!(
                    "component import cycle includes `{}`",
                    path.strip_prefix(self.root)
                        .unwrap_or(path)
                        .to_string_lossy()
                ),
            ));
        }

        self.component_stack.push(normalized.clone());
        let result = self.load_component(component, path);
        self.component_stack.pop();
        result
    }

    fn load_component(&mut self, component: &str, path: &Path) -> DoweResult<Vec<SourceNode>> {
        let source = fs::read_to_string(path)
            .map_err(|error| DoweError::at_path(path, error.to_string()))?;
        let file = parse_source_file(self.root, path, source)?;
        let imports = view_imports(self.root, &file)?;
        let root_node = single_export(&file)?;
        if root_node.name != "component" {
            return Err(node_error(
                root_node,
                format!("import `{component}` must export a component"),
            ));
        }
        if !root_node.props.is_empty() {
            return Err(node_error(
                root_node,
                "component export cannot declare props",
            ));
        }
        let export_name = root_node
            .args
            .first()
            .and_then(SourceValue::as_required_string)
            .ok_or_else(|| node_error(root_node, "component export must declare a name"))?;
        if export_name != component {
            return Err(node_error(
                root_node,
                format!("export `{export_name}` does not match import `{component}`"),
            ));
        }
        reject_component_state_nodes(root_node)?;
        if root_node.children.is_empty() {
            return Err(node_error(
                root_node,
                "component export must contain view nodes",
            ));
        }

        let mut used = HashSet::new();
        let children = self.expand_node_children(&imports, &root_node.children, &mut used)?;
        reject_unused_imports(&file.path, &imports, &used)?;
        self.components.insert(
            path.to_path_buf(),
            ParsedComponentModule {
                name: component.to_string(),
                children: children.clone(),
            },
        );
        Ok(children)
    }

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
