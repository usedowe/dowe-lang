fn android_runtime_tree() -> String {
    let icon = |name: &str, color: &str, modifier: &str| {
        let icon = solar_control_icon(name).expect("bundled Tree icon");
        format!(
            "DoweSvg(viewBox = {}, modifier = {}, color = {}, paths = {})",
            compose_svg_view_box(&icon.props.view_box),
            modifier,
            color,
            compose_svg_paths(&icon.paths)
        )
    };
    let arrow = icon("alt-arrow-down", "contentColor.copy(alpha = 0.72f)", "Modifier.size(16.dp).rotate(arrowRotation)");
    let folder = icon("folder-with-files", "contentColor.copy(alpha = 0.88f)", "Modifier.size(16.dp)");
    let file = icon("file-text", "contentColor.copy(alpha = 0.72f)", "Modifier.size(16.dp)");
    let mut output = r#"@Composable
private fun DoweTree(state: DoweReactiveState, dataPath: String, bindPath: String?, defaultOpen: Boolean, emptyLabel: String, ariaLabel: String, onSelect: String?, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: Dp) {
    val nodes = state.treeNodes(dataPath)
    val localSelected = remember(dataPath) { mutableStateOf("") }
    val selected = bindPath?.let { state.text(it) } ?: localSelected.value
    val openIds = remember(dataPath, defaultOpen) { mutableStateMapOf<String, Boolean>() }
    val scope = rememberCoroutineScope()
    val shape = RoundedCornerShape(radius)
    Column(
        modifier = modifier
            .fillMaxWidth()
            .clip(shape)
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, shape))
            .semantics { contentDescription = ariaLabel }
            .padding(vertical = 4.dp),
        verticalArrangement = Arrangement.spacedBy(1.dp)
    ) {
        if (nodes.isEmpty()) {
            Text(text = emptyLabel, color = contentColor.copy(alpha = 0.64f), fontSize = 14.sp, modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp))
        } else {
            nodes.forEach { node ->
                DoweTreeNodeView(node, 0, selected, defaultOpen, openIds, state, bindPath, onSelect, localSelected, scope, contentColor)
            }
        }
    }
}

@Composable
private fun DoweTreeNodeView(node: DoweTreeNode, depth: Int, selected: String, defaultOpen: Boolean, openIds: MutableMap<String, Boolean>, state: DoweReactiveState, bindPath: String?, onSelect: String?, localSelected: androidx.compose.runtime.MutableState<String>, scope: kotlinx.coroutines.CoroutineScope, contentColor: Color) {
    val open = openIds[node.id] ?: defaultOpen
    val isSelected = !node.branch && selected == node.path
    val arrowRotation by animateFloatAsState(if (open) 0f else -90f, animationSpec = tween(160), label = "dowe-tree-arrow")
    Column(modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 28.dp)
                .clip(RoundedCornerShape(5.dp))
                .background(if (isSelected) contentColor.copy(alpha = 0.14f) else Color.Transparent)
                .clickable {
                    if (node.branch) {
                        openIds[node.id] = !open
                    } else {
                        if (bindPath != null) state.write(bindPath, node.path) else localSelected.value = node.path
                        onSelect?.let { action -> scope.launch { state.run(action, node.value) } }
                    }
                }
                .padding(horizontal = 4.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            repeat(depth) {
                Box(modifier = Modifier.width(16.dp).height(28.dp))
            }
            Box(modifier = Modifier.size(20.dp), contentAlignment = Alignment.Center) {
                if (node.branch) {
                    __TREE_ARROW_ICON__
                }
            }
            Box(modifier = Modifier.size(16.dp), contentAlignment = Alignment.Center) {
                if (node.branch) {
                    __TREE_FOLDER_ICON__
                } else {
                    __TREE_FILE_ICON__
                }
            }
            Text(text = node.label, color = contentColor, fontSize = 14.sp, fontWeight = if (node.branch) FontWeight.SemiBold else FontWeight.Normal, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
        }
        if (node.branch && open) {
            node.children.forEach { child ->
                DoweTreeNodeView(child, depth + 1, selected, defaultOpen, openIds, state, bindPath, onSelect, localSelected, scope, contentColor)
            }
        }
    }
}

"#.to_string();
    output = output.replace("__TREE_ARROW_ICON__", &arrow);
    output = output.replace("__TREE_FOLDER_ICON__", &folder);
    output = output.replace("__TREE_FILE_ICON__", &file);
    output
}
