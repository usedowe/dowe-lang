const GRAPH_VERSION: u32 = 1;
const GRAPH_FILE: &str = "database.graph.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseMigrationReport {
    pub created: usize,
    pub unchanged: usize,
    pub dynamic: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseMigration {
    pub sequence: u32,
    pub fingerprint: String,
    pub sql: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MigrationGraph {
    version: u32,
    databases: Vec<MigrationDatabase>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MigrationDatabase {
    binding: String,
    provider: String,
    database: String,
    head: Option<String>,
    nodes: Vec<MigrationNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MigrationNode {
    sequence: u32,
    fingerprint: String,
    parent: Option<String>,
    file: Option<String>,
    sql_fingerprint: Option<String>,
    snapshot: SchemaSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SchemaSnapshot {
    entities: Vec<EntitySnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct EntitySnapshot {
    binding: String,
    table: String,
    fields: Vec<FieldSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FieldSnapshot {
    name: String,
    field_type: String,
    primary: bool,
    required: bool,
    unique: bool,
    index: bool,
}


