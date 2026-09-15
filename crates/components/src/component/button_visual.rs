/// Shared metrics for text Buttons and their icon-bearing variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonVisualContract {
    pub min_height: u16,
    pub horizontal_padding: u16,
    pub vertical_padding: u16,
    pub text_size: u16,
}

impl ButtonVisualContract {
    pub const fn for_size(size: ButtonSize) -> Self {
        let (min_height, horizontal_padding, vertical_padding, text_size) = match size {
            ButtonSize::Xs => (28, 10, 6, 12),
            ButtonSize::Sm => (36, 12, 8, 14),
            ButtonSize::Md => (44, 16, 10, 14),
            ButtonSize::Lg => (48, 20, 12, 16),
            ButtonSize::Xl => (56, 24, 14, 18),
        };
        Self { min_height, horizontal_padding, vertical_padding, text_size }
    }
}
