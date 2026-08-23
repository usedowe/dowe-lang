#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontConfig {
    pub default_family: FontFamily,
    pub install: Vec<FontFamily>,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            default_family: FontFamily::Inter,
            install: Vec::new(),
        }
    }
}

impl FontConfig {
    pub fn effective_families(&self, used: &BTreeSet<FontFamily>) -> BTreeSet<FontFamily> {
        let mut fonts = BTreeSet::new();
        fonts.insert(self.default_family);
        fonts.extend(self.install.iter().copied());
        fonts.extend(used.iter().copied());
        fonts
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpacingProps {
    pub p: Option<ResponsiveValue<ScaleValue>>,
    pub p_binding: Option<PropBinding>,
    pub px: Option<ResponsiveValue<ScaleValue>>,
    pub px_binding: Option<PropBinding>,
    pub py: Option<ResponsiveValue<ScaleValue>>,
    pub py_binding: Option<PropBinding>,
    pub pl: Option<ResponsiveValue<ScaleValue>>,
    pub pl_binding: Option<PropBinding>,
    pub pr: Option<ResponsiveValue<ScaleValue>>,
    pub pr_binding: Option<PropBinding>,
    pub pt: Option<ResponsiveValue<ScaleValue>>,
    pub pt_binding: Option<PropBinding>,
    pub pb: Option<ResponsiveValue<ScaleValue>>,
    pub pb_binding: Option<PropBinding>,
}

impl SpacingProps {
    pub fn with_horizontal_padding_default(&self, value: ResponsiveValue<ScaleValue>) -> Self {
        if self.p.is_some() {
            return self.clone();
        }

        let mut spacing = self.clone();
        if spacing.px.is_none() {
            match (spacing.pl.is_some(), spacing.pr.is_some()) {
                (false, false) => spacing.px = Some(value.clone()),
                (false, true) => spacing.pl = Some(value),
                (true, false) => spacing.pr = Some(value),
                (true, true) => {}
            }
        }
        spacing
    }

    pub fn with_vertical_padding_defaults(
        &self,
        top: ResponsiveValue<ScaleValue>,
        bottom: ResponsiveValue<ScaleValue>,
    ) -> Self {
        if self.p.is_some() {
            return self.clone();
        }

        let mut spacing = self.clone();
        if spacing.py.is_none() && spacing.pt.is_none() {
            spacing.pt = Some(top);
        }
        if spacing.py.is_none() && spacing.pb.is_none() {
            spacing.pb = Some(bottom);
        }
        spacing
    }

    pub fn with_padding_default(&self, value: ResponsiveValue<ScaleValue>) -> Self {
        if self.p.is_some() {
            return self.clone();
        }
        if self.px.is_none()
            && self.py.is_none()
            && self.pl.is_none()
            && self.pr.is_none()
            && self.pt.is_none()
            && self.pb.is_none()
        {
            return Self {
                p: Some(value),
                ..Default::default()
            };
        }

        let mut spacing = self.clone();
        if spacing.px.is_none() {
            match (spacing.pl.is_some(), spacing.pr.is_some()) {
                (false, false) => spacing.px = Some(value.clone()),
                (false, true) => spacing.pl = Some(value.clone()),
                (true, false) => spacing.pr = Some(value.clone()),
                (true, true) => {}
            }
        }
        if spacing.py.is_none() {
            match (spacing.pt.is_some(), spacing.pb.is_some()) {
                (false, false) => spacing.py = Some(value),
                (false, true) => spacing.pt = Some(value),
                (true, false) => spacing.pb = Some(value),
                (true, true) => {}
            }
        }
        spacing
    }

    pub fn with_padding_axis_defaults(
        &self,
        horizontal: ResponsiveValue<ScaleValue>,
        vertical: ResponsiveValue<ScaleValue>,
    ) -> Self {
        if self.p.is_some() {
            return self.clone();
        }

        let mut spacing = self.clone();
        if spacing.px.is_none() {
            match (spacing.pl.is_some(), spacing.pr.is_some()) {
                (false, false) => spacing.px = Some(horizontal.clone()),
                (false, true) => spacing.pl = Some(horizontal.clone()),
                (true, false) => spacing.pr = Some(horizontal),
                (true, true) => {}
            }
        }
        if spacing.py.is_none() {
            match (spacing.pt.is_some(), spacing.pb.is_some()) {
                (false, false) => spacing.py = Some(vertical.clone()),
                (false, true) => spacing.pt = Some(vertical.clone()),
                (true, false) => spacing.pb = Some(vertical),
                (true, true) => {}
            }
        }
        spacing
    }
}

pub fn section_content_spacing(spacing: &SpacingProps) -> SpacingProps {
    spacing.with_padding_axis_defaults(
        ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: ScaleValue::from_half_steps(8),
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: ScaleValue::from_half_steps(12),
            },
        ]),
        ResponsiveValue::ordered(vec![
            ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value: ScaleValue::from_half_steps(20),
            },
            ResponsiveEntry {
                breakpoint: Breakpoint::Md,
                value: ScaleValue::from_half_steps(32),
            },
        ]),
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SizingProps {
    pub w: Option<ResponsiveValue<SizeValue>>,
    pub w_binding: Option<PropBinding>,
    pub h: Option<ResponsiveValue<SizeValue>>,
    pub h_binding: Option<PropBinding>,
    pub min_w: Option<ResponsiveValue<SizeValue>>,
    pub min_w_binding: Option<PropBinding>,
    pub min_h: Option<ResponsiveValue<SizeValue>>,
    pub min_h_binding: Option<PropBinding>,
    pub max_w: Option<ResponsiveValue<SizeValue>>,
    pub max_w_binding: Option<PropBinding>,
    pub max_h: Option<ResponsiveValue<SizeValue>>,
    pub max_h_binding: Option<PropBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GridItemProps {
    pub col_span: Option<ResponsiveValue<GridSpan>>,
    pub row_span: Option<ResponsiveValue<GridSpan>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponsiveValue<T> {
    pub entries: Vec<ResponsiveEntry<T>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponsiveEntry<T> {
    pub breakpoint: Breakpoint,
    pub value: T,
}

impl<T> ResponsiveValue<T> {
    pub fn scalar(value: T) -> Self {
        Self {
            entries: vec![ResponsiveEntry {
                breakpoint: Breakpoint::Xs,
                value,
            }],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactiveValue<T> {
    pub value: T,
    pub binding: Option<PropBinding>,
}

impl<T> ReactiveValue<T> {
    pub fn static_value(value: T) -> Self { Self { value, binding: None } }
    pub fn bound(value: T, binding: PropBinding) -> Self { Self { value, binding: Some(binding) } }
    pub fn is_dynamic(&self) -> bool { self.binding.is_some() }
}

impl<T> ResponsiveValue<T> {
    pub fn ordered(mut entries: Vec<ResponsiveEntry<T>>) -> Self {
        entries.sort_by_key(|entry| entry.breakpoint.order());
        let mut unique = Vec::new();

        for entry in entries {
            if let Some(index) = unique
                .iter()
                .position(|existing: &ResponsiveEntry<T>| existing.breakpoint == entry.breakpoint)
            {
                unique[index] = entry;
            } else {
                unique.push(entry);
            }
        }

        Self { entries: unique }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentProp {
    pub name: String,
    pub value: PropValue,
}

impl PropValue {
    pub fn binding(&self) -> Option<&PropBinding> {
        match self { Self::Binding(binding) => Some(binding), _ => None }
    }

    pub fn binding_fallback(&self) -> Option<PropValue> {
        self.binding().and_then(|binding| binding.fallback.as_deref().cloned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropValue {
    String(String),
    Number(String),
    Boolean(bool),
    Responsive(Vec<ResponsivePropEntry>),
    Binding(PropBinding),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropBinding {
    pub path: String,
    pub kind: PropValueKind,
    pub fallback: Option<Box<PropValue>>,
}

impl PropBinding {
    pub fn new(path: impl Into<String>, kind: PropValueKind) -> Self {
        Self { path: path.into(), kind, fallback: None }
    }

    pub fn string(path: impl Into<String>) -> Self {
        Self::new(path, PropValueKind::String)
    }

    pub fn with_fallback(mut self, fallback: PropValue) -> Self {
        self.fallback = Some(Box::new(fallback));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponsivePropEntry {
    pub breakpoint: String,
    pub value: PropScalar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropScalar {
    String(String),
    Number(String),
    Boolean(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Breakpoint {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

impl Breakpoint {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "xs" => Some(Self::Xs),
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            "xl" => Some(Self::Xl),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }

    pub fn min_width(self) -> u16 {
        match self {
            Self::Xs => 0,
            Self::Sm => 640,
            Self::Md => 768,
            Self::Lg => 1024,
            Self::Xl => 1280,
        }
    }

    fn order(self) -> u8 {
        match self {
            Self::Xs => 0,
            Self::Sm => 1,
            Self::Md => 2,
            Self::Lg => 3,
            Self::Xl => 4,
        }
    }
}

