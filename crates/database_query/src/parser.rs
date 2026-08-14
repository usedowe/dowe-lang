use crate::{
    QueryFilter, QueryIdentifier, QueryJoin, QueryOperand, QueryOrder, QueryProjection,
    QueryProjectionValue, QuerySource, QueryValue, SelectQuery,
};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryParseError {
    message: String,
}

impl QueryParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for QueryParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for QueryParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    String(String),
    Number(String),
    Parameter(usize),
    Star,
    Comma,
    Dot,
    Equal,
}

pub fn parse_select(sql: &str) -> Result<SelectQuery, QueryParseError> {
    let tokens = tokenize(sql)?;
    let mut parser = Parser { tokens, index: 0 };
    parser.keyword("select")?;
    let projections = parser.projections()?;
    parser.keyword("from")?;
    let source = parser.source()?;
    let mut joins = Vec::new();
    while parser.take_keyword("join") {
        let join_source = parser.source()?;
        parser.keyword("on")?;
        let left = parser.identifier()?;
        parser.equal()?;
        let right = parser.identifier()?;
        joins.push(QueryJoin {
            source: join_source,
            left,
            right,
        });
    }
    let mut filters = Vec::new();
    if parser.take_keyword("where") {
        loop {
            let left = parser.identifier()?;
            parser.equal()?;
            let right = parser.operand()?;
            filters.push(QueryFilter { left, right });
            if !parser.take_keyword("and") {
                break;
            }
        }
    }
    let mut order = Vec::new();
    if parser.take_keyword("order") {
        parser.keyword("by")?;
        loop {
            let field = parser.identifier()?;
            let descending = if parser.take_keyword("desc") {
                true
            } else {
                parser.take_keyword("asc");
                false
            };
            order.push(QueryOrder { field, descending });
            if !parser.take(&Token::Comma) {
                break;
            }
        }
    }
    let limit = if parser.take_keyword("limit") {
        Some(parser.unsigned("limit")?)
    } else {
        None
    };
    let offset = if parser.take_keyword("offset") {
        Some(parser.unsigned("offset")?)
    } else {
        None
    };
    if parser.index != parser.tokens.len() {
        return Err(QueryParseError::new(
            "query contains unsupported trailing syntax",
        ));
    }
    let mut output_names = BTreeSet::new();
    for projection in &projections {
        if let Some(name) = projection.output_name()
            && !output_names.insert(name)
        {
            return Err(QueryParseError::new(format!(
                "query projection output `{name}` is ambiguous; use `AS` aliases"
            )));
        }
    }
    Ok(SelectQuery {
        projections,
        source,
        joins,
        filters,
        order,
        limit,
        offset,
    })
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn projections(&mut self) -> Result<Vec<QueryProjection>, QueryParseError> {
        let mut projections = Vec::new();
        loop {
            let value = if self.take(&Token::Star) {
                QueryProjectionValue::Wildcard
            } else {
                QueryProjectionValue::Identifier(self.identifier()?)
            };
            let alias = if self.take_keyword("as") {
                Some(self.word("projection alias")?)
            } else {
                None
            };
            if matches!(value, QueryProjectionValue::Wildcard) && alias.is_some() {
                return Err(QueryParseError::new("query wildcard cannot use an alias"));
            }
            projections.push(QueryProjection { value, alias });
            if !self.take(&Token::Comma) {
                break;
            }
        }
        if projections.is_empty() {
            return Err(QueryParseError::new(
                "query must project at least one field",
            ));
        }
        Ok(projections)
    }

    fn source(&mut self) -> Result<QuerySource, QueryParseError> {
        let table = self.word("table")?;
        let alias = if self.take_keyword("as") {
            Some(self.word("table alias")?)
        } else {
            None
        };
        Ok(QuerySource { table, alias })
    }

    fn identifier(&mut self) -> Result<QueryIdentifier, QueryParseError> {
        let mut parts = vec![self.word("identifier")?];
        while self.take(&Token::Dot) {
            parts.push(self.word("identifier")?);
        }
        Ok(QueryIdentifier { parts })
    }

    fn operand(&mut self) -> Result<QueryOperand, QueryParseError> {
        let Some(token) = self.tokens.get(self.index).cloned() else {
            return Err(QueryParseError::new("query equality value is missing"));
        };
        match token {
            Token::Parameter(index) => {
                self.index += 1;
                Ok(QueryOperand::Value(QueryValue::Parameter(index)))
            }
            Token::String(value) => {
                self.index += 1;
                Ok(QueryOperand::Value(QueryValue::String(value)))
            }
            Token::Number(value) => {
                self.index += 1;
                Ok(QueryOperand::Value(QueryValue::Number(value)))
            }
            Token::Word(value) if value.eq_ignore_ascii_case("null") => {
                self.index += 1;
                Ok(QueryOperand::Value(QueryValue::Null))
            }
            Token::Word(value) if value.eq_ignore_ascii_case("true") => {
                self.index += 1;
                Ok(QueryOperand::Value(QueryValue::Bool(true)))
            }
            Token::Word(value) if value.eq_ignore_ascii_case("false") => {
                self.index += 1;
                Ok(QueryOperand::Value(QueryValue::Bool(false)))
            }
            Token::Word(_) => self.identifier().map(QueryOperand::Identifier),
            _ => Err(QueryParseError::new("query equality value is invalid")),
        }
    }

    fn unsigned(&mut self, name: &str) -> Result<usize, QueryParseError> {
        let Some(Token::Number(value)) = self.tokens.get(self.index) else {
            return Err(QueryParseError::new(format!(
                "query {name} must be a non-negative integer"
            )));
        };
        let value = value.parse::<usize>().map_err(|_| {
            QueryParseError::new(format!("query {name} must be a non-negative integer"))
        })?;
        self.index += 1;
        Ok(value)
    }

    fn word(&mut self, name: &str) -> Result<String, QueryParseError> {
        let Some(Token::Word(value)) = self.tokens.get(self.index) else {
            return Err(QueryParseError::new(format!("query {name} is missing")));
        };
        let value = value.clone();
        self.index += 1;
        Ok(value)
    }

    fn keyword(&mut self, keyword: &str) -> Result<(), QueryParseError> {
        if self.take_keyword(keyword) {
            Ok(())
        } else {
            Err(QueryParseError::new(format!(
                "query must declare `{keyword}`"
            )))
        }
    }

    fn take_keyword(&mut self, keyword: &str) -> bool {
        if self.tokens.get(self.index).is_some_and(
            |token| matches!(token, Token::Word(value) if value.eq_ignore_ascii_case(keyword)),
        ) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn equal(&mut self) -> Result<(), QueryParseError> {
        if self.take(&Token::Equal) {
            Ok(())
        } else {
            Err(QueryParseError::new("query predicate must use equality"))
        }
    }

    fn take(&mut self, expected: &Token) -> bool {
        if self.tokens.get(self.index) == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }
}

fn tokenize(sql: &str) -> Result<Vec<Token>, QueryParseError> {
    let chars = sql.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut index = 0usize;
    while index < chars.len() {
        let value = chars[index];
        if value.is_whitespace() {
            index += 1;
            continue;
        }
        match value {
            '*' => {
                tokens.push(Token::Star);
                index += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                index += 1;
            }
            '.' => {
                tokens.push(Token::Dot);
                index += 1;
            }
            '=' => {
                tokens.push(Token::Equal);
                index += 1;
            }
            '?' => {
                let start = index + 1;
                let end = take_while(&chars, start, |value| value.is_ascii_digit());
                if end == start {
                    return Err(QueryParseError::new("query parameter is invalid"));
                }
                let parameter = chars[start..end]
                    .iter()
                    .collect::<String>()
                    .parse::<usize>()
                    .map_err(|_| QueryParseError::new("query parameter is invalid"))?;
                if parameter == 0 {
                    return Err(QueryParseError::new("query parameters start at `?1`"));
                }
                tokens.push(Token::Parameter(parameter));
                index = end;
            }
            '\'' | '"' => {
                let quote = value;
                index += 1;
                let mut output = String::new();
                let mut closed = false;
                while index < chars.len() {
                    let value = chars[index];
                    index += 1;
                    if value == quote {
                        if index < chars.len() && chars[index] == quote {
                            output.push(quote);
                            index += 1;
                        } else {
                            closed = true;
                            break;
                        }
                    } else if value == '\\' && index < chars.len() {
                        output.push(chars[index]);
                        index += 1;
                    } else {
                        output.push(value);
                    }
                }
                if !closed {
                    return Err(QueryParseError::new("query string is unterminated"));
                }
                tokens.push(Token::String(output));
            }
            value if value.is_ascii_digit() || value == '-' => {
                let end = take_while(&chars, index + 1, |value| {
                    value.is_ascii_digit() || value == '.'
                });
                tokens.push(Token::Number(chars[index..end].iter().collect()));
                index = end;
            }
            value if value.is_ascii_alphabetic() || value == '_' => {
                let end = take_while(&chars, index + 1, |value| {
                    value.is_ascii_alphanumeric() || value == '_'
                });
                tokens.push(Token::Word(chars[index..end].iter().collect()));
                index = end;
            }
            _ => {
                return Err(QueryParseError::new(format!(
                    "query contains unsupported character `{value}`"
                )));
            }
        }
    }
    Ok(tokens)
}

fn take_while<F>(chars: &[char], mut index: usize, predicate: F) -> usize
where
    F: Fn(char) -> bool,
{
    while index < chars.len() && predicate(chars[index]) {
        index += 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::parse_select;

    #[test]
    fn parses_documented_relations_and_paging_subset() {
        let query = parse_select("SELECT users.name, roles.name AS roleName FROM users JOIN user_roles ON user_roles.userId = users.id JOIN roles ON user_roles.roleId = roles.id WHERE users.id = ?1 ORDER BY roleName DESC LIMIT 20 OFFSET 10").expect("query");
        assert_eq!(query.joins.len(), 2);
        assert_eq!(query.projections[1].output_name(), Some("roleName"));
        assert_eq!(query.limit, Some(20));
        assert_eq!(query.offset, Some(10));
        query.validate_parameters(1).expect("parameters");
    }

    #[test]
    fn rejects_ambiguous_projection_names() {
        let error = parse_select(
            "SELECT users.name, roles.name FROM users JOIN roles ON users.roleId = roles.id",
        )
        .expect_err("ambiguous");
        assert!(error.to_string().contains("AS"));
    }

    #[test]
    fn rejects_queries_outside_the_portable_subset() {
        let error = parse_select("SELECT COUNT(*) FROM users").expect_err("unsupported query");
        assert!(error.to_string().contains("unsupported"));
    }
}
