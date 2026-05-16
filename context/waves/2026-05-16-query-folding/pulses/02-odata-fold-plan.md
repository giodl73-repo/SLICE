---
wave: query-folding
pulse: 02
date: 2026-05-16
status: done
depends_on: ["query-folding/pulse-01"]
governing_roles: ["SCHEMA", "SIGNAL", "BENCH"]
---

# Pulse 02: OData fold plan

## Mission

Add OData predicate folding to the existing `slice.fold.v1` plan shape so
consumers backed by OData services can inspect pushdown filters before execution.

## Scope inventory

- Source artifacts:
  - `crates/slice-core/src/lib.rs`
  - `crates/slice-cli/src/main.rs`
  - `examples/icelines-odata-catalog.json`
- Generated/user artifacts:
  - JSON fold plans from `slice plan --backend odata`

## Pre-implementation scout

- Reuse the source partitioning and residual strategy from SQLite folding.
- Lower scalar comparisons, null checks, `in`/`not in`, `between`, and string
  functions to OData filter syntax.
- Keep array/object containment residual because service-specific collection
  semantics vary.

## Deliverables checklist

- [x] Backend-neutral fold predicate output.
- [x] OData lowering for scalar comparisons, nulls, membership, ranges, and
  string functions.
- [x] `slice plan --backend odata` CLI support.
- [x] ICELINES-style OData catalog example.
- [x] README/formalism/TRACKER updates.

## Validation gates

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace -- -D warnings`
- `cargo run -p slice-cli -- plan --backend odata --catalog examples\icelines-odata-catalog.json --expr "player.position eq 'C' and stats.ppg ge 0.8 and stats.tags has 'playoffs'"`
- `git diff --check`

## Non-goals

- Do not execute OData requests.
- Do not discover service metadata or infer navigation joins.
- Do not fold collection containment without an explicit service capability
  contract.
