# Pulse 04: Explain reports

## Goal

Expose machine-readable selector explanations so downstream CLIs and agents can
show which typed fields, operators, and literals a compiled selector depends on.

## Changes

- Add `ExplainReport` and `ExplainField` to `slice-core`.
- Store explain output on `CompiledExpr`.
- Include explain reports in `slice-mock-client` output for Mdport, MDCROP,
  FLETCH, and ICELINES-shaped selectors.
- Keep explain output product-neutral: no graph cuts, cache folding, ranking, or
  hockey semantics in `slice-core`.

## Validation

- `cargo fmt --check`
- `cargo test`
- `cargo run -p slice-mock-client`
- `cargo run -p slice-cli -- eval --expr "metadata.tags has 'context'" --input examples/mdport.json`
- `git diff --check`

## Status

Done.
