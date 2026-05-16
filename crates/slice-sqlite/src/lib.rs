use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use anyhow::{Context, Result};
use rusqlite::{params_from_iter, types::Value as SqlValue, Connection, OpenFlags};
use serde::Serialize;
use slice_core::{FoldCatalog, FoldPlan, Literal, ValueType};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SqliteInspectReport {
    pub schema: String,
    pub tables: Vec<SqliteTable>,
    pub draft_catalog: DraftFoldCatalog,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SqliteTable {
    pub name: String,
    pub columns: Vec<SqliteColumn>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SqliteColumn {
    pub name: String,
    pub declared_type: String,
    pub value_type: ValueType,
    pub nullable: bool,
    pub primary_key: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DraftFoldCatalog {
    pub fields: BTreeMap<String, DraftFoldField>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DraftFoldField {
    #[serde(rename = "type")]
    pub value_type: ValueType,
    pub source: String,
    pub column: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SqlitePlanReport {
    pub schema: String,
    pub plan: FoldPlan,
    pub validation: SqliteCatalogValidation,
    pub smoke: Vec<SqliteSmokeQuery>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SqliteCatalogValidation {
    pub valid: bool,
    pub checked_field_count: usize,
    pub diagnostics: Vec<SqliteCatalogDiagnostic>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SqliteCatalogDiagnostic {
    pub kind: String,
    pub message: String,
    pub field: String,
    pub source: String,
    pub column: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SqliteSmokeQuery {
    pub source: String,
    pub sql: String,
    pub param_count: usize,
    pub row_available: bool,
}

pub fn inspect_database(path: &Path) -> Result<SqliteInspectReport> {
    let connection = open_read_only(path)?;
    inspect_connection(&connection)
}

pub fn inspect_connection(connection: &Connection) -> Result<SqliteInspectReport> {
    let tables = inspect_tables(connection)?;
    let mut fields = BTreeMap::new();
    for table in &tables {
        for column in &table.columns {
            let path = format!("{}.{}", table.name, column.name);
            fields.insert(
                path,
                DraftFoldField {
                    value_type: column.value_type,
                    source: table.name.clone(),
                    column: format!("{}.{}", table.name, column.name),
                },
            );
        }
    }

    Ok(SqliteInspectReport {
        schema: "slice.sqlite.inspect.v1".to_string(),
        tables,
        draft_catalog: DraftFoldCatalog { fields },
    })
}

pub fn plan_database(path: &Path, catalog: &FoldCatalog, expr: &str) -> Result<SqlitePlanReport> {
    let connection = open_read_only(path)?;
    plan_connection(&connection, catalog, expr)
}

pub fn plan_connection(
    connection: &Connection,
    catalog: &FoldCatalog,
    expr: &str,
) -> Result<SqlitePlanReport> {
    let plan = slice_core::parse(expr)
        .with_context(|| format!("failed to parse SLICE expression {expr:?}"))?
        .plan_sqlite(catalog)
        .with_context(|| format!("failed to plan SQLite folds for expression {expr:?}"))?;
    let validation = validate_catalog(connection, catalog)?;
    let smoke = if validation.valid {
        run_smoke_queries(connection, &plan)?
    } else {
        Vec::new()
    };

    Ok(SqlitePlanReport {
        schema: "slice.sqlite.plan.v1".to_string(),
        plan,
        validation,
        smoke,
    })
}

pub fn validate_catalog(
    connection: &Connection,
    catalog: &FoldCatalog,
) -> Result<SqliteCatalogValidation> {
    let tables = inspect_tables(connection)?;
    let table_columns = tables
        .iter()
        .map(|table| {
            (
                table.name.clone(),
                table
                    .columns
                    .iter()
                    .map(|column| column.name.clone())
                    .collect::<BTreeSet<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();

    for (field, spec) in catalog.fields() {
        let source = spec.source().to_string();
        let physical_column = physical_column_name(spec.column(), &source);
        match table_columns.get(&source) {
            Some(columns) if columns.contains(&physical_column) => {}
            Some(_) => diagnostics.push(SqliteCatalogDiagnostic {
                kind: "unknown_column".to_string(),
                message: format!(
                    "field {field:?} maps to missing SQLite column {source}.{physical_column}"
                ),
                field: field.to_string(),
                source,
                column: physical_column,
            }),
            None => diagnostics.push(SqliteCatalogDiagnostic {
                kind: "unknown_source".to_string(),
                message: format!("field {field:?} maps to missing SQLite source {source:?}"),
                field: field.to_string(),
                source,
                column: physical_column,
            }),
        }
    }

    Ok(SqliteCatalogValidation {
        valid: diagnostics.is_empty(),
        checked_field_count: catalog.fields().count(),
        diagnostics,
    })
}

fn open_read_only(path: &Path) -> Result<Connection> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("failed to open SQLite database {}", path.display()))
}

fn inspect_tables(connection: &Connection) -> Result<Vec<SqliteTable>> {
    let mut statement = connection.prepare(
        "SELECT name FROM sqlite_schema \
         WHERE type = 'table' AND name NOT LIKE 'sqlite_%' \
         ORDER BY name",
    )?;
    let table_names = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    table_names
        .into_iter()
        .map(|name| inspect_table(connection, &name))
        .collect()
}

fn inspect_table(connection: &Connection, name: &str) -> Result<SqliteTable> {
    let pragma = format!("PRAGMA table_info({})", quote_sqlite_identifier(name));
    let mut statement = connection.prepare(&pragma)?;
    let columns = statement
        .query_map([], |row| {
            let declared_type = row.get::<_, String>(2)?;
            Ok(SqliteColumn {
                name: row.get(1)?,
                value_type: sqlite_declared_type(&declared_type),
                declared_type,
                nullable: row.get::<_, i64>(3)? == 0,
                primary_key: row.get::<_, i64>(5)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SqliteTable {
        name: name.to_string(),
        columns,
    })
}

fn run_smoke_queries(connection: &Connection, plan: &FoldPlan) -> Result<Vec<SqliteSmokeQuery>> {
    plan.sources
        .iter()
        .map(|source| {
            let sql = format!(
                "SELECT 1 FROM {} WHERE {} LIMIT 1",
                quote_sqlite_identifier(&source.source),
                source.predicate.text
            );
            let params = source
                .predicate
                .params
                .iter()
                .map(sql_value)
                .collect::<Result<Vec<_>>>()?;
            let row_available =
                match connection.query_row(&sql, params_from_iter(params), |_| Ok(())) {
                    Ok(()) => true,
                    Err(rusqlite::Error::QueryReturnedNoRows) => false,
                    Err(error) => return Err(error.into()),
                };
            Ok(SqliteSmokeQuery {
                source: source.source.clone(),
                sql,
                param_count: source.predicate.params.len(),
                row_available,
            })
        })
        .collect()
}

fn sql_value(literal: &Literal) -> Result<SqlValue> {
    match literal {
        Literal::String(value) => Ok(SqlValue::Text(value.clone())),
        Literal::Number(value) => Ok(SqlValue::Real(*value)),
        Literal::Bool(value) => Ok(SqlValue::Integer(i64::from(*value))),
        Literal::Null => Ok(SqlValue::Null),
        Literal::List(_) | Literal::Range { .. } => {
            anyhow::bail!("nested list/range literal cannot be bound as one SQLite parameter")
        }
    }
}

fn sqlite_declared_type(declared_type: &str) -> ValueType {
    let upper = declared_type.to_ascii_uppercase();
    if upper.contains("INT")
        || upper.contains("REAL")
        || upper.contains("FLOA")
        || upper.contains("DOUB")
        || upper.contains("NUM")
        || upper.contains("DEC")
    {
        ValueType::Number
    } else if upper.contains("BOOL") {
        ValueType::Bool
    } else if upper.contains("TEXT")
        || upper.contains("CHAR")
        || upper.contains("CLOB")
        || upper.contains("DATE")
        || upper.contains("TIME")
    {
        ValueType::String
    } else {
        ValueType::Any
    }
}

fn physical_column_name(column: &str, source: &str) -> String {
    column
        .strip_prefix(source)
        .and_then(|rest| rest.strip_prefix('.'))
        .unwrap_or(column)
        .rsplit('.')
        .next()
        .unwrap_or(column)
        .trim_matches('"')
        .to_string()
}

fn quote_sqlite_identifier(identifier: &str) -> String {
    identifier
        .split('.')
        .map(|segment| format!("\"{}\"", segment.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    #[test]
    fn inspects_tables_and_emits_draft_catalog() {
        let connection = open_fixture().unwrap();
        let report = inspect_connection(&connection).unwrap();

        assert_eq!(report.schema, "slice.sqlite.inspect.v1");
        assert!(report.draft_catalog.fields.contains_key("players.position"));
        assert_eq!(
            report
                .draft_catalog
                .fields
                .get("stats.ppg")
                .unwrap()
                .value_type,
            ValueType::Number
        );
    }

    #[test]
    fn plans_and_smoke_tests_per_source_predicates() {
        let connection = open_fixture().unwrap();
        let mut catalog = FoldCatalog::new();
        catalog
            .insert_sqlite(
                "player.position",
                ValueType::String,
                "players",
                "players.position",
            )
            .insert_sqlite("stats.ppg", ValueType::Number, "stats", "stats.ppg");

        let report = plan_connection(
            &connection,
            &catalog,
            "player.position eq 'C' and stats.ppg ge 0.8",
        )
        .unwrap();

        assert_eq!(report.plan.source_count, 2);
        assert!(report.validation.valid);
        assert_eq!(report.smoke.len(), 2);
        assert!(report.smoke.iter().all(|query| query.row_available));
    }

    #[test]
    fn reports_catalog_validation_diagnostics() {
        let connection = open_fixture().unwrap();
        let mut catalog = FoldCatalog::new();
        catalog.insert_sqlite(
            "player.position",
            ValueType::String,
            "players",
            "players.missing",
        );

        let validation = validate_catalog(&connection, &catalog).unwrap();

        assert!(!validation.valid);
        assert_eq!(validation.diagnostics[0].kind, "unknown_column");
    }

    #[test]
    fn keeps_unsupported_sqlite_predicates_residual() {
        let connection = open_fixture().unwrap();
        let mut catalog = FoldCatalog::new();
        catalog.insert_sqlite("stats.tags", ValueType::Array, "stats", "stats.tags_json");

        let report = plan_connection(&connection, &catalog, "stats.tags has 'playoffs'").unwrap();

        assert!(report.validation.valid);
        assert!(report.smoke.is_empty());
        assert_eq!(report.plan.diagnostics[0].kind, "unsupported_sqlite_fold");
        assert!(report.plan.residual.is_some());
    }

    fn open_fixture() -> Result<Connection> {
        let connection = Connection::open_in_memory()?;
        connection.execute_batch(
            "CREATE TABLE players (
                id TEXT PRIMARY KEY,
                position TEXT NOT NULL
            );
            CREATE TABLE stats (
                player_id TEXT NOT NULL,
                ppg REAL NOT NULL,
                tags_json TEXT NOT NULL
            );",
        )?;
        connection.execute(
            "INSERT INTO players (id, position) VALUES (?1, ?2)",
            params!["847-mock-swe-c", "C"],
        )?;
        connection.execute(
            "INSERT INTO stats (player_id, ppg, tags_json) VALUES (?1, ?2, ?3)",
            params!["847-mock-swe-c", 0.82_f64, r#"["playoffs","leader"]"#],
        )?;
        Ok(connection)
    }
}
