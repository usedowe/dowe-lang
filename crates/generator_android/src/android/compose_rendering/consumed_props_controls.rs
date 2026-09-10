fn register_compose_control_consumed_props(node: &ViewNode, context: &ComposeReactiveContext) {
    match node {
        ViewNode::Image { props: _ } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Image,
                "src",
                "ImageProps.src",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Image,
                "alt",
                "ImageProps.alt",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Image,
                "objectFit",
                "ImageProps.object_fit",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Image,
                "loading",
                "ImageProps.loading",
            );
        }
        ViewNode::Text { props, .. } | ViewNode::Title { props, .. } => {
            if props.size.is_some() || props.size_binding.is_some() {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::Text,
                    "size",
                    "TextProps.size",
                );
            }
            if props.weight.is_some() || props.weight_binding.is_some() {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::Text,
                    "weight",
                    "TextProps.weight",
                );
            }
            if props.letter_spacing.is_some() || props.letter_spacing_binding.is_some() {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::Text,
                    "spacing",
                    "TextProps.letter_spacing",
                );
            }
        }
        ViewNode::Input { props } | ViewNode::Select { props, .. } => {
            let component = if matches!(node, ViewNode::Input { .. }) {
                dowe_components::BuiltinComponent::Input
            } else {
                dowe_components::BuiltinComponent::Select
            };
            if props.color.is_some() || props.color_binding.is_some() {
                context.register_consumed_prop(component, "scheme", "VariantProps.color");
            }
            if props.variant.is_some() || props.variant_binding.is_some() {
                context.register_consumed_prop(component, "variant", "VariantProps.variant");
            }
            if props.size.is_some() || props.size_binding.is_some() {
                context.register_consumed_prop(component, "size", "VariantProps.size");
            }
            if props.element.bind.is_some() {
                context.register_consumed_prop(component, "bind", "ElementProps.bind");
            }
            if props.label.is_some() {
                context.register_consumed_prop(component, "label", "VariantProps.label");
            }
            if props.placeholder.is_some() {
                context.register_consumed_prop(
                    component,
                    "placeholder",
                    "VariantProps.placeholder",
                );
            }
            if props.i18n.is_some() {
                context.register_consumed_prop(component, "i18n", "VariantProps.i18n");
            }
        }
        ViewNode::Button { props, .. } => {
            if props.color.is_some() || props.color_binding.is_some() {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::Button,
                    "scheme",
                    "VariantProps.color",
                );
            }
            if props.variant.is_some() || props.variant_binding.is_some() {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::Button,
                    "variant",
                    "VariantProps.variant",
                );
            }
            if props.size.is_some() || props.size_binding.is_some() {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::Button,
                    "size",
                    "VariantProps.size",
                );
            }
            if props.style.rounded.is_some() || props.style.rounded_binding.is_some() {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::Button,
                    "rounded",
                    "VariantProps.style.rounded",
                );
            }
            if props.reactive.loading.is_some() {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::Button,
                    "loading",
                    "VariantProps.reactive.loading",
                );
            }
            if props.reactive.disabled.is_some() {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::Button,
                    "disabled",
                    "VariantProps.reactive.disabled",
                );
            }
        }
        _ => {}
    }
}
