#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideNavProps {
    pub style: VariantProps,
    pub size: SideNavSize,
    pub wide: bool,
    pub reactive_wide: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RailNavProps {
    pub style: VariantProps,
    pub size: SideNavSize,
    pub show_labels: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarProps {
    pub style: VariantProps,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavMenuProps {
    pub style: VariantProps,
    pub size: SideNavSize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScaffoldProps {
    pub style: StyleProps,
    pub boxed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawerProps {
    pub style: VariantProps,
    pub open: String,
    pub position: DrawerPosition,
    pub disable_overlay_close: bool,
    pub hide_close_button: bool,
}
