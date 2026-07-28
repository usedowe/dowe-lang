fn is_known_component_prop(component: BuiltinComponent, name: &str) -> bool {
    let shared_style = !matches!(
        component,
        BuiltinComponent::Option
            | BuiltinComponent::FabAction
            | BuiltinComponent::ComboOption
            | BuiltinComponent::CsvColumn
            | BuiltinComponent::DragGroup
            | BuiltinComponent::DragItem
            | BuiltinComponent::Svg
            | BuiltinComponent::Path
    ) && matches!(
        name,
        "id" | "show"
            | "font"
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
            | "rounded"
            | "border"
    );
    shared_style
        || is_known_layout_and_data_prop(component, name)
        || is_known_form_and_action_prop(component, name)
        || is_known_shell_and_feedback_prop(component, name)
        || is_known_display_prop(component, name)
        || is_known_selection_and_text_prop(component, name)
}
