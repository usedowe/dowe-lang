use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub imports: Vec<SourceImport>,
    pub nodes: Vec<SourceNode>,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceImport {
    pub local: String,
    pub path: String,
    pub location: SourceLocation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceNode {
    pub location: SourceLocation,
    pub name: String,
    pub args: Vec<SourceValue>,
    pub props: Vec<SourceProp>,
    pub children: Vec<SourceNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceProp {
    pub name: String,
    pub value: SourceValue,
    pub location: SourceLocation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub indent: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceValue {
    String(String),
    Number(String),
    Boolean(bool),
    Null,
    Bareword(String),
    Array(Vec<SourceValue>),
    Object(Vec<SourceObjectEntry>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceObjectEntry {
    KeyValue { key: String, value: SourceValue },
    Spread(String),
}

impl SourceValue {
    pub fn as_string_like(&self) -> Option<String> {
        match self {
            SourceValue::String(value)
            | SourceValue::Number(value)
            | SourceValue::Bareword(value) => Some(value.clone()),
            SourceValue::Boolean(value) => Some(value.to_string()),
            SourceValue::Null | SourceValue::Array(_) | SourceValue::Object(_) => None,
        }
    }

    pub fn as_required_string(&self) -> Option<String> {
        match self {
            SourceValue::String(value) | SourceValue::Bareword(value) if !value.is_empty() => {
                Some(value.clone())
            }
            _ => None,
        }
    }

    pub fn to_source(&self) -> String {
        match self {
            SourceValue::String(value) => format!("\"{}\"", value.replace('"', "\\\"")),
            SourceValue::Number(value) | SourceValue::Bareword(value) => value.clone(),
            SourceValue::Boolean(value) => value.to_string(),
            SourceValue::Null => "null".to_string(),
            SourceValue::Array(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(SourceValue::to_source)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            SourceValue::Object(entries) => {
                let values = entries
                    .iter()
                    .map(|entry| match entry {
                        SourceObjectEntry::KeyValue { key, value } => {
                            format!("{key}:{}", value.to_source())
                        }
                        SourceObjectEntry::Spread(value) => format!("...{value}"),
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("{{ {values} }}")
            }
        }
    }
}

impl SourceNode {
    pub fn prop(&self, name: &str) -> Option<&SourceProp> {
        self.props.iter().find(|prop| prop.name == name)
    }
}
