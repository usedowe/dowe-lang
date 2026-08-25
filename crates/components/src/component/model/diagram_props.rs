#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramProps {
    pub style: VariantProps,
    pub nodes: String,
    pub edges: String,
    pub fit_view: bool,
    pub pan_on_drag: bool,
    pub zoom_on_scroll: bool,
    pub minimap: bool,
    pub controls: bool,
    pub show_grid: bool,
    pub empty_label: String,
    pub on_node_click: Option<String>,
    pub on_node_drag: Option<String>,
    pub on_connect: Option<String>,
}
