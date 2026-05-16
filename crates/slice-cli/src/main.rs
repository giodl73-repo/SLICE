use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
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
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Eval {
            expr,
            input,
            jsonl,
            markdown_table,
            fields,
        } => run_eval(&expr, &input, jsonl, markdown_table, &fields),
    }
}

fn run_eval(
    expr: &str,
    input: &PathBuf,
    jsonl: bool,
    markdown_table: bool,
    fields: &[String],
) -> Result<()> {
    if jsonl && markdown_table {
        anyhow::bail!("--jsonl and --markdown-table are mutually exclusive");
    }
    let expr = slice_core::parse(expr).context("failed to parse SLICE expression")?;
    let content = fs::read_to_string(input)
        .with_context(|| format!("failed to read input {}", input.display()))?;

    if markdown_table {
        for row in markdown_table_rows(&content) {
            if expr.matches(&row) {
                print_row(&row, fields)?;
            }
        }
        return Ok(());
    }

    if jsonl {
        for line in content.lines().filter(|line| !line.trim().is_empty()) {
            let value: Value = serde_json::from_str(line).context("failed to parse JSONL row")?;
            if expr.matches(&value) {
                print_row(&value, fields)?;
            }
        }
        return Ok(());
    }

    let value: Value = serde_json::from_str(&content).context("failed to parse JSON input")?;
    match value {
        Value::Array(items) => {
            for item in items {
                if expr.matches(&item) {
                    print_row(&item, fields)?;
                }
            }
        }
        item if expr.matches(&item) => {
            print_row(&item, fields)?;
        }
        _ => {}
    }

    Ok(())
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
        let path = field
            .split('.')
            .filter(|segment| !segment.trim().is_empty())
            .collect::<Vec<_>>();
        if path.is_empty() {
            continue;
        }
        if let Some(value) = lookup_path(row, &path) {
            insert_projected_value(&mut output, &path, value.clone());
        }
    }
    Value::Object(output)
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
}
