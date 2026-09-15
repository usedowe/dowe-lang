#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaVisualContract {
    pub default_radius: u16,
    pub control_size: u16,
    pub overlay_alpha: u16,
}

impl MediaVisualContract {
    pub const fn standard() -> Self { Self { default_radius: 8, control_size: 40, overlay_alpha: 48 } }
}
