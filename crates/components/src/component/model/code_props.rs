#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeProps {
    pub style: VariantProps,
    pub language: CodeLanguage,
    pub source: String,
    pub tokens: Vec<CodeToken>,
    pub copy_label: String,
    pub copied_label: String,
    pub template_segments: Vec<CodeTemplateSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeTemplateSegment {
    Static {
        text: String,
        tokens: Vec<CodeToken>,
    },
    Binding(String),
}

