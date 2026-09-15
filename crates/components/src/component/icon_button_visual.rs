/// Shared visual roles for IconButton and every control that composes it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconButtonVisualContract {
    pub border_alpha: f32,
    pub disabled_alpha: f32,
    pub focus_ring_alpha: f32,
}

impl IconButtonVisualContract {
    pub const fn standard() -> Self {
        Self { border_alpha: 0.24, disabled_alpha: 0.42, focus_ring_alpha: 0.24 }
    }
}
