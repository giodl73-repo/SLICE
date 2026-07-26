# Pulse 06: Diagnostic reports

## Goal

Make parser and catalog-validation failures actionable for downstream adapters
before the first real CROP migration.

## Changes

- Add `slice.diagnostic.v1` via `SliceError::diagnostic()`.
- Preserve byte offsets for unknown fields, invalid operators, and invalid
  literals after parsing.
- Include structured error kind, message, expected value, token/path, operator,
  value type, literal, and allowed operators when available.
- Keep diagnostics product-neutral: CROP, Mdport, FLETCH, MDLOOM, and ICELINES
  decide how to render the report.

## Validation

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo run -p slice-mock-client`
- `git diff --check`

## Status

Done.
