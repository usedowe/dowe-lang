use super::QueryOutcome;
use crate::engine::{Database, QueryPlan, StoreRecord};
use crate::error::{StoreError, StoreResult};
use crate::names::validate_table_name;
use crate::value::StoreValue;
use dowe_database_query::{
    QueryIdentifier, QueryOperand, QueryProjectionValue, QueryValue, SelectQuery,
};
use serde_json::Value;

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

pub(super) fn portable_base_row(qualifier: &str, record: StoreRecord) -> StoreRecord {
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

fn namespace_fields(output: &mut StoreRecord, table: &str, record: &StoreRecord) {
    for (key, value) in record {
        output.insert(format!("{table}.{key}"), value.clone());
    }
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
