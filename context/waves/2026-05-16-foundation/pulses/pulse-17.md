# Pulse 17: ICELINES prepared-row runtime selectors

## Goal

Move ICELINES from dev-only selector examples to narrow runtime helpers over
already-prepared player bio/stat rows.

## Changes

- Recorded ICELINES commit `3848d51`.
- `icelines-query` now depends on `slice-core` at runtime for prepared player
  row selectors.
- Selection stays after ICELINES has projected hockey data into simple rows;
  SLICE does not own hockey query UX, stat aliases, windows, ranking,
  aggregation, leaderboards, or data requirements.

## Validation

- ICELINES: `cargo test -p icelines-query`
- ICELINES: `cargo clippy -p icelines-query -- -D warnings`
- ICELINES: `git diff --check`

## Status

Done.
