#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentError {
    message: String,
}

impl ComponentError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn unknown_component(name: &str) -> Self {
        Self::new(format!("unknown component `{name}`"))
    }

    pub fn unknown_prop(component: BuiltinComponent, name: &str) -> Self {
        Self::new(format!("unknown prop `{name}` on `{}`", component.as_str()))
    }

    pub fn invalid_prop(name: &str, expected: &str) -> Self {
        Self::new(format!(
            "invalid value for prop `{name}`: expected {expected}"
        ))
    }

    pub fn box_requires_child() -> Self {
        Self::new("Box requires at least one child")
    }

    pub fn text_requires_static_text(component: BuiltinComponent) -> Self {
        Self::new(format!("{} requires static text", component.as_str()))
    }

    pub fn text_cannot_contain_component_children(component: BuiltinComponent) -> Self {
        Self::new(format!(
            "{} cannot contain component children",
            component.as_str()
        ))
    }

    pub fn text_cannot_contain_children(component: BuiltinComponent) -> Self {
        Self::new(format!("{} cannot contain children", component.as_str()))
    }

    pub fn text_cannot_contain_dynamic_expressions(component: BuiltinComponent) -> Self {
        Self::new(format!(
            "{} cannot contain dynamic expressions",
            component.as_str()
        ))
    }

    pub fn children_outside_layout() -> Self {
        Self::new("children can only be used inside layouts")
    }

    pub fn children_not_allowed(component: BuiltinComponent) -> Self {
        Self::new(format!(
            "children are not allowed inside `{}`",
            component.as_str()
        ))
    }

    pub fn dynamic_expressions_not_supported() -> Self {
        Self::new("dynamic expressions are not supported in views")
    }

    pub fn dynamic_prop_not_supported(name: &str) -> Self {
        Self::new(format!(
            "dynamic expressions are not supported for prop `{name}`"
        ))
    }

    pub fn invalid_prop_combination(message: impl Into<String>) -> Self {
        Self::new(message)
    }
}

impl Display for ComponentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ComponentError {}
