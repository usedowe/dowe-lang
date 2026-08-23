#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreFilter {
    pub field: String,
    pub value: StoreLiteral,
    pub additional: Vec<StoreMatchField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreMatchField {
    pub field: String,
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreLiteral {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Reference(String),
    Array(Vec<StoreLiteral>),
    Object(Vec<(String, StoreLiteral)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoweType {
    Unknown,
    Null,
    Bool,
    Number,
    String,
    Array(Box<DoweType>),
    Object(Vec<DoweTypeField>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoweTypeField {
    pub name: String,
    pub value: DoweType,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLog {
    pub level: ServerLogLevel,
    pub values: Vec<ServerLogValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerLogLevel {
    Log,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerLogValue {
    String(String),
    Reference(String),
    Number(String),
    Boolean(bool),
    Null,
    JsonLiteral(String),
}
