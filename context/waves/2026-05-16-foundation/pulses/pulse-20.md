# Pulse 20: Membership and null operators

## Goal

Round out the practical row-selector operator set so TRACKER, JSONL artifacts,
and runtime adapters can express common repo/status and sparse-field filters
without local predicate code.

## Changes

- Added `in` and `not in` membership operators with literal lists.
- Added `is null` and `is not null` sparse-field operators.
- Extended parser tokenization for list delimiters and commas.
- Extended catalog validation, diagnostics, explain reports, and mock-client
  catalog derivation for the new operators.
- Kept the scope to product-neutral row predicates; no domain ranking,
  aggregation, fetching, or scripting semantics were added.

## Validation

- SLICE: `cargo test -p slice-core`
- SLICE: `cargo test`
- SLICE: `cargo clippy --workspace -- -D warnings`
- SLICE: `cargo run -q -p slice-cli -- eval --markdown-table --expr "slice_layer in ['Predicate AST/parser','CLI smoke/evaluation'] and tracker is not null" --input C:\src\TRACKER\dependency-systems\slice-usage.md --fields slice_layer,tracker`
- SLICE: `git diff --check`

## Status

Done.
