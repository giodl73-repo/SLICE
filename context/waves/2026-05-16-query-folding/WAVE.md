---
wave: query-folding
date_open: 2026-05-16
status: active
source_request: "Generate folded queries for SQL/OData/SQLite sources, including multi-source joins."
---

# Wave: Query folding

## Mission

Add backend-neutral query folding so SLICE can plan which typed expression
subtrees can be pushed into a remote source and which predicates must remain as
local residual filters.

## Claim boundary

SLICE owns expression analysis, typed source requirements, fold diagnostics,
per-source predicate fragments, parameters, and residual expression trees.
Consumers own physical schemas, joins, execution, auth, ranking, pagination,
domain semantics, and whether a folded plan is safe to execute.

## Inputs

- `slice-core` expression trees, typed catalogs, explain reports, requirements,
  and diagnostics.
- ICELINES' SQLite-backed data shape as the first real target pattern.
- Future SQL/OData stores that can accept predicate pushdown.

## Pulse status

| Pulse | Title | Status | Outcome |
|------:|-------|--------|---------|
| 01 | SQLite fold plan | done | Added per-source SQLite predicate planning, residuals, CLI plan output, and a mock SQLite join proof. |
| 02 | OData fold plan | done | Added OData predicate folding on the same per-source plan shape, including CLI output and residual diagnostics. |
| 03 | ICELINES fold adoption | done | ICELINES adopted prepared-player SQLite fold planning while keeping schema joins and execution local. |

## Validation gates

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace -- -D warnings`
- `cargo run -p slice-cli -- plan --backend sqlite --catalog examples\icelines-sqlite-catalog.json --expr "..."`
- `cargo run -p slice-cli -- plan --backend odata --catalog examples\icelines-odata-catalog.json --expr "..."`
- `cargo run -p slice-mock-client`
- `git diff --check`

## Done criteria

- SLICE emits a stable JSON fold plan with backend, source predicates,
  parameters, requirements, residual tree, and diagnostics.
- SQLite and OData are implemented backend targets.
- Multi-source expressions can fold independent `and` branches to both sides of
  a join while leaving unsupported branches as residual local filters.
- The mock client demonstrates an ICELINES-style SQLite join using folded
  predicates from more than one source.
- ICELINES exposes a prepared-player SQLite fold helper that consumers can attach
  to ICELINES-owned joins.

## Non-goals

- No database connections or query execution in `slice-core`.
- No SQL join planning, table discovery, migrations, or ORM behavior in SLICE.
- No product-specific ICELINES hockey semantics in SLICE.
- No source execution or service-specific OData metadata discovery.
