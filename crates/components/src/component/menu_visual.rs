#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuVisualContract {
    pub option_min_height: u16,
    pub option_padding: u16,
    pub menu_gap: u16,
    pub min_width: u16,
}

impl MenuVisualContract {
    pub const fn standard() -> Self { Self { option_min_height: 40, option_padding: 12, menu_gap: 4, min_width: 192 } }
}
