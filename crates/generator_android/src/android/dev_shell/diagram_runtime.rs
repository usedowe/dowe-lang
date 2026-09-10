fn dev_activity_diagram_runtime() -> &'static str {
    r#"    private DoweDiagramView doweDiagram(String nodesPath, String edgesPath, boolean fitView, boolean panOnDrag, boolean zoomOnScroll, boolean controls, boolean minimap, boolean showGrid, String emptyLabel, String onNodeClick, String onNodeDrag, String onConnect, int backgroundColor, int contentColor, float radius) {
        DoweDiagramView view = new DoweDiagramView(this, nodesPath, edgesPath, fitView, panOnDrag, zoomOnScroll, controls, minimap, showGrid, emptyLabel, backgroundColor, contentColor);
        view.setBackground(doweInputBackground(backgroundColor, doweAlpha(contentColor, 0.12f), (int) radius));
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(300)));
        view.setOnDiagramAction((name, item) -> { if (name != null) doweRunAction(name, item); });
        if (onNodeClick != null) view.setNodeClickListener(onNodeClick);
        if (onNodeDrag != null) view.setNodeDragListener(onNodeDrag);
        if (onConnect != null) view.setConnectListener(onConnect);
        return view;
    }
"#
}


fn dev_activity_diagram_view() -> &'static str {
    concat!(
        include!("diagram_view_declaration.rs"),
        include!("diagram_view_data.rs"),
        include!("diagram_view_rendering.rs"),
        include!("diagram_view_interactions.rs"),
    )
}
