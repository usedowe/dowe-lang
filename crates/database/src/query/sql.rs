use super::QueryOutcome;
use crate::engine::{Database, QueryPlan, StoreRecord};
use crate::error::{StoreError, StoreResult};
use crate::names::{validate_field_name, validate_table_name};
use crate::value::StoreValue;

use std::collections::BTreeMap;
pub(super) fn select(
    database: &Database,
    tokens: &[String],
    version: u64,
) -> StoreResult<QueryOutcome> {
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

pub(super) fn insert(database: &Database, tokens: &[String]) -> StoreResult<QueryOutcome> {
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

pub(super) fn update(database: &Database, tokens: &[String]) -> StoreResult<QueryOutcome> {
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

pub(super) fn delete(database: &Database, tokens: &[String]) -> StoreResult<QueryOutcome> {
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
