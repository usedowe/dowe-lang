pub(super) fn component_value_completions(
    component: BuiltinComponent,
    prop: &str,
) -> Option<Vec<LanguageCompletion>> {
    if !props_for_component(component.as_str()).contains(&prop) {
        return None;
    }
    component_visual_value_completions(component, prop)
        .or_else(|| component_common_value_completions(component, prop))
}
