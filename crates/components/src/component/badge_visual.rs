#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BadgeVisualContract {
    pub font_size: u16,
    pub font_weight: u16,
    pub height: u16,
    pub horizontal_padding: u16,
    pub vertical_padding: u16,
}

impl BadgeVisualContract {
    pub const fn standard() -> Self {
        Self {
            font_size: 12,
            font_weight: 600,
            height: 20,
            horizontal_padding: 6,
            vertical_padding: 2,
        }
    }
}
