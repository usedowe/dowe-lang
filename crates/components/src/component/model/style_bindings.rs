#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleBindingProperty {
    BackgroundColor,
    TextColor,
    Padding,
    PaddingInline,
    PaddingBlock,
    PaddingLeft,
    PaddingRight,
    PaddingTop,
    PaddingBottom,
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
    BorderWidth,
    BorderRadius,
}

impl StyleBindingProperty {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BackgroundColor => "bg",
            Self::TextColor => "color",
            Self::Padding => "p",
            Self::PaddingInline => "px",
            Self::PaddingBlock => "py",
            Self::PaddingLeft => "pl",
            Self::PaddingRight => "pr",
            Self::PaddingTop => "pt",
            Self::PaddingBottom => "pb",
            Self::Width => "w",
            Self::Height => "h",
            Self::MinWidth => "minW",
            Self::MinHeight => "minH",
            Self::MaxWidth => "maxW",
            Self::MaxHeight => "maxH",
            Self::BorderWidth => "border",
            Self::BorderRadius => "rounded",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleBinding {
    pub property: StyleBindingProperty,
    pub binding: PropBinding,
}

impl StyleBinding {
    pub fn runtime_key(&self) -> String {
        format!("{}:{}", self.property.as_str(), self.binding.path)
    }
}

impl StyleProps {
    pub fn bindings(&self) -> Vec<StyleBinding> {
        let mut bindings = Vec::new();
        let mut push = |property, binding: Option<&PropBinding>| {
            if let Some(binding) = binding {
                bindings.push(StyleBinding { property, binding: binding.clone() });
            }
        };
        push(StyleBindingProperty::BackgroundColor, self.bg_binding.as_ref());
        push(StyleBindingProperty::TextColor, self.text_binding.as_ref());
        push(StyleBindingProperty::BorderRadius, self.rounded_binding.as_ref());
        push(StyleBindingProperty::BorderWidth, self.border_binding.as_ref());
        push(StyleBindingProperty::Padding, self.spacing.p_binding.as_ref());
        push(StyleBindingProperty::PaddingInline, self.spacing.px_binding.as_ref());
        push(StyleBindingProperty::PaddingBlock, self.spacing.py_binding.as_ref());
        push(StyleBindingProperty::PaddingLeft, self.spacing.pl_binding.as_ref());
        push(StyleBindingProperty::PaddingRight, self.spacing.pr_binding.as_ref());
        push(StyleBindingProperty::PaddingTop, self.spacing.pt_binding.as_ref());
        push(StyleBindingProperty::PaddingBottom, self.spacing.pb_binding.as_ref());
        push(StyleBindingProperty::Width, self.sizing.w_binding.as_ref());
        push(StyleBindingProperty::Height, self.sizing.h_binding.as_ref());
        push(StyleBindingProperty::MinWidth, self.sizing.min_w_binding.as_ref());
        push(StyleBindingProperty::MinHeight, self.sizing.min_h_binding.as_ref());
        push(StyleBindingProperty::MaxWidth, self.sizing.max_w_binding.as_ref());
        push(StyleBindingProperty::MaxHeight, self.sizing.max_h_binding.as_ref());
        bindings
    }
}
