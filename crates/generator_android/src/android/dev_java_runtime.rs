fn dev_java_reactive_runtime() -> String {
    let source = concat!(
        include_str!("dev_java_runtime/svg.java"),
        include_str!("dev_java_runtime/models.java"),
        include_str!("dev_java_runtime/state.java"),
        include_str!("dev_java_runtime/styles-and-data.java"),
        include_str!("dev_java_runtime/stdlib.java"),
        include_str!("dev_java_runtime/actions-and-toast.java"),
        include_str!("dev_java_runtime/startup-and-network.java"),
        include_str!("dev_java_runtime/json.java"),
    );
    source
        .replace("__DOWE_PROP_VARIANTS__", &java_values(dowe_components::BuiltinComponent::Button, "variant"))
        .replace("__DOWE_PROP_SCHEMES__", &java_values(dowe_components::BuiltinComponent::Button, "scheme"))
        .replace("__DOWE_PROP_SIZES__", &java_values(dowe_components::BuiltinComponent::Button, "size"))
        .replace("__DOWE_PROP_ROUNDED__", &java_values(dowe_components::BuiltinComponent::Button, "rounded"))
        .replace("__DOWE_PROP_COLORS__", &java_values(dowe_components::BuiltinComponent::Button, "scheme"))
}

fn java_values(component: dowe_components::BuiltinComponent, name: &str) -> String {
    dowe_components::prop_allowed_values(component, name)
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ")
}
