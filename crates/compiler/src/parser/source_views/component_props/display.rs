fn is_known_display_prop(component: BuiltinComponent, name: &str) -> bool {
    match component {
            BuiltinComponent::AvatarGroup => matches!(
                name,
                "items"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "max"
                    | "autoFit"
                    | "inline"
                    | "bordered"
                    | "color"
            ),
            BuiltinComponent::ChatBox => matches!(
                name,
                "messages"
                    | "mode"
                    | "currentUserId"
                    | "userName"
                    | "userAvatar"
                    | "userStatus"
                    | "assistantName"
                    | "assistantAvatar"
                    | "showHeader"
                    | "placeholder"
                    | "showAttachments"
                    | "showVoiceNote"
                    | "showCamera"
                    | "loading"
                    | "sending"
                    | "streaming"
                    | "hasMore"
                    | "onSend"
                    | "onLoadMore"
                    | "onStop"
                    | "onVoiceNote"
                    | "onFileAttach"
                    | "onCameraCapture"
                    | "variant"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::Empty => matches!(
                name,
                "type"
                    | "title"
                    | "description"
                    | "href"
                    | "navigate"
                    | "history"
                    | "target"
                    | "externalMode"
                    | "onClick"
                    | "actionLabel"
                    | "variant"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::Marquee => matches!(
                name,
                "speed" | "pauseOnHover" | "reverse" | "orientation" | "fade" | "fadeColor" | "gap"
            ),
            BuiltinComponent::TypeWriter => matches!(
                name,
                "typeSpeed"
                    | "deleteSpeed"
                    | "afterTyped"
                    | "afterDeleted"
                    | "repeat"
                    | "bg"
                    | "color"
            ),
            BuiltinComponent::RichText => {
                matches!(
                    name,
                    "size" | "weight" | "spacing" | "bg" | "color" | "i18n" | "title"
                )
            }
            BuiltinComponent::Record => matches!(
                name,
                "name"
                    | "url"
                    | "disabled"
                    | "maxDuration"
                    | "onStart"
                    | "onPause"
                    | "onResume"
                    | "onStop"
                    | "onDiscard"
                    | "onConfirm"
                    | "variant"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::Camera => matches!(
                name,
                "facing" | "label" | "disabled" | "onStart" | "onCapture" | "onError" | "variant" | "scheme" | "color"
            ),
            BuiltinComponent::Microphone => matches!(
                name,
                "label" | "maxDuration" | "disabled" | "onStart" | "onStop" | "onError" | "variant" | "scheme" | "color"
            ),
            BuiltinComponent::ToggleGroup => matches!(
                name,
                "value"
                    | "selected"
                    | "size"
                    | "wide"
                    | "vertical"
                    | "disabled"
                    | "ariaLabel"
                    | "onChange"
                    | "variant"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::Collapsible => matches!(
                name,
                "label" | "defaultOpen" | "disabled" | "variant" | "scheme" | "color"
            ),
            BuiltinComponent::Countdown => matches!(
                name,
                "target"
                    | "showDays"
                    | "showHours"
                    | "showMinutes"
                    | "showSeconds"
                    | "size"
                    | "daysLabel"
                    | "hoursLabel"
                    | "minutesLabel"
                    | "secondsLabel"
                    | "onComplete"
                    | "variant"
                    | "scheme"
                    | "color"
            ),
            BuiltinComponent::Map => matches!(
                name,
                "centerLat"
                    | "centerLng"
                    | "zoom"
                    | "height"
                    | "width"
                    | "showControls"
                    | "showScale"
                    | "showLocationControl"
                    | "interactive"
                    | "routeStartLat"
                    | "routeStartLng"
                    | "routeEndLat"
                    | "routeEndLng"
                    | "onLocation"
                    | "onLocationError"
                    | "onRoute"
                    | "variant"
                    | "scheme"
                    | "color"
            ),
        _ => false,
    }
}
