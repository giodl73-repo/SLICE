# Pulse 02: Mock downstream client

## Goal

Add a local downstream validation harness that proves SLICE selectors over
representative consumer-shaped artifacts.

## Changes

- Add `slice-mock-client` workspace crate.
- Validate Mdport-shaped metadata selection.
- Validate CROP-like evidence unit metadata selection.
- Validate FLETCH-like active partition selection and keep quiver folding in the
  mock client layer.
- Validate an ICELINES-like player row selector without replacing ICELINES query
  semantics.

## Validation

- `cargo fmt --check`
- `cargo test`
- `cargo run -p slice-mock-client`
- `cargo run -p slice-cli -- eval --expr "metadata.tags has 'context'" --input examples/mdport.json`
- `git diff --check`

## Status

Done.
