# Pulse 14: MDPORT optional runtime selectors

## Goal

Move MDPORT from dev-only SLICE examples to an optional runtime adoption surface
without making selectors part of the `mdport.v1` schema.

## Changes

- Recorded MDPORT commit `9239143`.
- MDPORT now exposes a feature-gated `slice` helper for selecting documents and
  sections by metadata.
- The helper projects Mdport-owned document and section fields into SLICE rows;
  consumers that need custom tag arrays or domain fields still own their own
  adapters.

## Validation

- MDPORT: `cargo test`
- MDPORT: `cargo test --features slice`
- MDPORT: `cargo clippy --features slice -- -D warnings`
- MDPORT: `git diff --check`

## Status

Done.
