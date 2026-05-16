# Pulse 24: Expression-tree explain

## Goal

Make SLICE expressions inspectable before execution so agents and downstream
CLIs can reason about grouping, negation, typed fields, requirements, and
diagnostics without scanning rows.

## Changes

- Added parse-only explain reports with nested expression trees.
- Extended compiled explain reports with the same nested tree while preserving
  the existing flat `fields` compatibility list.
- Added `slice explain --expr ...` for parse explain JSON.
- Added `slice explain --catalog ... --expr ...` for typed explain,
  requirements, and structured diagnostics.
- Kept evaluation, fetching, ranking, and presentation outside `slice-core`.

## Validation

- SLICE: `cargo test -p slice-core`
- SLICE: `cargo test -p slice-cli`
- SLICE: `cargo run -q -p slice-cli -- explain --expr "(slice_layer in ['Predicate AST/parser','CLI smoke/evaluation'] or tracker eq '[x]') and not notes contains 'deprecated'" --catalog examples\tracker-slice-usage-catalog.json`

## Status

Done.
