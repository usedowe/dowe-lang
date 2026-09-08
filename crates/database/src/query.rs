use crate::engine::Database;
use crate::error::{StoreError, StoreResult};

pub enum QueryOutcome {
    Rows {
        rows: Vec<crate::engine::StoreRecord>,
        plan: crate::engine::QueryPlan,
    },
    Changed {
        count: usize,
        detail: String,
    },
}

mod portable;
mod sql;
mod tokenizer;

pub use portable::execute_portable_select;
pub use tokenizer::bind_query_params;

pub fn execute_sql(database: &Database, sql_text: &str) -> StoreResult<QueryOutcome> {
    let tokens = tokenizer::tokenize(sql_text)?;
    let Some(first) = tokens
        .first()
        .map(|token| token.eq_ignore_ascii_case("select"))
    else {
        return Err(StoreError::InvalidQuery("query is empty".to_string()));
    };
    if first {
        return sql::select(database, &tokens, database.current_version()?);
    }
    if tokens
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("insert"))
    {
        return sql::insert(database, &tokens);
    }
    if tokens
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("update"))
    {
        return sql::update(database, &tokens);
    }
    if tokens
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("delete"))
    {
        return sql::delete(database, &tokens);
    }
    Err(StoreError::InvalidQuery(
        "only select, insert, update, and delete are supported".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::bind_query_params;
    use crate::{StoreRecord, StoreValue, init_database, open_database};
    use dowe_database_query::parse_select;
    use serde_json::{Value, json};
    use tempfile::tempdir;

    include!("query/tests.rs");
}
