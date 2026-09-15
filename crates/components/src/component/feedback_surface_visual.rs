#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeedbackSurfaceVisualContract {
    pub panel_padding: u16,
    pub content_gap: u16,
    pub tooltip_padding_x: u16,
    pub tooltip_padding_y: u16,
}

impl FeedbackSurfaceVisualContract {
    pub const fn standard() -> Self { Self { panel_padding: 16, content_gap: 12, tooltip_padding_x: 12, tooltip_padding_y: 8 } }
}
