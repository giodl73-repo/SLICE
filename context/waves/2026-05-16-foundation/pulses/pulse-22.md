# Pulse 22: CLI catalog loading

## Goal

Bring runtime-adapter type checking to CLI queries so Markdown, JSON, and JSONL
selectors fail before row scanning when fields or operators do not match a
catalog.

## Changes

- Added `slice eval --catalog <path>` for JSON field catalogs.
- Catalogs can be either a direct object of `path: type` pairs or an object with
  a `fields` map.
- Supported field types: `string`, `number`, `bool`/`boolean`, `array`,
  `object`, `null`, and `any`.
- Added a TRACKER slice-usage example catalog under `examples/`.
- Existing parse-only CLI evaluation still works when no catalog is supplied.

## Validation

- SLICE: `cargo test -p slice-cli`
- SLICE: `cargo test`
- SLICE: `cargo clippy --workspace -- -D warnings`
- SLICE: `cargo run -q -p slice-cli -- eval --markdown-table --catalog examples\tracker-slice-usage-catalog.json --expr "(slice_layer in ['Predicate AST/parser','CLI smoke/evaluation'] or tracker eq '[x]') and not notes contains 'deprecated'" --input C:\src\TRACKER\dependency-systems\slice-usage.md --fields slice_layer,tracker`
- SLICE: invalid typed query `tracker ge 1` rejects before evaluation with the catalog.
- SLICE: `git diff --check`

## Status

Done.
