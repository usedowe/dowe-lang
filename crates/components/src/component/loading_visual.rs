#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoadingVisualContract {
    pub spinner_size: u16,
    pub stroke_width: u16,
    pub disabled_alpha: f32,
}

impl LoadingVisualContract {
    pub const fn standard() -> Self { Self { spinner_size: 18, stroke_width: 2, disabled_alpha: 0.5 } }
}
