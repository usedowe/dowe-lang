fn compose_form_component(node: &ViewNode) -> Option<dowe_components::BuiltinComponent> {
    match node {
        ViewNode::Input { .. } => Some(dowe_components::BuiltinComponent::Input),
        ViewNode::Select { .. } => Some(dowe_components::BuiltinComponent::Select),
        ViewNode::Checkbox { .. } => Some(dowe_components::BuiltinComponent::Checkbox),
        ViewNode::Toggle { .. } => Some(dowe_components::BuiltinComponent::Toggle),
        ViewNode::RadioGroup { props, .. } => Some(
            if matches!(props.presentation, dowe_components::RadioGroupPresentation::Card) {
                dowe_components::BuiltinComponent::RadioCard
            } else {
                dowe_components::BuiltinComponent::RadioGroup
            },
        ),
        ViewNode::Slider { .. } => Some(dowe_components::BuiltinComponent::Slider),
        ViewNode::Date { .. } => Some(dowe_components::BuiltinComponent::Date),
        ViewNode::DateRange { .. } => Some(dowe_components::BuiltinComponent::DateRange),
        ViewNode::Password { .. } => Some(dowe_components::BuiltinComponent::Password),
        ViewNode::Phone { .. } => Some(dowe_components::BuiltinComponent::Phone),
        ViewNode::Pin { .. } => Some(dowe_components::BuiltinComponent::Pin),
        ViewNode::Textarea { .. } => Some(dowe_components::BuiltinComponent::Textarea),
        ViewNode::Color { .. } => Some(dowe_components::BuiltinComponent::Color),
        ViewNode::Dropzone { .. } => Some(dowe_components::BuiltinComponent::Dropzone),
        ViewNode::ComboBox { .. } => Some(dowe_components::BuiltinComponent::ComboBox),
        ViewNode::CsvField { .. } => Some(dowe_components::BuiltinComponent::CsvField),
        ViewNode::DragDrop { .. } => Some(dowe_components::BuiltinComponent::DragDrop),
        ViewNode::Editor { .. } => Some(dowe_components::BuiltinComponent::Editor),
        ViewNode::ImageCropper { .. } => Some(dowe_components::BuiltinComponent::ImageCropper),
        _ => None,
    }
}

fn compose_form_variant_props(node: &ViewNode) -> Option<&dowe_components::VariantProps> {
    match node {
        ViewNode::Input { props } | ViewNode::Select { props, .. } => Some(props),
        ViewNode::Checkbox { props } => Some(&props.style),
        ViewNode::Color { props } => Some(&props.style),
        ViewNode::Date { props } => Some(&props.style),
        ViewNode::DateRange { props } => Some(&props.style),
        ViewNode::RadioGroup { props, .. } => Some(&props.style),
        ViewNode::Toggle { props } => Some(&props.style),
        ViewNode::Slider { props } => Some(&props.style),
        ViewNode::Dropzone { props } => Some(&props.style),
        ViewNode::ComboBox { props, .. } => Some(&props.style),
        ViewNode::CsvField { props, .. } => Some(&props.style),
        ViewNode::DragDrop { props, .. } => Some(&props.style),
        ViewNode::Editor { props } => Some(&props.style),
        ViewNode::ImageCropper { props } => Some(&props.style),
        ViewNode::Password { props } => Some(&props.style),
        ViewNode::Phone { props } => Some(&props.style),
        ViewNode::Pin { props } => Some(&props.style),
        ViewNode::Textarea { props } => Some(&props.style),
        _ => None,
    }
}
