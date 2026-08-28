#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewPropOwner {
    Component(BuiltinComponent),
    Item(ViewItemKind),
    CommonStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewItemKind {
    Tab,
    Accordion,
    Carousel,
    Option,
    TableColumn,
    NavMenu,
    SideNav,
    RailNav,
    BottomBar,
    SvgPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewPropDefinition {
    pub owner: ViewPropOwner,
    pub prop: &'static str,
    pub ir_field: IrFieldPath,
    pub kind: PropValueKind,
    pub reactive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropDomain {
    Form,
    Media,
    Chart,
    Navigation,
    Structural,
    Variant,
    Style,
}

impl ViewItemKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tab => "Tab",
            Self::Accordion => "Accordion",
            Self::Carousel => "Carousel",
            Self::Option => "Option",
            Self::TableColumn => "TableColumn",
            Self::NavMenu => "NavMenu",
            Self::SideNav => "SideNav",
            Self::RailNav => "RailNav",
            Self::BottomBar => "BottomBar",
            Self::SvgPath => "SvgPath",
        }
    }
}

pub const VIEW_IR_SCHEMA_VERSION: u32 = 1;

include!(concat!(env!("OUT_DIR"), "/view_prop_inventory.rs"));

pub use GENERATED_VIEW_PROP_INVENTORY as VIEW_PROP_INVENTORY;

pub fn validate_component_props_from_inventory(
    component: BuiltinComponent,
    props: &[ComponentProp],
    domain: PropDomain,
) -> ComponentResult<()> {
    for prop in props {
        let matches_domain = match domain {
            PropDomain::Style => common_style_prop_declared(&prop.name),
            PropDomain::Navigation => matches!(
                component,
                BuiltinComponent::AppBar
                    | BuiltinComponent::Footer
                    | BuiltinComponent::BottomBar
                    | BuiltinComponent::NavMenu
                    | BuiltinComponent::SideNav
                    | BuiltinComponent::RailNav
                    | BuiltinComponent::Sidebar
                    | BuiltinComponent::Scaffold
                    | BuiltinComponent::Drawer
            ),
            PropDomain::Structural => matches!(
                component,
                BuiltinComponent::Tabs
                    | BuiltinComponent::Tab
                    | BuiltinComponent::Stepper
                    | BuiltinComponent::Step
                    | BuiltinComponent::Accordion
                    | BuiltinComponent::Carousel
                    | BuiltinComponent::Option
                    | BuiltinComponent::Table
                    | BuiltinComponent::Path
            ),
            PropDomain::Chart => matches!(
                component,
                BuiltinComponent::Candlestick
                    | BuiltinComponent::ArcChart
                    | BuiltinComponent::AreaChart
                    | BuiltinComponent::BarChart
                    | BuiltinComponent::LineChart
                    | BuiltinComponent::PieChart
            ),
            PropDomain::Media => matches!(
                component,
                BuiltinComponent::Audio
                    | BuiltinComponent::Video
                    | BuiltinComponent::Iframe
                    | BuiltinComponent::Device
                    | BuiltinComponent::Image
                    | BuiltinComponent::Camera
                    | BuiltinComponent::Microphone
            ),
            PropDomain::Variant => matches!(
                prop.name.as_str(),
                "variant" | "scheme" | "size" | "rounded" | "loading" | "disabled"
            ),
            PropDomain::Form => matches!(
                component,
                BuiltinComponent::Input
                    | BuiltinComponent::Select
                    | BuiltinComponent::Checkbox
                    | BuiltinComponent::Toggle
                    | BuiltinComponent::RadioGroup
                    | BuiltinComponent::Slider
                    | BuiltinComponent::Date
                    | BuiltinComponent::DateRange
                    | BuiltinComponent::Password
                    | BuiltinComponent::Phone
                    | BuiltinComponent::Pin
                    | BuiltinComponent::Textarea
                    | BuiltinComponent::Color
                    | BuiltinComponent::Dropzone
            ),
        };
        if matches_domain && !view_prop_declared(component, &prop.name) {
            if matches!(
                domain,
                PropDomain::Chart
                    | PropDomain::Media
                    | PropDomain::Navigation
                    | PropDomain::Structural
                    | PropDomain::Form
            ) {
                return Err(ComponentError::unknown_prop(component, &prop.name));
            }
        }
    }
    Ok(())
}

pub fn common_style_prop_name(prop: &str) -> bool {
    matches!(
        prop,
        "id"
            | "show"
            | "font"
            | "bg"
            | "color"
            | "cover"
            | "overlay"
            | "background"
            | "centerX"
            | "centerY"
            | "flex"
            | "animation"
            | "rotate"
            | "scale"
            | "translateX"
            | "translateY"
            | "transition"
            | "gesture"
            | "colSpan"
            | "rowSpan"
            | "p"
            | "px"
            | "py"
            | "pl"
            | "pr"
            | "pt"
            | "pb"
            | "w"
            | "h"
            | "minW"
            | "minH"
            | "maxW"
            | "maxH"
            | "rounded"
            | "border"
            | "borderColor"
            | "shadow"
            | "shadowColor"
    )
}

pub fn common_style_prop_declared(prop: &str) -> bool {
    common_style_prop_name(prop)
        && VIEW_PROP_INVENTORY.iter().any(|definition| {
            definition.prop == prop && matches!(definition.owner, ViewPropOwner::CommonStyle)
        })
}

pub fn view_prop_declared(component: BuiltinComponent, prop: &str) -> bool {
    let item = match component {
        BuiltinComponent::Tab | BuiltinComponent::Step => Some(ViewItemKind::Tab),
        BuiltinComponent::Option => Some(ViewItemKind::Option),
        BuiltinComponent::Accordion => Some(ViewItemKind::Accordion),
        BuiltinComponent::Carousel => Some(ViewItemKind::Carousel),
        BuiltinComponent::Table => Some(ViewItemKind::TableColumn),
        BuiltinComponent::NavMenu => Some(ViewItemKind::NavMenu),
        BuiltinComponent::SideNav => Some(ViewItemKind::SideNav),
        BuiltinComponent::RailNav => Some(ViewItemKind::RailNav),
        BuiltinComponent::BottomBar => Some(ViewItemKind::BottomBar),
        BuiltinComponent::Path => Some(ViewItemKind::SvgPath),
        _ => None,
    };
    VIEW_PROP_INVENTORY.iter().any(|definition| {
        definition.prop == prop
            && match definition.owner {
                ViewPropOwner::CommonStyle => true,
                ViewPropOwner::Component(owner) => owner == component,
                ViewPropOwner::Item(owner) => item == Some(owner),
            }
    })
}
