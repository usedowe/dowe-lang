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
    pub px: Option<ResponsiveValue<ScaleValue>>,
    pub py: Option<ResponsiveValue<ScaleValue>>,
    pub pl: Option<ResponsiveValue<ScaleValue>>,
    pub pr: Option<ResponsiveValue<ScaleValue>>,
    pub pt: Option<ResponsiveValue<ScaleValue>>,
    pub pb: Option<ResponsiveValue<ScaleValue>>,
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
    pub h: Option<ResponsiveValue<SizeValue>>,
    pub min_w: Option<ResponsiveValue<SizeValue>>,
    pub min_h: Option<ResponsiveValue<SizeValue>>,
    pub max_w: Option<ResponsiveValue<SizeValue>>,
    pub max_h: Option<ResponsiveValue<SizeValue>>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropValue {
    String(String),
    Number(String),
    Boolean(bool),
    Responsive(Vec<ResponsivePropEntry>),
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

