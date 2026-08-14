use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectQuery {
    pub projections: Vec<QueryProjection>,
    pub source: QuerySource,
    pub joins: Vec<QueryJoin>,
    pub filters: Vec<QueryFilter>,
    pub order: Vec<QueryOrder>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl SelectQuery {
    pub fn validate_parameters(&self, count: usize) -> Result<(), String> {
        let mut parameters = self
            .filters
            .iter()
            .filter_map(|filter| match filter.right {
                QueryOperand::Value(QueryValue::Parameter(index)) => Some(index),
                _ => None,
            })
            .collect::<Vec<_>>();
        parameters.sort_unstable();
        parameters.dedup();
        let expected = (1..=count).collect::<Vec<_>>();
        if parameters != expected {
            return Err(format!(
                "query parameters must use every placeholder from `?1` through `?{count}` exactly by position"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryProjection {
    pub value: QueryProjectionValue,
    pub alias: Option<String>,
}

impl QueryProjection {
    pub fn output_name(&self) -> Option<&str> {
        self.alias.as_deref().or_else(|| match &self.value {
            QueryProjectionValue::Identifier(identifier) => {
                identifier.parts.last().map(String::as_str)
            }
            QueryProjectionValue::Wildcard => None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryProjectionValue {
    Identifier(QueryIdentifier),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuerySource {
    pub table: String,
    pub alias: Option<String>,
}

impl QuerySource {
    pub fn qualifier(&self) -> &str {
        self.alias.as_deref().unwrap_or(&self.table)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryJoin {
    pub source: QuerySource,
    pub left: QueryIdentifier,
    pub right: QueryIdentifier,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryFilter {
    pub left: QueryIdentifier,
    pub right: QueryOperand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryOperand {
    Identifier(QueryIdentifier),
    Value(QueryValue),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryValue {
    Parameter(usize),
    Null,
    Bool(bool),
    Number(String),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryOrder {
    pub field: QueryIdentifier,
    pub descending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryIdentifier {
    pub parts: Vec<String>,
}

impl QueryIdentifier {
    pub fn key(&self) -> String {
        self.parts.join(".")
    }
}
