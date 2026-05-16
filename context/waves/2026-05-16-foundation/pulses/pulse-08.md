# Pulse 08: CROP parity mock

## Goal

Prove that CROP frontmatter-query parity can be built as an adapter around
SLICE rather than as product logic inside `slice-core`.

## Changes

- Add a CROP frontmatter parity path to `slice-mock-client`.
- Derive a field catalog from parsed query clauses for dynamic frontmatter keys.
- Materialize required fields before evaluation, using `null` for missing keys
  so CROP's current `ne` behavior is preserved.
- Convert array-like frontmatter strings such as `[computing, systems]` into
  arrays for `has` evaluation.

## Boundary

`slice-core` still only parses, validates, explains, reports requirements, and
evaluates typed rows. CROP remains responsible for frontmatter extraction,
recipe schemas, view policy, graph selection, and output rendering.

## Validation

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo run -p slice-mock-client`
- `git diff --check`

## Status

Done.
