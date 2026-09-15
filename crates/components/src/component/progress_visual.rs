#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressVisualContract {
    pub track_height: u16,
    pub radius: u16,
    pub disabled_alpha: u16,
}

impl ProgressVisualContract {
    pub const fn standard() -> Self { Self { track_height: 6, radius: 999, disabled_alpha: 50 } }
}
