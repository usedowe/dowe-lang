#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BannerVisualContract {
    pub padding: u16,
    pub content_gap: u16,
    pub action_height: u16,
    pub disabled_alpha: u16,
}

impl BannerVisualContract {
    pub const fn standard() -> Self { Self { padding: 16, content_gap: 12, action_height: 40, disabled_alpha: 50 } }
}
