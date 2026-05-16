# Pulse 21: OData-style expression syntax

## Goal

Move SLICE from simple conjunction selectors toward a stronger OData-style row
expression kernel while keeping the layer product-neutral.

## Changes

- Added boolean expression grouping with `or`, unary `not`, and parentheses.
- Preserved existing top-level `and` behavior while adding precedence:
  parentheses, unary `not`, `and`, then `or`.
- Added `between` for numeric ranges.
- Added `starts_with` / `ends_with` string predicates.
- Added `has any` / `has all` quantifiers for arrays, strings, and object keys.
- Updated explain and requirement collection to walk nested expression trees.
- Updated the mock client to derive catalogs for the expanded operator set.

## Validation

- SLICE: `cargo test`
- SLICE: `cargo clippy --workspace -- -D warnings`
- SLICE: `cargo run -q -p slice-cli -- eval --markdown-table --expr "(slice_layer in ['Predicate AST/parser','CLI smoke/evaluation'] or tracker eq '[x]') and not notes contains 'deprecated'" --input C:\src\TRACKER\dependency-systems\slice-usage.md --fields slice_layer,tracker`
- SLICE: `git diff --check`

## Status

Done.
