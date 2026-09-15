#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableVisualContract {
    pub header_padding_y: u16,
    pub cell_padding: u16,
    pub text_size: u16,
    pub divider_width: u16,
}

impl TableVisualContract {
    pub const fn standard() -> Self { Self { header_padding_y: 12, cell_padding: 16, text_size: 14, divider_width: 1 } }
}
