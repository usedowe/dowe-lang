#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvatarVisualContract {
    pub group_overlap: u16,
    pub indicator_border: u16,
    pub counter_border: u16,
}

impl AvatarVisualContract {
    pub const fn standard() -> Self { Self { group_overlap: 12, indicator_border: 3, counter_border: 3 } }
}
