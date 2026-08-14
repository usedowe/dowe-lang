use crate::{QueryOperand, QueryProjectionValue, QueryValue, SelectQuery};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryDialect {
    Postgres,
    D1,
}

pub fn render_select(query: &SelectQuery, dialect: QueryDialect) -> String {
    let projections = query
        .projections
        .iter()
        .map(|projection| match &projection.value {
            QueryProjectionValue::Wildcard => "*".to_string(),
            QueryProjectionValue::Identifier(identifier) => format!(
                "{} AS {}",
                quote_parts(&identifier.parts),
                quote(projection.output_name().unwrap_or_default())
            ),
        })
        .collect::<Vec<_>>()
        .join(", ");
    let mut sql = format!("SELECT {projections} FROM {}", quote(&query.source.table));
    if let Some(alias) = &query.source.alias {
        sql.push_str(&format!(" AS {}", quote(alias)));
    }
    for join in &query.joins {
        sql.push_str(&format!(" JOIN {}", quote(&join.source.table)));
        if let Some(alias) = &join.source.alias {
            sql.push_str(&format!(" AS {}", quote(alias)));
        }
        sql.push_str(&format!(
            " ON {} = {}",
            quote_parts(&join.left.parts),
            quote_parts(&join.right.parts)
        ));
    }
    if !query.filters.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(
            &query
                .filters
                .iter()
                .map(|filter| {
                    format!(
                        "{} = {}",
                        quote_parts(&filter.left.parts),
                        operand(&filter.right, dialect)
                    )
                })
                .collect::<Vec<_>>()
                .join(" AND "),
        );
    }
    if !query.order.is_empty() {
        sql.push_str(" ORDER BY ");
        sql.push_str(
            &query
                .order
                .iter()
                .map(|order| {
                    format!(
                        "{} {}",
                        quote_parts(&order.field.parts),
                        if order.descending { "DESC" } else { "ASC" }
                    )
                })
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    if let Some(limit) = query.limit {
        sql.push_str(&format!(" LIMIT {limit}"));
    }
    if let Some(offset) = query.offset {
        sql.push_str(&format!(" OFFSET {offset}"));
    }
    sql
}

fn operand(operand: &QueryOperand, dialect: QueryDialect) -> String {
    match operand {
        QueryOperand::Identifier(identifier) => quote_parts(&identifier.parts),
        QueryOperand::Value(QueryValue::Parameter(index)) => match dialect {
            QueryDialect::Postgres => format!("${index}"),
            QueryDialect::D1 => format!("?{index}"),
        },
        QueryOperand::Value(QueryValue::Null) => "NULL".to_string(),
        QueryOperand::Value(QueryValue::Bool(value)) => match dialect {
            QueryDialect::Postgres => value.to_string().to_ascii_uppercase(),
            QueryDialect::D1 => usize::from(*value).to_string(),
        },
        QueryOperand::Value(QueryValue::Number(value)) => value.clone(),
        QueryOperand::Value(QueryValue::String(value)) => {
            format!("'{}'", value.replace('\'', "''"))
        }
    }
}

fn quote_parts(parts: &[String]) -> String {
    parts
        .iter()
        .map(|part| quote(part))
        .collect::<Vec<_>>()
        .join(".")
}

fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::{QueryDialect, render_select};
    use crate::parse_select;

    #[test]
    fn quotes_case_sensitive_postgres_identifiers_and_aliases() {
        let query = parse_select("SELECT posts.id, users.name AS authorName FROM posts JOIN users ON posts.authorId = users.id WHERE posts.authorId = ?1").expect("query");
        assert_eq!(
            render_select(&query, QueryDialect::Postgres),
            "SELECT \"posts\".\"id\" AS \"id\", \"users\".\"name\" AS \"authorName\" FROM \"posts\" JOIN \"users\" ON \"posts\".\"authorId\" = \"users\".\"id\" WHERE \"posts\".\"authorId\" = $1"
        );
    }
}
