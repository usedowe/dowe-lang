fn is_known_selection_and_text_prop(component: BuiltinComponent, name: &str) -> bool {
    match component {
            BuiltinComponent::Accordion => {
                matches!(name, "variant" | "scheme" | "multiple" | "color")
            }
            BuiltinComponent::Carousel => matches!(
                name,
                "autoplay"
                    | "autoplayInterval"
                    | "disableLoop"
                    | "hideControls"
                    | "hideIndicators"
                    | "showNavigation"
                    | "showCounter"
                    | "orientation"
                    | "scheme"
                    | "size"
                    | "indicatorType"
                    | "title"
                    | "slideWidth"
                    | "slideHeight"
                    | "slidesPerView"
                    | "gap"
                    | "color"
            ),
            BuiltinComponent::Checkbox => {
                matches!(
                    name,
                    "bind" | "checked" | "label" | "name" | "disabled" | "scheme" | "color"
                )
            }
            BuiltinComponent::Color => matches!(
                name,
                "bind"
                    | "value"
                    | "label"
                    | "placeholder"
                    | "labelFloating"
                    | "helpText"
                    | "errorText"
                    | "showHex"
                    | "showRgb"
                    | "showCmyk"
                    | "showOklch"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "color"
            ),
            BuiltinComponent::Date => matches!(
                name,
                "bind"
                    | "value"
                    | "label"
                    | "placeholder"
                    | "labelFloating"
                    | "helpText"
                    | "errorText"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "min"
                    | "max"
                    | "color"
            ),
            BuiltinComponent::DateRange => matches!(
                name,
                "start"
                    | "end"
                    | "startValue"
                    | "endValue"
                    | "label"
                    | "placeholder"
                    | "labelFloating"
                    | "helpText"
                    | "errorText"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "min"
                    | "max"
                    | "color"
            ),
            BuiltinComponent::RadioGroup => matches!(
                name,
                "bind" | "label" | "name" | "info" | "error" | "scheme" | "size" | "color"
            ),
            BuiltinComponent::Toggle => matches!(
                name,
                "bind"
                    | "checked"
                    | "label"
                    | "labelLeft"
                    | "labelRight"
                    | "name"
                    | "disabled"
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
            BuiltinComponent::Title | BuiltinComponent::Text => {
                matches!(
                    name,
                    "size" | "weight" | "spacing" | "bg" | "color" | "i18n"
                ) || (component == BuiltinComponent::Title && name == "as")
            }
        _ => false,
    }
}
