---
wave: query-folding
pulse: 04
date: 2026-05-16
status: done
depends_on: ["query-folding/pulse-01", "query-folding/pulse-03"]
governing_roles: ["SCHEMA", "SIGNAL", "BENCH"]
---

# Pulse 04: SQLite CLI/runtime layer

## Mission

Let SLICE support SQLite-backed CLIs more directly by adding an optional
SQLite-facing layer that can inspect a database, derive or validate a fold
catalog, plan predicates, and run safe read-only smoke queries while preserving
the pure `slice-core` boundary.

## Scope inventory

- Source artifacts:
  - `crates/slice-core/src/lib.rs`
  - `crates/slice-cli/src/main.rs`
  - New optional SQLite runtime surface, either a `slice-sqlite` crate or a
    narrowly scoped CLI module.
  - `crates/slice-mock-client/src/lib.rs`
  - `examples/icelines-sqlite-catalog.json`
- Generated/user artifacts:
  - SQLite-derived fold catalog JSON.
  - Read-only query plan and smoke execution reports.
  - Mock SQLite fixture output proving the contract.

## Pre-implementation scout

- Re-check the existing `slice.fold.v1` plan shape and keep it stable unless a
  backwards-compatible extension is needed.
- Identify whether adding `rusqlite` belongs in a separate crate to keep
  `slice-core` free of database dependencies.
- Inspect `slice-cli` command structure and decide whether the surface should be
  `slice sqlite ...` or an extension of `slice plan --backend sqlite`.
- Verify how the ICELINES catalog maps logical fields to physical tables and
  columns so generated catalogs do not pretend to know product semantics.

## Deliverables checklist

- [x] Add a SQLite runtime boundary outside `slice-core`.
- [x] Add a CLI command to inspect SQLite tables/columns and emit a draft
      SLICE fold catalog.
- [x] Add a CLI command or flag that combines catalog validation, fold planning,
      and read-only SQLite smoke execution.
- [x] Support consumer-supplied source maps for more than one source so
      independent predicates can be folded to both sides of a consumer-owned
      join.
- [x] Keep joins explicit: SLICE may accept a join contract for smoke execution,
      but must not infer product joins from field names.
- [x] Add mock SQLite tests covering catalog generation, multi-source folding,
      residual local filtering, and unsupported operator diagnostics.
- [x] Update README/formalism docs with the new boundary.

## Validation gates

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace -- -D warnings`
- `cargo run -p slice-cli -- sqlite inspect --db <fixture.db>`
- `cargo run -p slice-cli -- sqlite plan --db <fixture.db> --catalog examples\icelines-sqlite-catalog.json --expr "player.position eq 'C' and stats.ppg ge 0.8"`
- `cargo run -p slice-mock-client`
- `git diff --check`

## Non-goals

- Do not add SQLite, OData, or network dependencies to `slice-core`.
- Do not make SLICE an ORM, migration tool, database admin tool, or product
  query engine.
- Do not infer ICELINES hockey semantics, stat aliases, ranking, windows,
  pagination, or league rules.
- Do not execute writes; SQLite CLI/runtime support is read-only unless a future
  wave explicitly defines a safe fixture-management contract.

## Evidence

- Added `slice-sqlite` as a separate crate so `slice-core` remains free of
  SQLite/database dependencies.
- Added `slice sqlite inspect --db ...` for table/column inspection and draft
  fold-catalog emission.
- Added `slice sqlite plan --db ... --catalog ... --expr ...` for catalog
  validation, `slice.fold.v1` planning, and read-only per-source smoke queries.
- Updated `slice-mock-client` to prove the runtime boundary against the
  ICELINES-style in-memory SQLite fixture.
- Validation passed:
  - `cargo fmt --check`
  - `cargo test`
  - `cargo clippy --workspace -- -D warnings`
  - `cargo run -p slice-cli -- sqlite inspect --db <fixture.db>`
  - `cargo run -p slice-cli -- sqlite plan --db <fixture.db> --catalog examples\icelines-sqlite-catalog.json --expr "player.position eq 'C' and stats.ppg ge 0.8"`
  - `cargo run -p slice-mock-client`
  - `git diff --check`
