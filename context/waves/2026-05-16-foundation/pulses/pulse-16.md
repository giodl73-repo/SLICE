# Pulse 16: FLETCH row runtime selectors

## Goal

Move FLETCH from dev-only selector examples to narrow runtime helpers over
already-prepared cache-index and active-partition rows.

## Changes

- Recorded FLETCH commit `b7ddbe7`.
- `fletch-core` now depends on `slice-core` at runtime for cache-index and
  active-partition row selectors.
- Selection stays before FLETCH's cache-index gates, active partition rollups,
  and quiver folding; SLICE does not own fetching, caching, quiver policy, or
  domain decisions.

## Validation

- FLETCH: `cargo test -p fletch-core`
- FLETCH: `cargo clippy -p fletch-core -- -D warnings`
- FLETCH: `git diff --check`

## Status

Done.
