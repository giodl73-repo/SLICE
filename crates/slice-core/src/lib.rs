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
    root: ExprNode,
}

#[derive(Debug, Clone, PartialEq)]
enum ExprNode {
    Clause(Clause),
    All(Vec<ExprNode>),
    Any(Vec<ExprNode>),
    Not(Box<ExprNode>),
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
    Between,
    StartsWith,
    EndsWith,
    HasAny,
    HasAll,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    List(Vec<Literal>),
    Range {
        min: Box<Literal>,
        max: Box<Literal>,
    },
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FoldCatalog {
    fields: BTreeMap<String, FoldFieldSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldFieldSpec {
    value_type: ValueType,
    source: String,
    column: String,
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
    pub tree: ExplainNode,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExplainField {
    pub path: String,
    pub value_type: ValueType,
    pub operator: Operator,
    pub literal: Literal,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExplainNode {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<ExplainField>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ExplainNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParsedExplainReport {
    pub schema: String,
    pub clause_count: usize,
    pub fields: Vec<ParsedExplainField>,
    pub tree: ParsedExplainNode,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParsedExplainField {
    pub path: String,
    pub operator: Operator,
    pub literal: Literal,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParsedExplainNode {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<ParsedExplainField>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ParsedExplainNode>,
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
pub struct FoldPlan {
    pub schema: String,
    pub backend: String,
    pub source_count: usize,
    pub sources: Vec<FoldSourcePlan>,
    pub requirements: RequirementReport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub residual: Option<ParsedExplainNode>,
    pub diagnostics: Vec<FoldDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FoldSourcePlan {
    pub source: String,
    pub predicate: FoldPredicate,
    pub folded_clause_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FoldPredicate {
    pub language: String,
    pub text: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<Literal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FoldDiagnostic {
    pub kind: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<Operator>,
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

impl FoldCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_sqlite(
        &mut self,
        path: impl Into<String>,
        value_type: ValueType,
        source: impl Into<String>,
        column: impl Into<String>,
    ) -> &mut Self {
        self.fields.insert(
            path.into(),
            FoldFieldSpec {
                value_type,
                source: source.into(),
                column: column.into(),
            },
        );
        self
    }

    pub fn get(&self, path: &str) -> Option<&FoldFieldSpec> {
        self.fields.get(path)
    }

    pub fn fields(&self) -> impl Iterator<Item = (&str, &FoldFieldSpec)> {
        self.fields.iter().map(|(path, spec)| (path.as_str(), spec))
    }

    pub fn field_catalog(&self) -> FieldCatalog {
        let mut catalog = FieldCatalog::new();
        for (path, field) in &self.fields {
            catalog.insert(path, field.value_type);
        }
        catalog
    }
}

impl FoldFieldSpec {
    pub fn value_type(&self) -> ValueType {
        self.value_type
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn column(&self) -> &str {
        &self.column
    }
}

impl Expr {
    pub fn matches(&self, value: &Value) -> bool {
        self.root.matches(value)
    }

    pub fn clauses(&self) -> Vec<&Clause> {
        let mut clauses = Vec::new();
        self.root.collect_clauses(&mut clauses);
        clauses
    }

    pub fn validate(&self, catalog: &FieldCatalog) -> Result<(), SliceError> {
        for clause in self.clauses() {
            clause.validate(catalog)?;
        }
        Ok(())
    }

    pub fn explain(&self, catalog: &FieldCatalog) -> Result<ExplainReport, SliceError> {
        let fields = self
            .clauses()
            .into_iter()
            .map(|clause| clause.explain(catalog))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ExplainReport {
            schema: "slice.explain.v1".to_string(),
            clause_count: fields.len(),
            fields,
            tree: self.root.explain(catalog)?,
        })
    }

    pub fn explain_parse(&self) -> ParsedExplainReport {
        let fields = self
            .clauses()
            .into_iter()
            .map(Clause::explain_parse)
            .collect::<Vec<_>>();
        ParsedExplainReport {
            schema: "slice.parse_explain.v1".to_string(),
            clause_count: fields.len(),
            fields,
            tree: self.root.explain_parse(),
        }
    }

    pub fn requirements(&self, catalog: &FieldCatalog) -> Result<RequirementReport, SliceError> {
        let mut fields = BTreeMap::<String, RequirementField>::new();
        for clause in self.clauses() {
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

    pub fn plan_sqlite(&self, catalog: &FoldCatalog) -> Result<FoldPlan, SliceError> {
        self.plan_fold(catalog, FoldBackend::Sqlite)
    }

    pub fn plan_odata(&self, catalog: &FoldCatalog) -> Result<FoldPlan, SliceError> {
        self.plan_fold(catalog, FoldBackend::OData)
    }

    fn plan_fold(
        &self,
        catalog: &FoldCatalog,
        backend: FoldBackend,
    ) -> Result<FoldPlan, SliceError> {
        let field_catalog = catalog.field_catalog();
        self.validate(&field_catalog)?;
        let requirements = self.requirements(&field_catalog)?;
        let mut accumulator = FoldAccumulator::default();
        fold_top_level_and(&self.root, catalog, backend, &mut accumulator);
        let residual = combine_residuals(accumulator.residuals).map(|node| node.explain_parse());
        let sources = accumulator
            .sources
            .into_iter()
            .map(|(source, source_fold)| FoldSourcePlan {
                source,
                predicate: combine_fold_predicates("and", backend, source_fold.predicates),
                folded_clause_count: source_fold.folded_clause_count,
            })
            .collect::<Vec<_>>();

        Ok(FoldPlan {
            schema: "slice.fold.v1".to_string(),
            backend: backend.name().to_string(),
            source_count: sources.len(),
            sources,
            requirements,
            residual,
            diagnostics: accumulator.diagnostics,
        })
    }
}

impl ExprNode {
    fn matches(&self, value: &Value) -> bool {
        match self {
            ExprNode::Clause(clause) => clause.matches(value),
            ExprNode::All(children) => children.iter().all(|child| child.matches(value)),
            ExprNode::Any(children) => children.iter().any(|child| child.matches(value)),
            ExprNode::Not(child) => !child.matches(value),
        }
    }

    fn collect_clauses<'a>(&'a self, clauses: &mut Vec<&'a Clause>) {
        match self {
            ExprNode::Clause(clause) => clauses.push(clause),
            ExprNode::All(children) | ExprNode::Any(children) => {
                for child in children {
                    child.collect_clauses(clauses);
                }
            }
            ExprNode::Not(child) => child.collect_clauses(clauses),
        }
    }

    fn explain(&self, catalog: &FieldCatalog) -> Result<ExplainNode, SliceError> {
        match self {
            ExprNode::Clause(clause) => Ok(ExplainNode {
                kind: "clause".to_string(),
                field: Some(clause.explain(catalog)?),
                children: Vec::new(),
            }),
            ExprNode::All(children) => explain_children("all", children, catalog),
            ExprNode::Any(children) => explain_children("any", children, catalog),
            ExprNode::Not(child) => Ok(ExplainNode {
                kind: "not".to_string(),
                field: None,
                children: vec![child.explain(catalog)?],
            }),
        }
    }

    fn explain_parse(&self) -> ParsedExplainNode {
        match self {
            ExprNode::Clause(clause) => ParsedExplainNode {
                kind: "clause".to_string(),
                field: Some(clause.explain_parse()),
                children: Vec::new(),
            },
            ExprNode::All(children) => parse_explain_children("all", children),
            ExprNode::Any(children) => parse_explain_children("any", children),
            ExprNode::Not(child) => ParsedExplainNode {
                kind: "not".to_string(),
                field: None,
                children: vec![child.explain_parse()],
            },
        }
    }
}

fn explain_children(
    kind: &str,
    children: &[ExprNode],
    catalog: &FieldCatalog,
) -> Result<ExplainNode, SliceError> {
    Ok(ExplainNode {
        kind: kind.to_string(),
        field: None,
        children: children
            .iter()
            .map(|child| child.explain(catalog))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn parse_explain_children(kind: &str, children: &[ExprNode]) -> ParsedExplainNode {
    ParsedExplainNode {
        kind: kind.to_string(),
        field: None,
        children: children
            .iter()
            .map(ExprNode::explain_parse)
            .collect::<Vec<_>>(),
    }
}

#[derive(Debug, Default)]
struct FoldAccumulator {
    sources: BTreeMap<String, SourceFold>,
    residuals: Vec<ExprNode>,
    diagnostics: Vec<FoldDiagnostic>,
}

#[derive(Debug, Default)]
struct SourceFold {
    predicates: Vec<FoldPredicate>,
    folded_clause_count: usize,
}

#[derive(Debug)]
struct SingleFold {
    source: String,
    predicate: FoldPredicate,
    folded_clause_count: usize,
}

#[derive(Debug, Clone, Copy)]
enum FoldBackend {
    Sqlite,
    OData,
}

impl FoldBackend {
    fn name(self) -> &'static str {
        match self {
            FoldBackend::Sqlite => "sqlite",
            FoldBackend::OData => "odata",
        }
    }

    fn language(self) -> &'static str {
        match self {
            FoldBackend::Sqlite => "sql",
            FoldBackend::OData => "odata",
        }
    }

    fn and(self) -> &'static str {
        match self {
            FoldBackend::Sqlite => " AND ",
            FoldBackend::OData => " and ",
        }
    }

    fn or(self) -> &'static str {
        match self {
            FoldBackend::Sqlite => " OR ",
            FoldBackend::OData => " or ",
        }
    }

    fn not(self, text: &str) -> String {
        match self {
            FoldBackend::Sqlite => format!("(NOT {text})"),
            FoldBackend::OData => format!("(not {text})"),
        }
    }
}

fn fold_top_level_and(
    node: &ExprNode,
    catalog: &FoldCatalog,
    backend: FoldBackend,
    accumulator: &mut FoldAccumulator,
) {
    match node {
        ExprNode::All(children) => {
            for child in children {
                fold_top_level_and(child, catalog, backend, accumulator);
            }
        }
        _ => match fold_single_source(node, catalog, backend) {
            Ok(fold) => {
                let source = accumulator.sources.entry(fold.source).or_default();
                source.predicates.push(fold.predicate);
                source.folded_clause_count += fold.folded_clause_count;
            }
            Err(diagnostic) => {
                accumulator.diagnostics.push(diagnostic);
                accumulator.residuals.push(node.clone());
            }
        },
    }
}

fn fold_single_source(
    node: &ExprNode,
    catalog: &FoldCatalog,
    backend: FoldBackend,
) -> Result<SingleFold, FoldDiagnostic> {
    match node {
        ExprNode::Clause(clause) => fold_clause(clause, catalog, backend),
        ExprNode::All(children) => fold_boolean_children("and", children, catalog, backend),
        ExprNode::Any(children) => fold_boolean_children("or", children, catalog, backend),
        ExprNode::Not(child) => {
            let child = fold_single_source(child, catalog, backend)?;
            Ok(SingleFold {
                source: child.source,
                predicate: FoldPredicate {
                    language: backend.language().to_string(),
                    text: backend.not(&child.predicate.text),
                    params: child.predicate.params,
                },
                folded_clause_count: child.folded_clause_count,
            })
        }
    }
}

fn fold_boolean_children(
    kind: &str,
    children: &[ExprNode],
    catalog: &FoldCatalog,
    backend: FoldBackend,
) -> Result<SingleFold, FoldDiagnostic> {
    let mut folded = Vec::<SingleFold>::new();
    for child in children {
        folded.push(fold_single_source(child, catalog, backend)?);
    }

    let Some(first) = folded.first() else {
        return Err(FoldDiagnostic {
            kind: "empty_boolean".to_string(),
            message: "empty boolean expression cannot be folded".to_string(),
            path: None,
            operator: None,
        });
    };
    if folded.iter().any(|fold| fold.source != first.source) {
        return Err(FoldDiagnostic {
            kind: "cross_source_boolean".to_string(),
            message: format!("{kind} expression spans multiple sources and remains residual"),
            path: None,
            operator: None,
        });
    }

    let source = first.source.clone();
    let folded_clause_count = folded
        .iter()
        .map(|fold| fold.folded_clause_count)
        .sum::<usize>();
    let predicates = folded
        .into_iter()
        .map(|fold| fold.predicate)
        .collect::<Vec<_>>();
    Ok(SingleFold {
        source,
        predicate: combine_fold_predicates(kind, backend, predicates),
        folded_clause_count,
    })
}

fn fold_clause(
    clause: &Clause,
    catalog: &FoldCatalog,
    backend: FoldBackend,
) -> Result<SingleFold, FoldDiagnostic> {
    match backend {
        FoldBackend::Sqlite => sqlite_clause(clause, catalog),
        FoldBackend::OData => odata_clause(clause, catalog),
    }
}

fn sqlite_clause(clause: &Clause, catalog: &FoldCatalog) -> Result<SingleFold, FoldDiagnostic> {
    let path = clause.path.join(".");
    let Some(field) = catalog.get(&path) else {
        return Err(FoldDiagnostic {
            kind: "unknown_fold_field".to_string(),
            message: format!("{path} is not present in the fold catalog"),
            path: Some(path),
            operator: Some(clause.op),
        });
    };
    let column = quote_sqlite_identifier(field.column());
    let predicate = match clause.op {
        Operator::Eq if matches!(clause.literal, Literal::Null) => FoldPredicate {
            language: "sql".to_string(),
            text: format!("({column} IS NULL)"),
            params: Vec::new(),
        },
        Operator::Ne if matches!(clause.literal, Literal::Null) => FoldPredicate {
            language: "sql".to_string(),
            text: format!("({column} IS NOT NULL)"),
            params: Vec::new(),
        },
        Operator::Eq => binary_sqlite_predicate(&column, "=", &clause.literal),
        Operator::Ne => binary_sqlite_predicate(&column, "<>", &clause.literal),
        Operator::Gt => binary_sqlite_predicate(&column, ">", &clause.literal),
        Operator::Ge => binary_sqlite_predicate(&column, ">=", &clause.literal),
        Operator::Lt => binary_sqlite_predicate(&column, "<", &clause.literal),
        Operator::Le => binary_sqlite_predicate(&column, "<=", &clause.literal),
        Operator::IsNull => FoldPredicate {
            language: "sql".to_string(),
            text: format!("({column} IS NULL)"),
            params: Vec::new(),
        },
        Operator::IsNotNull => FoldPredicate {
            language: "sql".to_string(),
            text: format!("({column} IS NOT NULL)"),
            params: Vec::new(),
        },
        Operator::In | Operator::NotIn => {
            sqlite_list_predicate(&column, clause.op, &clause.literal).ok_or_else(|| {
                unsupported_fold(
                    FoldBackend::Sqlite,
                    &path,
                    clause.op,
                    "list literal is required",
                )
            })?
        }
        Operator::Between => {
            sqlite_between_predicate(&column, &clause.literal).ok_or_else(|| {
                unsupported_fold(
                    FoldBackend::Sqlite,
                    &path,
                    clause.op,
                    "range literal is required",
                )
            })?
        }
        Operator::Contains | Operator::StartsWith | Operator::EndsWith => {
            sqlite_like_predicate(&column, clause.op, &clause.literal).ok_or_else(|| {
                unsupported_fold(
                    FoldBackend::Sqlite,
                    &path,
                    clause.op,
                    "string literal is required for LIKE folding",
                )
            })?
        }
        Operator::Has | Operator::HasAny | Operator::HasAll => {
            return Err(unsupported_fold(
                FoldBackend::Sqlite,
                &path,
                clause.op,
                "array/object containment stays as a residual local filter",
            ));
        }
    };

    Ok(SingleFold {
        source: field.source().to_string(),
        predicate,
        folded_clause_count: 1,
    })
}

fn binary_sqlite_predicate(column: &str, operator: &str, literal: &Literal) -> FoldPredicate {
    FoldPredicate {
        language: "sql".to_string(),
        text: format!("({column} {operator} ?)"),
        params: vec![literal.clone()],
    }
}

fn sqlite_list_predicate(
    column: &str,
    operator: Operator,
    literal: &Literal,
) -> Option<FoldPredicate> {
    let Literal::List(values) = literal else {
        return None;
    };
    if values.is_empty() {
        return None;
    }
    let placeholders = std::iter::repeat_n("?", values.len())
        .collect::<Vec<_>>()
        .join(", ");
    let sql_operator = match operator {
        Operator::In => "IN",
        Operator::NotIn => "NOT IN",
        _ => return None,
    };
    Some(FoldPredicate {
        language: "sql".to_string(),
        text: format!("({column} {sql_operator} ({placeholders}))"),
        params: values.clone(),
    })
}

fn sqlite_between_predicate(column: &str, literal: &Literal) -> Option<FoldPredicate> {
    let Literal::Range { min, max } = literal else {
        return None;
    };
    Some(FoldPredicate {
        language: "sql".to_string(),
        text: format!("({column} BETWEEN ? AND ?)"),
        params: vec![(**min).clone(), (**max).clone()],
    })
}

fn sqlite_like_predicate(
    column: &str,
    operator: Operator,
    literal: &Literal,
) -> Option<FoldPredicate> {
    let Literal::String(value) = literal else {
        return None;
    };
    let pattern = match operator {
        Operator::Contains => format!("%{value}%"),
        Operator::StartsWith => format!("{value}%"),
        Operator::EndsWith => format!("%{value}"),
        _ => return None,
    };
    Some(FoldPredicate {
        language: "sql".to_string(),
        text: format!("({column} LIKE ?)"),
        params: vec![Literal::String(pattern)],
    })
}

fn odata_clause(clause: &Clause, catalog: &FoldCatalog) -> Result<SingleFold, FoldDiagnostic> {
    let path = clause.path.join(".");
    let Some(field) = catalog.get(&path) else {
        return Err(FoldDiagnostic {
            kind: "unknown_fold_field".to_string(),
            message: format!("{path} is not present in the fold catalog"),
            path: Some(path),
            operator: Some(clause.op),
        });
    };
    let property = odata_property_path(field.column());
    let predicate = match clause.op {
        Operator::Eq => binary_odata_predicate(&property, "eq", &clause.literal),
        Operator::Ne => binary_odata_predicate(&property, "ne", &clause.literal),
        Operator::Gt => binary_odata_predicate(&property, "gt", &clause.literal),
        Operator::Ge => binary_odata_predicate(&property, "ge", &clause.literal),
        Operator::Lt => binary_odata_predicate(&property, "lt", &clause.literal),
        Operator::Le => binary_odata_predicate(&property, "le", &clause.literal),
        Operator::IsNull => FoldPredicate {
            language: "odata".to_string(),
            text: format!("({property} eq null)"),
            params: Vec::new(),
        },
        Operator::IsNotNull => FoldPredicate {
            language: "odata".to_string(),
            text: format!("({property} ne null)"),
            params: Vec::new(),
        },
        Operator::In | Operator::NotIn => {
            odata_list_predicate(&property, clause.op, &clause.literal).ok_or_else(|| {
                unsupported_fold(
                    FoldBackend::OData,
                    &path,
                    clause.op,
                    "list literal is required",
                )
            })?
        }
        Operator::Between => {
            odata_between_predicate(&property, &clause.literal).ok_or_else(|| {
                unsupported_fold(
                    FoldBackend::OData,
                    &path,
                    clause.op,
                    "range literal is required",
                )
            })?
        }
        Operator::Contains | Operator::StartsWith | Operator::EndsWith => {
            odata_string_function(&property, clause.op, &clause.literal).ok_or_else(|| {
                unsupported_fold(
                    FoldBackend::OData,
                    &path,
                    clause.op,
                    "string literal is required for OData function folding",
                )
            })?
        }
        Operator::Has | Operator::HasAny | Operator::HasAll => {
            return Err(unsupported_fold(
                FoldBackend::OData,
                &path,
                clause.op,
                "array/object containment stays as a residual local filter",
            ));
        }
    };

    Ok(SingleFold {
        source: field.source().to_string(),
        predicate,
        folded_clause_count: 1,
    })
}

fn binary_odata_predicate(property: &str, operator: &str, literal: &Literal) -> FoldPredicate {
    FoldPredicate {
        language: "odata".to_string(),
        text: format!("({property} {operator} {})", odata_literal(literal)),
        params: Vec::new(),
    }
}

fn odata_list_predicate(
    property: &str,
    operator: Operator,
    literal: &Literal,
) -> Option<FoldPredicate> {
    let Literal::List(values) = literal else {
        return None;
    };
    if values.is_empty() {
        return None;
    }
    let values = values
        .iter()
        .map(odata_literal)
        .collect::<Vec<_>>()
        .join(", ");
    let text = match operator {
        Operator::In => format!("({property} in ({values}))"),
        Operator::NotIn => format!("(not ({property} in ({values})))"),
        _ => return None,
    };
    Some(FoldPredicate {
        language: "odata".to_string(),
        text,
        params: Vec::new(),
    })
}

fn odata_between_predicate(property: &str, literal: &Literal) -> Option<FoldPredicate> {
    let Literal::Range { min, max } = literal else {
        return None;
    };
    Some(FoldPredicate {
        language: "odata".to_string(),
        text: format!(
            "(({property} ge {}) and ({property} le {}))",
            odata_literal(min),
            odata_literal(max)
        ),
        params: Vec::new(),
    })
}

fn odata_string_function(
    property: &str,
    operator: Operator,
    literal: &Literal,
) -> Option<FoldPredicate> {
    let Literal::String(value) = literal else {
        return None;
    };
    let value = odata_quoted_string(value);
    let function = match operator {
        Operator::Contains => "contains",
        Operator::StartsWith => "startswith",
        Operator::EndsWith => "endswith",
        _ => return None,
    };
    Some(FoldPredicate {
        language: "odata".to_string(),
        text: format!("({function}({property}, {value}))"),
        params: Vec::new(),
    })
}

fn odata_literal(literal: &Literal) -> String {
    match literal {
        Literal::String(value) => odata_quoted_string(value),
        Literal::Number(value) => value.to_string(),
        Literal::Bool(value) => value.to_string(),
        Literal::Null => "null".to_string(),
        Literal::List(values) => values
            .iter()
            .map(odata_literal)
            .collect::<Vec<_>>()
            .join(", "),
        Literal::Range { min, max } => format!("{}, {}", odata_literal(min), odata_literal(max)),
    }
}

fn odata_quoted_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn odata_property_path(column: &str) -> String {
    column.replace('.', "/")
}

fn combine_fold_predicates(
    kind: &str,
    backend: FoldBackend,
    predicates: Vec<FoldPredicate>,
) -> FoldPredicate {
    let separator = match kind {
        "or" => backend.or(),
        _ => backend.and(),
    };
    let mut params = Vec::new();
    let text = predicates
        .into_iter()
        .map(|predicate| {
            params.extend(predicate.params);
            predicate.text
        })
        .collect::<Vec<_>>()
        .join(separator);
    FoldPredicate {
        language: backend.language().to_string(),
        text: format!("({text})"),
        params,
    }
}

fn unsupported_fold(
    backend: FoldBackend,
    path: &str,
    operator: Operator,
    reason: &str,
) -> FoldDiagnostic {
    FoldDiagnostic {
        kind: format!("unsupported_{}_fold", backend.name()),
        message: reason.to_string(),
        path: Some(path.to_string()),
        operator: Some(operator),
    }
}

fn combine_residuals(residuals: Vec<ExprNode>) -> Option<ExprNode> {
    match residuals.len() {
        0 => None,
        1 => residuals.into_iter().next(),
        _ => Some(ExprNode::All(residuals)),
    }
}

fn quote_sqlite_identifier(identifier: &str) -> String {
    identifier
        .split('.')
        .map(|part| format!("\"{}\"", part.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(".")
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
            Operator::Between => value_between(value, &self.literal),
            Operator::StartsWith => value_starts_with(value, &self.literal),
            Operator::EndsWith => value_ends_with(value, &self.literal),
            Operator::HasAny => value_has_any(value, &self.literal),
            Operator::HasAll => value_has_all(value, &self.literal),
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

    fn explain_parse(&self) -> ParsedExplainField {
        ParsedExplainField {
            path: self.path.join("."),
            operator: self.op,
            literal: self.literal.clone(),
        }
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

fn value_has_any(value: &Value, literal: &Literal) -> bool {
    match literal {
        Literal::List(items) => items.iter().any(|item| value_has(value, item)),
        _ => false,
    }
}

fn value_has_all(value: &Value, literal: &Literal) -> bool {
    match literal {
        Literal::List(items) => items.iter().all(|item| value_has(value, item)),
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

fn value_between(value: &Value, literal: &Literal) -> bool {
    let Literal::Range { min, max } = literal else {
        return false;
    };
    numeric_compare(value, min, |left, right| left >= right)
        && numeric_compare(value, max, |left, right| left <= right)
}

fn value_starts_with(value: &Value, literal: &Literal) -> bool {
    match (value, literal) {
        (Value::String(left), Literal::String(right)) => left.starts_with(right),
        _ => false,
    }
}

fn value_ends_with(value: &Value, literal: &Literal) -> bool {
    match (value, literal) {
        (Value::String(left), Literal::String(right)) => left.ends_with(right),
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
        Operator::Between => matches!(value_type, ValueType::Number | ValueType::Any),
        Operator::StartsWith | Operator::EndsWith => {
            matches!(value_type, ValueType::String | ValueType::Any)
        }
        Operator::HasAny | Operator::HasAll => matches!(
            value_type,
            ValueType::Array | ValueType::Object | ValueType::String | ValueType::Any
        ),
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
        Operator::Between,
        Operator::StartsWith,
        Operator::EndsWith,
        Operator::HasAny,
        Operator::HasAll,
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
        Literal::Number(_) => matches!(value_type, ValueType::Number | ValueType::Array),
        Literal::Bool(_) => matches!(value_type, ValueType::Bool | ValueType::Array),
        Literal::Null => matches!(value_type, ValueType::Null | ValueType::Array),
        Literal::List(items) => items
            .iter()
            .all(|item| literal_valid_for_type(item, value_type)),
        Literal::Range { min, max } => {
            literal_valid_for_type(min, value_type) && literal_valid_for_type(max, value_type)
        }
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
    LeftParen,
    RightParen,
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
        let root = self.parse_or()?;
        if let Some(token) = self.peek() {
            return Err(SliceError::TrailingInput {
                token: token_text(token),
                offset: token.offset,
            });
        }
        Ok(Expr { root })
    }

    fn parse_or(&mut self) -> Result<ExprNode, SliceError> {
        let mut nodes = vec![self.parse_and()?];
        while self.peek_ident("or") {
            self.cursor += 1;
            nodes.push(self.parse_and()?);
        }
        Ok(collapse_boolean(nodes, ExprNode::Any))
    }

    fn parse_and(&mut self) -> Result<ExprNode, SliceError> {
        let mut nodes = vec![self.parse_unary()?];
        while self.peek_ident("and") {
            self.cursor += 1;
            nodes.push(self.parse_unary()?);
        }
        Ok(collapse_boolean(nodes, ExprNode::All))
    }

    fn parse_unary(&mut self) -> Result<ExprNode, SliceError> {
        if self.peek_ident("not") {
            self.cursor += 1;
            return Ok(ExprNode::Not(Box::new(self.parse_unary()?)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<ExprNode, SliceError> {
        if matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::LeftParen)
        ) {
            self.cursor += 1;
            let node = self.parse_or()?;
            self.expect_token("closing parenthesis", |kind| {
                matches!(kind, TokenKind::RightParen)
            })?;
            return Ok(node);
        }
        Ok(ExprNode::Clause(self.parse_clause()?))
    }

    fn parse_clause(&mut self) -> Result<Clause, SliceError> {
        let (path, path_offset) = self.parse_path()?;
        let op_token = self.expect_ident("operator")?;
        let (op, literal, literal_offset) = match op_token.0.as_str() {
            "eq" => self.parse_standard_clause_literal(Operator::Eq)?,
            "ne" => self.parse_standard_clause_literal(Operator::Ne)?,
            "has" if self.peek_ident("any") => {
                self.cursor += 1;
                let (literal, literal_offset) = self.parse_literal_list()?;
                (Operator::HasAny, literal, literal_offset)
            }
            "has" if self.peek_ident("all") => {
                self.cursor += 1;
                let (literal, literal_offset) = self.parse_literal_list()?;
                (Operator::HasAll, literal, literal_offset)
            }
            "has" => self.parse_standard_clause_literal(Operator::Has)?,
            "contains" => self.parse_standard_clause_literal(Operator::Contains)?,
            "starts_with" | "startswith" => {
                self.parse_standard_clause_literal(Operator::StartsWith)?
            }
            "ends_with" | "endswith" => self.parse_standard_clause_literal(Operator::EndsWith)?,
            "gt" => self.parse_standard_clause_literal(Operator::Gt)?,
            "ge" => self.parse_standard_clause_literal(Operator::Ge)?,
            "lt" => self.parse_standard_clause_literal(Operator::Lt)?,
            "le" => self.parse_standard_clause_literal(Operator::Le)?,
            "between" => {
                let (min, literal_offset) = self.parse_literal()?;
                self.expect_specific_ident("and")?;
                let (max, _) = self.parse_literal()?;
                (
                    Operator::Between,
                    Literal::Range {
                        min: Box::new(min),
                        max: Box::new(max),
                    },
                    literal_offset,
                )
            }
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

    fn expect_token(
        &mut self,
        expected: &'static str,
        predicate: impl FnOnce(&TokenKind) -> bool,
    ) -> Result<Token, SliceError> {
        let Some(token) = self.next() else {
            return Err(SliceError::Expected {
                expected,
                offset: self.source.len(),
            });
        };
        if predicate(&token.kind) {
            Ok(token)
        } else {
            Err(SliceError::UnexpectedToken {
                token: token_text(&token),
                offset: token.offset,
            })
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

    fn peek_ident(&self, expected: &str) -> bool {
        matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::Ident(value)) if value == expected
        )
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.cursor).cloned();
        self.cursor += usize::from(token.is_some());
        token
    }
}

fn collapse_boolean(
    mut nodes: Vec<ExprNode>,
    build: impl FnOnce(Vec<ExprNode>) -> ExprNode,
) -> ExprNode {
    if nodes.len() == 1 {
        nodes.pop().expect("one node exists")
    } else {
        build(nodes)
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
        if ch == '(' {
            tokens.push(Token {
                kind: TokenKind::LeftParen,
                offset,
            });
            continue;
        }
        if ch == ')' {
            tokens.push(Token {
                kind: TokenKind::RightParen,
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
            if matches!(*next, '[' | ']' | '(' | ')' | ',') {
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
        TokenKind::LeftParen => "(".to_string(),
        TokenKind::RightParen => ")".to_string(),
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
            parse("repo in ['MDCROP', 'MDLOOM'] and status not in ['blocked', 'stale']").unwrap();

        assert!(expr.matches(&json!({"repo": "MDCROP", "status": "ready"})));
        assert!(!expr.matches(&json!({"repo": "MDPORT", "status": "ready"})));
        assert!(!expr.matches(&json!({"repo": "MDLOOM", "status": "blocked"})));
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
    fn matches_boolean_grouping_with_or_not_and_parentheses() {
        let expr =
            parse("(repo in ['MDCROP', 'MDLOOM'] or tracker eq '[x]') and not status eq 'blocked'")
                .unwrap();

        assert!(expr.matches(&json!({"repo": "MDCROP", "tracker": "[ ]", "status": "ready"})));
        assert!(expr.matches(&json!({"repo": "TRACKER", "tracker": "[x]", "status": "ready"})));
        assert!(!expr.matches(&json!({"repo": "MDCROP", "tracker": "[ ]", "status": "blocked"})));
        assert!(!expr.matches(&json!({"repo": "MDPORT", "tracker": "[ ]", "status": "ready"})));
    }

    #[test]
    fn explain_parse_preserves_expression_tree() {
        let expr =
            parse("(repo eq 'MDCROP' or tracker eq '[x]') and not status eq 'blocked'").unwrap();
        let explain = expr.explain_parse();

        assert_eq!(explain.schema, "slice.parse_explain.v1");
        assert_eq!(explain.clause_count, 3);
        assert_eq!(explain.tree.kind, "all");
        assert_eq!(explain.tree.children[0].kind, "any");
        assert_eq!(explain.tree.children[1].kind, "not");
        assert_eq!(
            explain.tree.children[0].children[0]
                .field
                .as_ref()
                .map(|field| field.path.as_str()),
            Some("repo")
        );
    }

    #[test]
    fn compiled_explain_preserves_typed_expression_tree() {
        let mut catalog = FieldCatalog::new();
        catalog
            .insert("repo", ValueType::String)
            .insert("tracker", ValueType::String)
            .insert("status", ValueType::String);
        let compiled = compile(
            "(repo eq 'MDCROP' or tracker eq '[x]') and not status eq 'blocked'",
            &catalog,
        )
        .unwrap();

        assert_eq!(compiled.explain().clause_count, 3);
        assert_eq!(compiled.explain().tree.kind, "all");
        assert_eq!(compiled.explain().tree.children[0].kind, "any");
        assert_eq!(
            compiled.explain().tree.children[0].children[0]
                .field
                .as_ref()
                .map(|field| field.value_type),
            Some(ValueType::String)
        );
    }

    #[test]
    fn sqlite_fold_plan_partitions_top_level_and_across_sources() {
        let mut catalog = FoldCatalog::new();
        catalog
            .insert_sqlite(
                "player.position",
                ValueType::String,
                "players",
                "players.position",
            )
            .insert_sqlite("stats.ppg", ValueType::Number, "stats", "stats.ppg")
            .insert_sqlite("tags", ValueType::Array, "stats", "stats.tags");
        let expr =
            parse("player.position eq 'C' and stats.ppg ge 0.8 and tags has 'playoffs'").unwrap();
        let plan = expr.plan_sqlite(&catalog).unwrap();

        assert_eq!(plan.backend, "sqlite");
        assert_eq!(plan.source_count, 2);
        assert_eq!(plan.sources[0].source, "players");
        assert_eq!(plan.sources[0].predicate.language, "sql");
        assert_eq!(
            plan.sources[0].predicate.text,
            "((\"players\".\"position\" = ?))"
        );
        assert_eq!(plan.sources[1].source, "stats");
        assert_eq!(plan.sources[1].predicate.text, "((\"stats\".\"ppg\" >= ?))");
        assert_eq!(plan.diagnostics[0].kind, "unsupported_sqlite_fold");
        assert_eq!(
            plan.residual
                .as_ref()
                .and_then(|node| node.field.as_ref())
                .map(|field| field.path.as_str()),
            Some("tags")
        );
    }

    #[test]
    fn sqlite_fold_plan_keeps_cross_source_or_residual() {
        let mut catalog = FoldCatalog::new();
        catalog
            .insert_sqlite(
                "player.position",
                ValueType::String,
                "players",
                "players.position",
            )
            .insert_sqlite("stats.ppg", ValueType::Number, "stats", "stats.ppg");
        let expr = parse("player.position eq 'C' or stats.ppg ge 0.8").unwrap();
        let plan = expr.plan_sqlite(&catalog).unwrap();

        assert!(plan.sources.is_empty());
        assert_eq!(plan.diagnostics[0].kind, "cross_source_boolean");
        assert_eq!(
            plan.residual.as_ref().map(|node| node.kind.as_str()),
            Some("any")
        );
    }

    #[test]
    fn odata_fold_plan_uses_odata_operators_and_functions() {
        let mut catalog = FoldCatalog::new();
        catalog
            .insert_sqlite("player.position", ValueType::String, "players", "position")
            .insert_sqlite("player.name", ValueType::String, "players", "name")
            .insert_sqlite("stats.ppg", ValueType::Number, "stats", "ppg")
            .insert_sqlite("stats.tags", ValueType::Array, "stats", "tags");
        let expr = parse(
            "player.position eq 'C' and player.name starts_with 'A' and stats.ppg between 0.8 and 1.2 and stats.tags has 'playoffs'",
        )
        .unwrap();
        let plan = expr.plan_odata(&catalog).unwrap();

        assert_eq!(plan.backend, "odata");
        assert_eq!(plan.source_count, 2);
        assert_eq!(plan.sources[0].source, "players");
        assert_eq!(plan.sources[0].predicate.language, "odata");
        assert_eq!(
            plan.sources[0].predicate.text,
            "((position eq 'C') and (startswith(name, 'A')))"
        );
        assert_eq!(
            plan.sources[1].predicate.text,
            "(((ppg ge 0.8) and (ppg le 1.2)))"
        );
        assert_eq!(plan.diagnostics[0].kind, "unsupported_odata_fold");
        assert_eq!(
            plan.residual
                .as_ref()
                .and_then(|node| node.field.as_ref())
                .map(|field| field.path.as_str()),
            Some("stats.tags")
        );
    }

    #[test]
    fn matches_range_string_and_array_quantifier_clauses() {
        let expr = parse(
            "priority between 2 and 4 and path starts_with 'docs/' and path ends_with '.md' and tags has all ['slice', 'runtime']",
        )
        .unwrap();

        assert!(expr.matches(&json!({
            "priority": 3,
            "path": "docs/plans/runtime.md",
            "tags": ["slice", "runtime", "tracker"]
        })));
        assert!(!expr.matches(&json!({
            "priority": 5,
            "path": "docs/plans/runtime.md",
            "tags": ["slice", "runtime"]
        })));
        assert!(!expr.matches(&json!({
            "priority": 3,
            "path": "src/runtime.rs",
            "tags": ["slice", "runtime"]
        })));
        assert!(!expr.matches(&json!({
            "priority": 3,
            "path": "docs/plans/runtime.md",
            "tags": ["slice"]
        })));
    }

    #[test]
    fn matches_array_has_any_clause() {
        let expr = parse("tags has any ['runtime', 'adoption']").unwrap();

        assert!(expr.matches(&json!({"tags": ["slice", "adoption"]})));
        assert!(!expr.matches(&json!({"tags": ["slice", "docs"]})));
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
                Operator::StartsWith,
                Operator::EndsWith,
                Operator::HasAny,
                Operator::HasAll,
            ])
        );
    }

    #[test]
    fn validates_membership_and_null_operators_against_catalog() {
        let mut catalog = FieldCatalog::new();
        catalog
            .insert("repo", ValueType::String)
            .insert("priority", ValueType::Number)
            .insert("owner", ValueType::String)
            .insert("path", ValueType::String)
            .insert("tags", ValueType::Array);

        let compiled = compile(
            "repo in ['MDCROP', 'MDLOOM'] and priority between 1 and 5 and owner is not null and path starts_with 'docs/' and tags has any ['runtime']",
            &catalog,
        )
        .unwrap();

        assert_eq!(compiled.requirements().field_count, 5);
        assert!(compiled.matches(&json!({
            "repo": "MDCROP",
            "priority": 2,
            "owner": "docs",
            "path": "docs/runtime.md",
            "tags": ["runtime"]
        })));
    }

    #[test]
    fn rejects_membership_literals_that_do_not_match_catalog_type() {
        let mut catalog = FieldCatalog::new();
        catalog.insert("repo", ValueType::String);

        let err = compile("repo in ['MDCROP', 1]", &catalog).unwrap_err();

        assert_eq!(
            err,
            SliceError::InvalidLiteral {
                path: "repo".to_string(),
                literal: Literal::List(vec![
                    Literal::String("MDCROP".to_string()),
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
