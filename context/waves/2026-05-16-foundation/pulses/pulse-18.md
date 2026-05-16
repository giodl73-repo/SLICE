# Pulse 18: Markdown table selectors

## Goal

Make SLICE useful for portfolio planning artifacts, not only product runtime
rows, by selecting Markdown table rows directly from TRACKER and wave docs.

## Changes

- Added `slice eval --markdown-table` for Markdown table input.
- Markdown headers are normalized into selector-friendly field names, for
  example `Consumer repo` becomes `consumer_repo`.
- Matching Markdown table rows are emitted as JSONL so downstream tools can pipe
  them into existing JSON workflows.
- Existing JSON and JSONL evaluation paths remain unchanged.

## Validation

- SLICE: `cargo test -p slice-cli`
- SLICE: `cargo test`
- SLICE: `cargo run -q -p slice-cli -- eval --expr "metadata.tags has 'context'" --input examples\pebble.json`
- SLICE: `cargo run -q -p slice-cli -- eval --markdown-table --expr "icelines eq '[x]'" --input C:\src\TRACKER\dependency-systems\slice-usage.md`
- SLICE: `git diff --check`

## Status

Done.
