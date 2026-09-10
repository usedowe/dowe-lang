fn swift_reactive_runtime() -> String {
    [
        include!("runtime_foundations.rs"),
        include!("runtime_svg_import.rs"),
        include!("runtime_state_core.rs"),
        include!("runtime_state_queries.rs"),
        include!("runtime_state_actions.rs"),
    ]
    .concat()
    .replace("__DOWE_VARIANTS__", &runtime_swift_values(dowe_components::BuiltinComponent::Button, "variant"))
    .replace("__DOWE_SCHEMES__", &runtime_swift_values(dowe_components::BuiltinComponent::Button, "scheme"))
    .replace("__DOWE_SIZES__", &runtime_swift_values(dowe_components::BuiltinComponent::Button, "size"))
    .replace("__DOWE_ROUNDED__", &runtime_swift_values(dowe_components::BuiltinComponent::Button, "rounded"))
    .replace("__DOWE_COLORS__", &runtime_swift_values(dowe_components::BuiltinComponent::Button, "scheme"))
    .replace("__DOWE_ICON_NAMES__", &runtime_swift_icons())
}

fn runtime_swift_icons() -> String {
    dowe_components::all_icon_names()
        .iter()
        .map(|value| format!("\"{}\"", value.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ")
}

fn runtime_swift_values(component: dowe_components::BuiltinComponent, name: &str) -> String {
    dowe_components::prop_allowed_values(component, name)
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ")
}
