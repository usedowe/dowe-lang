#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropValueKind {
    String,
    Number,
    Boolean,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropValidator {
    None,
    ColorToken,
    IconName,
    Enum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentPropContract {
    pub kind: PropValueKind,
    pub validator: PropValidator,
}

impl ComponentPropContract {
    pub const fn new(kind: PropValueKind) -> Self {
        Self {
            kind,
            validator: PropValidator::None,
        }
    }

    pub const fn with_validator(self, validator: PropValidator) -> Self {
        Self { validator, ..self }
    }
}

pub fn validate_prop_value(contract: ComponentPropContract, value: &str) -> bool {
    match contract.validator {
        PropValidator::None => true,
        PropValidator::ColorToken => {
            value == "currentColor" || ColorToken::from_name(value).is_some()
        }
        PropValidator::IconName => all_icon_names().iter().any(|name| name == value),
        PropValidator::Enum => !value.is_empty(),
    }
}

pub fn prop_color_tokens() -> &'static [&'static str] {
    &[
        "primary",
        "secondary",
        "accent",
        "muted",
        "success",
        "info",
        "warning",
        "danger",
        "background",
        "surface",
    ]
}

pub fn prop_allowed_values(component: BuiltinComponent, name: &str) -> &'static [&'static str] {
    match name {
        "variant" => {
            if matches!(component, BuiltinComponent::SideNav) {
                &["solid", "outlined", "ghost"]
            } else {
                &["solid", "outlined", "ghost"]
            }
        }
        "scheme" => &[
            "primary",
            "secondary",
            "accent",
            "muted",
            "success",
            "info",
            "warning",
            "danger",
        ],
        "size" => {
            if matches!(component, BuiltinComponent::SideNav) {
                &["sm", "md", "lg"]
            } else {
                &["xs", "sm", "md", "lg", "xl"]
            }
        }
        "rounded" => &["xs", "sm", "md", "lg", "xl", "full"],
        _ => &[],
    }
}

pub fn prop_validator_name(validator: PropValidator) -> &'static str {
    match validator {
        PropValidator::None => "valid value",
        PropValidator::ColorToken => "currentColor or Dowe color token",
        PropValidator::IconName => "known Dowe icon name",
        PropValidator::Enum => "valid component enum value",
    }
}

pub fn component_prop_contract(
    component: BuiltinComponent,
    name: &str,
) -> Option<ComponentPropContract> {
    let string = ComponentPropContract::new(PropValueKind::String);
    let number = ComponentPropContract::new(PropValueKind::Number);
    let boolean = ComponentPropContract::new(PropValueKind::Boolean);
    match name {
        "size" | "weight" | "spacing"
            if matches!(component, BuiltinComponent::Text | BuiltinComponent::Title) =>
        {
            Some(string)
        }
        "variant" | "scheme" | "rounded" | "size"
            if !matches!(
                component,
                BuiltinComponent::Option | BuiltinComponent::Path | BuiltinComponent::CsvColumn
            ) =>
        {
            Some(string.with_validator(PropValidator::Enum))
        }
        "color" | "bg" | "borderColor" | "shadowColor"
            if !matches!(
                component,
                BuiltinComponent::Option | BuiltinComponent::Path | BuiltinComponent::Svg
            ) =>
        {
            Some(string.with_validator(PropValidator::ColorToken))
        }
        "p" | "px" | "py" | "pl" | "pr" | "pt" | "pb" | "w" | "h" | "minW" | "minH" | "maxW"
        | "maxH" | "border"
            if !matches!(component, BuiltinComponent::Option | BuiltinComponent::Path) =>
        {
            Some(number)
        }
        "show" if !matches!(component, BuiltinComponent::Option | BuiltinComponent::Path) => {
            Some(boolean)
        }
        "fill" | "stroke" if component == BuiltinComponent::Icon => {
            Some(string.with_validator(PropValidator::ColorToken))
        }
        "name" if component == BuiltinComponent::Icon => {
            Some(string.with_validator(PropValidator::IconName))
        }
        "loading" | "disabled"
            if matches!(component, BuiltinComponent::Button | BuiltinComponent::Swap) =>
        {
            Some(boolean)
        }
        "bind" if component == BuiltinComponent::Swap => Some(boolean),
        "variant" | "scheme" | "size" if component == BuiltinComponent::SideNav => {
            Some(string.with_validator(PropValidator::Enum))
        }
        "wide" if component == BuiltinComponent::SideNav => Some(boolean),
        "src" if component == BuiltinComponent::Image => Some(string),
        "open"
            if matches!(
                component,
                BuiltinComponent::Drawer
                    | BuiltinComponent::Modal
                    | BuiltinComponent::AlertDialog
                    | BuiltinComponent::Command
            ) =>
        {
            Some(boolean)
        }
        "mobileMenuOpen" if component == BuiltinComponent::AppBar => Some(boolean),
        "source" if component == BuiltinComponent::Toast => {
            Some(ComponentPropContract::new(PropValueKind::Any))
        }
        "items" if component == BuiltinComponent::AvatarGroup => {
            Some(ComponentPropContract::new(PropValueKind::Any))
        }
        "data" if component == BuiltinComponent::Svg => {
            Some(ComponentPropContract::new(PropValueKind::Any))
        }
        "messages" if component == BuiltinComponent::ChatBox => {
            Some(ComponentPropContract::new(PropValueKind::Any))
        }
        "loading" | "sending" | "streaming" | "hasMore"
            if component == BuiltinComponent::ChatBox =>
        {
            Some(boolean)
        }
        "start" | "end" if component == BuiltinComponent::DateRange => Some(string),
        "value" if component == BuiltinComponent::ToggleGroup => Some(string),
        "data" | "series"
            if matches!(
                component,
                BuiltinComponent::Candlestick
                    | BuiltinComponent::ArcChart
                    | BuiltinComponent::AreaChart
                    | BuiltinComponent::BarChart
                    | BuiltinComponent::LineChart
                    | BuiltinComponent::PieChart
                    | BuiltinComponent::Table
            ) =>
        {
            Some(ComponentPropContract::new(PropValueKind::Any))
        }
        "scene" | "onPointer" | "onKey" | "onMotion" if component == BuiltinComponent::Canvas => {
            Some(ComponentPropContract::new(PropValueKind::Any))
        }
        _ => None,
    }
}

pub fn accepts_reactive_prop(component: BuiltinComponent, name: &str) -> bool {
    component_prop_contract(component, name).is_some()
}

pub fn prop_binding(path: impl Into<String>, kind: PropValueKind) -> PropValue {
    PropValue::Binding(PropBinding::new(path, kind))
}

pub fn default_binding_value(contract: ComponentPropContract) -> PropValue {
    match contract.kind {
        PropValueKind::Boolean => PropValue::Boolean(false),
        PropValueKind::Number => PropValue::Number("1".to_string()),
        PropValueKind::String => PropValue::String(
            match contract.validator {
                PropValidator::ColorToken => "primary",
                PropValidator::Enum => "md",
                PropValidator::IconName | PropValidator::None => "",
            }
            .to_string(),
        ),
        PropValueKind::Any => PropValue::String(String::new()),
    }
}
