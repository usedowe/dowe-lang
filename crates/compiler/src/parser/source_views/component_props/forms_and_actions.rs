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
                | "rounded"
                | "onChange"
                | "onInput"
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
                | "rounded"
                | "onChange"
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
                | "rounded"
                | "onChange"
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
                | "onSave"
                | "variant"
                | "scheme"
                | "size"
                | "rounded"
                | "onChange"
                | "onInput"
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
                | "rounded"
                | "onChange"
                | "name"
                | "color"
        ),
        BuiltinComponent::Password => matches!(
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
                | "rounded"
                | "onChange"
                | "onInput"
                | "name"
                | "color"
        ),
        BuiltinComponent::Phone => matches!(
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
                | "rounded"
                | "onChange"
                | "onInput"
                | "name"
                | "color"
        ),
        BuiltinComponent::Pin => matches!(
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
                | "rounded"
                | "onChange"
                | "onInput"
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
                | "rounded"
                | "onChange"
                | "onInput"
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
                | "disabled"
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
        BuiltinComponent::Swap => matches!(
            name,
            "iconOn"
                | "iconOff"
                | "label"
                | "bind"
                | "variant"
                | "scheme"
                | "size"
                | "rounded"
                | "disabled"
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
                | "variant"
                | "scheme"
                | "size"
                | "rounded"
                | "onChange"
                | "onInput"
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
                | "rounded"
                | "onChange"
                | "color"
        ),
        _ => false,
    }
}
