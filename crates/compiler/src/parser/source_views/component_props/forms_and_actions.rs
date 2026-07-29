fn is_known_form_and_action_prop(component: BuiltinComponent, name: &str) -> bool {
    match component {
            BuiltinComponent::Table => {
                matches!(
                    name,
                    "data"
                        | "variant"
                        | "scheme"
                        | "size"
                        | "striped"
                        | "bordered"
                        | "dividers"
                        | "emptyTitle"
                        | "emptyDescription"
                )
            }
            BuiltinComponent::Divider => matches!(name, "orientation" | "scheme"),
            BuiltinComponent::Option => matches!(name, "value" | "label" | "description"),
            BuiltinComponent::ComboBox => matches!(
                name,
                "bind"
                    | "value"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "label"
                    | "placeholder"
                    | "labelFloating"
                    | "searchPlaceholder"
                    | "emptyText"
                    | "loadingText"
                    | "loadingMoreText"
                    | "clearable"
                    | "disabled"
                    | "helpText"
                    | "errorText"
                    | "color"
            ),
            BuiltinComponent::ComboOption => matches!(
                name,
                "value" | "label" | "description" | "src" | "icon" | "disabled"
            ),
            BuiltinComponent::CsvField => matches!(
                name,
                "buttonText"
                    | "modalTitle"
                    | "instructions"
                    | "cancelText"
                    | "confirmText"
                    | "clearText"
                    | "previewTitle"
                    | "multiple"
                    | "showPreview"
                    | "previewRows"
                    | "previewPageSize"
                    | "errorText"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "color"
            ),
            BuiltinComponent::CsvColumn => matches!(name, "name" | "label"),
            BuiltinComponent::DragDrop => matches!(
                name,
                "emptyText"
                    | "direction"
                    | "allowGroupTransfer"
                    | "disabled"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "color"
            ),
            BuiltinComponent::DragGroup => matches!(name, "id" | "title"),
            BuiltinComponent::DragItem => {
                matches!(name, "id" | "label" | "description" | "disabled")
            }
            BuiltinComponent::Editor => matches!(
                name,
                "bind"
                    | "value"
                    | "placeholder"
                    | "label"
                    | "helpText"
                    | "errorText"
                    | "minHeight"
                    | "hideToolbar"
                    | "disabled"
                    | "readonly"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "color"
            ),
            BuiltinComponent::ImageCropper => matches!(
                name,
                "bind"
                    | "src"
                    | "alt"
                    | "accept"
                    | "placeholder"
                    | "label"
                    | "helpText"
                    | "errorText"
                    | "aspectRatio"
                    | "minWidth"
                    | "minHeight"
                    | "maxWidth"
                    | "maxHeight"
                    | "shape"
                    | "disabled"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "color"
            ),
            BuiltinComponent::PasswordField => matches!(
                name,
                "bind"
                    | "value"
                    | "placeholder"
                    | "label"
                    | "labelFloating"
                    | "helpText"
                    | "errorText"
                    | "hideStrength"
                    | "weakLabel"
                    | "mediumLabel"
                    | "strongLabel"
                    | "disabled"
                    | "readonly"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "color"
            ),
            BuiltinComponent::PhoneField => matches!(
                name,
                "bind"
                    | "value"
                    | "country"
                    | "dialCodeName"
                    | "placeholder"
                    | "label"
                    | "labelFloating"
                    | "searchPlaceholder"
                    | "emptyText"
                    | "loadingText"
                    | "priorityCountries"
                    | "disabled"
                    | "helpText"
                    | "errorText"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "color"
            ),
            BuiltinComponent::PinField => matches!(
                name,
                "bind"
                    | "value"
                    | "length"
                    | "type"
                    | "label"
                    | "helpText"
                    | "errorText"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "color"
            ),
            BuiltinComponent::Textarea => matches!(
                name,
                "bind"
                    | "value"
                    | "placeholder"
                    | "label"
                    | "labelFloating"
                    | "helpText"
                    | "errorText"
                    | "rows"
                    | "cols"
                    | "maxLength"
                    | "resize"
                    | "disabled"
                    | "readonly"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "name"
                    | "color"
            ),
            BuiltinComponent::Button => matches!(
                name,
                "onClick"
                    | "iconStart"
                    | "iconEnd"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "loading"
                    | "href"
                    | "navigate"
                    | "history"
                    | "target"
                    | "externalMode"
            ),
            BuiltinComponent::Brand => matches!(
                name,
                "href" | "label" | "borderColor" | "shadow" | "shadowColor"
            ),
            BuiltinComponent::Banner => matches!(
                name,
                "href" | "label" | "borderColor" | "shadow" | "shadowColor"
            ),
            BuiltinComponent::IconButton => matches!(
                name,
                "icon"
                    | "label"
                    | "onClick"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "href"
                    | "navigate"
                    | "history"
                    | "target"
                    | "externalMode"
            ),
            BuiltinComponent::ToggleTheme => {
                matches!(
                    name,
                    "variant" | "scheme" | "size" | "lightLabel" | "darkLabel" | "color"
                )
            }
            BuiltinComponent::SelectTheme => {
                matches!(
                    name,
                    "label" | "placeholder" | "variant" | "scheme" | "size"
                )
            }
            BuiltinComponent::Fab => matches!(
                name,
                "position"
                    | "fixed"
                    | "offsetX"
                    | "offsetY"
                    | "icon"
                    | "label"
                    | "onClick"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "color"
            ),
            BuiltinComponent::FabAction => matches!(
                name,
                "label" | "icon" | "scheme" | "href" | "target" | "navigate" | "onClick" | "color"
            ),
            BuiltinComponent::Slider => matches!(
                name,
                "bind"
                    | "value"
                    | "min"
                    | "max"
                    | "step"
                    | "label"
                    | "name"
                    | "hideLabel"
                    | "scheme"
                    | "size"
                    | "color"
            ),
            BuiltinComponent::Dropzone => matches!(
                name,
                "accept"
                    | "multiple"
                    | "maxSize"
                    | "name"
                    | "label"
                    | "helpText"
                    | "errorText"
                    | "placeholder"
                    | "disabled"
                    | "variant"
                    | "scheme"
                    | "size"
                    | "color"
            ),
        _ => false,
    }
}
