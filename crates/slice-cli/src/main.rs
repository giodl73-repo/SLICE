use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Parser)]
#[command(name = "slice")]
#[command(about = "Evaluate SLICE expressions over structured artifacts.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Explain a SLICE expression as machine-readable JSON.
    Explain {
        /// SLICE expression to explain.
        #[arg(long)]
        expr: String,
        /// Optional JSON catalog for typed validation and requirements.
        #[arg(long)]
        catalog: Option<PathBuf>,
    },
    /// Plan backend predicate folding without scanning input rows.
    Plan {
        /// SLICE expression to plan.
        #[arg(long)]
        expr: String,
        /// JSON fold catalog mapping field paths to type/source/column metadata.
        #[arg(long)]
        catalog: PathBuf,
        /// Backend target for folded predicates.
        #[arg(long, value_enum, default_value_t = PlanBackend::Sqlite)]
        backend: PlanBackend,
    },
    /// Evaluate an expression over JSON, JSONL, or Markdown table input.
    Eval {
        /// SLICE expression, for example: metadata.tags has 'context'
        #[arg(long)]
        expr: String,
        /// JSON, JSONL, or Markdown input file.
        #[arg(long)]
        input: PathBuf,
        /// Treat the input as newline-delimited JSON.
        #[arg(long)]
        jsonl: bool,
        /// Treat the input as Markdown tables and emit matching rows as JSONL.
        #[arg(long)]
        markdown_table: bool,
        /// Comma-separated field paths to emit from matching rows.
        #[arg(long, value_delimiter = ',')]
        fields: Vec<String>,
        /// JSON catalog mapping field paths to value types for preflight validation.
        #[arg(long)]
        catalog: Option<PathBuf>,
        /// Field path to sort matching rows by.
        #[arg(long)]
        sort_by: Option<String>,
        /// Sort descending when used with --sort-by.
        #[arg(long)]
        desc: bool,
        /// Number of matching rows to skip after sorting.
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Maximum number of matching rows to emit after sorting and offset.
        #[arg(long)]
        limit: Option<usize>,
        /// Print only the count of matching rows, before offset and limit.
        #[arg(long)]
        count: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Explain { expr, catalog } => run_explain(&expr, catalog.as_ref()),
        Command::Plan {
            expr,
            catalog,
            backend,
        } => run_plan(&expr, &catalog, backend),
        Command::Eval {
            expr,
            input,
            jsonl,
            markdown_table,
            fields,
            catalog,
            sort_by,
            desc,
            offset,
            limit,
            count,
        } => run_eval(
            &expr,
            &input,
            jsonl,
            markdown_table,
            &fields,
            catalog.as_ref(),
            ResultOptions {
                sort_by,
                desc,
                offset,
                limit,
                count,
            },
        ),
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum PlanBackend {
    Sqlite,
}

impl std::fmt::Display for PlanBackend {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanBackend::Sqlite => formatter.write_str("sqlite"),
        }
    }
}

#[derive(Debug, Serialize)]
struct ExplainOutput<'a> {
    schema: &'static str,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    explain: Option<&'a slice_core::ExplainReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse: Option<slice_core::ParsedExplainReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    requirements: Option<&'a slice_core::RequirementReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostic: Option<slice_core::DiagnosticReport>,
}

fn run_explain(expr: &str, catalog: Option<&PathBuf>) -> Result<()> {
    if let Some(catalog_path) = catalog {
        let content = fs::read_to_string(catalog_path)
            .with_context(|| format!("failed to read catalog {}", catalog_path.display()))?;
        let catalog = parse_catalog(&content).context("failed to parse SLICE catalog")?;
        match slice_core::compile(expr, &catalog) {
            Ok(compiled) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&ExplainOutput {
                        schema: "slice.cli.explain.v1",
                        status: "ok",
                        explain: Some(compiled.explain()),
                        parse: None,
                        requirements: Some(compiled.requirements()),
                        diagnostic: None,
                    })?
                );
            }
            Err(error) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&ExplainOutput {
                        schema: "slice.cli.explain.v1",
                        status: "error",
                        explain: None,
                        parse: None,
                        requirements: None,
                        diagnostic: Some(error.diagnostic()),
                    })?
                );
                anyhow::bail!("SLICE expression validation failed");
            }
        }
        return Ok(());
    }

    match slice_core::parse(expr) {
        Ok(parsed) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&ExplainOutput {
                    schema: "slice.cli.explain.v1",
                    status: "ok",
                    explain: None,
                    parse: Some(parsed.explain_parse()),
                    requirements: None,
                    diagnostic: None,
                })?
            );
        }
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&ExplainOutput {
                    schema: "slice.cli.explain.v1",
                    status: "error",
                    explain: None,
                    parse: None,
                    requirements: None,
                    diagnostic: Some(error.diagnostic()),
                })?
            );
            anyhow::bail!("SLICE expression parse failed");
        }
    }
    Ok(())
}

fn run_plan(expr: &str, catalog: &PathBuf, backend: PlanBackend) -> Result<()> {
    let content = fs::read_to_string(catalog)
        .with_context(|| format!("failed to read catalog {}", catalog.display()))?;
    let catalog = parse_fold_catalog(&content).context("failed to parse SLICE fold catalog")?;
    let parsed = match slice_core::parse(expr) {
        Ok(parsed) => parsed,
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&ExplainOutput {
                    schema: "slice.cli.plan.v1",
                    status: "error",
                    explain: None,
                    parse: None,
                    requirements: None,
                    diagnostic: Some(error.diagnostic()),
                })?
            );
            anyhow::bail!("SLICE expression parse failed");
        }
    };

    match backend {
        PlanBackend::Sqlite => match parsed.plan_sqlite(&catalog) {
            Ok(plan) => println!("{}", serde_json::to_string_pretty(&plan)?),
            Err(error) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&ExplainOutput {
                        schema: "slice.cli.plan.v1",
                        status: "error",
                        explain: None,
                        parse: None,
                        requirements: None,
                        diagnostic: Some(error.diagnostic()),
                    })?
                );
                anyhow::bail!("SLICE expression fold planning failed for {backend}");
            }
        },
    }
    Ok(())
}

#[derive(Debug)]
struct ResultOptions {
    sort_by: Option<String>,
    desc: bool,
    offset: usize,
    limit: Option<usize>,
    count: bool,
}

fn run_eval(
    expr: &str,
    input: &PathBuf,
    jsonl: bool,
    markdown_table: bool,
    fields: &[String],
    catalog: Option<&PathBuf>,
    options: ResultOptions,
) -> Result<()> {
    if jsonl && markdown_table {
        anyhow::bail!("--jsonl and --markdown-table are mutually exclusive");
    }
    let selector = load_selector(expr, catalog)?;
    let content = fs::read_to_string(input)
        .with_context(|| format!("failed to read input {}", input.display()))?;
    let mut rows = Vec::new();

    if markdown_table {
        for row in markdown_table_rows(&content) {
            if selector.matches(&row) {
                rows.push(row);
            }
        }
    } else if jsonl {
        for line in content.lines().filter(|line| !line.trim().is_empty()) {
            let value: Value = serde_json::from_str(line).context("failed to parse JSONL row")?;
            if selector.matches(&value) {
                rows.push(value);
            }
        }
    } else {
        let value: Value = serde_json::from_str(&content).context("failed to parse JSON input")?;
        match value {
            Value::Array(items) => {
                for item in items {
                    if selector.matches(&item) {
                        rows.push(item);
                    }
                }
            }
            item if selector.matches(&item) => {
                rows.push(item);
            }
            _ => {}
        }
    }

    emit_rows(rows, fields, &options)
}

fn emit_rows(mut rows: Vec<Value>, fields: &[String], options: &ResultOptions) -> Result<()> {
    let selected_count = rows.len();
    if options.count {
        println!("{}", serde_json::json!({ "count": selected_count }));
        return Ok(());
    }
    if let Some(sort_by) = &options.sort_by {
        let path = split_field_path(sort_by);
        rows.sort_by(|left, right| compare_path_values(left, right, &path));
        if options.desc {
            rows.reverse();
        }
    }
    let iter = rows.into_iter().skip(options.offset);
    let rows: Box<dyn Iterator<Item = Value>> = if let Some(limit) = options.limit {
        Box::new(iter.take(limit))
    } else {
        Box::new(iter)
    };
    for row in rows {
        print_row(&row, fields)?;
    }
    Ok(())
}

enum Selector {
    Parsed(slice_core::Expr),
    Compiled(Box<slice_core::CompiledExpr>),
}

impl Selector {
    fn matches(&self, value: &Value) -> bool {
        match self {
            Selector::Parsed(expr) => expr.matches(value),
            Selector::Compiled(expr) => expr.matches(value),
        }
    }
}

fn load_selector(expr: &str, catalog: Option<&PathBuf>) -> Result<Selector> {
    let Some(catalog_path) = catalog else {
        return slice_core::parse(expr)
            .map(Selector::Parsed)
            .context("failed to parse SLICE expression");
    };
    let content = fs::read_to_string(catalog_path)
        .with_context(|| format!("failed to read catalog {}", catalog_path.display()))?;
    let catalog = parse_catalog(&content).context("failed to parse SLICE catalog")?;
    slice_core::compile(expr, &catalog)
        .map(Box::new)
        .map(Selector::Compiled)
        .context("failed to compile SLICE expression with catalog")
}

fn parse_catalog(content: &str) -> Result<slice_core::FieldCatalog> {
    let value: Value = serde_json::from_str(content).context("catalog must be JSON")?;
    let fields = value
        .get("fields")
        .unwrap_or(&value)
        .as_object()
        .context("catalog must be an object or contain an object-valued 'fields' key")?;
    let mut catalog = slice_core::FieldCatalog::new();
    for (path, value_type) in fields {
        let type_name = value_type
            .as_str()
            .with_context(|| format!("catalog field {path:?} must map to a string value type"))?;
        catalog.insert(path, parse_value_type(type_name)?);
    }
    Ok(catalog)
}

fn parse_fold_catalog(content: &str) -> Result<slice_core::FoldCatalog> {
    let value: Value = serde_json::from_str(content).context("catalog must be JSON")?;
    let fields = value
        .get("fields")
        .unwrap_or(&value)
        .as_object()
        .context("catalog must be an object or contain an object-valued 'fields' key")?;
    let mut catalog = slice_core::FoldCatalog::new();
    for (path, spec) in fields {
        match spec {
            Value::String(type_name) => {
                catalog.insert_sqlite(path, parse_value_type(type_name)?, "default", path);
            }
            Value::Object(object) => {
                let type_name = object
                    .get("type")
                    .and_then(Value::as_str)
                    .with_context(|| format!("fold catalog field {path:?} must include a type"))?;
                let source = object
                    .get("source")
                    .and_then(Value::as_str)
                    .unwrap_or("default");
                let column = object.get("column").and_then(Value::as_str).unwrap_or(path);
                catalog.insert_sqlite(path, parse_value_type(type_name)?, source, column);
            }
            _ => anyhow::bail!(
                "fold catalog field {path:?} must map to a string type or object spec"
            ),
        }
    }
    Ok(catalog)
}

fn parse_value_type(type_name: &str) -> Result<slice_core::ValueType> {
    match type_name.to_ascii_lowercase().as_str() {
        "string" => Ok(slice_core::ValueType::String),
        "number" => Ok(slice_core::ValueType::Number),
        "bool" | "boolean" => Ok(slice_core::ValueType::Bool),
        "array" => Ok(slice_core::ValueType::Array),
        "object" => Ok(slice_core::ValueType::Object),
        "null" => Ok(slice_core::ValueType::Null),
        "any" => Ok(slice_core::ValueType::Any),
        other => anyhow::bail!("unsupported catalog value type {other:?}"),
    }
}

fn print_row(row: &Value, fields: &[String]) -> Result<()> {
    let output = if fields.is_empty() {
        row.clone()
    } else {
        project_fields(row, fields)
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

fn project_fields(row: &Value, fields: &[String]) -> Value {
    let mut output = serde_json::Map::new();
    for field in fields {
        let path = split_field_path(field);
        if path.is_empty() {
            continue;
        }
        if let Some(value) = lookup_path(row, &path) {
            insert_projected_value(&mut output, &path, value.clone());
        }
    }
    Value::Object(output)
}

fn split_field_path(field: &str) -> Vec<&str> {
    field
        .split('.')
        .filter(|segment| !segment.trim().is_empty())
        .collect::<Vec<_>>()
}

fn lookup_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = match current {
            Value::Object(object) => object.get(*segment)?,
            _ => return None,
        };
    }
    Some(current)
}

fn insert_projected_value(
    object: &mut serde_json::Map<String, Value>,
    path: &[&str],
    value: Value,
) {
    if let Some((head, tail)) = path.split_first() {
        if tail.is_empty() {
            object.insert((*head).to_string(), value);
            return;
        }
        let entry = object
            .entry((*head).to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(serde_json::Map::new());
        }
        if let Value::Object(child) = entry {
            insert_projected_value(child, tail, value);
        }
    }
}

fn compare_path_values(left: &Value, right: &Value, path: &[&str]) -> std::cmp::Ordering {
    let left = lookup_path(left, path);
    let right = lookup_path(right, path);
    match (left, right) {
        (Some(left), Some(right)) => compare_values(left, right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn compare_values(left: &Value, right: &Value) -> std::cmp::Ordering {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => left
            .as_f64()
            .partial_cmp(&right.as_f64())
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::String(left), Value::String(right)) => left.cmp(right),
        (Value::Bool(left), Value::Bool(right)) => left.cmp(right),
        (Value::Null, Value::Null) => std::cmp::Ordering::Equal,
        (Value::Null, _) => std::cmp::Ordering::Greater,
        (_, Value::Null) => std::cmp::Ordering::Less,
        _ => type_rank(left).cmp(&type_rank(right)),
    }
}

fn type_rank(value: &Value) -> u8 {
    match value {
        Value::Bool(_) => 0,
        Value::Number(_) => 1,
        Value::String(_) => 2,
        Value::Array(_) => 3,
        Value::Object(_) => 4,
        Value::Null => 5,
    }
}

fn markdown_table_rows(content: &str) -> Vec<Value> {
    let lines = content.lines().collect::<Vec<_>>();
    let mut rows = Vec::new();
    let mut index = 0usize;
    while index + 1 < lines.len() {
        if !is_table_line(lines[index]) || !is_separator_line(lines[index + 1]) {
            index += 1;
            continue;
        }

        let headers = split_table_row(lines[index])
            .into_iter()
            .map(normalize_header)
            .collect::<Vec<_>>();
        index += 2;

        while index < lines.len() && is_table_line(lines[index]) {
            let cells = split_table_row(lines[index]);
            let mut object = serde_json::Map::new();
            for (header, cell) in headers.iter().zip(cells.iter()) {
                object.insert(header.clone(), parse_markdown_cell(cell));
            }
            rows.push(Value::Object(object));
            index += 1;
        }
    }
    rows
}

fn is_table_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.matches('|').count() >= 2
}

fn is_separator_line(line: &str) -> bool {
    if !is_table_line(line) {
        return false;
    }
    split_table_row(line).into_iter().all(|cell| {
        let trimmed = cell.trim();
        !trimmed.is_empty()
            && trimmed
                .chars()
                .all(|character| matches!(character, '-' | ':' | ' '))
            && trimmed.chars().any(|character| character == '-')
    })
}

fn split_table_row(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn normalize_header(header: String) -> String {
    let mut normalized = String::new();
    let mut previous_was_separator = false;
    for character in header.trim().chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator {
            normalized.push('_');
            previous_was_separator = true;
        }
    }
    normalized.trim_matches('_').to_string()
}

fn parse_markdown_cell(cell: &str) -> Value {
    let trimmed = cell.trim();
    if trimmed.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }
    if trimmed.eq_ignore_ascii_case("null") || trimmed.eq_ignore_ascii_case("n/a") {
        return Value::Null;
    }
    if let Ok(number) = trimmed.parse::<f64>() {
        return serde_json::Number::from_f64(number)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(trimmed.to_string()));
    }
    Value::String(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_table_rows_normalize_headers_and_cells() {
        let rows = markdown_table_rows(
            r#"
| Consumer repo | Status | Runtime | Count |
|---|---:|---|---:|
| CROP | [x] | true | 2 |
| TRACKER | [ ] | false | n/a |
"#,
        );

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["consumer_repo"], "CROP");
        assert_eq!(rows[0]["status"], "[x]");
        assert_eq!(rows[0]["runtime"], true);
        assert_eq!(rows[0]["count"], 2.0);
        assert_eq!(rows[1]["count"], Value::Null);
    }

    #[test]
    fn markdown_table_rows_can_be_selected_with_slice() {
        let rows = markdown_table_rows(
            r#"
| Repo | Status | Runtime |
|---|---|---|
| CROP | [x] | true |
| TRACKER | [ ] | false |
"#,
        );
        let expr = slice_core::parse("status eq '[x]' and runtime eq true").unwrap();
        let selected = rows
            .iter()
            .filter(|row| expr.matches(row))
            .map(|row| row["repo"].as_str().unwrap())
            .collect::<Vec<_>>();

        assert_eq!(selected, ["CROP"]);
    }

    #[test]
    fn project_fields_keeps_requested_nested_paths() {
        let value = serde_json::json!({
            "metadata": {
                "status": "ready",
                "tags": ["context"],
            },
            "source": "example.md",
        });

        let projected = project_fields(
            &value,
            &[
                "metadata.status".to_string(),
                "source".to_string(),
                "missing".to_string(),
            ],
        );

        assert_eq!(
            projected,
            serde_json::json!({
                "metadata": {
                    "status": "ready",
                },
                "source": "example.md",
            })
        );
    }

    #[test]
    fn project_fields_supports_flat_markdown_table_rows() {
        let value = serde_json::json!({
            "slice_layer": "CLI smoke/evaluation",
            "tracker": "[x]",
            "notes": "ready",
        });

        let projected = project_fields(&value, &["slice_layer".to_string(), "tracker".to_string()]);

        assert_eq!(
            projected,
            serde_json::json!({
                "slice_layer": "CLI smoke/evaluation",
                "tracker": "[x]",
            })
        );
    }

    #[test]
    fn parse_catalog_accepts_field_map() {
        let catalog = parse_catalog(
            r#"{
                "fields": {
                    "repo": "string",
                    "priority": "number",
                    "active": "bool",
                    "tags": "array"
                }
            }"#,
        )
        .unwrap();

        assert_eq!(
            catalog.get("repo").map(slice_core::FieldSpec::value_type),
            Some(slice_core::ValueType::String)
        );
        assert_eq!(
            catalog
                .get("priority")
                .map(slice_core::FieldSpec::value_type),
            Some(slice_core::ValueType::Number)
        );
        assert_eq!(
            catalog.get("active").map(slice_core::FieldSpec::value_type),
            Some(slice_core::ValueType::Bool)
        );
        assert_eq!(
            catalog.get("tags").map(slice_core::FieldSpec::value_type),
            Some(slice_core::ValueType::Array)
        );
    }

    #[test]
    fn parse_catalog_rejects_unknown_value_type() {
        let err = parse_catalog(r#"{"repo": "text"}"#).unwrap_err();

        assert!(err.to_string().contains("unsupported catalog value type"));
    }

    #[test]
    fn parse_fold_catalog_accepts_source_and_column_specs() {
        let catalog = parse_fold_catalog(
            r#"{
                "fields": {
                    "player.position": {
                        "type": "string",
                        "source": "players",
                        "column": "players.position"
                    },
                    "stats.ppg": "number"
                }
            }"#,
        )
        .unwrap();

        let position = catalog.get("player.position").unwrap();
        assert_eq!(position.value_type(), slice_core::ValueType::String);
        assert_eq!(position.source(), "players");
        assert_eq!(position.column(), "players.position");
        let ppg = catalog.get("stats.ppg").unwrap();
        assert_eq!(ppg.source(), "default");
        assert_eq!(ppg.column(), "stats.ppg");
    }

    #[test]
    fn emit_rows_sorts_offsets_limits_and_projects() {
        let mut rows = vec![
            serde_json::json!({"repo": "PEBBLE", "priority": 2}),
            serde_json::json!({"repo": "CROP", "priority": 1}),
            serde_json::json!({"repo": "PROOF", "priority": 3}),
        ];
        let path = split_field_path("priority");
        rows.sort_by(|left, right| compare_path_values(left, right, &path));
        let selected = rows
            .into_iter()
            .skip(1)
            .take(1)
            .map(|row| project_fields(&row, &["repo".to_string()]))
            .collect::<Vec<_>>();

        assert_eq!(selected, vec![serde_json::json!({"repo": "PEBBLE"})]);
    }

    #[test]
    fn compare_path_values_puts_missing_values_last() {
        let path = split_field_path("priority");
        let mut rows = vec![
            serde_json::json!({"repo": "missing"}),
            serde_json::json!({"repo": "present", "priority": 1}),
        ];

        rows.sort_by(|left, right| compare_path_values(left, right, &path));

        assert_eq!(rows[0]["repo"], "present");
        assert_eq!(rows[1]["repo"], "missing");
    }
}
