#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormControlVisualContract {
    pub min_height: u16,
    pub horizontal_padding: u16,
    pub text_size: u16,
    pub radius: u16,
}

impl FormControlVisualContract {
    pub const fn standard() -> Self { Self { min_height: 40, horizontal_padding: 12, text_size: 14, radius: 8 } }
}
