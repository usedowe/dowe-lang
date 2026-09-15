#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryFamily {
    Container,
    Action,
    Field,
    Selection,
    Navigation,
    Overlay,
    Collection,
    Media,
    Typography,
    Visualization,
    Feedback,
    Contextual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryPlacement {
    Flow,
    Inline,
    Overlay,
    Anchored,
    ParentOwned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeometryRegion {
    pub name: &'static str,
    pub placement: GeometryPlacement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentGeometryContract {
    pub component: BuiltinComponent,
    pub name: &'static str,
    pub dependencies: &'static [&'static str],
    pub family: GeometryFamily,
    pub regions: &'static [GeometryRegion],
}

pub const VIEW_GEOMETRY_SCHEMA_VERSION: u32 = 1;

const fn geometry_region(name: &'static str, placement: GeometryPlacement) -> GeometryRegion {
    GeometryRegion { name, placement }
}

pub fn component_geometry_contract(component: BuiltinComponent) -> ComponentGeometryContract {
    use BuiltinComponent as C;
    use GeometryFamily as F;
    use GeometryPlacement as P;
    let (family, regions): (_, &'static [GeometryRegion]) = match component {
        C::Box | C::Section | C::Flex | C::Grid | C::Card | C::Brand | C::Banner => (
            F::Container,
            const { &[geometry_region("content", P::Flow)] },
        ),
        C::Button | C::IconButton | C::Swap | C::ToggleTheme => (
            F::Action,
            const {
                &[
                    geometry_region("leading", P::Inline),
                    geometry_region("label", P::Inline),
                    geometry_region("trailing", P::Inline),
                ]
            },
        ),
        C::Fab => (
            F::Action,
            const {
                &[
                    geometry_region("trigger", P::Flow),
                    geometry_region("actions", P::Anchored),
                ]
            },
        ),
        C::Input
        | C::Select
        | C::SelectTheme
        | C::ComboBox
        | C::Password
        | C::Phone
        | C::Textarea
        | C::Color
        | C::Date
        | C::DateRange
        | C::Pin => (
            F::Field,
            const {
                &[
                    geometry_region("label", P::Flow),
                    geometry_region("control", P::Flow),
                    geometry_region("message", P::Flow),
                    geometry_region("popup", P::Anchored),
                ]
            },
        ),
        C::Checkbox | C::Toggle | C::Slider | C::RadioGroup | C::RadioCard | C::ToggleGroup => (
            F::Selection,
            const {
                &[
                    geometry_region("control", P::Inline),
                    geometry_region("label", P::Inline),
                ]
            },
        ),
        C::AppBar | C::Footer => (
            F::Navigation,
            const {
                &[
                    geometry_region("top", P::Flow),
                    geometry_region("start", P::Inline),
                    geometry_region("center", P::Inline),
                    geometry_region("end", P::Inline),
                    geometry_region("bottom", P::Flow),
                ]
            },
        ),
        C::Scaffold => (
            F::Navigation,
            const {
                &[
                    geometry_region("appBar", P::Flow),
                    geometry_region("start", P::Inline),
                    geometry_region("main", P::Inline),
                    geometry_region("end", P::Inline),
                    geometry_region("bottomBar", P::Flow),
                    geometry_region("overlays", P::Overlay),
                ]
            },
        ),
        C::Sidebar => (
            F::Navigation,
            const {
                &[
                    geometry_region("header", P::Flow),
                    geometry_region("body", P::Flow),
                    geometry_region("footer", P::Flow),
                ]
            },
        ),
        C::NavMenu | C::SideNav | C::RailNav | C::BottomBar => (
            F::Navigation,
            const {
                &[
                    geometry_region("items", P::Flow),
                    geometry_region("popup", P::Anchored),
                ]
            },
        ),
        C::Drawer | C::Modal | C::AlertDialog | C::Command | C::Splash => (
            F::Overlay,
            const {
                &[
                    geometry_region("backdrop", P::Overlay),
                    geometry_region("panel", P::Overlay),
                ]
            },
        ),
        C::Tooltip | C::Dropdown => (
            F::Overlay,
            const {
                &[
                    geometry_region("trigger", P::Flow),
                    geometry_region("popup", P::Anchored),
                ]
            },
        ),
        C::Toast => (
            F::Overlay,
            const { &[geometry_region("notification", P::Anchored)] },
        ),
        C::Carousel => (
            F::Collection,
            const {
                &[
                    geometry_region("title", P::Flow),
                    geometry_region("stage", P::Flow),
                    geometry_region("slides", P::Flow),
                    geometry_region("navigation", P::Overlay),
                    geometry_region("controls", P::Flow),
                    geometry_region("indicators", P::Inline),
                    geometry_region("counter", P::Inline),
                ]
            },
        ),
        C::Tabs | C::Stepper => (
            F::Collection,
            const {
                &[
                    geometry_region("navigation", P::Flow),
                    geometry_region("panel", P::Flow),
                ]
            },
        ),
        C::Accordion | C::Collapsible | C::Tree => (
            F::Collection,
            const {
                &[
                    geometry_region("trigger", P::Flow),
                    geometry_region("content", P::Flow),
                ]
            },
        ),
        C::Table | C::CsvField | C::Record => (
            F::Collection,
            const {
                &[
                    geometry_region("header", P::Flow),
                    geometry_region("rows", P::Flow),
                    geometry_region("controls", P::Flow),
                ]
            },
        ),
        C::DragDrop | C::Dropzone | C::Editor | C::ImageCropper | C::ChatBox => (
            F::Collection,
            const {
                &[
                    geometry_region("toolbar", P::Flow),
                    geometry_region("content", P::Flow),
                    geometry_region("controls", P::Flow),
                ]
            },
        ),
        C::Marquee | C::AvatarGroup => (
            F::Collection,
            const { &[geometry_region("items", P::Flow)] },
        ),
        C::Image | C::Avatar | C::Icon | C::Svg | C::Iframe => {
            (F::Media, const { &[geometry_region("content", P::Flow)] })
        }
        C::Video | C::Audio | C::Camera | C::Microphone | C::Device => (
            F::Media,
            const {
                &[
                    geometry_region("content", P::Flow),
                    geometry_region("controls", P::Flow),
                ]
            },
        ),
        C::Title | C::Text | C::TypeWriter | C::RichText | C::Code => (
            F::Typography,
            const { &[geometry_region("content", P::Flow)] },
        ),
        C::Canvas
        | C::Game
        | C::Candlestick
        | C::Diagram
        | C::ArcChart
        | C::AreaChart
        | C::BarChart
        | C::LineChart
        | C::PieChart
        | C::Map => (
            F::Visualization,
            const {
                &[
                    geometry_region("viewport", P::Flow),
                    geometry_region("legend", P::Flow),
                    geometry_region("controls", P::Overlay),
                ]
            },
        ),
        C::Divider | C::Skeleton | C::Badge | C::Chip | C::Alert | C::Empty | C::Countdown => (
            F::Feedback,
            const { &[geometry_region("content", P::Flow)] },
        ),
        C::Option
        | C::ComboOption
        | C::CsvColumn
        | C::DragGroup
        | C::DragItem
        | C::FabAction
        | C::Tab
        | C::Step
        | C::Path
        | C::Draw => (
            F::Contextual,
            const { &[geometry_region("content", P::ParentOwned)] },
        ),
    };
    let dependencies: &'static [&'static str] = match component {
        C::IconButton => &["Icon"],
        C::Carousel => &["IconButton", "Pagination"],
        _ => &[],
    };
    ComponentGeometryContract {
        component,
        name: component.as_str(),
        dependencies,
        family,
        regions,
    }
}

pub fn view_geometry_contract(name: &str) -> Option<ComponentGeometryContract> {
    if name == "Pagination" {
        return Some(ComponentGeometryContract {
            component: BuiltinComponent::ToggleGroup,
            name: "Pagination",
            dependencies: &["IconButton"],
            family: GeometryFamily::Collection,
            regions: const {
                &[
                    geometry_region("previous", GeometryPlacement::Inline),
                    geometry_region("indicators", GeometryPlacement::Inline),
                    geometry_region("counter", GeometryPlacement::Inline),
                    geometry_region("next", GeometryPlacement::Inline),
                ]
            },
        });
    }
    BuiltinComponent::from_name(name).map(component_geometry_contract)
}
