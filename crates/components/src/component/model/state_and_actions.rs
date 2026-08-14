#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewConstant {
    pub id: String,
    pub name: String,
    pub value: ViewSignalValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewSignal {
    pub id: String,
    pub name: String,
    pub storage_key: String,
    pub scope: ViewSignalScope,
    pub storage: ViewSignalStorage,
    pub initial: ViewSignalValue,
    pub schema: Option<ViewSignalValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewSignalScope {
    Page,
    Global,
}

impl ViewSignalScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Global => "global",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewSignalStorage {
    None,
    Local,
}

impl ViewSignalStorage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Local => "local",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewSignalValue {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<ViewSignalValue>),
    Object(Vec<(String, ViewSignalValue)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewAction {
    pub id: String,
    pub name: String,
    pub params: Vec<ViewFunctionParameter>,
    pub return_type: Option<ViewFunctionReturn>,
    pub kind: ViewActionKind,
}

impl ViewAction {
    pub fn init(id: String, statements: Vec<ViewFunctionStatement>) -> Self {
        Self {
            id,
            name: "$dowe:init".to_string(),
            params: Vec::new(),
            return_type: None,
            kind: ViewActionKind::Sequence(statements),
        }
    }

    pub fn is_init(&self) -> bool {
        self.name == "$dowe:init"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewFunctionParameter {
    pub name: String,
    pub type_name: String,
    pub schema: ViewSignalValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewFunctionReturn {
    pub type_name: String,
    pub schema: ViewSignalValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewActionKind {
    Sequence(Vec<ViewFunctionStatement>),
    Request(ViewRequestAction),
    Assign(ViewAssignAction),
    Reset(ViewResetAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewFunctionStatement {
    Validate {
        target: String,
    },
    Request {
        result: String,
        action: ViewRequestAction,
    },
    If {
        result: String,
        success: Vec<ViewFunctionStatement>,
        error: Vec<ViewFunctionStatement>,
    },
    Assign(ViewAssignAction),
    Reset(ViewResetAction),
    Toast(ViewToastAction),
    Redirect {
        path: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewToastAction {
    pub kind: String,
    pub title: String,
    pub message: String,
    pub duration: Option<u32>,
    pub scheme: Option<String>,
    pub variant: Option<String>,
    pub position: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewRequestAction {
    pub method: ViewRequestMethod,
    pub path: String,
    pub base_env: Option<String>,
    pub headers: Vec<ViewRequestHeader>,
    pub body: Option<String>,
    pub update: Option<String>,
    pub reset: Option<String>,
    pub success_alert: Option<String>,
    pub success_message: Option<String>,
    pub error_alert: Option<String>,
    pub error_message: Option<String>,
    pub autoload: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewRequestHeader {
    pub name: String,
    pub value: ViewRequestHeaderValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewRequestHeaderValue {
    Static(String),
    Signal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewRequestMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl ViewRequestMethod {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "PUT" => Some(Self::Put),
            "PATCH" => Some(Self::Patch),
            "DELETE" => Some(Self::Delete),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewAssignAction {
    pub target: String,
    pub source: String,
    pub literal: Option<ViewSignalValue>,
    pub call: Option<StdlibCall>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewResetAction {
    pub target: String,
}
