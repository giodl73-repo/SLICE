# SLICE Principles

## SLICE-P-01: Selector Kernel, Not Product Engine

**Status:** ACTIVE

**Statement:** SLICE owns reusable selector parsing, typing, evaluation,
explain, diagnostics, requirements, and fold planning; consumers own schemas,
joins, ranking, execution, auth, rendering, and domain policy.

**Decision rule:** Any new capability must identify what remains in MDCROP,
MDPORT, FLETCH, PROOF, ICELINES, or another consumer before it enters SLICE.

**Evidence:** `README.md`, `PRODUCT_PLAN.md`, `docs/specs/formalism.md`, and
`.roles/parliament/adapter-boundary-keeper.md`.

## SLICE-P-02: Compatibility Is Observable Behavior

**Status:** ACTIVE

**Statement:** Grammar, operator semantics, missing-field behavior, report
schemas, fold output, deterministic ordering, and diagnostics are protected
contracts even before 1.0.

**Decision rule:** Refactors are compatible only when observable selector,
report, diagnostic, and fold behavior remains stable or receives documented
versioning and migration guidance.

**Evidence:** `docs/compatibility.md` and `.roles/editorial/contract-checker.md`.

## SLICE-P-03: Consumers Prove Adoption At Their Boundary

**Status:** ACTIVE

**Statement:** Runtime adoption is accepted only when consumer-owned tests show
the same rows, reports, or artifacts remain selected without moving product
semantics into SLICE.

**Decision rule:** Migration claims must cite local mock-client evidence and,
when applicable, downstream consumer tests or rehearsals.

**Evidence:** `docs/plans/consumer-migration.md`,
`crates/slice-mock-client/src/lib.rs`, and `docs/compatibility.md`.

## SLICE-P-04: Diagnostics Are Machine Contracts

**Status:** ACTIVE

**Statement:** Parse, type, catalog, and fold failures expose stable
machine-readable reports, not just human parser strings.

**Decision rule:** Error-shape changes must preserve `slice.diagnostic.v1` and
related explain/requirements contracts or follow compatibility rules.

**Evidence:** `docs/specs/formalism.md`, `crates/slice-core/src/lib.rs`, and
`docs/reviews/formalism-role-review.md`.

## SLICE-P-05: Fold Plans Are Not Query Execution

**Status:** ACTIVE

**Statement:** SQLite and OData fold plans describe what can be pushed to a
backend; consumers still own joins, execution, auth, ranking, pagination policy,
and result display.

**Decision rule:** Backend planning may emit predicates, parameters,
requirements, residuals, and diagnostics, but `slice-core` must not execute
product queries or infer joins from field names.

**Evidence:** `README.md`, `docs/specs/formalism.md`, and
`context/waves/2026-05-16-query-folding/WAVE.md`.
