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
}
