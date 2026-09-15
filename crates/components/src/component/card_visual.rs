#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardVisualContract {
    pub padding: u16,
    pub content_gap: u16,
    pub border_width: u16,
}

impl CardVisualContract {
    pub const fn standard() -> Self { Self { padding: 16, content_gap: 12, border_width: 1 } }
}
