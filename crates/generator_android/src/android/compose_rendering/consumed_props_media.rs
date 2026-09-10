fn register_compose_media_consumed_props(node: &ViewNode, context: &ComposeReactiveContext) {
    match node {
        ViewNode::Audio { .. } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Audio,
                "src",
                "AudioProps.src",
            );
        }
        ViewNode::Video { .. } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Video,
                "src",
                "VideoProps.src",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Video,
                "poster",
                "VideoProps.poster",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Video,
                "autoplay",
                "VideoProps.autoplay",
            );
        }
        ViewNode::Iframe { .. } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Iframe,
                "src",
                "IframeProps.src",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Iframe,
                "sandbox",
                "IframeProps.sandbox",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Iframe,
                "allow",
                "IframeProps.allow",
            );
        }
        ViewNode::Device { .. } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Device,
                "device",
                "DeviceProps.device",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Device,
                "bind",
                "DeviceProps.bind",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Device,
                "hideControls",
                "DeviceProps.hide_controls",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Device,
                "hideButtons",
                "DeviceProps.hide_controls",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Device,
                "zoom",
                "DeviceProps.options",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Device,
                "fit",
                "DeviceProps.options",
            );
        }
        ViewNode::Camera { .. } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Camera,
                "facing",
                "CameraProps.facing",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Camera,
                "onCapture",
                "CameraProps.on_capture",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Camera,
                "onError",
                "CameraProps.on_error",
            );
        }
        ViewNode::Microphone { .. } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Microphone,
                "onError",
                "MicrophoneProps.on_error",
            );
        }
        ViewNode::Editor { props } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Editor,
                "language",
                "EditorProps.language",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Editor,
                "hideToolbar",
                "EditorProps.hide_toolbar",
            );
            if props.on_save.is_some() {
                context.register_consumed_prop(
                    dowe_components::BuiltinComponent::Editor,
                    "onSave",
                    "EditorProps.on_save",
                );
            }
        }
        ViewNode::Date { .. } => {}
        ViewNode::DateRange { .. } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::DateRange,
                "start",
                "DateRangeProps.start",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::DateRange,
                "end",
                "DateRangeProps.end",
            );
        }
        ViewNode::Password { .. } => {}
        ViewNode::Phone { .. } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Phone,
                "country",
                "PhoneProps.country",
            );
        }
        ViewNode::Pin { .. } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Pin,
                "length",
                "PinProps.length",
            );
        }
        ViewNode::Textarea { .. } => {}
        ViewNode::Color { .. } => {}
        ViewNode::Dropzone { .. } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Dropzone,
                "multiple",
                "DropzoneProps.multiple",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Dropzone,
                "accept",
                "DropzoneProps.accept",
            );
        }
        ViewNode::Checkbox { .. } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Checkbox,
                "checked",
                "CheckboxProps.checked",
            );
        }
        ViewNode::Toggle { .. } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Toggle,
                "checked",
                "ToggleProps.checked",
            );
        }
        ViewNode::RadioGroup { props, .. } => {
            context.register_consumed_prop(
                if matches!(props.presentation, dowe_components::RadioGroupPresentation::Card) {
                    dowe_components::BuiltinComponent::RadioCard
                } else {
                    dowe_components::BuiltinComponent::RadioGroup
                },
                "orientation",
                "RadioGroupProps.orientation",
            );
        }
        ViewNode::Slider { .. } => {
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Slider,
                "min",
                "SliderProps.min",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Slider,
                "max",
                "SliderProps.max",
            );
            context.register_consumed_prop(
                dowe_components::BuiltinComponent::Slider,
                "step",
                "SliderProps.step",
            );
        }
        _ => {}
    }
}
