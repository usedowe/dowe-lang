#[derive(Default)]
struct BuiltViews {
    chunks: Vec<dowe_generator_web::GeneratedChunk>,
    outputs: PlatformRouteOutputs,
}

pub fn parse_views_file(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
    translations: &TranslationCatalog,
    design_config: &DesignConfig,
) -> DoweResult<ParsedViews> {
    let declarations = view_declarations(file)?;
    let built = build_views_declarations(root, file, declarations, environment, design_config)?;
    finalize_built_views(root, &file.path, built, translations)
}

fn build_views_declarations(
    root: &Path,
    file: &SourceFile,
    declarations: Vec<ViewDeclaration>,
    environment: &EnvironmentConfig,
    design_config: &DesignConfig,
) -> DoweResult<BuiltViews> {
    let imports = view_imports(root, file)?;
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
        components: HashMap::new(),
        component_stack: Vec::new(),
        chunks: Vec::new(),
        chunk_indexes: HashMap::new(),
        outputs: PlatformRouteOutputs::default(),
        environment,
        design_config,
    };

    for declaration in declarations {
        context.visit_route(&declaration, "/", Vec::new(), ViewPlatform::all().to_vec())?;
    }

    Ok(BuiltViews {
        chunks: context.chunks,
        outputs: context.outputs,
    })
}

fn finalize_built_views(
    root: &Path,
    diagnostic_path: &Path,
    built: BuiltViews,
    translations: &TranslationCatalog,
) -> DoweResult<ParsedViews> {
    let BuiltViews { chunks, outputs } = built;
    let PlatformRouteOutputs {
        web,
        desktop,
        android,
        ios,
    } = outputs;
    validate_navigation(&web.pages)?;
    validate_navigation(&desktop.pages)?;
    validate_navigation(&android.pages)?;
    validate_navigation(&ios.pages)?;
    let routes = ViewTargetRoutes {
        web: web.routes,
        desktop: desktop.routes,
        android: android.routes,
        ios: ios.routes,
    };
    validate_view_i18n_keys(diagnostic_path, &routes, translations)?;
    let translation_chunks = build_translation_chunks(root, translations);
    let web = web_output_for(web.pages, &chunks, &translation_chunks, translations);
    let desktop_web = web_output_for(desktop.pages, &chunks, &translation_chunks, translations);
    Ok(ParsedViews {
        web,
        desktop_web,
        routes,
    })
}

pub fn parse_views_entry(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
    translations: &TranslationCatalog,
    design_config: &DesignConfig,
) -> DoweResult<ParsedViews> {
    let main = single_main(file)?;
    let views_child = main.children.iter().find(|child| child.name == "views");
    let Some(views_node) = views_child else {
        return Err(DoweError::at_path(
            &file.path,
            "`main.dowe` must declare `views:<name>`",
        ));
    };
    if let Some(references) = views_references(views_node)? {
        let mut combined = BuiltViews::default();
        for reference in references {
            let import = file
                .imports
                .iter()
                .find(|import| import.local == reference)
                .ok_or_else(|| node_error(views_node, format!("missing import `{reference}`")))?;
            let path = resolve_import(root, &file.path, import)?;
            let source = fs::read_to_string(&path)
                .map_err(|error| DoweError::at_path(&path, error.to_string()))?;
            let module_file = parse_source_file(root, &path, source)?;
            let declarations = view_declarations_named(&module_file, Some(&reference))?;
            let built = build_views_declarations(
                root,
                &module_file,
                declarations,
                environment,
                design_config,
            )?;
            merge_built_views(&mut combined, built, &module_file.path)?;
        }
        return finalize_built_views(root, &file.path, combined, translations);
    }
    let declarations = views_node
        .children
        .iter()
        .map(|node| parse_route_node(node, false))
        .collect::<DoweResult<Vec<_>>>()?;
    let built = build_views_declarations(
        root,
        file,
        non_empty_view_declarations(file, declarations)?,
        environment,
        design_config,
    )?;
    finalize_built_views(root, &file.path, built, translations)
}

fn merge_built_views(target: &mut BuiltViews, source: BuiltViews, path: &Path) -> DoweResult<()> {
    for chunk in source.chunks {
        if let Some(existing) = target
            .chunks
            .iter()
            .find(|existing| existing.id == chunk.id)
        {
            if existing != &chunk {
                return Err(DoweError::at_path(
                    path,
                    format!("conflicting generated view chunk `{}`", chunk.id),
                ));
            }
        } else {
            target.chunks.push(chunk);
        }
    }
    merge_platform_output(
        &mut target.outputs,
        ViewPlatform::Web,
        source.outputs.web,
        path,
    )?;
    merge_platform_output(
        &mut target.outputs,
        ViewPlatform::Desktop,
        source.outputs.desktop,
        path,
    )?;
    merge_platform_output(
        &mut target.outputs,
        ViewPlatform::Android,
        source.outputs.android,
        path,
    )?;
    merge_platform_output(
        &mut target.outputs,
        ViewPlatform::Ios,
        source.outputs.ios,
        path,
    )
}

fn merge_platform_output(
    target: &mut PlatformRouteOutputs,
    platform: ViewPlatform,
    source: PlatformRouteOutput,
    path: &Path,
) -> DoweResult<()> {
    for (page, route) in source.pages.into_iter().zip(source.routes) {
        target.add_page(platform, page, route, path)?;
    }
    Ok(())
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
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
) -> DoweResult<ViewNode> {
    let types = TypeRegistry::parse_file(root, file)?;
    let stores = view_store_imports(root, file)?;
    let root_node = single_export(file)?;
    let imports = view_imports(root, file)?;
    let design_config = DesignConfig::default();
    let mut context = RouteBuildContext {
        root,
        views_path: &file.path,
        imports: HashMap::new(),
        modules: HashMap::new(),
        components: HashMap::new(),
        component_stack: Vec::new(),
        chunks: Vec::new(),
        chunk_indexes: HashMap::new(),
        outputs: PlatformRouteOutputs::default(),
        environment,
        design_config: &design_config,
    };
    let root_node = context.expand_export_node(root_node, &imports)?;
    match root_node.name.as_str() {
        "layout" => export_tree_with_stores(&root_node, true, environment, &types, &stores),
        "page" => export_tree_with_stores(&root_node, false, environment, &types, &stores),
        _ => Err(node_error(
            &root_node,
            "view modules must export a layout or page",
        )),
    }
}
