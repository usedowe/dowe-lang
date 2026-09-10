fn compose_side_nav_metrics(size: SideNavSize) -> (u16, u16, u16, u16, u16) {
    match size {
        SideNavSize::Sm => (8, 6, 8, 12, 10),
        SideNavSize::Md => (12, 8, 10, 14, 12),
        SideNavSize::Lg => (16, 12, 12, 16, 14),
    }
}
