# Pulse 14: PEBBLE optional runtime selectors

## Goal

Move PEBBLE from dev-only SLICE examples to an optional runtime adoption surface
without making selectors part of the `pebble.v1` schema.

## Changes

- Recorded PEBBLE commit `9239143`.
- PEBBLE now exposes a feature-gated `slice` helper for selecting documents and
  sections by metadata.
- The helper projects Pebble-owned document and section fields into SLICE rows;
  consumers that need custom tag arrays or domain fields still own their own
  adapters.

## Validation

- PEBBLE: `cargo test`
- PEBBLE: `cargo test --features slice`
- PEBBLE: `cargo clippy --features slice -- -D warnings`
- PEBBLE: `git diff --check`

## Status

Done.
