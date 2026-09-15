/// Shared visual metrics for simple surfaces rendered by every target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DividerVisualContract {
    pub thickness: u16,
}

impl DividerVisualContract {
    pub const fn standard() -> Self { Self { thickness: 1 } }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkeletonVisualContract {
    pub text_height: u16,
    pub default_radius: u16,
    pub pulse_alpha: f32,
    pub pulse_duration_ms: u16,
}

impl SkeletonVisualContract {
    pub const fn standard() -> Self {
        Self { text_height: 16, default_radius: 6, pulse_alpha: 0.45, pulse_duration_ms: 900 }
    }
}
