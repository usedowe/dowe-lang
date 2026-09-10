fn component_visual_value_completions(
    component: BuiltinComponent,
    prop: &str,
) -> Option<Vec<LanguageCompletion>> {
    match (component, prop) {
        (BuiltinComponent::Draw, "drawMode") => {
            Some(quoted_values(["pen", "rect", "circle", "select", "erase"]))
        }
        (BuiltinComponent::Canvas, "drawMode") => Some(quoted_values(["pen", "rect", "circle"])),
        (
            BuiltinComponent::Box
            | BuiltinComponent::Section
            | BuiltinComponent::Flex
            | BuiltinComponent::Grid
            | BuiltinComponent::Card,
            "flex",
        ) => {
            let mut values = quoted_values(
                FlexItem::all()
                    .iter()
                    .filter(|value| **value != FlexItem::Fill)
                    .map(|value| value.as_str()),
            );
            values.push(completion(
                "1",
                LanguageCompletionKind::Value,
                "static flex fill value",
            ));
            Some(values)
        }
        (BuiltinComponent::Icon, "name") => Some(quoted_values(dowe_components::all_icon_names())),
        (BuiltinComponent::IconButton, "icon")
        | (BuiltinComponent::Swap, "iconOn" | "iconOff")
        | (BuiltinComponent::SideNav | BuiltinComponent::RailNav, "icon")
        | (BuiltinComponent::Button | BuiltinComponent::Input, "iconStart" | "iconEnd")
        | (BuiltinComponent::Chip, "startIcon" | "endIcon") => {
            Some(quoted_values(dowe_components::solar_icon_names()))
        }
        (BuiltinComponent::Icon, "fill" | "stroke") => Some(quoted_values(
            ["currentColor"]
                .into_iter()
                .chain(ColorToken::all().iter().map(|value| value.as_str())),
        )),
        (
            BuiltinComponent::Card
            | BuiltinComponent::Code
            | BuiltinComponent::Video
            | BuiltinComponent::Candlestick
            | BuiltinComponent::Diagram
            | BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart
            | BuiltinComponent::Table
            | BuiltinComponent::Tree
            | BuiltinComponent::AppBar
            | BuiltinComponent::Footer
            | BuiltinComponent::BottomBar
            | BuiltinComponent::Sidebar
            | BuiltinComponent::Drawer
            | BuiltinComponent::Input
            | BuiltinComponent::Select
            | BuiltinComponent::ComboBox
            | BuiltinComponent::CsvField
            | BuiltinComponent::DragDrop
            | BuiltinComponent::Editor
            | BuiltinComponent::ImageCropper
            | BuiltinComponent::Password
            | BuiltinComponent::Phone
            | BuiltinComponent::Pin
            | BuiltinComponent::Textarea
            | BuiltinComponent::Button
            | BuiltinComponent::IconButton
            | BuiltinComponent::Alert
            | BuiltinComponent::ToggleTheme
            | BuiltinComponent::SelectTheme
            | BuiltinComponent::Dropzone
            | BuiltinComponent::ChatBox
            | BuiltinComponent::Empty
            | BuiltinComponent::ToggleGroup
            | BuiltinComponent::Collapsible
            | BuiltinComponent::Countdown
            | BuiltinComponent::Map
            | BuiltinComponent::Image
            | BuiltinComponent::Accordion
            | BuiltinComponent::Toast
            | BuiltinComponent::Checkbox
            | BuiltinComponent::Color
            | BuiltinComponent::Date
            | BuiltinComponent::DateRange
            | BuiltinComponent::Toggle,
            "variant",
        ) => Some(quoted_values(
            ComponentVariant::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Chip
            | BuiltinComponent::Modal
            | BuiltinComponent::AlertDialog
            | BuiltinComponent::Command,
            "variant",
        ) => Some(quoted_values(
            ComponentVariant::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Avatar
            | BuiltinComponent::AvatarGroup
            | BuiltinComponent::Badge
            | BuiltinComponent::Tooltip
            | BuiltinComponent::Fab
            | BuiltinComponent::Record,
            "variant",
        ) => Some(solid_values()),
        (BuiltinComponent::Audio, "variant") => Some(solid_values()),
        (BuiltinComponent::Carousel, "variant") => Some(quoted_values(
            CarouselVariant::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Tabs, "variant") => Some(quoted_values(
            TabsVariant::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Card
            | BuiltinComponent::Code
            | BuiltinComponent::Video
            | BuiltinComponent::Candlestick
            | BuiltinComponent::Diagram
            | BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart
            | BuiltinComponent::Table
            | BuiltinComponent::Divider
            | BuiltinComponent::AppBar
            | BuiltinComponent::Footer
            | BuiltinComponent::BottomBar
            | BuiltinComponent::Sidebar
            | BuiltinComponent::Tabs
            | BuiltinComponent::Stepper
            | BuiltinComponent::Drawer
            | BuiltinComponent::Avatar
            | BuiltinComponent::Badge
            | BuiltinComponent::Chip
            | BuiltinComponent::Modal
            | BuiltinComponent::AlertDialog
            | BuiltinComponent::Tooltip
            | BuiltinComponent::Toast
            | BuiltinComponent::Dropdown
            | BuiltinComponent::Command
            | BuiltinComponent::Dropzone
            | BuiltinComponent::ComboBox
            | BuiltinComponent::CsvField
            | BuiltinComponent::DragDrop
            | BuiltinComponent::Editor
            | BuiltinComponent::ImageCropper
            | BuiltinComponent::Password
            | BuiltinComponent::Phone
            | BuiltinComponent::Pin
            | BuiltinComponent::Textarea
            | BuiltinComponent::AvatarGroup
            | BuiltinComponent::ChatBox
            | BuiltinComponent::Empty
            | BuiltinComponent::Collapsible
            | BuiltinComponent::Countdown
            | BuiltinComponent::RadioGroup
            | BuiltinComponent::RadioCard
            | BuiltinComponent::SelectTheme
            | BuiltinComponent::Tree,
            "scheme",
        ) => Some(quoted_values(
            ColorFamily::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Input
            | BuiltinComponent::Select
            | BuiltinComponent::Button
            | BuiltinComponent::Alert
            | BuiltinComponent::ToggleTheme
            | BuiltinComponent::Fab
            | BuiltinComponent::FabAction
            | BuiltinComponent::Slider
            | BuiltinComponent::SideNav
            | BuiltinComponent::RailNav,
            "scheme",
        ) => Some(quoted_values(
            ColorFamily::all()
                .iter()
                .filter(|value| {
                    **value != ColorFamily::Background && **value != ColorFamily::Surface
                })
                .map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Record
            | BuiltinComponent::ToggleGroup
            | BuiltinComponent::Map
            | BuiltinComponent::Audio
            | BuiltinComponent::Image
            | BuiltinComponent::Accordion
            | BuiltinComponent::Carousel
            | BuiltinComponent::Checkbox
            | BuiltinComponent::Color
            | BuiltinComponent::Date
            | BuiltinComponent::DateRange
            | BuiltinComponent::Toggle,
            "scheme",
        ) => Some(quoted_values(
            ColorFamily::all()
                .iter()
                .filter(|value| {
                    **value != ColorFamily::Background && **value != ColorFamily::Surface
                })
                .map(|value| value.as_str()),
        )),
        (BuiltinComponent::Avatar, "size") => Some(quoted_values(
            AvatarSize::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Button
            | BuiltinComponent::AvatarGroup
            | BuiltinComponent::Chip
            | BuiltinComponent::ToggleTheme
            | BuiltinComponent::SelectTheme
            | BuiltinComponent::Fab
            | BuiltinComponent::ToggleGroup,
            "size",
        ) => Some(quoted_values(
            ButtonSize::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Input
            | BuiltinComponent::Select
            | BuiltinComponent::Slider
            | BuiltinComponent::RadioGroup
            | BuiltinComponent::RadioCard
            | BuiltinComponent::Dropzone
            | BuiltinComponent::ComboBox
            | BuiltinComponent::DragDrop
            | BuiltinComponent::Editor
            | BuiltinComponent::Password
            | BuiltinComponent::Phone
            | BuiltinComponent::Pin
            | BuiltinComponent::Textarea,
            "size",
        ) => Some(control_size_values()),
        (
            BuiltinComponent::Carousel
            | BuiltinComponent::Color
            | BuiltinComponent::Date
            | BuiltinComponent::DateRange,
            "size",
        ) => Some(control_size_values()),
        (BuiltinComponent::CsvField | BuiltinComponent::ImageCropper, "size") => Some(
            quoted_values(ButtonSize::all().iter().map(|value| value.as_str())),
        ),
        (BuiltinComponent::DragDrop, "direction") => {
            Some(quoted_values(["horizontal", "vertical"]))
        }
        (BuiltinComponent::RadioGroup | BuiltinComponent::RadioCard, "orientation") => {
            Some(quoted_values(["vertical", "horizontal"]))
        }
        (BuiltinComponent::Image, "aspect") => Some(quoted_values(
            ImageAspect::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Image, "objectFit") => Some(quoted_values(
            ImageObjectFit::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Image, "loading") => Some(quoted_values(
            ImageLoading::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Carousel, "orientation") => Some(quoted_values(
            CarouselOrientation::all()
                .iter()
                .map(|value| value.as_str()),
        )),
        (BuiltinComponent::Carousel, "indicatorType") => Some(quoted_values(
            CarouselIndicatorType::all()
                .iter()
                .map(|value| value.as_str()),
        )),
        (BuiltinComponent::ImageCropper, "shape") => Some(quoted_values(["circle", "square"])),
        (BuiltinComponent::Pin, "type") => Some(quoted_values(["text", "password", "number"])),
        (BuiltinComponent::Table, "size") => Some(quoted_values(
            TableSize::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart,
            "size",
        ) => Some(quoted_values(
            ChartSize::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart,
            "palette",
        ) => Some(quoted_values(
            ChartPalette::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart,
            "legendPosition",
        ) => Some(quoted_values(
            ChartLegendPosition::all()
                .iter()
                .map(|value| value.as_str()),
        )),
        (BuiltinComponent::AreaChart | BuiltinComponent::LineChart, "curve") => Some(
            quoted_values(ChartCurve::all().iter().map(|value| value.as_str())),
        ),
        (BuiltinComponent::Code | BuiltinComponent::Editor, "language") => Some(quoted_values(
            CodeLanguage::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Video, "aspect") => Some(quoted_values(
            VideoAspect::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Camera, "facing") => Some(quoted_values(
            CameraFacing::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Iframe, "loading") => Some(quoted_values(["lazy", "eager"])),
        (BuiltinComponent::Iframe, "allow") => Some(quoted_values([
            "fullscreen",
            "autoplay",
            "camera; microphone",
            "clipboard-read; clipboard-write",
        ])),
        (BuiltinComponent::Iframe, "sandbox") => Some(quoted_values([
            "",
            "scripts",
            "scripts same-origin",
            "scripts same-origin forms",
        ])),
        (BuiltinComponent::Device, "device") => {
            Some(quoted_values(["mobile", "tablet", "laptop", "monitor"]))
        }
        (BuiltinComponent::AppBar, "position") => Some(quoted_values(
            BarPosition::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Canvas | BuiltinComponent::Draw, "fit") => {
            Some(quoted_values(["contain", "cover", "stretch"]))
        }
        (BuiltinComponent::Canvas | BuiltinComponent::Draw, "background") => Some(quoted_values(
            ColorToken::all()
                .iter()
                .map(|value| value.as_str())
                .chain(["transparent"]),
        )),
        (BuiltinComponent::Divider, "orientation") => Some(quoted_values(
            DividerOrientation::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::ChatBox, "mode") => Some(quoted_values(
            ChatBoxMode::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Empty, "type") => Some(quoted_values(
            EmptyKind::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Marquee, "speed") => Some(quoted_values(
            MarqueeSpeed::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Marquee, "orientation") => Some(quoted_values(
            MarqueeOrientation::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Countdown, "size") => Some(quoted_values(
            CountdownSize::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::SideNav | BuiltinComponent::RailNav | BuiltinComponent::NavMenu,
            "size",
        ) => Some(quoted_values(
            SideNavSize::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Box, "position") => Some(quoted_values(
            BoxPosition::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Drawer, "position") => Some(quoted_values(
            DrawerPosition::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Tabs, "position") => Some(quoted_values(
            TabsPosition::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Stepper, "orientation") => {
            Some(quoted_values(["horizontal", "vertical"]))
        }
        (BuiltinComponent::Avatar, "status") => Some(quoted_values(
            AvatarStatus::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Badge | BuiltinComponent::Toast | BuiltinComponent::Fab, "position") => {
            Some(quoted_values(
                OverlayCornerPosition::all()
                    .iter()
                    .map(|value| value.as_str()),
            ))
        }
        (BuiltinComponent::Tooltip, "position") => Some(quoted_values(
            OverlayPosition::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Skeleton, "variant") => Some(quoted_values(
            SkeletonVariant::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Skeleton, "animation") => Some(quoted_values(
            SkeletonAnimation::all().iter().map(|value| value.as_str()),
        )),
        _ => None,
    }
}
