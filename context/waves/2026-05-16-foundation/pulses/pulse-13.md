# Pulse 13: ICELINES selector examples

## Goal

Show that ICELINES can use SLICE only at the simple prepared-row selector layer
without replacing ICELINES' hockey query language or typed query IR.

## Changes

- Recorded ICELINES commit `b4e05b2` as a dev-only selector example.
- Documented the ICELINES boundary in the consumer migration plan.
- Kept stat IDs, aliases, windows, career aggregation, leaderboards, ranking,
  similarity, percentiles, and data requirements out of SLICE.

## Validation

- ICELINES: `cargo fmt --check`
- ICELINES: `cargo test -p icelines-query --test slice_simple_selector`
- ICELINES: `git diff --check`

## Status

Done.
