#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeProps {
    pub style: VariantProps,
    pub data: String,
    pub bind: Option<String>,
    pub default_open: bool,
    pub empty_label: String,
    pub aria_label: String,
    pub on_select: Option<String>,
}
