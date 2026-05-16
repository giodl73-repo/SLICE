# Pulse 23: CLI result shaping

## Goal

Make `slice eval` useful as a practical query pipeline over JSON, JSONL, and
Markdown table rows, not only as a predicate filter.

## Changes

- Added `--sort-by <field>` for sorting matching rows by a dotted field path.
- Added `--desc` for descending sort order.
- Added `--offset` and `--limit` for paging result rows.
- Added `--count` to emit the selected row count before paging.
- Kept result shaping in the CLI layer; `slice-core` remains the reusable
  expression kernel.

## Validation

- SLICE: `cargo test -p slice-cli`
- SLICE: `cargo test`
- SLICE: `cargo clippy --workspace -- -D warnings`
- SLICE: `cargo run -q -p slice-cli -- eval --markdown-table --catalog examples\tracker-slice-usage-catalog.json --expr "tracker is not null" --input C:\src\TRACKER\dependency-systems\slice-usage.md --sort-by slice_layer --limit 2 --fields slice_layer,tracker`
- SLICE: `cargo run -q -p slice-cli -- eval --markdown-table --catalog examples\tracker-slice-usage-catalog.json --expr "tracker is not null" --input C:\src\TRACKER\dependency-systems\slice-usage.md --count`
- SLICE: `git diff --check`

## Status

Done.
