fn register_swift_consumed_props(node: &ViewNode, context: &SwiftReactiveContext) {
    if let Some(component) = swift_form_component(node) {
        let element = dowe_components::node_element_props(node);
        if element.and_then(|props| props.bind.as_deref()).is_some() {
            context.register_consumed_prop(component, "bind", "ElementProps.bind");
        }
        if let Some(props) = swift_form_variant_props(node) {
            if props.variant.is_some()
                || props.variant_binding.is_some()
                || props.reactive.variant.is_some()
            {
                context.register_consumed_prop(component, "variant", "VariantProps.variant");
            }
            if props.color.is_some() || props.color_binding.is_some() || props.reactive.scheme.is_some() {
                context.register_consumed_prop(component, "scheme", "VariantProps.color");
            }
            if props.size.is_some() || props.size_binding.is_some() || props.reactive.size.is_some() {
                context.register_consumed_prop(component, "size", "VariantProps.size");
            }
            if props.style.rounded.is_some()
                || props.style.rounded_binding.is_some()
                || props.reactive.rounded.is_some()
            {
                context.register_consumed_prop(component, "rounded", "VariantProps.style.rounded");
            }
        }
        if element.and_then(|props| props.on_change.as_ref()).is_some() {
            context.register_consumed_prop(component, "onChange", "ElementProps.on_change");
        }
        if element.and_then(|props| props.on_input.as_ref()).is_some() {
            context.register_consumed_prop(component, "onInput", "ElementProps.on_input");
        }
    }
    register_swift_structure_consumed_props(node, context);
    register_swift_media_consumed_props(node, context);
    register_swift_display_consumed_props(node, context);
    register_swift_control_consumed_props(node, context);
}
