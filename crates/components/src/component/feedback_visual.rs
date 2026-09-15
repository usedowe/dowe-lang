#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlertVisualContract {
    pub panel_padding: u16,
    pub content_gap: u16,
    pub title_size: u16,
    pub description_size: u16,
    pub button_gap: u16,
    pub close_button_size: u16,
}

impl AlertVisualContract {
    pub const fn standard() -> Self { Self { panel_padding: 20, content_gap: 16, title_size: 18, description_size: 14, button_gap: 12, close_button_size: 24 } }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmptyVisualContract {
    pub panel_padding: u16,
    pub content_gap: u16,
    pub icon_size: u16,
    pub title_size: u16,
    pub description_size: u16,
    pub action_horizontal_padding: u16,
    pub action_vertical_padding: u16,
}

impl EmptyVisualContract {
    pub const fn standard() -> Self { Self { panel_padding: 24, content_gap: 12, icon_size: 112, title_size: 20, description_size: 14, action_horizontal_padding: 16, action_vertical_padding: 9 } }
}
