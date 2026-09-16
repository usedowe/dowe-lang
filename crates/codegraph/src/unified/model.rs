use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeNamespace {
    Frontend,
    Backend,
    Shared,
    Product,
    Design,
    Asset,
    I18n,
    Contract,
    Verification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnifiedNodeKind {
    Project,
    Layout,
    Page,
    Component,
    Entity,
    Handler,
    Route,
    Middleware,
    Schema,
    Requirement,
    Capability,
    UseCase,
    Actor,
    DesignPattern,
    Asset,
    Translation,
    Contract,
    Invariant,
    Test,
    Source,
    Documentation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeRelation {
    Contains,
    Imports,
    Renders,
    Uses,
    Calls,
    Triggers,
    HandledBy,
    Reads,
    Writes,
    Validates,
    DependsOn,
    Implements,
    SpecifiedBy,
    StyledBy,
    References,
    VerifiedBy,
    Provides,
    Consumes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    Verified,
    Inferred,
    Assumed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeNode {
    pub id: String,
    pub namespace: KnowledgeNamespace,
    pub kind: UnifiedNodeKind,
    pub name: String,
    pub path: Option<String>,
    pub summary: Option<String>,
    pub evidence: EvidenceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEdge {
    pub from: String,
    pub relation: KnowledgeRelation,
    pub to: String,
    pub evidence: EvidenceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedCodeGraph {
    pub version: u32,
    pub root: String,
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnifiedScope {
    All,
    Frontend,
    Backend,
    Shared,
    Product,
    Design,
    Asset,
    I18n,
    Contract,
    Verification,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedGraphQuery {
    pub query: String,
    #[serde(default)]
    pub scope: UnifiedScope,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub depth: usize,
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedGraphQueryResult {
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
    pub impact: Vec<KnowledgeNode>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedNodeQueryResult {
    pub nodes: Vec<KnowledgeNode>,
    pub truncated: bool,
}

impl Default for UnifiedScope {
    fn default() -> Self {
        Self::All
    }
}
