use super::show_condition_entries;
use super::{prop_error, quoted_static_string_error, static_value_has_bareword};
use crate::error::DoweResult;
use crate::parser::source_ast::{SourceProp, SourceValue};
use crate::parser::source_views::{is_dynamic_reference, is_known_component_prop};
use dowe_components::{BuiltinComponent, ComponentError};

pub(super) fn validate_component_prop_source(
    component: BuiltinComponent,
    prop: &SourceProp,
) -> DoweResult<()> {
    if matches!(
        component,
        BuiltinComponent::Drawer
            | BuiltinComponent::Modal
            | BuiltinComponent::AlertDialog
            | BuiltinComponent::Command
    ) && prop.name == "bind"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("bind", "signal bool path").to_string(),
        ));
    }
    if component == BuiltinComponent::Toast
        && prop.name == "source"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("source", "signal object path").to_string(),
        ));
    }
    if component == BuiltinComponent::Draw
        && matches!(prop.name.as_str(), "bind" | "selected")
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop(
                &prop.name,
                if prop.name == "bind" {
                    "signal array path"
                } else {
                    "signal string path"
                },
            )
            .to_string(),
        ));
    }
    if component == BuiltinComponent::AvatarGroup
        && prop.name == "items"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("items", "signal array path").to_string(),
        ));
    }
    if component == BuiltinComponent::Svg
        && prop.name == "data"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("data", "signal or each-item data path").to_string(),
        ));
    }
    if component == BuiltinComponent::ChatBox
        && prop.name == "messages"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("messages", "signal array path").to_string(),
        ));
    }
    if component == BuiltinComponent::ChatBox
        && matches!(
            prop.name.as_str(),
            "loading" | "sending" | "streaming" | "hasMore"
        )
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop(&prop.name, "signal bool path").to_string(),
        ));
    }
    if component == BuiltinComponent::Button
        && matches!(prop.name.as_str(), "loading" | "disabled")
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop(&prop.name, "signal bool path").to_string(),
        ));
    }
    if component == BuiltinComponent::DateRange
        && matches!(prop.name.as_str(), "start" | "end")
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop(&prop.name, "signal string path").to_string(),
        ));
    }
    if component == BuiltinComponent::ToggleGroup
        && prop.name == "value"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("value", "signal string path").to_string(),
        ));
    }
    if component == BuiltinComponent::Tree
        && matches!(prop.name.as_str(), "data" | "bind")
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop(&prop.name, "signal or constant path").to_string(),
        ));
    }
    if !is_known_component_prop(component, &prop.name)
        || allows_bare_component_reference(component, prop)
        || matches!(&prop.value, SourceValue::Bareword(_))
            && dowe_components::accepts_reactive_prop(component, &prop.name)
    {
        return Ok(());
    }
    if static_value_has_bareword(&prop.value) {
        Err(quoted_static_string_error(prop))
    } else {
        Ok(())
    }
}

pub(super) fn allows_bare_component_reference(
    component: BuiltinComponent,
    prop: &SourceProp,
) -> bool {
    match (component, prop.name.as_str(), &prop.value) {
        (_, "show", SourceValue::Bareword(_))
            if !matches!(component, BuiltinComponent::Option | BuiltinComponent::Path) =>
        {
            true
        }
        (_, "show", SourceValue::Object(entries)) if show_condition_entries(entries) => true,
        (
            BuiltinComponent::Button | BuiltinComponent::IconButton | BuiltinComponent::Swap,
            "variant" | "scheme" | "size" | "rounded",
            SourceValue::Bareword(_),
        ) => true,
        (BuiltinComponent::Card, "animation", SourceValue::Bareword(_)) => true,
        (BuiltinComponent::Avatar, "icon", SourceValue::Bareword(_))
        | (BuiltinComponent::Icon, "fill" | "stroke", SourceValue::Bareword(_))
        | (BuiltinComponent::Icon, "name", SourceValue::Bareword(_)) => true,
        (
            BuiltinComponent::Button | BuiltinComponent::Swap,
            "loading" | "disabled",
            SourceValue::Bareword(_),
        ) => true,
        (
            BuiltinComponent::SideNav,
            "variant" | "scheme" | "size" | "wide",
            SourceValue::Bareword(_),
        ) => true,
        (BuiltinComponent::Image | BuiltinComponent::Iframe, "src", SourceValue::Bareword(_)) => {
            true
        }
        (BuiltinComponent::Button, "iconStart" | "iconEnd", SourceValue::Object(_)) => true,
        (
            BuiltinComponent::Input
            | BuiltinComponent::Select
            | BuiltinComponent::ComboBox
            | BuiltinComponent::CsvField
            | BuiltinComponent::DragDrop
            | BuiltinComponent::Editor
            | BuiltinComponent::ImageCropper
            | BuiltinComponent::Slider
            | BuiltinComponent::Checkbox
            | BuiltinComponent::Color
            | BuiltinComponent::Date
            | BuiltinComponent::RadioGroup
            | BuiltinComponent::RadioCard
            | BuiltinComponent::Toggle
            | BuiltinComponent::Password
            | BuiltinComponent::Phone
            | BuiltinComponent::Pin
            | BuiltinComponent::Textarea
            | BuiltinComponent::Swap,
            "bind",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Input
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
            | BuiltinComponent::Checkbox
            | BuiltinComponent::Color
            | BuiltinComponent::Date
            | BuiltinComponent::DateRange
            | BuiltinComponent::RadioGroup
            | BuiltinComponent::RadioCard
            | BuiltinComponent::Toggle
            | BuiltinComponent::Slider
            | BuiltinComponent::Dropzone,
            "onChange" | "onInput",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::DateRange, "start" | "end", SourceValue::Bareword(_))
        | (BuiltinComponent::ToggleGroup, "bind", SourceValue::Bareword(_))
        | (BuiltinComponent::Candlestick, "data", SourceValue::Bareword(_))
        | (
            BuiltinComponent::Diagram,
            "nodes" | "edges" | "onNodeClick" | "onNodeDrag" | "onConnect",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Canvas | BuiltinComponent::Draw,
            "scene" | "bind" | "selected" | "onPointer" | "onKey" | "onMotion" | "drawMode"
            | "onLayerAdd" | "onLayerChange" | "onLayerRemove" | "onLayerSelect",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart,
            "data" | "series",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::Table, "data", SourceValue::Bareword(_))
        | (BuiltinComponent::Svg, "data", SourceValue::Bareword(_))
        | (BuiltinComponent::AvatarGroup, "items", SourceValue::Bareword(_))
        | (
            BuiltinComponent::ChatBox,
            "messages" | "loading" | "sending" | "streaming" | "hasMore",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Button
            | BuiltinComponent::IconButton
            | BuiltinComponent::Avatar
            | BuiltinComponent::Empty
            | BuiltinComponent::Box
            | BuiltinComponent::Card
            | BuiltinComponent::Chip,
            "onClick",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::ChatBox,
            "onSend" | "onLoadMore" | "onStop" | "onVoiceNote" | "onFileAttach" | "onCameraCapture",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Camera,
            "onStart" | "onCapture" | "onError",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Microphone,
            "onStart" | "onStop" | "onError",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Record,
            "onStart" | "onPause" | "onResume" | "onStop" | "onDiscard" | "onConfirm",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::ToggleGroup, "onChange", SourceValue::Bareword(_))
        | (BuiltinComponent::Tree, "data" | "bind" | "onSelect", SourceValue::Bareword(_))
        | (BuiltinComponent::Editor, "onSave", SourceValue::Bareword(_))
        | (BuiltinComponent::Countdown, "onComplete", SourceValue::Bareword(_))
        | (
            BuiltinComponent::Map,
            "onLocation" | "onLocationError" | "onRoute",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Fab | BuiltinComponent::FabAction,
            "onClick",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::Alert, "visible" | "onClose", SourceValue::Bareword(_))
        | (
            BuiltinComponent::Chip | BuiltinComponent::Modal | BuiltinComponent::AlertDialog,
            "onClose" | "onConfirm" | "onCancel",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Drawer
            | BuiltinComponent::Modal
            | BuiltinComponent::AlertDialog
            | BuiltinComponent::Command,
            "bind",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::Toast, "source", SourceValue::Bareword(_)) => true,
        (BuiltinComponent::Alert, "message", SourceValue::Bareword(value)) => {
            is_dynamic_reference(value)
        }
        _ => false,
    }
}
