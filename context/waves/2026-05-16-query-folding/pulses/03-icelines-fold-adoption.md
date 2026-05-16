---
wave: query-folding
pulse: 03
date: 2026-05-16
status: done
depends_on: ["query-folding/pulse-01"]
governing_roles: ["SCHEMA", "SIGNAL", "BENCH"]
---

# Pulse 03: ICELINES fold adoption

## Mission

Adopt SLICE SQLite fold planning in ICELINES for prepared player rows without
moving ICELINES schema, joins, execution, ranking, or hockey semantics into
SLICE.

## Scope inventory

- Source artifacts:
  - `C:\src\ICELINES\Cargo.toml`
  - `C:\src\ICELINES\icelines-query\src\slice_selectors.rs`
  - `C:\src\ICELINES\icelines-query\tests\slice_simple_selector.rs`
  - `C:\src\ICELINES\design\specs\slice-selectors.md`
- Generated/user artifacts:
  - `slice.fold.v1` plans from `icelines_query::plan_prepared_player_sqlite_selector`

## Pre-implementation scout

- Reuse ICELINES' existing prepared-row SLICE selector boundary.
- Build a fold catalog that maps `player.*` fields to the `players` source and
  `stats.*` fields to the `stats` source.
- Keep SQL joins and row materialization in ICELINES.

## Deliverables checklist

- [x] Update ICELINES to a SLICE revision with fold planning.
- [x] Add `prepared_player_sqlite_fold_catalog`.
- [x] Add `plan_prepared_player_sqlite_selector`.
- [x] Add tests showing player/stat predicates fold to separate sources.
- [x] Update ICELINES and SLICE docs.

## Validation gates

- ICELINES: `cargo test -p icelines-query slice`
- ICELINES: `git diff --check`

## Non-goals

- Do not execute SQLite from SLICE.
- Do not replace ICELINES query UX or hockey IR.
- Do not infer joins from SLICE expressions.

## Evidence

- ICELINES query tests pass for prepared-row selection, requirements, and
  SQLite fold planning.
