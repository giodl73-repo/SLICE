use std::collections::BTreeMap;

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use slice_core::{FieldCatalog, ValueType};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MockClientReport {
    pub schema: String,
    pub pebble: SelectionReport,
    pub crop: SelectionReport,
    pub fletch: FletchSelectionReport,
    pub icelines: SelectionReport,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SelectionReport {
    pub expression: String,
    pub input_count: usize,
    pub selected_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FletchSelectionReport {
    pub expression: String,
    pub input_count: usize,
    pub selected_partition_ids: Vec<String>,
    pub quiver_candidates: Vec<QuiverCandidate>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct QuiverCandidate {
    pub dataset_id: String,
    pub partition_ids: Vec<String>,
    pub cache_keys: Vec<String>,
}

pub fn run_mock_client() -> Result<MockClientReport> {
    let pebble = select_ids(
        "metadata.tags has 'context' and metadata.status eq 'ready'",
        pebble_rows(),
        "id",
        pebble_catalog(),
    )?;
    let crop = select_ids(
        "metadata.tags has 'frontmatter' and metadata.status eq 'ready'",
        crop_rows(),
        "id",
        crop_catalog(),
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

    let passed = pebble.selected_ids == ["pebble:guide"]
        && crop.selected_ids == ["crop:unit:frontmatter"]
        && fletch.selected_partition_ids == ["partition:icelines:leaders"]
        && fletch.quiver_candidates
            == [QuiverCandidate {
                dataset_id: "icelines:leaders".to_string(),
                partition_ids: vec!["partition:icelines:leaders".to_string()],
                cache_keys: vec!["cache:icelines:leaders:2026".to_string()],
            }]
        && icelines.selected_ids == ["847-mock-swe-c"];

    Ok(MockClientReport {
        schema: "slice.mock-client.v1".to_string(),
        pebble,
        crop,
        fletch,
        icelines,
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
        input_count,
        selected_partition_ids,
        quiver_candidates,
    })
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

fn pebble_catalog() -> FieldCatalog {
    let mut catalog = FieldCatalog::new();
    catalog
        .insert("metadata.tags", ValueType::Array)
        .insert("metadata.status", ValueType::String);
    catalog
}

fn crop_catalog() -> FieldCatalog {
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

fn pebble_rows() -> Vec<Value> {
    vec![
        json!({
            "id": "pebble:guide",
            "schema": "pebble.v1",
            "kind": "document",
            "metadata": {
                "status": "ready",
                "tags": ["context", "query"]
            }
        }),
        json!({
            "id": "pebble:draft",
            "schema": "pebble.v1",
            "kind": "document",
            "metadata": {
                "status": "draft",
                "tags": ["context"]
            }
        }),
    ]
}

fn crop_rows() -> Vec<Value> {
    vec![
        json!({
            "id": "crop:unit:frontmatter",
            "kind": "evidence-unit",
            "metadata": {
                "status": "ready",
                "tags": ["frontmatter", "metadata"]
            }
        }),
        json!({
            "id": "crop:unit:body",
            "kind": "evidence-unit",
            "metadata": {
                "status": "ready",
                "tags": ["body"]
            }
        }),
    ]
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
        assert_eq!(report.pebble.selected_ids, ["pebble:guide"]);
        assert_eq!(report.crop.selected_ids, ["crop:unit:frontmatter"]);
        assert_eq!(
            report.fletch.selected_partition_ids,
            ["partition:icelines:leaders"]
        );
        assert_eq!(report.icelines.selected_ids, ["847-mock-swe-c"]);
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
}
