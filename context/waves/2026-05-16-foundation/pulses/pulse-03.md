# Pulse 03: Typed path catalogs

## Goal

Let downstream adapters validate SLICE selector paths and operator/type
compatibility before evaluating rows.

## Changes

- Add numeric comparison operators: `gt`, `ge`, `lt`, and `le`.
- Add `FieldCatalog`, `FieldSpec`, and `ValueType` to `slice-core`.
- Add `compile(expr, catalog)` for parse-plus-validation.
- Update `slice-mock-client` to compile selectors against consumer-shaped field
  catalogs before evaluation.
- Use an ICELINES-like numeric `stats.ppg ge 0.8` predicate in the mock client.

## Validation

- `cargo fmt --check`
- `cargo test`
- `cargo run -p slice-mock-client`
- `cargo run -p slice-cli -- eval --expr "metadata.tags has 'context'" --input examples/pebble.json`
- `git diff --check`

## Status

Done.
