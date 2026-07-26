use std::collections::BTreeMap;

use anyhow::{Context, Result};
use rusqlite::{params, params_from_iter, types::Value as SqlValue, Connection};
use serde::Serialize;
use serde_json::{json, Map, Value};
use slice_core::{
    ExplainReport, FieldCatalog, FoldCatalog, FoldPlan, Literal, Operator, RequirementReport,
    ValueType,
};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MockClientReport {
    pub schema: String,
    pub mdport: SelectionReport,
    pub mdcrop: SelectionReport,
    pub mdcrop_frontmatter_parity: MdcropFrontmatterParityReport,
    pub fletch: FletchSelectionReport,
    pub icelines: SelectionReport,
    pub icelines_sqlite: SqliteFoldSelectionReport,
    pub icelines_sqlite_runtime: SqliteRuntimeReport,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SelectionReport {
    pub expression: String,
    pub explain: ExplainReport,
    pub requirements: RequirementReport,
    pub input_count: usize,
    pub selected_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FletchSelectionReport {
    pub expression: String,
    pub explain: ExplainReport,
    pub requirements: RequirementReport,
    pub input_count: usize,
    pub selected_partition_ids: Vec<String>,
    pub quiver_candidates: Vec<QuiverCandidate>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MdcropFrontmatterParityReport {
    pub expression: String,
    pub explain: ExplainReport,
    pub requirements: RequirementReport,
    pub input_count: usize,
    pub selected_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SqliteFoldSelectionReport {
    pub expression: String,
    pub plan: FoldPlan,
    pub folded_candidate_count: usize,
    pub selected_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SqliteRuntimeReport {
    pub table_count: usize,
    pub draft_catalog_field_count: usize,
    pub validation_valid: bool,
    pub smoke_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct QuiverCandidate {
    pub dataset_id: String,
    pub partition_ids: Vec<String>,
    pub cache_keys: Vec<String>,
}

pub fn run_mock_client() -> Result<MockClientReport> {
    let mdport = select_ids(
        "metadata.tags has 'context' and metadata.status eq 'ready'",
        mdport_rows(),
        "id",
        mdport_catalog(),
    )?;
    let mdcrop = select_ids(
        "metadata.tags has 'frontmatter' and metadata.status eq 'ready'",
        mdcrop_rows(),
        "id",
        mdcrop_catalog(),
    )?;
    let mdcrop_frontmatter_parity = select_mdcrop_frontmatter_sources(
        "tags has 'computing' and status eq 'ready' and owner ne 'docs'",
        mdcrop_frontmatter_rows(),
    )?;
    let fletch = select_fletch_partitions(
        "active eq true and dataset.id contains 'icelines'",
        fletch_partition_rows(),
        fletch_catalog(),
    )?;
    let icelines = select_ids(
        "player.position eq 'C' and player.nationality eq 'SWE' and stats.ppg ge 0.8",
        icelines_rows(),
        "player.id",
        icelines_catalog(),
    )?;
    let icelines_sqlite = select_icelines_sqlite_folded()?;
    let icelines_sqlite_runtime = inspect_icelines_sqlite_runtime()?;

    let passed = mdport.selected_ids == ["mdport:guide"]
        && mdcrop.selected_ids == ["mdcrop:unit:frontmatter"]
        && mdcrop_frontmatter_parity.selected_sources == ["maxim/systems.md"]
        && fletch.selected_partition_ids == ["partition:icelines:leaders"]
        && fletch.quiver_candidates
            == [QuiverCandidate {
                dataset_id: "icelines:leaders".to_string(),
                partition_ids: vec!["partition:icelines:leaders".to_string()],
                cache_keys: vec!["cache:icelines:leaders:2026".to_string()],
            }]
        && icelines.selected_ids == ["847-mock-swe-c"]
        && icelines_sqlite.selected_ids == ["847-mock-swe-c"]
        && icelines_sqlite_runtime.validation_valid
        && icelines_sqlite_runtime.smoke_sources == ["players", "stats"];

    Ok(MockClientReport {
        schema: "slice.mock-client.v1".to_string(),
        mdport,
        mdcrop,
        mdcrop_frontmatter_parity,
        fletch,
        icelines,
        icelines_sqlite,
        icelines_sqlite_runtime,
        passed,
    })
}

fn select_ids(
    expr: &str,
    rows: Vec<Value>,
    id_path: &str,
    catalog: FieldCatalog,
) -> Result<SelectionReport> {
    let selector = slice_core::compile(expr, &catalog)
        .with_context(|| format!("failed to compile expression {expr:?}"))?;
    let input_count = rows.len();
    let id_segments = split_path(id_path);
    let selected_ids = rows
        .iter()
        .filter(|row| selector.matches(row))
        .map(|row| string_path(row, &id_segments))
        .collect::<Result<Vec<_>>>()?;

    Ok(SelectionReport {
        expression: expr.to_string(),
        explain: selector.explain().clone(),
        requirements: selector.requirements().clone(),
        input_count,
        selected_ids,
    })
}

fn select_fletch_partitions(
    expr: &str,
    rows: Vec<Value>,
    catalog: FieldCatalog,
) -> Result<FletchSelectionReport> {
    let selector = slice_core::compile(expr, &catalog)
        .with_context(|| format!("failed to compile expression {expr:?}"))?;
    let input_count = rows.len();
    let selected = rows
        .into_iter()
        .filter(|row| selector.matches(row))
        .collect::<Vec<_>>();

    let selected_partition_ids = selected
        .iter()
        .map(|row| string_path(row, &["partition".to_string(), "id".to_string()]))
        .collect::<Result<Vec<_>>>()?;
    let quiver_candidates = fold_fletch_rows_into_quiver_candidates(&selected)?;

    Ok(FletchSelectionReport {
        expression: expr.to_string(),
        explain: selector.explain().clone(),
        requirements: selector.requirements().clone(),
        input_count,
        selected_partition_ids,
        quiver_candidates,
    })
}

fn select_mdcrop_frontmatter_sources(
    expr: &str,
    rows: Vec<MdcropFrontmatterRow>,
) -> Result<MdcropFrontmatterParityReport> {
    let catalog = mdcrop_frontmatter_catalog(expr)?;
    let selector = slice_core::compile(expr, &catalog)
        .with_context(|| format!("failed to compile MDCROP frontmatter expression {expr:?}"))?;
    let input_count = rows.len();
    let required_paths = selector
        .requirements()
        .fields
        .iter()
        .map(|field| field.path.as_str())
        .collect::<Vec<_>>();
    let selected_sources = rows
        .iter()
        .filter_map(|row| {
            let value = materialize_mdcrop_frontmatter_row(row, &required_paths);
            selector.matches(&value).then(|| row.source.clone())
        })
        .collect::<Vec<_>>();

    Ok(MdcropFrontmatterParityReport {
        expression: expr.to_string(),
        explain: selector.explain().clone(),
        requirements: selector.requirements().clone(),
        input_count,
        selected_sources,
    })
}

fn select_icelines_sqlite_folded() -> Result<SqliteFoldSelectionReport> {
    let expr = "player.position eq 'C' and stats.ppg ge 0.8 and stats.tags has 'playoffs'";
    let catalog = icelines_sqlite_fold_catalog();
    let selector = slice_core::compile(expr, &catalog.field_catalog())
        .with_context(|| format!("failed to compile SQLite fold expression {expr:?}"))?;
    let plan = slice_core::parse(expr)
        .with_context(|| format!("failed to parse SQLite fold expression {expr:?}"))?
        .plan_sqlite(&catalog)
        .with_context(|| format!("failed to plan SQLite fold expression {expr:?}"))?;

    let connection = open_mock_icelines_sqlite()?;
    let where_sql = plan
        .sources
        .iter()
        .map(|source| source.predicate.text.as_str())
        .collect::<Vec<_>>()
        .join(" AND ");
    let sql = format!(
        "SELECT players.id, players.position, players.nationality, stats.ppg, stats.goals, stats.tags_json \
         FROM players JOIN stats ON stats.player_id = players.id WHERE {where_sql}"
    );
    let params = plan
        .sources
        .iter()
        .flat_map(|source| source.predicate.params.iter().map(sql_value))
        .collect::<Result<Vec<_>>>()?;
    let mut statement = connection.prepare(&sql)?;
    let candidates = statement
        .query_map(params_from_iter(params), |row| {
            let tags_json: String = row.get(5)?;
            let tags = serde_json::from_str::<Value>(&tags_json).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?;
            Ok(json!({
                "player": {
                    "id": row.get::<_, String>(0)?,
                    "position": row.get::<_, String>(1)?,
                    "nationality": row.get::<_, String>(2)?
                },
                "stats": {
                    "ppg": row.get::<_, f64>(3)?,
                    "goals": row.get::<_, i64>(4)?,
                    "tags": tags
                }
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let folded_candidate_count = candidates.len();
    let selected_ids = candidates
        .iter()
        .filter(|row| selector.matches(row))
        .map(|row| string_path(row, &["player".to_string(), "id".to_string()]))
        .collect::<Result<Vec<_>>>()?;

    Ok(SqliteFoldSelectionReport {
        expression: expr.to_string(),
        plan,
        folded_candidate_count,
        selected_ids,
    })
}

fn inspect_icelines_sqlite_runtime() -> Result<SqliteRuntimeReport> {
    let expr = "player.position eq 'C' and stats.ppg ge 0.8 and stats.tags has 'playoffs'";
    let catalog = icelines_sqlite_fold_catalog();
    let connection = open_mock_icelines_sqlite()?;
    let inspect = slice_sqlite::inspect_connection(&connection)?;
    let runtime_plan = slice_sqlite::plan_connection(&connection, &catalog, expr)?;
    let smoke_sources = runtime_plan
        .smoke
        .iter()
        .map(|query| query.source.clone())
        .collect::<Vec<_>>();

    Ok(SqliteRuntimeReport {
        table_count: inspect.tables.len(),
        draft_catalog_field_count: inspect.draft_catalog.fields.len(),
        validation_valid: runtime_plan.validation.valid,
        smoke_sources,
    })
}

fn open_mock_icelines_sqlite() -> Result<Connection> {
    let connection = Connection::open_in_memory()?;
    connection.execute_batch(
        "CREATE TABLE players (
            id TEXT PRIMARY KEY,
            position TEXT NOT NULL,
            nationality TEXT NOT NULL
        );
        CREATE TABLE stats (
            player_id TEXT NOT NULL,
            ppg REAL NOT NULL,
            goals INTEGER NOT NULL,
            tags_json TEXT NOT NULL
        );",
    )?;
    connection.execute(
        "INSERT INTO players (id, position, nationality) VALUES (?1, ?2, ?3)",
        params!["847-mock-swe-c", "C", "SWE"],
    )?;
    connection.execute(
        "INSERT INTO stats (player_id, ppg, goals, tags_json) VALUES (?1, ?2, ?3, ?4)",
        params![
            "847-mock-swe-c",
            0.82_f64,
            32_i64,
            r#"["playoffs","leader"]"#
        ],
    )?;
    connection.execute(
        "INSERT INTO players (id, position, nationality) VALUES (?1, ?2, ?3)",
        params!["847-mock-can-d", "D", "CAN"],
    )?;
    connection.execute(
        "INSERT INTO stats (player_id, ppg, goals, tags_json) VALUES (?1, ?2, ?3, ?4)",
        params!["847-mock-can-d", 0.91_f64, 18_i64, r#"["playoffs"]"#],
    )?;
    connection.execute(
        "INSERT INTO players (id, position, nationality) VALUES (?1, ?2, ?3)",
        params!["847-mock-usa-c", "C", "USA"],
    )?;
    connection.execute(
        "INSERT INTO stats (player_id, ppg, goals, tags_json) VALUES (?1, ?2, ?3, ?4)",
        params!["847-mock-usa-c", 0.74_f64, 20_i64, r#"["leader"]"#],
    )?;
    connection.execute(
        "INSERT INTO players (id, position, nationality) VALUES (?1, ?2, ?3)",
        params!["847-mock-fin-c", "C", "FIN"],
    )?;
    connection.execute(
        "INSERT INTO stats (player_id, ppg, goals, tags_json) VALUES (?1, ?2, ?3, ?4)",
        params!["847-mock-fin-c", 0.86_f64, 24_i64, r#"["leader"]"#],
    )?;
    Ok(connection)
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

fn mdcrop_frontmatter_catalog(expr: &str) -> Result<FieldCatalog> {
    let parsed = slice_core::parse(expr)
        .with_context(|| format!("failed to parse MDCROP frontmatter expression {expr:?}"))?;
    let mut catalog = FieldCatalog::new();
    for clause in parsed.clauses() {
        let path = clause.path().join(".");
        let value_type = match clause.op() {
            Operator::Has => ValueType::Any,
            Operator::Eq
            | Operator::Ne
            | Operator::Contains
            | Operator::In
            | Operator::NotIn
            | Operator::StartsWith
            | Operator::EndsWith => ValueType::String,
            Operator::Gt | Operator::Ge | Operator::Lt | Operator::Le | Operator::Between => {
                ValueType::Any
            }
            Operator::IsNull | Operator::IsNotNull => ValueType::Any,
            Operator::HasAny | Operator::HasAll => ValueType::Any,
        };
        catalog.insert(path, value_type);
    }
    Ok(catalog)
}

fn materialize_mdcrop_frontmatter_row(
    row: &MdcropFrontmatterRow,
    required_paths: &[&str],
) -> Value {
    let mut object = Map::new();
    for path in required_paths {
        let value = row
            .fields
            .get(*path)
            .map(|field| mdcrop_frontmatter_value(field))
            .unwrap_or(Value::Null);
        object.insert((*path).to_string(), value);
    }
    Value::Object(object)
}

fn mdcrop_frontmatter_value(value: &str) -> Value {
    let trimmed = value.trim();
    if let Some(inner) = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    {
        let values = inner
            .split(',')
            .map(|part| part.trim().trim_matches('"').trim_matches('\'').trim())
            .filter(|part| !part.is_empty())
            .map(|part| Value::String(part.to_string()))
            .collect::<Vec<_>>();
        return Value::Array(values);
    }
    Value::String(trimmed.to_string())
}

fn fold_fletch_rows_into_quiver_candidates(rows: &[Value]) -> Result<Vec<QuiverCandidate>> {
    let mut by_dataset = BTreeMap::<String, QuiverCandidate>::new();
    for row in rows {
        let dataset_id = string_path(row, &["dataset".to_string(), "id".to_string()])?;
        let partition_id = string_path(row, &["partition".to_string(), "id".to_string()])?;
        let cache_key = string_path(row, &["cache".to_string(), "key".to_string()])?;
        let candidate = by_dataset
            .entry(dataset_id.clone())
            .or_insert_with(|| QuiverCandidate {
                dataset_id,
                partition_ids: Vec::new(),
                cache_keys: Vec::new(),
            });
        candidate.partition_ids.push(partition_id);
        candidate.cache_keys.push(cache_key);
    }
    Ok(by_dataset.into_values().collect())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MdcropFrontmatterRow {
    source: String,
    fields: BTreeMap<String, String>,
}

fn split_path(path: &str) -> Vec<String> {
    path.split('.').map(str::to_string).collect()
}

fn string_path(row: &Value, path: &[String]) -> Result<String> {
    let mut value = row;
    for segment in path {
        value = value
            .get(segment)
            .with_context(|| format!("missing path segment {segment:?} in {row}"))?;
    }
    value
        .as_str()
        .map(str::to_string)
        .with_context(|| format!("path {:?} is not a string in {row}", path))
}

fn mdport_catalog() -> FieldCatalog {
    let mut catalog = FieldCatalog::new();
    catalog
        .insert("metadata.tags", ValueType::Array)
        .insert("metadata.status", ValueType::String);
    catalog
}

fn mdcrop_catalog() -> FieldCatalog {
    let mut catalog = FieldCatalog::new();
    catalog
        .insert("metadata.tags", ValueType::Array)
        .insert("metadata.status", ValueType::String);
    catalog
}

fn fletch_catalog() -> FieldCatalog {
    let mut catalog = FieldCatalog::new();
    catalog
        .insert("active", ValueType::Bool)
        .insert("dataset.id", ValueType::String);
    catalog
}

fn icelines_catalog() -> FieldCatalog {
    let mut catalog = FieldCatalog::new();
    catalog
        .insert("player.position", ValueType::String)
        .insert("player.nationality", ValueType::String)
        .insert("stats.ppg", ValueType::Number);
    catalog
}

fn icelines_sqlite_fold_catalog() -> FoldCatalog {
    let mut catalog = FoldCatalog::new();
    catalog
        .insert_sqlite(
            "player.position",
            ValueType::String,
            "players",
            "players.position",
        )
        .insert_sqlite(
            "player.nationality",
            ValueType::String,
            "players",
            "players.nationality",
        )
        .insert_sqlite("stats.ppg", ValueType::Number, "stats", "stats.ppg")
        .insert_sqlite("stats.goals", ValueType::Number, "stats", "stats.goals")
        .insert_sqlite("stats.tags", ValueType::Array, "stats", "stats.tags_json");
    catalog
}

fn mdport_rows() -> Vec<Value> {
    vec![
        json!({
            "id": "mdport:guide",
            "schema": "mdport.v1",
            "kind": "document",
            "metadata": {
                "status": "ready",
                "tags": ["context", "query"]
            }
        }),
        json!({
            "id": "mdport:draft",
            "schema": "mdport.v1",
            "kind": "document",
            "metadata": {
                "status": "draft",
                "tags": ["context"]
            }
        }),
    ]
}

fn mdcrop_rows() -> Vec<Value> {
    vec![
        json!({
            "id": "mdcrop:unit:frontmatter",
            "kind": "evidence-unit",
            "metadata": {
                "status": "ready",
                "tags": ["frontmatter", "metadata"]
            }
        }),
        json!({
            "id": "mdcrop:unit:body",
            "kind": "evidence-unit",
            "metadata": {
                "status": "ready",
                "tags": ["body"]
            }
        }),
    ]
}

fn mdcrop_frontmatter_rows() -> Vec<MdcropFrontmatterRow> {
    vec![
        mdcrop_frontmatter_row(
            "maxim/systems.md",
            &[
                ("tags", "[computing, systems]"),
                ("status", "ready"),
                ("version", "1.0"),
            ],
        ),
        mdcrop_frontmatter_row(
            "maxim/draft.md",
            &[("tags", "[computing]"), ("status", "draft")],
        ),
        mdcrop_frontmatter_row("maxim/math.md", &[("tags", "[math]"), ("status", "ready")]),
    ]
}

fn mdcrop_frontmatter_row(source: &str, fields: &[(&str, &str)]) -> MdcropFrontmatterRow {
    MdcropFrontmatterRow {
        source: source.to_string(),
        fields: fields
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect(),
    }
}

fn fletch_partition_rows() -> Vec<Value> {
    vec![
        json!({
            "partition": {
                "id": "partition:icelines:leaders"
            },
            "dataset": {
                "id": "icelines:leaders"
            },
            "cache": {
                "key": "cache:icelines:leaders:2026"
            },
            "active": true,
            "verified": true
        }),
        json!({
            "partition": {
                "id": "partition:icelines:archive"
            },
            "dataset": {
                "id": "icelines:archive"
            },
            "cache": {
                "key": "cache:icelines:archive:2024"
            },
            "active": false,
            "verified": true
        }),
        json!({
            "partition": {
                "id": "partition:maxim:query-languages"
            },
            "dataset": {
                "id": "maxim:query-languages"
            },
            "cache": {
                "key": "cache:maxim:query-languages"
            },
            "active": true,
            "verified": true
        }),
    ]
}

fn icelines_rows() -> Vec<Value> {
    vec![
        json!({
            "player": {
                "id": "847-mock-swe-c",
                "position": "C",
                "nationality": "SWE"
            },
            "stats": {
                "ppg": 0.82
            }
        }),
        json!({
            "player": {
                "id": "847-mock-can-d",
                "position": "D",
                "nationality": "CAN"
            },
            "stats": {
                "ppg": 0.71
            }
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_client_validates_all_consumer_shapes() {
        let report = run_mock_client().unwrap();

        assert!(report.passed, "{report:#?}");
        assert_eq!(report.mdport.selected_ids, ["mdport:guide"]);
        assert_eq!(report.mdport.explain.clause_count, 2);
        assert_eq!(report.mdport.requirements.field_count, 2);
        assert_eq!(report.mdcrop.selected_ids, ["mdcrop:unit:frontmatter"]);
        assert_eq!(
            report.mdcrop_frontmatter_parity.selected_sources,
            ["maxim/systems.md"]
        );
        assert_eq!(
            report.mdcrop_frontmatter_parity.requirements.fields[1].path,
            "status"
        );
        assert_eq!(
            report.fletch.selected_partition_ids,
            ["partition:icelines:leaders"]
        );
        assert_eq!(report.fletch.explain.fields[0].path, "active");
        assert_eq!(report.fletch.requirements.fields[1].path, "dataset.id");
        assert_eq!(report.icelines.selected_ids, ["847-mock-swe-c"]);
        assert_eq!(report.icelines.explain.fields[2].path, "stats.ppg");
        assert_eq!(report.icelines.requirements.field_count, 3);
        assert_eq!(report.icelines_sqlite.selected_ids, ["847-mock-swe-c"]);
        assert_eq!(report.icelines_sqlite.plan.source_count, 2);
        assert!(report.icelines_sqlite.plan.residual.is_some());
    }

    #[test]
    fn fletch_folding_stays_outside_slice_core() {
        let report = run_mock_client().unwrap();

        assert_eq!(report.fletch.quiver_candidates.len(), 1);
        assert_eq!(
            report.fletch.quiver_candidates[0].cache_keys,
            ["cache:icelines:leaders:2026"]
        );
    }

    #[test]
    fn mdcrop_frontmatter_parity_preserves_missing_ne() {
        let report = select_mdcrop_frontmatter_sources(
            "owner ne 'docs'",
            vec![mdcrop_frontmatter_row(
                "docs/without-owner.md",
                &[("status", "ready")],
            )],
        )
        .unwrap();

        assert_eq!(report.selected_sources, ["docs/without-owner.md"]);
    }

    #[test]
    fn sqlite_fold_mock_filters_join_candidates_then_applies_residual() {
        let report = select_icelines_sqlite_folded().unwrap();

        assert_eq!(report.folded_candidate_count, 2);
        assert_eq!(report.selected_ids, ["847-mock-swe-c"]);
        assert_eq!(report.plan.sources[0].source, "players");
        assert_eq!(report.plan.sources[1].source, "stats");
        assert_eq!(report.plan.diagnostics[0].kind, "unsupported_sqlite_fold");
    }
}
