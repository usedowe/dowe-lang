impl RouteBuildContext<'_> {
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
            if node.name == "invoke"
                && let Some(function) = node
                    .prop("fn")
                    .and_then(|prop| prop.value.as_required_string())
                && imports.contains_key(&function)
            {
                used.insert(function.to_string());
            }
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
}
