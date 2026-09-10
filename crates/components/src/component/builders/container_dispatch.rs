include!("container_layout_and_overlays.rs");
include!("container_media_and_forms.rs");
include!("container_editor_and_charts.rs");
include!("container_terminal_components.rs");

pub fn container_component_node(
    component: BuiltinComponent,
    props: Vec<ComponentProp>,
    children: Vec<ViewNode>,
    allow_children: bool,
) -> ComponentResult<ViewNode> {
    if matches!(
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
    ) {
        validate_component_props_from_inventory(component, &props, PropDomain::Navigation)?;
        validate_component_props_from_inventory(component, &props, PropDomain::Style)?;
    }

    if matches!(
        component,
        BuiltinComponent::Tabs
            | BuiltinComponent::Tab
            | BuiltinComponent::Stepper
            | BuiltinComponent::Step
            | BuiltinComponent::Accordion
            | BuiltinComponent::Tree
            | BuiltinComponent::Carousel
            | BuiltinComponent::Option
            | BuiltinComponent::Table
            | BuiltinComponent::Path
    ) {
        validate_component_props_from_inventory(component, &props, PropDomain::Structural)?;
        validate_component_props_from_inventory(component, &props, PropDomain::Style)?;
    }

    if matches!(
        component,
        BuiltinComponent::Candlestick
            | BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart
    ) {
        validate_component_props_from_inventory(component, &props, PropDomain::Chart)?;
        validate_component_props_from_inventory(component, &props, PropDomain::Style)?;
    }

    if matches!(
        component,
        BuiltinComponent::Audio
            | BuiltinComponent::Video
            | BuiltinComponent::Iframe
            | BuiltinComponent::Device
            | BuiltinComponent::Image
            | BuiltinComponent::Camera
            | BuiltinComponent::Microphone
    ) {
        validate_component_props_from_inventory(component, &props, PropDomain::Media)?;
        validate_component_props_from_inventory(component, &props, PropDomain::Style)?;
    }

    if matches!(
        component,
        BuiltinComponent::Input
            | BuiltinComponent::Select
            | BuiltinComponent::ComboBox
            | BuiltinComponent::CsvField
            | BuiltinComponent::DragDrop
            | BuiltinComponent::Editor
            | BuiltinComponent::ImageCropper
            | BuiltinComponent::Checkbox
            | BuiltinComponent::Toggle
            | BuiltinComponent::RadioGroup
            | BuiltinComponent::RadioCard
            | BuiltinComponent::Date
            | BuiltinComponent::DateRange
            | BuiltinComponent::Password
            | BuiltinComponent::Phone
            | BuiltinComponent::Pin
            | BuiltinComponent::Textarea
            | BuiltinComponent::Color
            | BuiltinComponent::Dropzone
            | BuiltinComponent::Slider
    ) {
        validate_component_props_from_inventory(component, &props, PropDomain::Form)?;
        validate_component_props_from_inventory(component, &props, PropDomain::Variant)?;
        validate_component_props_from_inventory(component, &props, PropDomain::Style)?;
    }

    if let Some(result) =
        container_layout_and_overlays(component, &props, &children, allow_children)?
    {
        return Ok(result);
    }
    if let Some(result) = container_media_and_forms(component, &props, &children, allow_children)? {
        return Ok(result);
    }
    if let Some(result) = container_editor_and_charts(component, &props, &children, allow_children)?
    {
        return Ok(result);
    }
    if let Some(result) =
        container_terminal_components(component, &props, &children, allow_children)?
    {
        return Ok(result);
    }
    unreachable!("all BuiltinComponent variants are covered by container dispatchers");
}
