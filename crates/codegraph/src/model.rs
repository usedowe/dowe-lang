use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodeGraphMode {
    Dowe,
    Project,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Workspace,
    Crate,
    Module,
    File,
    Symbol,
    Spec,
    Contract,
    Acceptance,
    Test,
    Doc,
    AgentHarness,
    GeneratedArtifact,
    OwnershipArea,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Contains,
    DependsOn,
    Owns,
    Implements,
    Tests,
    Documents,
    DeclaresContract,
    Validates,
    Generates,
    Duplicates,
    Reexports,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeGraph {
    pub mode: CodeGraphMode,
    pub root: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub path: Option<String>,
    pub name: String,
    pub owner: Option<String>,
    pub fingerprint: String,
    pub metrics: Option<FileMetrics>,
    pub source_range: Option<SourceRange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceRange {
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMetrics {
    pub total_lines: usize,
    pub code_lines: usize,
    pub functions: usize,
    pub types: usize,
    pub public_items: usize,
    pub inline_tests: usize,
    pub responsibilities: usize,
    pub imports: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BuildOptions {
    pub mode: Option<CodeGraphMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CheckOptions {
    pub mode: Option<CodeGraphMode>,
    pub use_baseline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport {
    pub diagnostics: Vec<Diagnostic>,
}

impl CheckReport {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

impl Default for CheckReport {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub path: String,
    pub message: String,
    pub action: String,
    pub owner: Option<String>,
    pub metric: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeExplanation {
    pub node: Node,
    pub incoming: Vec<Edge>,
    pub outgoing: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WrittenReports {
    pub graph_path: String,
    pub report_path: String,
    pub markdown_path: String,
    pub ownership_path: String,
    pub duplication_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Baseline {
    pub diagnostics: Vec<BaselineDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineDiagnostic {
    pub code: String,
    pub path: String,
    pub metric: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Waiver {
    pub path: String,
    pub limit: usize,
    pub reason: String,
    pub spec: String,
    pub expires: String,
    pub owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FunctionFingerprint {
    pub path: String,
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub normalized: String,
}
