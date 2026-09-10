fn component_common_value_completions(
    component: BuiltinComponent,
    prop: &str,
) -> Option<Vec<LanguageCompletion>> {
    match (component, prop) {
        (_, "animation") => Some(quoted_values(
            ViewAnimation::all().iter().map(|value| value.as_str()),
        )),
        (_, "transition") => Some(quoted_values(
            ViewTransition::all().iter().map(|value| value.as_str()),
        )),
        (_, "gesture") => Some(quoted_values(
            ViewGesture::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Section, "background") => Some(quoted_values(
            SectionBackground::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Box | BuiltinComponent::Section, "centerX" | "centerY") => {
            Some(boolean_values())
        }
        (BuiltinComponent::Section, "boxed") => Some(boolean_values()),
        (BuiltinComponent::Tree, "defaultOpen") => Some(boolean_values()),
        (BuiltinComponent::RichText, "title") => Some(boolean_values()),
        (BuiltinComponent::Title | BuiltinComponent::Text | BuiltinComponent::RichText, "size") => {
            Some(quoted_values(
                TextSize::all().iter().map(|value| value.as_str()),
            ))
        }
        (BuiltinComponent::Title | BuiltinComponent::Text, "align") => Some(quoted_values(
            TextAlign::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Title, "as") => {
            Some(quoted_values(["h1", "h2", "h3", "h4", "h5", "h6"]))
        }
        (
            BuiltinComponent::Title | BuiltinComponent::Text | BuiltinComponent::RichText,
            "weight",
        ) => Some(quoted_values(
            TextWeight::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Title | BuiltinComponent::Text | BuiltinComponent::RichText,
            "spacing",
        ) => Some(quoted_values(
            TextSpacing::all().iter().map(|value| value.as_str()),
        )),
        (_, "font") => Some(quoted_values(
            FontFamily::all().iter().map(|value| value.as_str()),
        )),
        (_, "bg" | "color" | "upColor" | "downColor" | "fadeColor") => Some(quoted_values(
            ColorToken::all().iter().map(|value| value.as_str()),
        )),
        (_, "borderColor" | "shadowColor") => Some(quoted_values(
            ColorFamily::all().iter().map(|value| value.as_str()),
        )),
        (_, "shadow") => Some(quoted_values(
            ShadowSize::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Path, "fill") => Some(quoted_values(
            ["none", "currentColor"]
                .into_iter()
                .chain(ColorToken::all().iter().map(|value| value.as_str())),
        )),
        (BuiltinComponent::Path, "fillRule") => Some(quoted_values(["nonzero", "evenodd"])),
        (_, "h" | "minH" | "maxH") => Some(quoted_values(["full", "auto"])),
        (_, "w" | "minW") => Some(quoted_values(
            ["full"]
                .into_iter()
                .chain(ContainerSize::all().iter().map(|value| value.as_str()))
                .chain([
                    "10%", "20%", "30%", "40%", "50%", "60%", "70%", "80%", "90%", "100%",
                ]),
        )),
        (_, "maxW") => {
            Some(quoted_values(["full"].into_iter().chain(
                ContainerSize::all().iter().map(|value| value.as_str()),
            )))
        }
        (_, "rounded") => Some(quoted_values(
            RoundedSize::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Flex, "justify") => Some(quoted_values(
            Justify::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Flex, "direction") => Some(quoted_values(
            FlexDirection::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Flex, "align") => Some(quoted_values(
            Align::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Grid, prop @ ("justify" | "align")) => Some(quoted_values(
            GridAlignment::all()
                .iter()
                .filter(|value| {
                    if prop == "justify" {
                        !matches!(value, GridAlignment::Baseline | GridAlignment::BaselineLast)
                    } else {
                        !matches!(
                            value,
                            GridAlignment::Between
                                | GridAlignment::Around
                                | GridAlignment::Evenly
                                | GridAlignment::Normal
                        )
                    }
                })
                .map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Button
            | BuiltinComponent::Avatar
            | BuiltinComponent::FabAction
            | BuiltinComponent::Empty,
            "navigate",
        ) => Some(quoted_values(
            NavigationOperation::all()
                .iter()
                .map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Button
            | BuiltinComponent::Avatar
            | BuiltinComponent::FabAction
            | BuiltinComponent::Empty,
            "history",
        ) => Some(quoted_values(["back"])),
        (
            BuiltinComponent::Button
            | BuiltinComponent::Avatar
            | BuiltinComponent::FabAction
            | BuiltinComponent::Empty,
            "target",
        ) => Some(quoted_values(
            WebTarget::all().iter().map(|value| value.as_str()),
        )),
        (
            BuiltinComponent::Button
            | BuiltinComponent::Avatar
            | BuiltinComponent::FabAction
            | BuiltinComponent::Empty,
            "externalMode",
        ) => Some(quoted_values(
            NativeExternalMode::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Fab | BuiltinComponent::FabAction, "icon") => Some(quoted_values(
            ViewIcon::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::ToggleGroup, "icon") => Some(quoted_values(
            ViewIcon::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Alert, "type") => Some(quoted_values(
            AlertKind::all().iter().map(|value| value.as_str()),
        )),
        (BuiltinComponent::Toast, "type") => Some(quoted_values(
            ToastKind::all().iter().map(|value| value.as_str()),
        )),
        _ => None,
    }
}
