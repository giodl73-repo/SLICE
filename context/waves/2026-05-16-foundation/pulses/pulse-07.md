# Pulse 07: Requirement reports

## Goal

Expose the minimal typed field set a downstream adapter must materialize before
evaluating a selector.

## Changes

- Add `slice.requirements.v1` via `CompiledExpr::requirements()`.
- Deduplicate fields across clauses while preserving catalog value types.
- Include requirement reports in the local mock client alongside explain output.
- Keep fetch, cache, graph, rendering, ranking, and domain planning outside
  `slice-core`.

## Validation

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo run -p slice-mock-client`
- `git diff --check`

## Status

Done.
