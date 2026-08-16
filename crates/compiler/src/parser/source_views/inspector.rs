fn build_view_inspector_map(
    root_node: &SourceNode,
    tree: &ViewNode,
    usages_by_path: &HashMap<PathBuf, Vec<dowe_generator_web::ViewInspectorLocation>>,
) -> dowe_generator_web::ViewInspectorMap {
    let mut sources = Vec::new();
    collect_inspector_sources(&root_node.children, usages_by_path, &mut sources);
    let expected = count_inspector_elements(tree);
    if sources.len() > expected {
        sources.truncate(expected);
    }
    while sources.len() < expected {
        sources.push(InspectorSource {
            kind: "View".to_string(),
            path: root_node
                .location
                .relative_path
                .to_string_lossy()
                .to_string(),
            start_line: root_node.location.line,
            end_line: root_node.location.line,
            usages: Vec::new(),
        });
    }
    let nodes = sources
        .into_iter()
        .enumerate()
        .map(|(index, source)| {
            let id = inspector_id(index, &source);
            dowe_generator_web::ViewInspectorNode {
                id,
                kind: source.kind,
                source_path: source.path,
                start_line: source.start_line,
                end_line: source.end_line,
                usages: source.usages,
            }
        })
        .collect();
    dowe_generator_web::ViewInspectorMap { nodes }
}

#[derive(Clone)]
struct InspectorSource {
    kind: String,
    path: String,
    start_line: usize,
    end_line: usize,
    usages: Vec<dowe_generator_web::ViewInspectorLocation>,
}

fn collect_inspector_sources(
    nodes: &[SourceNode],
    usages_by_path: &HashMap<PathBuf, Vec<dowe_generator_web::ViewInspectorLocation>>,
    output: &mut Vec<InspectorSource>,
) {
    for node in nodes {
        let structural = matches!(
            node.name.as_str(),
            "meta"
                | "const"
                | "signal"
                | "fn"
                | "init"
                | "action"
                | "if"
                | "else"
                | "each"
                | "children"
                | "Splash"
                | "top"
                | "start"
                | "center"
                | "end"
                | "bottom"
                | "header"
                | "body"
                | "footer"
                | "overlays"
                | "appBar"
                | "main"
                | "content"
                | "actions"
                | "tab"
                | "item"
                | "option"
                | "column"
                | "group"
                | "step"
                | "slide"
        );
        let is_component = !structural && node.name.chars().next().is_some_and(char::is_uppercase);
        if is_component {
            let end_line = source_end_line(node);
            let source = InspectorSource {
                kind: node.name.clone(),
                path: node.location.relative_path.to_string_lossy().to_string(),
                start_line: node.location.line,
                end_line,
                usages: usages_by_path
                    .get(&node.location.path)
                    .cloned()
                    .unwrap_or_default(),
            };
            output.push(source.clone());
            if !matches!(node.name.as_str(), "Text" | "Title") {
                output.extend(
                    node.children
                        .iter()
                        .filter(|child| child.name.starts_with('"'))
                        .map(|_| source.clone()),
                );
            }
        }
        for child in &node.children {
            if !child.name.starts_with('"') {
                collect_inspector_sources(std::slice::from_ref(child), usages_by_path, output);
            }
        }
    }
}

fn source_end_line(node: &SourceNode) -> usize {
    node.children
        .iter()
        .map(source_end_line)
        .chain(node.props.iter().map(|prop| prop.location.line))
        .fold(node.location.line, usize::max)
}

fn count_inspector_elements(node: &ViewNode) -> usize {
    let own = usize::from(node_element_props(node).is_some());
    own + node_child_groups(node)
        .into_iter()
        .flat_map(|children| children.iter())
        .map(count_inspector_elements)
        .sum::<usize>()
}

fn inspector_id(index: usize, source: &InspectorSource) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in format!(
        "{}:{}:{}:{}:{}",
        source.path, source.start_line, source.end_line, source.kind, index
    )
    .bytes()
    {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("dn_{hash:016x}")
}
