fn prop_type(component: &str, prop: &str) -> String {
    if component == "Icon" && prop == "name" {
        return "quoted catalog icon name or string path from a constant or each over a constant collection"
            .to_string();
    }
    if component == "Button" && matches!(prop, "loading" | "disabled") {
        return "boolean Signal or View Store path".to_string();
    }
    if (component == "Image" || component == "Iframe") && prop == "src" {
        return "quoted packaged, HTTPS, or internal route source, or string constant, Signal, or each-item path"
            .to_string();
    }
    if component == "Tree" && prop == "data" {
        return "Signal or constant object or array path".to_string();
    }
    if let Some(values) = BuiltinComponent::from_name(component)
        .and_then(|component| component_value_completions(component, prop))
        .filter(|values| !values.is_empty())
    {
        return values
            .iter()
            .map(|value| value.label.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
    }
    match prop {
        "disabled"
        | "checked"
        | "multiple"
        | "autoplay"
        | "defaultOpen"
        | "open"
        | "loading"
        | "sending"
        | "streaming"
        | "hasMore"
        | "bordered"
        | "blurred"
        | "boxed"
        | "floating"
        | "dockOnScroll"
        | "fixed"
        | "hideLabel"
        | "labelFloating"
        | "disableOverlayClose"
        | "hideCloseButton"
        | "hideControls"
        | "hideIndicators"
        | "showNavigation"
        | "showCounter"
        | "disableLoop"
        | "showHex"
        | "showRgb"
        | "showCmyk"
        | "showOklch" => "boolean".to_string(),
        "centerX" | "centerY" => "boolean | responsive boolean".to_string(),
        "template" => "boolean".to_string(),
        "w" | "h" | "minW" | "minH" | "maxW" | "maxH" => {
            "Dowe size, auto or responsive Dowe size".to_string()
        }
        "fillRule" => "quoted nonzero | evenodd".to_string(),
        "rotate" => "number from -180 to 180 | responsive number".to_string(),
        "scale" => "number from 0.5 to 2 | responsive number".to_string(),
        "translateX" | "translateY" => {
            "Dowe scale from -96 to 96 | responsive Dowe scale".to_string()
        }
        "gap" => "Dowe scale value or px value | responsive gap".to_string(),
        "flex" => "\"initial\" | \"auto\" | \"none\" | 1 | responsive flex value".to_string(),
        "p" | "px" | "py" | "pl" | "pr" | "pt" | "pb" | "top" | "right" | "bottom" | "left"
        | "columns" | "rows" | "colSpan" | "rowSpan" | "min" | "max" | "step" | "maxSize"
        | "maxPoints" | "offsetX" | "offsetY" | "autoplayInterval" | "slideWidth"
        | "slideHeight" | "slidesPerView" => "number | responsive number".to_string(),
        name if name.starts_with("on") => "fn reference".to_string(),
        "i18n" | "descriptionI18n" | "statusI18n" => "quoted translation key".to_string(),
        "bind" | "data" | "series" | "items" | "messages" | "scene" | "visible" | "start"
        | "end" => "Signal path".to_string(),
        "show" => {
            "boolean | responsive boolean | boolean Signal path | numeric condition".to_string()
        }
        _ => "Dowe value".to_string(),
    }
}

fn prop_description(component: &str, prop: &str) -> &'static str {
    if component == "Icon" && prop == "name" {
        return "Validates every possible name from constant data and generates only those icon payloads. Mutable or unresolved names are rejected; no target generates the complete catalog.";
    }
    if component == "Tree" {
        return match prop {
            "defaultOpen" => {
                "Sets the initial expanded state for branches that have not been toggled."
            }
            "emptyLabel" => "Sets the visible copy used when the Tree has no valid nodes.",
            "ariaLabel" => "Names the Tree surface for assistive technology.",
            "onSelect" => {
                "References a fn that runs for selected leaves with the original node in item scope."
            }
            _ => prop_description_generic(prop),
        };
    }
    prop_description_generic(prop)
}

fn prop_description_generic(prop: &str) -> &'static str {
    match prop {
        name if name.starts_with("on") => {
            "References a visible Dowe fn executed by this component event."
        }
        "bind" => "Creates a two-way binding to a compatible Signal path.",
        "show" => {
            "Controls whether the component participates in layout. Accepts a boolean, responsive booleans, a boolean Signal path, or `{ when:<number Signal> gt|gte|lt|lte:<number> }`."
        }
        "scheme" => "Selects the component's semantic Dowe color family.",
        "variant" => "Selects the component's visual treatment.",
        "boxed" => {
            "Constrains and centers the component's generated content body while preserving its full-width structural container."
        }
        "gap" => {
            "Sets spacing between direct children. Accepts a Dowe scale or px value and responsive gap values; Section defaults to zero."
        }
        "flex" => {
            "Sets this Box, Section, Flex, Grid, or Card item's flex behavior when its direct parent is Section, Box, Flex, or Card. Use initial, auto, none, or 1; Grid children ignore it."
        }
        "centerX" => {
            "Centers Box or Section children horizontally. Accepts a boolean or responsive boolean value and defaults to false."
        }
        "centerY" => {
            "Centers Box or Section children vertically when the container has available height. Accepts a boolean or responsive boolean value and defaults to false."
        }
        "dockOnScroll" => {
            "Animates a fixed floating AppBar into the viewport top edge after the document passes 100px of scroll. Requires `floating:true` and `position:\"fixed\"`."
        }
        "position" => {
            "Controls Box flow and overlay placement with static, relative, absolute, or fixed positioning."
        }
        "align" => {
            "Controls logical text alignment on Text and Title. Use start, center, end, or justify; responsive values keep the same target-neutral meaning."
        }
        "top" | "right" | "bottom" | "left" => {
            "Offsets an absolute or fixed Box using a scalar or responsive Dowe scale value."
        }
        "rotate" => "Rotates the component by a validated number of degrees.",
        "scale" => "Scales the component uniformly around its center.",
        "translateX" | "translateY" => {
            "Moves the component visually on one axis without changing document flow."
        }
        "animation" => "Runs the selected entrance animation when the component appears.",
        "transition" => "Selects the timing preset used by interactive gesture state changes.",
        "gesture" => {
            "Adds portable hover or press feedback while respecting reduced-motion settings."
        }
        "maxW" => "Limits the component width without forcing it to occupy the full limit.",
        "h" => {
            "Sets the component height. Accepts a Dowe scale, full, auto, vh-<scale>, or responsive values."
        }
        "minH" => {
            "Sets the component minimum height. Accepts a Dowe scale, full, auto, vh-<scale>, or responsive values."
        }
        "maxH" => {
            "Limits the component height without adding implicit overflow behavior. Accepts a Dowe scale, full, auto, vh-<scale>, or responsive values."
        }
        "fillRule" => "Selects the portable fill rule used to resolve compound Path regions.",
        "size" => {
            "Selects the component's canonical Dowe size. Text and Title sizes use the shared fluid scale, so a scalar value is responsive by default."
        }
        "startIcon" | "endIcon" => {
            "Selects a quoted Solar icon resolved through the shared Icon catalog and sized from the Chip size."
        }
        "i18n" => {
            "References a translation key for the component's primary visible text while preserving the authored text as fallback."
        }
        "descriptionI18n" => {
            "References a translation key for the secondary description while preserving `description` as fallback."
        }
        "statusI18n" => {
            "References a translation key for the status copy while preserving `status` as fallback."
        }
        _ => "Accepted by this component and validated by the shared Dowe compiler.",
    }
}

fn return_kind(kind: StdlibReturnKind) -> &'static str {
    match kind {
        StdlibReturnKind::Unknown => "unknown",
        StdlibReturnKind::Null => "null",
        StdlibReturnKind::Bool => "boolean",
        StdlibReturnKind::Number => "number",
        StdlibReturnKind::String => "string",
        StdlibReturnKind::Array => "array",
        StdlibReturnKind::Object => "object",
    }
}
