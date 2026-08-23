#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantBindingProperty {
    Variant,
    Scheme,
    Size,
    Rounded,
    Loading,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantBinding {
    pub property: VariantBindingProperty,
    pub binding: PropBinding,
}

impl VariantBindingProperty {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Variant => "variant",
            Self::Scheme => "scheme",
            Self::Size => "size",
            Self::Rounded => "rounded",
            Self::Loading => "loading",
            Self::Disabled => "disabled",
        }
    }
}

impl VariantProps {
    pub fn bindings(&self) -> Vec<VariantBinding> {
        let mut bindings = Vec::new();
        let mut push = |property, path: Option<&String>, kind| {
            if let Some(path) = path {
                bindings.push(VariantBinding {
                    property,
                    binding: PropBinding::new(path.clone(), kind),
                });
            }
        };
        push(VariantBindingProperty::Variant, self.reactive.variant.as_ref(), PropValueKind::String);
        push(VariantBindingProperty::Scheme, self.reactive.scheme.as_ref(), PropValueKind::String);
        push(VariantBindingProperty::Size, self.reactive.size.as_ref(), PropValueKind::String);
        push(VariantBindingProperty::Rounded, self.reactive.rounded.as_ref(), PropValueKind::String);
        push(VariantBindingProperty::Loading, self.reactive.loading.as_ref(), PropValueKind::Boolean);
        push(VariantBindingProperty::Disabled, self.reactive.disabled.as_ref(), PropValueKind::Boolean);
        bindings
    }
}
