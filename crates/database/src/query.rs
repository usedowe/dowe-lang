use crate::engine::{Database, QueryPlan, StoreRecord};
use crate::error::{StoreError, StoreResult};
use crate::names::{validate_field_name, validate_table_name};
use crate::value::StoreValue;
use dowe_database_query::{
    QueryIdentifier, QueryOperand, QueryProjectionValue, QueryValue, SelectQuery,
};
use serde_json::Value;
use std::collections::BTreeMap;

pub enum QueryOutcome {
    Rows {
        rows: Vec<StoreRecord>,
        plan: QueryPlan,
    },
    Changed {
        count: usize,
        detail: String,
    },
}

pub fn execute_sql(database: &Database, sql: &str) -> StoreResult<QueryOutcome> {
    let tokens = tokenize(sql)?;
    let Some(first) = tokens
        .first()
        .map(|token| token.eq_ignore_ascii_case("select"))
    else {
        return Err(StoreError::InvalidQuery("query is empty".to_string()));
    };
    if first {
        return select(database, &tokens, database.current_version()?);
    }
    if tokens
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("insert"))
    {
        return insert(database, &tokens);
    }
    if tokens
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("update"))
    {
        return update(database, &tokens);
    }
    if tokens
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("delete"))
    {
        return delete(database, &tokens);
    }
    Err(StoreError::InvalidQuery(
        "only select, insert, update, and delete are supported".to_string(),
    ))
}

pub fn bind_query_params(sql: &str, params: &[Value]) -> StoreResult<String> {
    let mut output = String::with_capacity(sql.len());
    let chars = sql.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    while index < chars.len() {
        let value = chars[index];
        if in_string {
            output.push(value);
            if escaped {
                escaped = false;
            } else if value == '\\' {
                escaped = true;
            } else if value == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if value == '"' {
            in_string = true;
            output.push(value);
            index += 1;
            continue;
        }
        if value != '?' {
            output.push(value);
            index += 1;
            continue;
        }
        let start = index + 1;
        let mut end = start;
        while end < chars.len() && chars[end].is_ascii_digit() {
            end += 1;
        }
        if end == start {
            output.push(value);
            index += 1;
            continue;
        }
        let parameter = chars[start..end]
            .iter()
            .collect::<String>()
            .parse::<usize>()
            .map_err(|_| StoreError::InvalidQuery("query parameter is invalid".to_string()))?;
        let value = params.get(parameter.saturating_sub(1)).ok_or_else(|| {
            StoreError::InvalidQuery(format!("query parameter `?{parameter}` is missing"))
        })?;
        output.push_str(&serde_json::to_string(value)?);
        index = end;
    }
    Ok(output)
}

pub fn execute_portable_select(
    database: &Database,
    query: &SelectQuery,
    params: &[Value],
) -> StoreResult<QueryOutcome> {
    query
        .validate_parameters(params.len())
        .map_err(StoreError::InvalidQuery)?;
    validate_table_name(&query.source.table)?;
    let version = database.current_version()?;
    let mut rows = database
        .records_at(&query.source.table, version)?
        .into_iter()
        .map(|record| portable_base_row(query.source.qualifier(), record))
        .collect::<Vec<_>>();

    for join in &query.joins {
        validate_table_name(&join.source.table)?;
        let right_rows = database.records_at(&join.source.table, version)?;
        let mut joined = Vec::new();
        for left in rows {
            for right in &right_rows {
                let mut candidate = left.clone();
                namespace_fields(&mut candidate, join.source.qualifier(), right);
                let Some(left_value) = lookup_identifier(&candidate, &join.left) else {
                    continue;
                };
                let Some(right_value) = lookup_identifier(&candidate, &join.right) else {
                    continue;
                };
                if left_value.comparable_text() == right_value.comparable_text() {
                    joined.push(candidate);
                }
            }
        }
        rows = joined;
    }

    for filter in &query.filters {
        match &filter.right {
            QueryOperand::Identifier(identifier) => rows.retain(|record| {
                let Some(left) = lookup_identifier(record, &filter.left) else {
                    return false;
                };
                let Some(right) = lookup_identifier(record, identifier) else {
                    return false;
                };
                left.comparable_text() == right.comparable_text()
            }),
            operand => {
                let expected = operand_value(operand, params)?;
                rows.retain(|record| {
                    lookup_identifier(record, &filter.left)
                        .is_some_and(|value| value.comparable_text() == expected.comparable_text())
                });
            }
        }
    }

    for order in query.order.iter().rev() {
        rows.sort_by(|left, right| {
            let ordering = lookup_order_value(left, &order.field, query)
                .map(StoreValue::comparable_text)
                .cmp(
                    &lookup_order_value(right, &order.field, query)
                        .map(StoreValue::comparable_text),
                );
            if order.descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    if let Some(offset) = query.offset {
        rows = rows.into_iter().skip(offset).collect();
    }
    if let Some(limit) = query.limit {
        rows.truncate(limit);
    }

    let rows = rows
        .into_iter()
        .map(|record| project_portable(&record, query))
        .collect::<Vec<_>>();
    Ok(QueryOutcome::Rows {
        rows,
        plan: QueryPlan {
            indexed: false,
            detail: if query.joins.is_empty() {
                "portable table scan".to_string()
            } else {
                "portable join".to_string()
            },
        },
    })
}

fn portable_base_row(qualifier: &str, record: StoreRecord) -> StoreRecord {
    let mut row = record.clone();
    namespace_fields(&mut row, qualifier, &record);
    row
}

fn operand_value(operand: &QueryOperand, params: &[Value]) -> StoreResult<StoreValue> {
    match operand {
        QueryOperand::Identifier(_) => Err(StoreError::InvalidQuery(
            "portable query identifier value requires a result row".to_string(),
        )),
        QueryOperand::Value(QueryValue::Parameter(index)) => params
            .get(index.saturating_sub(1))
            .cloned()
            .map(StoreValue::from_json)
            .ok_or_else(|| {
                StoreError::InvalidQuery(format!("query parameter `?{index}` is missing"))
            }),
        QueryOperand::Value(QueryValue::Null) => Ok(StoreValue::Null),
        QueryOperand::Value(QueryValue::Bool(value)) => Ok(StoreValue::Bool(*value)),
        QueryOperand::Value(QueryValue::Number(value)) => {
            Ok(StoreValue::from_json(serde_json::from_str(value)?))
        }
        QueryOperand::Value(QueryValue::String(value)) => Ok(StoreValue::String(value.clone())),
    }
}

fn lookup_identifier<'a>(
    record: &'a StoreRecord,
    identifier: &QueryIdentifier,
) -> Option<&'a StoreValue> {
    let field = identifier.key();
    lookup(record, &field)
}

fn lookup_order_value<'a>(
    record: &'a StoreRecord,
    identifier: &QueryIdentifier,
    query: &SelectQuery,
) -> Option<&'a StoreValue> {
    lookup_identifier(record, identifier).or_else(|| {
        let alias = (identifier.parts.len() == 1).then(|| identifier.parts[0].as_str())?;
        query.projections.iter().find_map(|projection| {
            if projection.alias.as_deref() != Some(alias) {
                return None;
            }
            let QueryProjectionValue::Identifier(identifier) = &projection.value else {
                return None;
            };
            lookup_identifier(record, identifier)
        })
    })
}

fn project_portable(record: &StoreRecord, query: &SelectQuery) -> StoreRecord {
    if query.projections.len() == 1
        && matches!(query.projections[0].value, QueryProjectionValue::Wildcard)
    {
        return record
            .iter()
            .filter(|(field, _)| !field.contains('.'))
            .map(|(field, value)| (field.clone(), value.clone()))
            .collect();
    }
    let mut output = StoreRecord::new();
    for projection in &query.projections {
        let QueryProjectionValue::Identifier(identifier) = &projection.value else {
            continue;
        };
        if let Some(value) = lookup_identifier(record, identifier)
            && let Some(name) = projection.output_name()
        {
            output.insert(name.to_string(), value.clone());
        }
    }
    output
}

fn select(database: &Database, tokens: &[String], version: u64) -> StoreResult<QueryOutcome> {
    let from = position(tokens, "from")?;
    if from < 2 || from + 1 >= tokens.len() {
        return Err(StoreError::InvalidQuery(
            "select must include fields and from table".to_string(),
        ));
    }
    let fields = tokens[1..from].join(" ");
    let fields = fields
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let table = &tokens[from + 1];
    validate_table_name(table)?;
    let mut stop = tokens.len();
    for keyword in ["where", "join", "order", "limit", "offset"] {
        if let Ok(index) = position_after(tokens, keyword, from + 2) {
            stop = stop.min(index);
        }
    }
    if stop != from + 2 {
        return Err(StoreError::InvalidQuery(
            "unexpected tokens after source table".to_string(),
        ));
    }
    let predicate = position(tokens, "where")
        .ok()
        .map(|index| parse_predicate_value(tokens, index))
        .transpose()?;
    let has_join = position(tokens, "join").is_ok();
    let mut detail = "table scan".to_string();
    let mut indexed = false;
    let mut rows = if !has_join
        && let Some((field, expected)) = &predicate
        && let Some(records) =
            database.indexed_records_at(table, simple_field(field), expected, version)?
    {
        indexed = true;
        detail = format!("indexed filter on {}", simple_field(field));
        records
    } else {
        database.records_at(table, version)?
    };

    if let Ok(join_index) = position(tokens, "join") {
        rows = join(database, table, &rows, tokens, join_index, version)?;
        detail = "join".to_string();
    }

    if let Some((field, expected)) = predicate {
        let simple_field = simple_field(&field);
        indexed = indexed || database.has_index(table, simple_field);
        if indexed {
            detail = format!("indexed filter on {simple_field}");
        }
        let expected = expected.comparable_text();
        rows.retain(|record| {
            lookup(record, &field).is_some_and(|value| value.comparable_text() == expected)
        });
    }

    if let Ok(order_index) = position(tokens, "order")
        && tokens
            .get(order_index + 1)
            .is_some_and(|value| value.eq_ignore_ascii_case("by"))
        && let Some(field) = tokens.get(order_index + 2)
    {
        rows.sort_by(|left, right| {
            lookup(left, field)
                .map(StoreValue::comparable_text)
                .cmp(&lookup(right, field).map(StoreValue::comparable_text))
        });
    }

    if let Ok(offset_index) = position(tokens, "offset")
        && let Some(offset) = tokens
            .get(offset_index + 1)
            .and_then(|value| value.parse::<usize>().ok())
    {
        rows = rows.into_iter().skip(offset).collect();
    }

    if let Ok(limit_index) = position(tokens, "limit")
        && let Some(limit) = tokens
            .get(limit_index + 1)
            .and_then(|value| value.parse::<usize>().ok())
    {
        rows.truncate(limit);
    }

    let rows = project(rows, &fields);
    Ok(QueryOutcome::Rows {
        rows,
        plan: QueryPlan { indexed, detail },
    })
}

fn insert(database: &Database, tokens: &[String]) -> StoreResult<QueryOutcome> {
    if tokens.len() < 4 || !tokens[1].eq_ignore_ascii_case("into") {
        return Err(StoreError::InvalidQuery(
            "insert must use `insert into <table> <json>`".to_string(),
        ));
    }
    let table = &tokens[2];
    validate_table_name(table)?;
    let json = tokens[3..].join(" ");
    let value = serde_json::from_str::<serde_json::Value>(&json)?;
    let Some(object) = value.as_object() else {
        return Err(StoreError::InvalidQuery(
            "insert value must be a JSON object".to_string(),
        ));
    };
    let mut record = StoreRecord::new();
    for (key, value) in object {
        validate_field_name(key)?;
        record.insert(key.clone(), StoreValue::from_json(value.clone()));
    }
    let _ = database.insert(table, record)?;
    Ok(QueryOutcome::Changed {
        count: 1,
        detail: "insert".to_string(),
    })
}

fn update(database: &Database, tokens: &[String]) -> StoreResult<QueryOutcome> {
    if tokens.len() < 8 {
        return Err(StoreError::InvalidQuery(
            "update must use `update <table> set <field> = <value> where <field> = <value>`"
                .to_string(),
        ));
    }
    let table = &tokens[1];
    validate_table_name(table)?;
    let set = position(tokens, "set")?;
    let where_index = position(tokens, "where")?;
    if tokens.get(set + 2).map(String::as_str) != Some("=") {
        return Err(StoreError::InvalidQuery(
            "update set must use equality".to_string(),
        ));
    }
    let patch_field = tokens
        .get(set + 1)
        .ok_or_else(|| StoreError::InvalidQuery("missing set field".to_string()))?;
    validate_field_name(patch_field)?;
    let patch_value = parse_value_token(
        tokens
            .get(set + 3)
            .ok_or_else(|| StoreError::InvalidQuery("missing set value".to_string()))?,
    );
    let (filter_field, expected) = parse_predicate_value(tokens, where_index)?;
    let mut patch = StoreRecord::new();
    patch.insert(patch_field.clone(), patch_value);
    let count = database.update(table, &filter_field, &expected, patch)?;
    Ok(QueryOutcome::Changed {
        count,
        detail: "update".to_string(),
    })
}

fn delete(database: &Database, tokens: &[String]) -> StoreResult<QueryOutcome> {
    if tokens.len() < 7 || !tokens[1].eq_ignore_ascii_case("from") {
        return Err(StoreError::InvalidQuery(
            "delete must use `delete from <table> where <field> = <value>`".to_string(),
        ));
    }
    let table = &tokens[2];
    validate_table_name(table)?;
    let where_index = position(tokens, "where")?;
    let (field, expected) = parse_predicate_value(tokens, where_index)?;
    let count = database.delete(table, &field, &expected)?;
    Ok(QueryOutcome::Changed {
        count,
        detail: "delete".to_string(),
    })
}

fn join(
    database: &Database,
    left_table: &str,
    left_rows: &[StoreRecord],
    tokens: &[String],
    join_index: usize,
    version: u64,
) -> StoreResult<Vec<StoreRecord>> {
    let right_table = tokens
        .get(join_index + 1)
        .ok_or_else(|| StoreError::InvalidQuery("join must declare a table".to_string()))?;
    validate_table_name(right_table)?;
    if !tokens
        .get(join_index + 2)
        .is_some_and(|value| value.eq_ignore_ascii_case("on"))
    {
        return Err(StoreError::InvalidQuery(
            "join must declare `on`".to_string(),
        ));
    }
    if tokens.get(join_index + 4).map(String::as_str) != Some("=") {
        return Err(StoreError::InvalidQuery(
            "join predicate must use equality".to_string(),
        ));
    }
    let left_field = tokens
        .get(join_index + 3)
        .ok_or_else(|| StoreError::InvalidQuery("missing join left field".to_string()))?;
    let right_field = tokens
        .get(join_index + 5)
        .ok_or_else(|| StoreError::InvalidQuery("missing join right field".to_string()))?;
    let right_rows = database.records_at(right_table, version)?;
    let mut output = Vec::new();

    for left in left_rows {
        for right in &right_rows {
            let Some(left_value) = lookup(left, left_field) else {
                continue;
            };
            let Some(right_value) = lookup(right, right_field) else {
                continue;
            };
            if left_value.comparable_text() == right_value.comparable_text() {
                let mut row = StoreRecord::new();
                namespace_fields(&mut row, left_table, left);
                namespace_fields(&mut row, right_table, right);
                output.push(row);
            }
        }
    }
    Ok(output)
}

fn project(rows: Vec<StoreRecord>, fields: &[String]) -> Vec<StoreRecord> {
    if fields.len() == 1 && fields[0] == "*" {
        return rows;
    }
    rows.into_iter()
        .map(|record| {
            let mut output = BTreeMap::new();
            for field in fields {
                if let Some(value) = lookup(&record, field) {
                    output.insert(field.clone(), value.clone());
                }
            }
            output
        })
        .collect()
}

fn namespace_fields(output: &mut StoreRecord, table: &str, record: &StoreRecord) {
    for (key, value) in record {
        output.insert(format!("{table}.{key}"), value.clone());
    }
}

fn parse_predicate_value(
    tokens: &[String],
    where_index: usize,
) -> StoreResult<(String, StoreValue)> {
    if tokens.get(where_index + 2).map(String::as_str) != Some("=") {
        return Err(StoreError::InvalidQuery(
            "where predicate must use equality".to_string(),
        ));
    }
    let field = tokens
        .get(where_index + 1)
        .ok_or_else(|| StoreError::InvalidQuery("missing where field".to_string()))?
        .clone();
    let value = parse_value_token(
        tokens
            .get(where_index + 3)
            .ok_or_else(|| StoreError::InvalidQuery("missing where value".to_string()))?,
    );
    Ok((field, value))
}

fn parse_value_token(value: &str) -> StoreValue {
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        return StoreValue::String(value[1..value.len() - 1].to_string());
    }
    if value == "true" {
        return StoreValue::Bool(true);
    }
    if value == "false" {
        return StoreValue::Bool(false);
    }
    if value == "null" {
        return StoreValue::Null;
    }
    if let Ok(value) = value.parse::<i64>() {
        return StoreValue::Int(value);
    }
    StoreValue::String(value.to_string())
}

fn lookup<'a>(record: &'a StoreRecord, field: &str) -> Option<&'a StoreValue> {
    record
        .get(field)
        .or_else(|| record.get(simple_field(field)))
}

fn simple_field(field: &str) -> &str {
    field
        .rsplit_once('.')
        .map(|(_, field)| field)
        .unwrap_or(field)
}

fn position(tokens: &[String], keyword: &str) -> StoreResult<usize> {
    position_after(tokens, keyword, 0)
}

fn position_after(tokens: &[String], keyword: &str, start: usize) -> StoreResult<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, token)| token.eq_ignore_ascii_case(keyword).then_some(index))
        .ok_or_else(|| StoreError::InvalidQuery(format!("missing `{keyword}`")))
}

fn tokenize(sql: &str) -> StoreResult<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut brace_depth = 0usize;

    for value in sql.chars() {
        if in_string {
            current.push(value);
            if escaped {
                escaped = false;
            } else if value == '\\' {
                escaped = true;
            } else if value == '"' {
                in_string = false;
            }
            continue;
        }

        match value {
            '"' => {
                in_string = true;
                current.push(value);
            }
            '{' => {
                brace_depth += 1;
                current.push(value);
            }
            '}' => {
                brace_depth = brace_depth.saturating_sub(1);
                current.push(value);
            }
            ',' if brace_depth == 0 => {
                current.push(value);
            }
            '=' if brace_depth == 0 => {
                push_current(&mut tokens, &mut current);
                tokens.push("=".to_string());
            }
            value if value.is_whitespace() && brace_depth == 0 => {
                push_current(&mut tokens, &mut current);
            }
            _ => current.push(value),
        }
    }

    if in_string {
        return Err(StoreError::InvalidQuery(
            "query has an unterminated string".to_string(),
        ));
    }
    push_current(&mut tokens, &mut current);
    Ok(tokens)
}

fn push_current(tokens: &mut Vec<String>, current: &mut String) {
    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }
    current.clear();
}

#[cfg(test)]
mod tests {
    use super::bind_query_params;
    use crate::{StoreRecord, StoreValue, init_database, open_database};
    use dowe_database_query::parse_select;
    use serde_json::{Value, json};
    use tempfile::tempdir;

    #[test]
    fn binds_dowe_query_parameters_outside_literals() {
        assert_eq!(
            bind_query_params(
                "select * from users where name = ?1 and template = \"?2\"",
                &[json!("Ana"), json!("ignored")],
            )
            .expect("query"),
            "select * from users where name = \"Ana\" and template = \"?2\""
        );
    }

    #[test]
    fn executes_documented_multi_join_query_with_portable_aliases() {
        let root = tempdir().expect("root");
        init_database(root.path(), "app").expect("database");
        let database = open_database(root.path(), "app").expect("open");
        for (table, value) in [
            (
                "users",
                json!({ "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV", "name": "Ana" }),
            ),
            (
                "roles",
                json!({ "id": "01ARZ3NDEKTSV4RRFFQ69G5FAW", "name": "admin" }),
            ),
            (
                "user_roles",
                json!({
                    "id": "01ARZ3NDEKTSV4RRFFQ69G5FAX",
                    "userId": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
                    "roleId": "01ARZ3NDEKTSV4RRFFQ69G5FAW"
                }),
            ),
        ] {
            let Value::Object(record) = value else {
                unreachable!();
            };
            let record = record
                .into_iter()
                .map(|(field, value)| (field, StoreValue::from_json(value)))
                .collect::<StoreRecord>();
            database.insert(table, record).expect("insert");
        }
        let query = parse_select("SELECT users.name, roles.name AS roleName FROM users JOIN user_roles ON user_roles.userId = users.id JOIN roles ON user_roles.roleId = roles.id WHERE users.id = ?1").expect("query");
        let result = database
            .query_portable_json(&query, &[json!("01ARZ3NDEKTSV4RRFFQ69G5FAV")])
            .expect("result");
        assert_eq!(result, json!([{ "name": "Ana", "roleName": "admin" }]));
    }

    #[test]
    fn executes_portable_identifier_filters() {
        let root = tempdir().expect("root");
        init_database(root.path(), "app").expect("database");
        let database = open_database(root.path(), "app").expect("open");
        let record = [
            (
                "id".to_string(),
                StoreValue::String("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            ),
            (
                "externalId".to_string(),
                StoreValue::String("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            ),
        ]
        .into_iter()
        .collect::<StoreRecord>();
        database.insert("users", record).expect("insert");
        let query = parse_select("SELECT id FROM users WHERE id = externalId").expect("query");

        let result = database.query_portable_json(&query, &[]).expect("result");

        assert_eq!(result, json!([{ "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV" }]));
    }
}
