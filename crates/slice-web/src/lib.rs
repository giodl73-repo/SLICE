//! Product-neutral, bounded browser/native adapter. Selector semantics stay in slice-core.
use serde::Deserialize;
use serde_json::{json, Value};
use slice_core::{FieldCatalog, ValueType};
use std::collections::BTreeMap;

pub const MAX_RECORD_BYTES: usize = 1_048_576;
pub const MAX_ROWS: usize = 2_000;
pub const MAX_DEPTH: usize = 24;
pub const MAX_NODES: usize = 50_000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    records_json: String,
    expression: String,
    #[serde(default)]
    catalog_json: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    fields: BTreeMap<String, String>,
}

fn error(phase: &str, message: impl Into<String>) -> Value {
    json!({"schema":"slice.playground.v1", "ok":false,"phase":phase,
        "diagnostic":{"kind":phase,"message":message.into()}})
}

fn limits() -> Value {
    json!({"record_bytes":MAX_RECORD_BYTES,"rows":MAX_ROWS,"json_depth":MAX_DEPTH,
        "json_nodes":MAX_NODES,"expression_bytes":2048,"expression_depth":16,
        "expression_words":128,"not_tokens":16,"catalog_bytes":16384,"catalog_fields":128,
        "request_bytes":3*1024*1024})
}

// Admission only. Follow the current core's lexical boundaries (not JSON escapes)
// so quotes embedded in identifiers cannot hide recursive syntax from the guard.
// No selector is evaluated, normalized or rewritten here.
fn expression_bound(source: &str) -> bool {
    if source.len() > 2048 {
        return false;
    }
    let mut depth = 0usize;
    let mut words = 0;
    let mut nots = 0;
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            continue;
        }
        if c == '\'' || c == '"' {
            for next in chars.by_ref() {
                if next == c {
                    break;
                }
            }
            continue;
        }
        if matches!(c, '(' | '[' | ')' | ']' | ',' | '.') {
            if c == '(' || c == '[' {
                depth += 1;
            }
            if depth > 16 {
                return false;
            }
            if c == ')' || c == ']' {
                depth = depth.saturating_sub(1);
            }
            continue;
        }
        let mut word = String::from(c);
        while let Some(&next) = chars.peek() {
            if next.is_whitespace() || matches!(next, '[' | ']' | '(' | ')' | ',') {
                break;
            }
            if next == '.' {
                let decimal_point =
                    word.chars().all(|ch| ch.is_ascii_digit()) && !word.contains('.');
                if !decimal_point {
                    break;
                }
            }
            word.push(next);
            chars.next();
        }
        words += 1;
        nots += usize::from(word == "not");
        if words > 128 || nots > 16 {
            return false;
        }
    }
    true
}

fn run(raw: &str) -> Value {
    if raw.len() > 3 * 1024 * 1024 {
        return error("limit", "Request exceeds 3 MiB.");
    }
    let request: Request = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(e) => return error("request", e.to_string()),
    };
    if request.records_json.len() > MAX_RECORD_BYTES {
        return error("limit", "Records exceed 1 MiB.");
    }
    if !expression_bound(&request.expression) {
        return error(
            "limit",
            "Selector exceeds the byte, nesting, word or not-token limit.",
        );
    }
    let rows: Value = match serde_json::from_str(&request.records_json) {
        Ok(v) => v,
        Err(e) => {
            return error(
                "records",
                format!(
                    "Invalid JSON at line {}, column {}: {e}",
                    e.line(),
                    e.column()
                ),
            )
        }
    };
    let Some(records) = rows.as_array() else {
        return error("records", "Use a JSON array of objects.");
    };
    if records.len() > MAX_ROWS {
        return error("limit", "Use at most 2,000 records.");
    }
    if !records.iter().all(Value::is_object) {
        return error("records", "Every record must be an object.");
    }
    let mut stack = vec![(&rows, 0)];
    let mut nodes = 0;
    while let Some((value, depth)) = stack.pop() {
        nodes += 1;
        if depth > MAX_DEPTH || nodes > MAX_NODES {
            return error("limit", "Records exceed 24 levels or 50,000 JSON nodes.");
        }
        match value {
            Value::Array(values) => stack.extend(values.iter().map(|v| (v, depth + 1))),
            Value::Object(values) => stack.extend(values.values().map(|v| (v, depth + 1))),
            _ => (),
        }
    }
    let mut catalog = None;
    if let Some(text) = request
        .catalog_json
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        if text.len() > 16_384 {
            return error("limit", "Catalog exceeds 16 KiB.");
        }
        let fields: Catalog = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => return error("catalog", e.to_string()),
        };
        if fields.fields.len() > 128 {
            return error("limit", "Use at most 128 catalog fields.");
        }
        let mut typed = FieldCatalog::new();
        for (name, kind) in fields.fields {
            if name.is_empty() || name.len() > 128 {
                return error("catalog", "Catalog paths must contain 1–128 bytes.");
            }
            let value_type = match kind.to_ascii_lowercase().as_str() {
                "string" => ValueType::String,
                "number" => ValueType::Number,
                "bool" | "boolean" => ValueType::Bool,
                "array" => ValueType::Array,
                "object" => ValueType::Object,
                "null" => ValueType::Null,
                "any" => ValueType::Any,
                _ => {
                    return error(
                        "catalog",
                        format!("Unsupported catalog type {kind:?} for {name}."),
                    )
                }
            };
            typed.insert(name, value_type);
        }
        catalog = Some(typed);
    }
    let expr = match slice_core::parse(&request.expression) {
        Ok(expr) => expr,
        Err(e) => {
            return json!({"schema":"slice.playground.v1","ok":false,"phase":"expression","diagnostic":e.diagnostic()})
        }
    };
    let (explain, requirements) = if let Some(typed) = catalog.as_ref() {
        match (expr.explain(typed), expr.requirements(typed)) {
            (Ok(explain), Ok(required)) => (json!(explain), json!(required)),
            (Err(e), _) | (_, Err(e)) => {
                return json!({"schema":"slice.playground.v1","ok":false,"phase":"expression","diagnostic":e.diagnostic()})
            }
        }
    } else {
        (json!(expr.explain_parse()), Value::Null)
    };
    let indices: Vec<usize> = records
        .iter()
        .enumerate()
        .filter_map(|(i, row)| expr.matches(row).then_some(i))
        .collect();
    let matches: Vec<&Value> = indices.iter().map(|&i| &records[i]).collect();
    json!({"schema":"slice.playground.v1","ok":true,"input_count":records.len(),"match_count":matches.len(),
        "matches":matches,"match_indices":indices,"explain":explain,"requirements":requirements,"limits":limits()})
}

/// The exact same canonical response is used by the native CLI and browser worker.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn evaluate(request: &str) -> String {
    run(request).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn query(records: Value, expression: &str, catalog: Option<Value>) -> Value {
        run(
            &json!({"records_json":records.to_string(),"expression":expression,
            "catalog_json":catalog.map(|v|v.to_string())})
            .to_string(),
        )
    }
    #[test]
    fn matches_are_original_rows_in_input_order_and_core_explain_is_preserved() {
        let rows = json!([{"id":"a","score":4},{"id":"b","score":8},{"id":"c","score":6}]);
        let result = query(
            rows.clone(),
            "score ge 6",
            Some(json!({"fields":{"score":"number"}})),
        );
        assert_eq!(result["match_indices"], json!([1, 2]));
        assert_eq!(result["matches"], json!([rows[1], rows[2]]));
        let mut catalog = FieldCatalog::new();
        catalog.insert("score", ValueType::Number);
        let core = slice_core::compile("score ge 6", &catalog).unwrap();
        assert_eq!(result["explain"], json!(core.explain()));
        assert_eq!(result["requirements"], json!(core.requirements()));
    }
    #[test]
    fn untyped_boolean_null_and_empty_results_keep_native_semantics() {
        let rows = json!([{"id":"a","tags":["ready"],"owner":null},{"id":"b","tags":[] }]);
        for source in [
            "owner is null",
            "not tags has 'ready'",
            "id in ['a','b']",
            "id eq 'missing'",
        ] {
            let expected: Vec<&Value> = rows
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| slice_core::parse(source).unwrap().matches(r))
                .collect();
            let result = query(rows.clone(), source, None);
            assert_eq!(result["matches"], json!(expected));
            assert!(result["requirements"].is_null());
        }
        assert_eq!(query(json!([]), "id eq 'a'", None)["match_count"], 0);
    }
    #[test]
    fn diagnostics_retain_exact_core_byte_offsets_including_unicode() {
        let text = "name eq 'café' and missing eq 'x'";
        let catalog = json!({"fields":{"name":"string"}});
        let result = query(json!([]), text, Some(catalog));
        let mut typed = FieldCatalog::new();
        typed.insert("name", ValueType::String);
        assert_eq!(
            result["diagnostic"],
            json!(slice_core::compile(text, &typed).unwrap_err().diagnostic())
        );
        assert_eq!(result["phase"], "expression");
        assert!(result.get("matches").is_none());
    }
    #[test]
    fn invalid_shapes_catalogs_and_malformed_input_are_structured() {
        for records in [json!({}), json!([1]), json!([null])] {
            assert_eq!(query(records, "id eq 'a'", None)["phase"], "records");
        }
        for catalog in [
            json!({"fields":{"x":"bad"}}),
            json!({"fields":{"x":1}}),
            json!({"other":{}}),
        ] {
            assert_eq!(
                query(json!([]), "x eq 1", Some(catalog))["phase"],
                "catalog"
            );
        }
        assert_eq!(run("{}")["phase"], "request");
        assert_eq!(
            run(&json!({"records_json":"[broken","expression":"x eq 1"}).to_string())["phase"],
            "records"
        );
        assert_eq!(
            query(json!([]), "x frobnicate 1", None)["phase"],
            "expression"
        );
    }
    #[test]
    fn bounds_refuse_large_rows_depth_nodes_and_recursive_selectors() {
        assert_eq!(
            query(json!(vec![json!({}); MAX_ROWS + 1]), "x is null", None)["phase"],
            "limit"
        );
        let mut deep = json!({});
        for _ in 0..MAX_DEPTH {
            deep = json!({"nested":deep});
        }
        assert_eq!(query(json!([deep]), "x is null", None)["phase"], "limit");
        let wide = json!([{"values":vec![0;MAX_NODES]}]);
        assert_eq!(query(wide, "x is null", None)["phase"], "limit");
        for expression in [
            format!("{}x is null", "not ".repeat(17)),
            format!("{}x is null{}", "(".repeat(17), ")".repeat(17)),
            "x".repeat(2049),
        ] {
            assert_eq!(query(json!([]), &expression, None)["phase"], "limit");
        }
        let raw = json!({"records_json":" ".repeat(MAX_RECORD_BYTES+1),"expression":"x is null"});
        assert_eq!(run(&raw.to_string())["phase"], "limit");
    }
    #[test]
    fn admission_does_not_count_quoted_not_or_brackets_as_recursion() {
        let text = "not ([".repeat(18);
        let source = format!("name eq '{text}'");
        assert_eq!(
            query(json!([{"name":text}]), &source, None)["match_count"],
            1
        );
        assert_eq!(query(json!([{"x'":1}]), "x' eq 1", None)["match_count"], 1);
        for prefix in ["x' eq 1 and ", "name eq 'x\\' and "] {
            let nested = format!("{prefix}{}id is null{}", "(".repeat(17), ")".repeat(17));
            assert_eq!(query(json!([]), &nested, None)["phase"], "limit");
        }
    }
}
