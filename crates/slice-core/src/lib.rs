use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
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
    #[error("unknown field path at byte {offset}: {path}")]
    UnknownField { path: String, offset: usize },
    #[error("operator {operator:?} is not valid for {path} ({value_type:?}) at byte {offset}")]
    InvalidOperator {
        path: String,
        operator: Operator,
        value_type: ValueType,
        offset: usize,
    },
    #[error("literal {literal:?} is not valid for {path} ({value_type:?}) at byte {offset}")]
    InvalidLiteral {
        path: String,
        literal: Literal,
        value_type: ValueType,
        offset: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    clauses: Vec<Clause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    path: Vec<String>,
    path_offset: usize,
    op: Operator,
    operator_offset: usize,
    literal: Literal,
    literal_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    Eq,
    Ne,
    Has,
    Contains,
    Gt,
    Ge,
    Lt,
    Le,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    List(Vec<Literal>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    String,
    Number,
    Bool,
    Array,
    Object,
    Null,
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSpec {
    value_type: ValueType,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FieldCatalog {
    fields: BTreeMap<String, FieldSpec>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledExpr {
    expr: Expr,
    explain: ExplainReport,
    requirements: RequirementReport,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExplainReport {
    pub schema: String,
    pub clause_count: usize,
    pub fields: Vec<ExplainField>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExplainField {
    pub path: String,
    pub value_type: ValueType,
    pub operator: Operator,
    pub literal: Literal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequirementReport {
    pub schema: String,
    pub field_count: usize,
    pub fields: Vec<RequirementField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequirementField {
    pub path: String,
    pub value_type: ValueType,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DiagnosticReport {
    pub schema: String,
    pub kind: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<Operator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_type: Option<ValueType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal: Option<Literal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_operators: Option<Vec<Operator>>,
}

pub fn parse(source: &str) -> Result<Expr, SliceError> {
    Parser::new(source).parse()
}

pub fn compile(source: &str, catalog: &FieldCatalog) -> Result<CompiledExpr, SliceError> {
    let expr = parse(source)?;
    expr.validate(catalog)?;
    let explain = expr.explain(catalog)?;
    let requirements = expr.requirements(catalog)?;
    Ok(CompiledExpr {
        expr,
        explain,
        requirements,
    })
}

impl SliceError {
    pub fn diagnostic(&self) -> DiagnosticReport {
        match self {
            SliceError::Expected { expected, offset } => DiagnosticReport {
                schema: "slice.diagnostic.v1".to_string(),
                kind: "expected".to_string(),
                message: self.to_string(),
                offset: Some(*offset),
                expected: Some((*expected).to_string()),
                token: None,
                path: None,
                operator: None,
                value_type: None,
                literal: None,
                allowed_operators: None,
            },
            SliceError::UnexpectedToken { token, offset } => DiagnosticReport {
                schema: "slice.diagnostic.v1".to_string(),
                kind: "unexpected_token".to_string(),
                message: self.to_string(),
                offset: Some(*offset),
                expected: None,
                token: Some(token.clone()),
                path: None,
                operator: None,
                value_type: None,
                literal: None,
                allowed_operators: None,
            },
            SliceError::UnsupportedOperator { operator, offset } => DiagnosticReport {
                schema: "slice.diagnostic.v1".to_string(),
                kind: "unsupported_operator".to_string(),
                message: self.to_string(),
                offset: Some(*offset),
                expected: Some("operator".to_string()),
                token: Some(operator.clone()),
                path: None,
                operator: None,
                value_type: None,
                literal: None,
                allowed_operators: Some(all_operators()),
            },
            SliceError::TrailingInput { token, offset } => DiagnosticReport {
                schema: "slice.diagnostic.v1".to_string(),
                kind: "trailing_input".to_string(),
                message: self.to_string(),
                offset: Some(*offset),
                expected: None,
                token: Some(token.clone()),
                path: None,
                operator: None,
                value_type: None,
                literal: None,
                allowed_operators: None,
            },
            SliceError::UnknownField { path, offset } => DiagnosticReport {
                schema: "slice.diagnostic.v1".to_string(),
                kind: "unknown_field".to_string(),
                message: self.to_string(),
                offset: Some(*offset),
                expected: Some("known catalog field path".to_string()),
                token: None,
                path: Some(path.clone()),
                operator: None,
                value_type: None,
                literal: None,
                allowed_operators: None,
            },
            SliceError::InvalidOperator {
                path,
                operator,
                value_type,
                offset,
            } => DiagnosticReport {
                schema: "slice.diagnostic.v1".to_string(),
                kind: "invalid_operator".to_string(),
                message: self.to_string(),
                offset: Some(*offset),
                expected: Some("operator compatible with field type".to_string()),
                token: None,
                path: Some(path.clone()),
                operator: Some(*operator),
                value_type: Some(*value_type),
                literal: None,
                allowed_operators: Some(operators_for_type(*value_type)),
            },
            SliceError::InvalidLiteral {
                path,
                literal,
                value_type,
                offset,
            } => DiagnosticReport {
                schema: "slice.diagnostic.v1".to_string(),
                kind: "invalid_literal".to_string(),
                message: self.to_string(),
                offset: Some(*offset),
                expected: Some(format!("{value_type:?} literal")),
                token: None,
                path: Some(path.clone()),
                operator: None,
                value_type: Some(*value_type),
                literal: Some(literal.clone()),
                allowed_operators: None,
            },
        }
    }
}

impl FieldSpec {
    pub fn new(value_type: ValueType) -> Self {
        Self { value_type }
    }

    pub fn value_type(&self) -> ValueType {
        self.value_type
    }
}

impl FieldCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, path: impl Into<String>, value_type: ValueType) -> &mut Self {
        self.fields.insert(path.into(), FieldSpec::new(value_type));
        self
    }

    pub fn get(&self, path: &str) -> Option<&FieldSpec> {
        self.fields.get(path)
    }
}

impl Expr {
    pub fn matches(&self, value: &Value) -> bool {
        self.clauses.iter().all(|clause| clause.matches(value))
    }

    pub fn clauses(&self) -> &[Clause] {
        &self.clauses
    }

    pub fn validate(&self, catalog: &FieldCatalog) -> Result<(), SliceError> {
        for clause in &self.clauses {
            clause.validate(catalog)?;
        }
        Ok(())
    }

    pub fn explain(&self, catalog: &FieldCatalog) -> Result<ExplainReport, SliceError> {
        let fields = self
            .clauses
            .iter()
            .map(|clause| clause.explain(catalog))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ExplainReport {
            schema: "slice.explain.v1".to_string(),
            clause_count: fields.len(),
            fields,
        })
    }

    pub fn requirements(&self, catalog: &FieldCatalog) -> Result<RequirementReport, SliceError> {
        let mut fields = BTreeMap::<String, RequirementField>::new();
        for clause in &self.clauses {
            let requirement = clause.requirement(catalog)?;
            fields
                .entry(requirement.path.clone())
                .or_insert(requirement);
        }
        let fields = fields.into_values().collect::<Vec<_>>();
        Ok(RequirementReport {
            schema: "slice.requirements.v1".to_string(),
            field_count: fields.len(),
            fields,
        })
    }
}

impl CompiledExpr {
    pub fn matches(&self, value: &Value) -> bool {
        self.expr.matches(value)
    }

    pub fn expr(&self) -> &Expr {
        &self.expr
    }

    pub fn explain(&self) -> &ExplainReport {
        &self.explain
    }

    pub fn requirements(&self) -> &RequirementReport {
        &self.requirements
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
            Operator::Gt => numeric_compare(value, &self.literal, |left, right| left > right),
            Operator::Ge => numeric_compare(value, &self.literal, |left, right| left >= right),
            Operator::Lt => numeric_compare(value, &self.literal, |left, right| left < right),
            Operator::Le => numeric_compare(value, &self.literal, |left, right| left <= right),
            Operator::In => value_in(value, &self.literal),
            Operator::NotIn => !value_in(value, &self.literal),
            Operator::IsNull => matches!(value, Value::Null),
            Operator::IsNotNull => !matches!(value, Value::Null),
        }
    }

    fn validate(&self, catalog: &FieldCatalog) -> Result<(), SliceError> {
        let path = self.path.join(".");
        let Some(field) = catalog.get(&path) else {
            return Err(SliceError::UnknownField {
                path,
                offset: self.path_offset,
            });
        };

        if !operator_valid_for_type(self.op, field.value_type) {
            return Err(SliceError::InvalidOperator {
                path,
                operator: self.op,
                value_type: field.value_type,
                offset: self.operator_offset,
            });
        }

        if !matches!(self.op, Operator::IsNull | Operator::IsNotNull)
            && !literal_valid_for_type(&self.literal, field.value_type)
        {
            return Err(SliceError::InvalidLiteral {
                path,
                literal: self.literal.clone(),
                value_type: field.value_type,
                offset: self.literal_offset,
            });
        }

        Ok(())
    }

    fn explain(&self, catalog: &FieldCatalog) -> Result<ExplainField, SliceError> {
        let path = self.path.join(".");
        let Some(field) = catalog.get(&path) else {
            return Err(SliceError::UnknownField {
                path,
                offset: self.path_offset,
            });
        };
        Ok(ExplainField {
            path,
            value_type: field.value_type,
            operator: self.op,
            literal: self.literal.clone(),
        })
    }

    fn requirement(&self, catalog: &FieldCatalog) -> Result<RequirementField, SliceError> {
        let path = self.path.join(".");
        let Some(field) = catalog.get(&path) else {
            return Err(SliceError::UnknownField {
                path,
                offset: self.path_offset,
            });
        };
        Ok(RequirementField {
            path,
            value_type: field.value_type,
        })
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

fn value_in(value: &Value, literal: &Literal) -> bool {
    match literal {
        Literal::List(items) => items.iter().any(|item| literal_eq(value, item)),
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

fn numeric_compare(
    value: &Value,
    literal: &Literal,
    compare: impl FnOnce(f64, f64) -> bool,
) -> bool {
    match (value.as_f64(), literal) {
        (Some(left), Literal::Number(right)) => compare(left, *right),
        _ => false,
    }
}

fn operator_valid_for_type(operator: Operator, value_type: ValueType) -> bool {
    match operator {
        Operator::Eq | Operator::Ne => matches!(
            value_type,
            ValueType::String
                | ValueType::Number
                | ValueType::Bool
                | ValueType::Null
                | ValueType::Any
        ),
        Operator::Has => matches!(
            value_type,
            ValueType::Array | ValueType::Object | ValueType::String | ValueType::Any
        ),
        Operator::Contains => matches!(
            value_type,
            ValueType::Array | ValueType::Object | ValueType::String | ValueType::Any
        ),
        Operator::Gt | Operator::Ge | Operator::Lt | Operator::Le => {
            matches!(value_type, ValueType::Number | ValueType::Any)
        }
        Operator::In | Operator::NotIn => matches!(
            value_type,
            ValueType::String
                | ValueType::Number
                | ValueType::Bool
                | ValueType::Null
                | ValueType::Any
        ),
        Operator::IsNull | Operator::IsNotNull => true,
    }
}

fn all_operators() -> Vec<Operator> {
    vec![
        Operator::Eq,
        Operator::Ne,
        Operator::Has,
        Operator::Contains,
        Operator::Gt,
        Operator::Ge,
        Operator::Lt,
        Operator::Le,
        Operator::In,
        Operator::NotIn,
        Operator::IsNull,
        Operator::IsNotNull,
    ]
}

fn operators_for_type(value_type: ValueType) -> Vec<Operator> {
    all_operators()
        .into_iter()
        .filter(|operator| operator_valid_for_type(*operator, value_type))
        .collect()
}

fn literal_valid_for_type(literal: &Literal, value_type: ValueType) -> bool {
    if matches!(value_type, ValueType::Any) {
        return true;
    }

    match literal {
        Literal::String(_) => matches!(
            value_type,
            ValueType::String | ValueType::Array | ValueType::Object
        ),
        Literal::Number(_) => matches!(value_type, ValueType::Number),
        Literal::Bool(_) => matches!(value_type, ValueType::Bool),
        Literal::Null => matches!(value_type, ValueType::Null),
        Literal::List(items) => items
            .iter()
            .all(|item| literal_valid_for_type(item, value_type)),
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
    LeftBracket,
    RightBracket,
    Comma,
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
        let (path, path_offset) = self.parse_path()?;
        let op_token = self.expect_ident("operator")?;
        let (op, literal, literal_offset) = match op_token.0.as_str() {
            "eq" => self.parse_standard_clause_literal(Operator::Eq)?,
            "ne" => self.parse_standard_clause_literal(Operator::Ne)?,
            "has" => self.parse_standard_clause_literal(Operator::Has)?,
            "contains" => self.parse_standard_clause_literal(Operator::Contains)?,
            "gt" => self.parse_standard_clause_literal(Operator::Gt)?,
            "ge" => self.parse_standard_clause_literal(Operator::Ge)?,
            "lt" => self.parse_standard_clause_literal(Operator::Lt)?,
            "le" => self.parse_standard_clause_literal(Operator::Le)?,
            "in" => {
                let (literal, literal_offset) = self.parse_literal_list()?;
                (Operator::In, literal, literal_offset)
            }
            "not" => {
                self.expect_specific_ident("in")?;
                let (literal, literal_offset) = self.parse_literal_list()?;
                (Operator::NotIn, literal, literal_offset)
            }
            "is" => {
                let Some(next) = self.next() else {
                    return Err(SliceError::Expected {
                        expected: "null or not null",
                        offset: self.source.len(),
                    });
                };
                match next.kind {
                    TokenKind::Null => (Operator::IsNull, Literal::Null, next.offset),
                    TokenKind::Ident(word) if word == "not" => {
                        let null_offset = self.expect_null()?;
                        (Operator::IsNotNull, Literal::Null, null_offset)
                    }
                    _ => {
                        return Err(SliceError::UnexpectedToken {
                            token: token_text(&next),
                            offset: next.offset,
                        })
                    }
                }
            }
            operator => {
                return Err(SliceError::UnsupportedOperator {
                    operator: operator.to_string(),
                    offset: op_token.1,
                })
            }
        };
        Ok(Clause {
            path,
            path_offset,
            op,
            operator_offset: op_token.1,
            literal,
            literal_offset,
        })
    }

    fn parse_standard_clause_literal(
        &mut self,
        op: Operator,
    ) -> Result<(Operator, Literal, usize), SliceError> {
        let (literal, literal_offset) = self.parse_literal()?;
        Ok((op, literal, literal_offset))
    }

    fn parse_path(&mut self) -> Result<(Vec<String>, usize), SliceError> {
        let first = self.expect_ident("field path")?;
        let mut path = vec![first.0];
        while matches!(self.peek().map(|token| &token.kind), Some(TokenKind::Dot)) {
            self.cursor += 1;
            path.push(self.expect_ident("field path segment")?.0);
        }
        Ok((path, first.1))
    }

    fn parse_literal(&mut self) -> Result<(Literal, usize), SliceError> {
        let Some(token) = self.next() else {
            return Err(SliceError::Expected {
                expected: "literal",
                offset: self.source.len(),
            });
        };
        let offset = token.offset;
        match token.kind {
            TokenKind::String(value) => Ok((Literal::String(value), offset)),
            TokenKind::Number(value) => Ok((Literal::Number(value), offset)),
            TokenKind::Bool(value) => Ok((Literal::Bool(value), offset)),
            TokenKind::Null => Ok((Literal::Null, offset)),
            _ => Err(SliceError::UnexpectedToken {
                token: token_text(&token),
                offset: token.offset,
            }),
        }
    }

    fn parse_literal_list(&mut self) -> Result<(Literal, usize), SliceError> {
        let Some(open) = self.next() else {
            return Err(SliceError::Expected {
                expected: "literal list",
                offset: self.source.len(),
            });
        };
        if !matches!(open.kind, TokenKind::LeftBracket) {
            return Err(SliceError::UnexpectedToken {
                token: token_text(&open),
                offset: open.offset,
            });
        }

        let mut items = Vec::new();
        if matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::RightBracket)
        ) {
            let close = self.next().expect("peeked token must exist");
            return Ok((Literal::List(items), close.offset));
        }

        loop {
            let (literal, _) = self.parse_literal()?;
            items.push(literal);
            let Some(separator) = self.next() else {
                return Err(SliceError::Expected {
                    expected: "',' or ']'",
                    offset: self.source.len(),
                });
            };
            match separator.kind {
                TokenKind::Comma => {}
                TokenKind::RightBracket => break,
                _ => {
                    return Err(SliceError::UnexpectedToken {
                        token: token_text(&separator),
                        offset: separator.offset,
                    })
                }
            }
        }
        Ok((Literal::List(items), open.offset))
    }

    fn expect_specific_ident(
        &mut self,
        expected: &'static str,
    ) -> Result<(String, usize), SliceError> {
        let token = self.expect_ident(expected)?;
        if token.0 == expected {
            Ok(token)
        } else {
            Err(SliceError::UnexpectedToken {
                token: token.0,
                offset: token.1,
            })
        }
    }

    fn expect_null(&mut self) -> Result<usize, SliceError> {
        let Some(token) = self.next() else {
            return Err(SliceError::Expected {
                expected: "null",
                offset: self.source.len(),
            });
        };
        match token.kind {
            TokenKind::Null => Ok(token.offset),
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
        if ch == '[' {
            tokens.push(Token {
                kind: TokenKind::LeftBracket,
                offset,
            });
            continue;
        }
        if ch == ']' {
            tokens.push(Token {
                kind: TokenKind::RightBracket,
                offset,
            });
            continue;
        }
        if ch == ',' {
            tokens.push(Token {
                kind: TokenKind::Comma,
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
            if next.is_whitespace() {
                break;
            }
            if matches!(*next, '[' | ']' | ',') {
                break;
            }
            if *next == '.' {
                let decimal_point = raw.chars().all(|c| c.is_ascii_digit()) && !raw.contains('.');
                if decimal_point {
                    raw.push(*next);
                    chars.next();
                    continue;
                }
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
        TokenKind::LeftBracket => "[".to_string(),
        TokenKind::RightBracket => "]".to_string(),
        TokenKind::Comma => ",".to_string(),
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
    fn matches_numeric_comparison_clause() {
        let expr = parse("stats.ppg ge 0.8").unwrap();

        assert!(expr.matches(&json!({"stats": {"ppg": 0.82}})));
        assert!(!expr.matches(&json!({"stats": {"ppg": 0.71}})));
    }

    #[test]
    fn matches_in_and_not_in_clauses() {
        let expr =
            parse("repo in ['CROP', 'PROOF'] and status not in ['blocked', 'stale']").unwrap();

        assert!(expr.matches(&json!({"repo": "CROP", "status": "ready"})));
        assert!(!expr.matches(&json!({"repo": "PEBBLE", "status": "ready"})));
        assert!(!expr.matches(&json!({"repo": "PROOF", "status": "blocked"})));
    }

    #[test]
    fn matches_numeric_and_bool_membership_clauses() {
        let expr = parse("score in [1, 2.5] and active in [true]").unwrap();

        assert!(expr.matches(&json!({"score": 2.5, "active": true})));
        assert!(!expr.matches(&json!({"score": 3, "active": true})));
        assert!(!expr.matches(&json!({"score": 1, "active": false})));
    }

    #[test]
    fn matches_null_query_clauses() {
        let is_null = parse("metadata.owner is null").unwrap();
        let is_not_null = parse("metadata.owner is not null").unwrap();

        assert!(is_null.matches(&json!({"metadata": {"owner": null}})));
        assert!(!is_null.matches(&json!({"metadata": {"owner": "docs"}})));
        assert!(is_not_null.matches(&json!({"metadata": {"owner": "docs"}})));
        assert!(!is_not_null.matches(&json!({"metadata": {"owner": null}})));
        assert!(!is_not_null.matches(&json!({"metadata": {}})));
    }

    #[test]
    fn validates_against_field_catalog() {
        let mut catalog = FieldCatalog::new();
        catalog
            .insert("metadata.tags", ValueType::Array)
            .insert("metadata.status", ValueType::String)
            .insert("stats.ppg", ValueType::Number);

        let compiled =
            compile("metadata.tags has 'context' and stats.ppg ge 0.8", &catalog).unwrap();

        assert_eq!(compiled.explain().clause_count, 2);
        assert_eq!(compiled.explain().fields[1].path, "stats.ppg");
        assert_eq!(compiled.explain().fields[1].value_type, ValueType::Number);
        assert_eq!(compiled.requirements().field_count, 2);
        assert_eq!(compiled.requirements().fields[0].path, "metadata.tags");
        assert_eq!(compiled.requirements().fields[1].path, "stats.ppg");

        assert!(compiled.matches(&json!({
            "metadata": {"tags": ["context"], "status": "ready"},
            "stats": {"ppg": 0.82}
        })));
    }

    #[test]
    fn rejects_unknown_catalog_field() {
        let catalog = FieldCatalog::new();
        let err = compile("metadata.status eq 'ready'", &catalog).unwrap_err();

        assert_eq!(
            err,
            SliceError::UnknownField {
                path: "metadata.status".to_string(),
                offset: 0
            }
        );
    }

    #[test]
    fn rejects_invalid_operator_for_catalog_type() {
        let mut catalog = FieldCatalog::new();
        catalog.insert("metadata.status", ValueType::String);

        let err = compile("metadata.status ge 1", &catalog).unwrap_err();

        assert_eq!(
            err,
            SliceError::InvalidOperator {
                path: "metadata.status".to_string(),
                operator: Operator::Ge,
                value_type: ValueType::String,
                offset: 16
            }
        );
    }

    #[test]
    fn reports_machine_readable_diagnostics() {
        let mut catalog = FieldCatalog::new();
        catalog.insert("metadata.status", ValueType::String);

        let err = compile("metadata.status ge 1", &catalog).unwrap_err();
        let diagnostic = err.diagnostic();

        assert_eq!(diagnostic.schema, "slice.diagnostic.v1");
        assert_eq!(diagnostic.kind, "invalid_operator");
        assert_eq!(diagnostic.offset, Some(16));
        assert_eq!(diagnostic.path, Some("metadata.status".to_string()));
        assert_eq!(diagnostic.operator, Some(Operator::Ge));
        assert_eq!(diagnostic.value_type, Some(ValueType::String));
        assert_eq!(
            diagnostic.allowed_operators,
            Some(vec![
                Operator::Eq,
                Operator::Ne,
                Operator::Has,
                Operator::Contains,
                Operator::In,
                Operator::NotIn,
                Operator::IsNull,
                Operator::IsNotNull,
            ])
        );
    }

    #[test]
    fn validates_membership_and_null_operators_against_catalog() {
        let mut catalog = FieldCatalog::new();
        catalog
            .insert("repo", ValueType::String)
            .insert("priority", ValueType::Number)
            .insert("owner", ValueType::String);

        let compiled = compile(
            "repo in ['CROP', 'PROOF'] and priority not in [0, 99] and owner is not null",
            &catalog,
        )
        .unwrap();

        assert_eq!(compiled.requirements().field_count, 3);
        assert!(compiled.matches(&json!({
            "repo": "CROP",
            "priority": 2,
            "owner": "docs"
        })));
    }

    #[test]
    fn rejects_membership_literals_that_do_not_match_catalog_type() {
        let mut catalog = FieldCatalog::new();
        catalog.insert("repo", ValueType::String);

        let err = compile("repo in ['CROP', 1]", &catalog).unwrap_err();

        assert_eq!(
            err,
            SliceError::InvalidLiteral {
                path: "repo".to_string(),
                literal: Literal::List(vec![
                    Literal::String("CROP".to_string()),
                    Literal::Number(1.0)
                ]),
                value_type: ValueType::String,
                offset: 8
            }
        );
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
