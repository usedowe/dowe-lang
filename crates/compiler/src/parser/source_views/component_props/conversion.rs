use super::prop_value;
use super::{
    parse_conditional_icon, parse_show_condition, show_condition_entries,
    validate_component_prop_source,
};
use crate::error::DoweResult;
use crate::parser::source_ast::{SourceNode, SourceProp, SourceValue};
use dowe_components::{BuiltinComponent, ComponentProp, PropValue};

pub(crate) fn component_props(
    node: &SourceNode,
    component: BuiltinComponent,
) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| component_prop(component, prop))
        .collect()
}

pub(crate) fn component_prop(
    component: BuiltinComponent,
    prop: &SourceProp,
) -> DoweResult<ComponentProp> {
    validate_component_prop_source(component, prop)?;
    let value = match (component, prop.name.as_str(), &prop.value) {
        (
            BuiltinComponent::Button | BuiltinComponent::IconButton | BuiltinComponent::Swap,
            "variant" | "scheme" | "size" | "rounded",
            SourceValue::Bareword(path),
        ) => PropValue::String(format!("@signal:{path}")),
        (
            BuiltinComponent::Button | BuiltinComponent::Swap,
            "loading",
            SourceValue::Bareword(path),
        ) => PropValue::String(format!("@signal:{path}")),
        (
            BuiltinComponent::Button | BuiltinComponent::Swap,
            "disabled",
            SourceValue::Bareword(path),
        ) => PropValue::String(format!("@signal:{path}")),
        (
            BuiltinComponent::SideNav,
            "variant" | "scheme" | "size" | "wide",
            SourceValue::Bareword(path),
        ) => PropValue::String(format!("@signal:{path}")),
        (
            BuiltinComponent::Image | BuiltinComponent::Iframe,
            "src",
            SourceValue::Bareword(path),
        ) => PropValue::String(format!("@signal:{path}")),
        (
            BuiltinComponent::Drawer
            | BuiltinComponent::Modal
            | BuiltinComponent::AlertDialog
            | BuiltinComponent::Command,
            "bind",
            SourceValue::Bareword(path),
        ) => PropValue::String(path.clone()),
        (BuiltinComponent::Avatar, "icon", SourceValue::Bareword(path)) => {
            PropValue::String(format!("@icon-binding:{path}"))
        }
        (BuiltinComponent::Icon, "fill" | "stroke", SourceValue::Bareword(path)) => {
            PropValue::Binding(
                dowe_components::PropBinding::new(
                    path.clone(),
                    if matches!(
                        prop.name.as_str(),
                        "p" | "px"
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
                            | "border"
                    ) {
                        dowe_components::PropValueKind::Number
                    } else {
                        dowe_components::PropValueKind::String
                    },
                )
                .with_fallback(
                    if matches!(
                        prop.name.as_str(),
                        "p" | "px"
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
                            | "border"
                    ) {
                        PropValue::Number("8".to_string())
                    } else {
                        PropValue::String(String::new())
                    },
                ),
            )
        }
        (BuiltinComponent::Icon, "name", SourceValue::Bareword(path)) => {
            PropValue::String(format!("@icon-binding:{path}"))
        }
        (BuiltinComponent::Button, "iconStart" | "iconEnd", SourceValue::Object(entries)) => {
            PropValue::String(parse_conditional_icon(prop, entries)?)
        }
        (BuiltinComponent::Card, "animation", SourceValue::Bareword(path)) => PropValue::Binding(
            dowe_components::PropBinding::new(path.clone(), dowe_components::PropValueKind::String)
                .with_fallback(PropValue::String("none".to_string())),
        ),
        (_, "show", SourceValue::Bareword(path)) => PropValue::String(format!("@signal:{path}")),
        (_, "show", SourceValue::Object(entries)) if show_condition_entries(entries) => {
            PropValue::String(parse_show_condition(prop, entries)?)
        }
        (_, _, SourceValue::Bareword(path))
            if dowe_components::accepts_reactive_prop(component, &prop.name) =>
        {
            let contract = dowe_components::component_prop_contract(component, &prop.name)
                .expect("reactive component prop contract");
            PropValue::Binding(
                dowe_components::PropBinding::new(path.clone(), contract.kind)
                    .with_fallback(dowe_components::default_binding_value(contract)),
            )
        }
        _ => prop_value(prop)?,
    };
    Ok(ComponentProp {
        name: prop.name.clone(),
        value,
    })
}
