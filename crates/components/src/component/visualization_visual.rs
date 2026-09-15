#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisualizationVisualContract {
    pub container_padding: u16,
    pub grid_line_width: u16,
    pub empty_min_height: u16,
}

impl VisualizationVisualContract {
    pub const fn standard() -> Self { Self { container_padding: 16, grid_line_width: 1, empty_min_height: 180 } }
}
