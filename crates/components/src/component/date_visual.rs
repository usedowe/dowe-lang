#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatePickerVisualContract {
    pub cell_size: u16,
    pub grid_gap: u16,
    pub nav_size: u16,
}

impl DatePickerVisualContract {
    pub const fn standard() -> Self { Self { cell_size: 32, grid_gap: 4, nav_size: 32 } }
}
