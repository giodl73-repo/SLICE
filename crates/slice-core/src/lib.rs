use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SliceError {
    #[error("expected {expected} at byte {offset}")]
    Expected {
        expected: &'static str,
        offset: usize,
    },
    #[error("unexpected token at byte {offset}: {token}")]
    UnexpectedToken { token: String, offset: usize },
    #[error("unsupported operator at byte {offset}: {operator}")]
    UnsupportedOperator { operator: String, offset: usize },
    #[error("trailing input at byte {offset}: {token}")]
    TrailingInput { token: String, offset: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    clauses: Vec<Clause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    path: Vec<String>,
    op: Operator,
    literal: Literal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Eq,
    Ne,
    Has,
    Contains,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

pub fn parse(source: &str) -> Result<Expr, SliceError> {
    Parser::new(source).parse()
}

impl Expr {
    pub fn matches(&self, value: &Value) -> bool {
        self.clauses.iter().all(|clause| clause.matches(value))
    }

    pub fn clauses(&self) -> &[Clause] {
        &self.clauses
    }
}

impl Clause {
    pub fn path(&self) -> &[String] {
        &self.path
    }

    pub fn op(&self) -> Operator {
        self.op
    }

    pub fn literal(&self) -> &Literal {
        &self.literal
    }

    fn matches(&self, root: &Value) -> bool {
        let Some(value) = lookup_path(root, &self.path) else {
            return false;
        };
        match self.op {
            Operator::Eq => literal_eq(value, &self.literal),
            Operator::Ne => !literal_eq(value, &self.literal),
            Operator::Has => value_has(value, &self.literal),
            Operator::Contains => value_contains(value, &self.literal),
        }
    }
}

fn lookup_path<'a>(mut value: &'a Value, path: &[String]) -> Option<&'a Value> {
    for segment in path {
        value = match value {
            Value::Object(object) => object.get(segment)?,
            _ => return None,
        };
    }
    Some(value)
}

fn literal_eq(value: &Value, literal: &Literal) -> bool {
    match (value, literal) {
        (Value::String(left), Literal::String(right)) => left == right,
        (Value::Number(left), Literal::Number(right)) => left.as_f64() == Some(*right),
        (Value::Bool(left), Literal::Bool(right)) => left == right,
        (Value::Null, Literal::Null) => true,
        _ => false,
    }
}

fn value_has(value: &Value, literal: &Literal) -> bool {
    match value {
        Value::Array(items) => items.iter().any(|item| literal_eq(item, literal)),
        Value::Object(object) => match literal {
            Literal::String(key) => object.contains_key(key),
            _ => false,
        },
        Value::String(text) => match literal {
            Literal::String(needle) => text.split_whitespace().any(|term| term == needle),
            _ => false,
        },
        _ => false,
    }
}

fn value_contains(value: &Value, literal: &Literal) -> bool {
    match value {
        Value::Array(items) => items.iter().any(|item| literal_eq(item, literal)),
        Value::Object(object) => match literal {
            Literal::String(key) => object.contains_key(key),
            _ => false,
        },
        Value::String(text) => match literal {
            Literal::String(needle) => text.contains(needle),
            _ => false,
        },
        _ => false,
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Token {
    kind: TokenKind,
    offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Ident(String),
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    Dot,
}

struct Parser<'a> {
    tokens: Vec<Token>,
    cursor: usize,
    source: &'a str,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            tokens: tokenize(source),
            cursor: 0,
            source,
        }
    }

    fn parse(mut self) -> Result<Expr, SliceError> {
        let mut clauses = vec![self.parse_clause()?];
        while let Some(token) = self.peek() {
            let TokenKind::Ident(word) = &token.kind else {
                return Err(SliceError::TrailingInput {
                    token: token_text(token),
                    offset: token.offset,
                });
            };
            if word != "and" {
                return Err(SliceError::TrailingInput {
                    token: word.clone(),
                    offset: token.offset,
                });
            }
            self.cursor += 1;
            clauses.push(self.parse_clause()?);
        }
        Ok(Expr { clauses })
    }

    fn parse_clause(&mut self) -> Result<Clause, SliceError> {
        let path = self.parse_path()?;
        let op_token = self.expect_ident("operator")?;
        let op = match op_token.0.as_str() {
            "eq" => Operator::Eq,
            "ne" => Operator::Ne,
            "has" => Operator::Has,
            "contains" => Operator::Contains,
            operator => {
                return Err(SliceError::UnsupportedOperator {
                    operator: operator.to_string(),
                    offset: op_token.1,
                })
            }
        };
        let literal = self.parse_literal()?;
        Ok(Clause { path, op, literal })
    }

    fn parse_path(&mut self) -> Result<Vec<String>, SliceError> {
        let mut path = vec![self.expect_ident("field path")?.0];
        while matches!(self.peek().map(|token| &token.kind), Some(TokenKind::Dot)) {
            self.cursor += 1;
            path.push(self.expect_ident("field path segment")?.0);
        }
        Ok(path)
    }

    fn parse_literal(&mut self) -> Result<Literal, SliceError> {
        let Some(token) = self.next() else {
            return Err(SliceError::Expected {
                expected: "literal",
                offset: self.source.len(),
            });
        };
        match token.kind {
            TokenKind::String(value) => Ok(Literal::String(value)),
            TokenKind::Number(value) => Ok(Literal::Number(value)),
            TokenKind::Bool(value) => Ok(Literal::Bool(value)),
            TokenKind::Null => Ok(Literal::Null),
            _ => Err(SliceError::UnexpectedToken {
                token: token_text(&token),
                offset: token.offset,
            }),
        }
    }

    fn expect_ident(&mut self, expected: &'static str) -> Result<(String, usize), SliceError> {
        let Some(token) = self.next() else {
            return Err(SliceError::Expected {
                expected,
                offset: self.source.len(),
            });
        };
        match token.kind {
            TokenKind::Ident(value) => Ok((value, token.offset)),
            _ => Err(SliceError::UnexpectedToken {
                token: token_text(&token),
                offset: token.offset,
            }),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.cursor).cloned();
        self.cursor += usize::from(token.is_some());
        token
    }
}

fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();

    while let Some((offset, ch)) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }
        if ch == '.' {
            tokens.push(Token {
                kind: TokenKind::Dot,
                offset,
            });
            continue;
        }
        if ch == '\'' || ch == '"' {
            let quote = ch;
            let mut value = String::new();
            for (_, next) in chars.by_ref() {
                if next == quote {
                    break;
                }
                value.push(next);
            }
            tokens.push(Token {
                kind: TokenKind::String(value),
                offset,
            });
            continue;
        }
        let mut raw = String::from(ch);
        while let Some((_, next)) = chars.peek() {
            if next.is_whitespace() || *next == '.' {
                break;
            }
            raw.push(*next);
            chars.next();
        }
        let kind = match raw.as_str() {
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            "null" => TokenKind::Null,
            _ => raw
                .parse::<f64>()
                .map(TokenKind::Number)
                .unwrap_or(TokenKind::Ident(raw)),
        };
        tokens.push(Token { kind, offset });
    }

    tokens
}

fn token_text(token: &Token) -> String {
    match &token.kind {
        TokenKind::Ident(value) => value.clone(),
        TokenKind::String(value) => format!("'{value}'"),
        TokenKind::Number(value) => value.to_string(),
        TokenKind::Bool(value) => value.to_string(),
        TokenKind::Null => "null".to_string(),
        TokenKind::Dot => ".".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn matches_equality_clause() {
        let expr = parse("metadata.status eq 'ready'").unwrap();

        assert!(expr.matches(&json!({"metadata": {"status": "ready"}})));
        assert!(!expr.matches(&json!({"metadata": {"status": "draft"}})));
    }

    #[test]
    fn matches_array_has_clause() {
        let expr = parse("metadata.tags has 'context'").unwrap();

        assert!(expr.matches(&json!({"metadata": {"tags": ["context", "query"]}})));
        assert!(!expr.matches(&json!({"metadata": {"tags": ["query"]}})));
    }

    #[test]
    fn matches_conjunction() {
        let expr = parse("metadata.tags has 'context' and metadata.status ne 'draft'").unwrap();

        assert!(expr.matches(&json!({
            "metadata": {"tags": ["context"], "status": "ready"}
        })));
        assert!(!expr.matches(&json!({
            "metadata": {"tags": ["context"], "status": "draft"}
        })));
    }

    #[test]
    fn rejects_unknown_operator() {
        let err = parse("metadata.tags approx 'context'").unwrap_err();

        assert_eq!(
            err,
            SliceError::UnsupportedOperator {
                operator: "approx".to_string(),
                offset: 14
            }
        );
    }
}
