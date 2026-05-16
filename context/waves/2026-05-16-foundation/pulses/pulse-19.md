# Pulse 19: CLI projection

## Goal

Make selected rows easier to consume in scripts and planning reports by emitting
only requested fields after selector evaluation.

## Changes

- Added `slice eval --fields` with comma-separated dotted field paths.
- Projection works for JSON arrays, JSONL rows, and Markdown table rows.
- Dotted JSON paths are emitted as nested JSON objects; flat Markdown table
  fields remain flat.
- Missing projected fields are omitted rather than synthesized.

## Validation

- SLICE: `cargo test -p slice-cli`
- SLICE: `cargo test`
- SLICE: `cargo clippy --workspace -- -D warnings`
- SLICE: `cargo run -q -p slice-cli -- eval --expr "metadata.tags has 'context'" --input examples\pebble.json --fields metadata.status,source`
- SLICE: `cargo run -q -p slice-cli -- eval --markdown-table --expr "tracker eq '[x]'" --input C:\src\TRACKER\dependency-systems\slice-usage.md --fields slice_layer,tracker,notes`
- SLICE: `git diff --check`

## Status

Done.
