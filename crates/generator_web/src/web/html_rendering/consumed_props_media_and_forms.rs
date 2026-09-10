fn register_media_and_form_consumed_props(node: &ViewNode, context: &ReactiveRenderContext) {
    match node {
        ViewNode::Audio { .. } => {
            context.register_consumed_prop(BuiltinComponent::Audio, "src", "AudioProps.src");
        }
        ViewNode::Video { .. } => {
            context.register_consumed_prop(BuiltinComponent::Video, "src", "VideoProps.src");
            context.register_consumed_prop(BuiltinComponent::Video, "poster", "VideoProps.poster");
            context.register_consumed_prop(
                BuiltinComponent::Video,
                "autoplay",
                "VideoProps.autoplay",
            );
        }
        ViewNode::Iframe { .. } => {
            context.register_consumed_prop(BuiltinComponent::Iframe, "src", "IframeProps.src");
            context.register_consumed_prop(
                BuiltinComponent::Iframe,
                "sandbox",
                "IframeProps.sandbox",
            );
            context.register_consumed_prop(BuiltinComponent::Iframe, "allow", "IframeProps.allow");
        }
        ViewNode::Device { .. } => {
            context.register_consumed_prop(
                BuiltinComponent::Device,
                "device",
                "DeviceProps.device",
            );
            context.register_consumed_prop(BuiltinComponent::Device, "bind", "DeviceProps.bind");
            context.register_consumed_prop(
                BuiltinComponent::Device,
                "hideControls",
                "DeviceProps.hide_controls",
            );
            context.register_consumed_prop(
                BuiltinComponent::Device,
                "hideButtons",
                "DeviceProps.hide_controls",
            );
            context.register_consumed_prop(
                BuiltinComponent::Device,
                "studioInspector",
                "DeviceProps.studio_inspector",
            );
            context.register_consumed_prop(BuiltinComponent::Device, "zoom", "DeviceProps.options");
            context.register_consumed_prop(BuiltinComponent::Device, "fit", "DeviceProps.options");
        }
        ViewNode::Camera { .. } => {
            context.register_consumed_prop(
                BuiltinComponent::Camera,
                "facing",
                "CameraProps.facing",
            );
            context.register_consumed_prop(
                BuiltinComponent::Camera,
                "onCapture",
                "CameraProps.on_capture",
            );
            context.register_consumed_prop(
                BuiltinComponent::Camera,
                "onError",
                "CameraProps.on_error",
            );
        }
        ViewNode::Microphone { .. } => {
            context.register_consumed_prop(
                BuiltinComponent::Microphone,
                "onError",
                "MicrophoneProps.on_error",
            );
        }
        ViewNode::Editor { props } => {
            context.register_consumed_prop(
                BuiltinComponent::Editor,
                "language",
                "EditorProps.language",
            );
            context.register_consumed_prop(
                BuiltinComponent::Editor,
                "hideToolbar",
                "EditorProps.hide_toolbar",
            );
            if props.on_save.is_some() {
                context.register_consumed_prop(
                    BuiltinComponent::Editor,
                    "onSave",
                    "EditorProps.on_save",
                );
            }
        }
        ViewNode::Date { .. } => {}
        ViewNode::DateRange { .. } => {
            context.register_consumed_prop(
                BuiltinComponent::DateRange,
                "start",
                "DateRangeProps.start",
            );
            context.register_consumed_prop(
                BuiltinComponent::DateRange,
                "end",
                "DateRangeProps.end",
            );
        }
        ViewNode::Password { .. } => {}
        ViewNode::Phone { .. } => {
            context.register_consumed_prop(
                BuiltinComponent::Phone,
                "country",
                "PhoneProps.country",
            );
        }
        ViewNode::Pin { .. } => {
            context.register_consumed_prop(BuiltinComponent::Pin, "length", "PinProps.length");
        }
        ViewNode::Textarea { .. } => {}
        ViewNode::Color { .. } => {}
        ViewNode::Dropzone { .. } => {
            context.register_consumed_prop(
                BuiltinComponent::Dropzone,
                "multiple",
                "DropzoneProps.multiple",
            );
            context.register_consumed_prop(
                BuiltinComponent::Dropzone,
                "accept",
                "DropzoneProps.accept",
            );
        }
        ViewNode::Checkbox { .. } => {
            context.register_consumed_prop(
                BuiltinComponent::Checkbox,
                "checked",
                "CheckboxProps.checked",
            );
        }
        ViewNode::Toggle { .. } => {
            context.register_consumed_prop(
                BuiltinComponent::Toggle,
                "checked",
                "ToggleProps.checked",
            );
        }
        ViewNode::RadioGroup { .. } => {
            let component = match node {
                ViewNode::RadioGroup { props, .. } if matches!(props.presentation, RadioGroupPresentation::Card) => {
                    BuiltinComponent::RadioCard
                }
                _ => BuiltinComponent::RadioGroup,
            };
            context.register_consumed_prop(component, "orientation", "RadioGroupProps.orientation");
        }
        ViewNode::Slider { .. } => {
            context.register_consumed_prop(BuiltinComponent::Slider, "min", "SliderProps.min");
            context.register_consumed_prop(BuiltinComponent::Slider, "max", "SliderProps.max");
            context.register_consumed_prop(BuiltinComponent::Slider, "step", "SliderProps.step");
        }
        _ => {}
    }
}
