/// Shared Chip metrics consumed by every target renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChipVisualContract {
    pub height: u16,
    pub horizontal_padding: u16,
    pub text_size: u16,
    pub content_gap: u16,
    pub close_alpha: f32,
}

impl ChipVisualContract {
    pub fn for_size(size: ButtonSize) -> Self {
        let (height, horizontal_padding, text_size) = match size {
            ButtonSize::Xs => (20, 12, 12),
            ButtonSize::Sm => (24, 12, 12),
            ButtonSize::Md => (32, 16, 14),
            ButtonSize::Lg => (40, 20, 18),
            ButtonSize::Xl => (48, 24, 24),
        };
        Self { height, horizontal_padding, text_size, content_gap: 8, close_alpha: 0.72 }
    }
}
