#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SideNavItem {
    Header(SideNavItemProps),
    Item(SideNavItemProps),
    Divider,
    Submenu {
        props: SideNavItemProps,
        open: bool,
        bordered: bool,
        items: Vec<SideNavItemProps>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideNavItemProps {
    pub label: String,
    pub i18n: Option<String>,
    pub description: Option<String>,
    pub description_i18n: Option<String>,
    pub status: Option<String>,
    pub status_i18n: Option<String>,
    pub icon: Option<SideNavIcon>,
    pub on_click: Option<String>,
    pub navigation: Option<NavigationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RailNavItem {
    Item(RailNavItemProps),
    Divider,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RailNavItemProps {
    pub label: String,
    pub i18n: Option<String>,
    pub icon: SideNavIcon,
    pub on_click: Option<String>,
    pub navigation: Option<NavigationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideNavIcon {
    pub props: SvgProps,
    pub paths: Vec<SvgPath>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgViewBox {
    pub min_x: String,
    pub min_y: String,
    pub width: String,
    pub height: String,
}

impl SvgViewBox {
    pub fn as_str(&self) -> String {
        format!(
            "{} {} {} {}",
            self.min_x, self.min_y, self.width, self.height
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgPath {
    pub data: String,
    pub fill: SvgPathFill,
    pub transform: Option<SvgTransform>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgTransform {
    pub a: String,
    pub b: String,
    pub c: String,
    pub d: String,
    pub e: String,
    pub f: String,
}

impl SvgTransform {
    pub fn as_str(&self) -> String {
        format!(
            "matrix({} {} {} {} {} {})",
            self.a, self.b, self.c, self.d, self.e, self.f
        )
    }
}

