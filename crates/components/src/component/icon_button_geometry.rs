#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconButtonGeometryContract {
    pub control_size: u16,
    pub icon_size: u16,
}

impl IconButtonGeometryContract {
    pub const fn for_size(size: ButtonSize) -> Self {
        let (control_size, icon_size) = match size {
            ButtonSize::Xs => (24, 16),
            ButtonSize::Sm => (32, 20),
            ButtonSize::Md => (40, 24),
            ButtonSize::Lg => (48, 32),
            ButtonSize::Xl => (56, 40),
        };
        Self {
            control_size,
            icon_size,
        }
    }
}
