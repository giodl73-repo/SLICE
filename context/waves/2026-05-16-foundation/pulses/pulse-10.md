# Pulse 10: PEBBLE selector examples

## Goal

Record PEBBLE's SLICE metadata selector examples.

## Outcome

PEBBLE commit `7460225` added dev-only `slice-core` integration tests for
document metadata and section metadata selectors.

## Boundary

PEBBLE keeps `pebble.v1` schema ownership, chunking, metadata maps,
provenance refs, and serialization. SLICE is used only as an example/test
selector kernel over adapter-projected rows.

## Validation

Validated in PEBBLE:

- `cargo fmt --check`
- `cargo test`
- `git diff --check`

## Status

Done.
