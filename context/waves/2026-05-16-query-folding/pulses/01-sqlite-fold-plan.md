---
wave: query-folding
pulse: 01
date: 2026-05-16
status: done
depends_on: ["foundation/pulse-24"]
governing_roles: ["SCHEMA", "SIGNAL", "BENCH"]
---

# Pulse 01: SQLite fold plan

## Mission

Make SLICE produce a typed fold plan for SQLite-compatible predicates, including
multi-source `and` partitioning and residual local expressions.

## Scope inventory

- Source artifacts:
  - `crates/slice-core/src/lib.rs`
  - `crates/slice-cli/src/main.rs`
  - `crates/slice-mock-client/src/lib.rs`
  - `examples/icelines-sqlite-catalog.json`
- Generated/user artifacts:
  - JSON fold plans from `slice plan --backend sqlite`
  - Mock-client report section for SQLite folded ICELINES selection

## Pre-implementation scout

- Confirm which SLICE operators map cleanly to SQLite predicates.
- Keep field-to-source and field-to-column mapping outside the old simple field
  catalog so existing consumers are not forced into SQL concepts.
- Treat multi-source `and` as independently foldable source predicates; do not
  fold cross-source `or` until a consumer supplies safe join semantics.

## Deliverables checklist

- [x] `slice-core` fold catalog and fold plan structs.
- [x] SQLite predicate lowering with parameters.
- [x] Residual expression tree for unsupported or unsafe subtrees.
- [x] `slice plan --backend sqlite` CLI output.
- [x] Mock SQLite join mdloom with folded predicates from `players` and `stats`.
- [x] README/formalism/TRACKER updates.

## Evidence

- `slice plan --backend sqlite` emits `slice.fold.v1` with `players` and `stats`
  source predicates plus a residual `stats.tags has 'playoffs'` filter.
- `slice-mock-client` builds an in-memory SQLite database, applies the folded
  source predicates through a consumer-owned join, and applies the residual
  SLICE filter locally.

## Validation gates

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace -- -D warnings`
- `cargo run -p slice-cli -- plan --backend sqlite --catalog examples\icelines-sqlite-catalog.json --expr "player.position eq 'C' and stats.ppg ge 0.8"`
- `cargo run -p slice-mock-client`
- `git diff --check`

## Non-goals

- Do not execute SQL from `slice-core`.
- Do not add OData rendering in this pulse.
- Do not infer joins or source schemas from field names.
