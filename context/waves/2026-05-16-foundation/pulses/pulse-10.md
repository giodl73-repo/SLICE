# Pulse 10: MDPORT selector examples

## Goal

Record MDPORT's SLICE metadata selector examples.

## Outcome

MDPORT commit `7460225` added dev-only `slice-core` integration tests for
document metadata and section metadata selectors.

## Boundary

MDPORT keeps `mdport.v1` schema ownership, chunking, metadata maps,
provenance refs, and serialization. SLICE is used only as an example/test
selector kernel over adapter-projected rows.

## Validation

Validated in MDPORT:

- `cargo fmt --check`
- `cargo test`
- `git diff --check`

## Status

Done.
