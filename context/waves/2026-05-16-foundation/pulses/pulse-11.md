# Pulse 11: FLETCH selector examples

## Goal

Record FLETCH's SLICE selector examples for cache-index and active-partition
rows.

## Outcome

FLETCH commit `1aeceea` added dev-only `slice-core` tests that select
cache-index rows and active-partition rows before FLETCH-side quiver candidate
folding.

## Boundary

SLICE only selects rows and reports requirements. FLETCH keeps cache manifests,
cache-index gates, active partition sets, rollups, quiver grouping,
fetch/cache execution, and policy decisions.

## Validation

Validated in FLETCH:

- `cargo fmt --check`
- `cargo test --workspace`
- `git diff --check`

## Status

Done.
