#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableProps {
    pub style: VariantProps,
    pub data: String,
    pub columns: Vec<TableColumn>,
    pub size: TableSize,
    pub striped: bool,
    pub bordered: bool,
    pub dividers: bool,
    pub empty_title: String,
    pub empty_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableColumn {
    pub field: String,
    pub label: String,
    pub align: TableColumnAlign,
    pub width: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DividerProps {
    pub style: StyleProps,
    pub orientation: DividerOrientation,
    pub color: ColorFamily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DividerOrientation {
    Horizontal,
    Vertical,
}

impl DividerOrientation {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "horizontal" => Some(Self::Horizontal),
            "vertical" => Some(Self::Vertical),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Horizontal, Self::Vertical]
    }
}

