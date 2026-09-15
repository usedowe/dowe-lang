#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationShellVisualContract {
    pub item_min_height: u16,
    pub item_padding: u16,
    pub rail_width: u16,
    pub item_gap: u16,
}

impl NavigationShellVisualContract {
    pub const fn standard() -> Self { Self { item_min_height: 40, item_padding: 12, rail_width: 72, item_gap: 4 } }
}
