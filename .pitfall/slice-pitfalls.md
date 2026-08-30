# SLICE Pitfalls

## SLICE-PF-01: Product Policy Leaks Into The Kernel

**Status:** MITIGATED

**Pattern:** A consumer asks SLICE to own graph cuts, schema design, cacheline
folding, report rendering, hockey ranking, auth, or execution policy.

**Domain:** `slice-core`, adapters, mock client, consumer migrations, and fold
planning.

**Detection difficulty:** The first reusable hook can look generic until a
second consumer reveals it was product-specific.

**Structural solution:** Keep field catalogs, adapters, joins, ranking, and
policy downstream; require adapter-boundary review for migrations.

**Evidence:** `docs/specs/formalism.md`, `docs/plans/consumer-migration.md`,
and `.roles/parliament/adapter-boundary-keeper.md`.

## SLICE-PF-02: Grammar Expansion Becomes A General Language

**Status:** MITIGATED

**Pattern:** Selector convenience adds broad programming-language features
without repeated consumer pressure and compatibility evidence.

**Domain:** parser, AST, operators, literals, boolean composition, CLI syntax,
and compatibility policy.

**Detection difficulty:** Each individual operator can look small while the
combined surface becomes hard for downstream repos to support.

**Structural solution:** Add grammar only with tests, compatibility notes, and
named consumer evidence.

**Evidence:** `docs/compatibility.md`, `.roles/parliament/expression-kernel-keeper.md`,
and `docs/reviews/slice-plan-role-review.md`.

## SLICE-PF-03: Fold Plan Becomes Backend Execution

**Status:** MITIGATED

**Pattern:** SQLite or OData fold plans are treated as permission for SLICE to
execute joins, enforce auth, rank results, or own backend-specific product
queries.

**Domain:** `slice-core` fold plans, `slice-sqlite`, CLI planning commands, and
consumer database integrations.

**Detection difficulty:** A successful fold looks close to a query plan, but
product execution still requires consumer-owned context.

**Structural solution:** Emit predicates, parameters, requirements, residuals,
and diagnostics only; leave execution and joins to consumers.

**Evidence:** `context/waves/2026-05-16-query-folding/WAVE.md`,
`docs/specs/formalism.md`, and README SQLite workbench docs.

## SLICE-PF-04: Diagnostics Regress To Strings

**Status:** MITIGATED

**Pattern:** Parser, catalog, type, or fold errors become opaque human strings
that agents and downstream CLIs cannot inspect mechanically.

**Domain:** `SliceError`, CLI explain output, diagnostic reports, validation
failures, and compatibility rules.

**Detection difficulty:** Human-readable errors can look sufficient until an
adapter needs byte offsets, expected types, or structured remediation.

**Structural solution:** Preserve `slice.diagnostic.v1`, parse explain,
requirements, and typed diagnostic details under compatibility policy.

**Evidence:** `docs/compatibility.md`, `crates/slice-core/src/lib.rs`, and the
PITFALL adoption update to `docs/reviews/formalism-role-review.md`.

## SLICE-PF-05: README Smoke Paths Drift From Submodule Layout

**Status:** MITIGATED

**Pattern:** README commands use a historical sibling `../TRACKER` path that no
longer resolves from the portfolio submodule checkout.

**Domain:** README validation commands, TRACKER Markdown-table examples,
portfolio adoption, and agent copy/paste workflows.

**Detection difficulty:** Core tests pass, but a documented smoke command fails
only when run from the repo's actual submodule location.

**Structural solution:** Keep README examples aligned with the checked-out
portfolio path and validate at least one Markdown-table command during adoption.

**Evidence:** PITFALL adoption updated README commands to
`../../../dependency-systems/slice-usage.md` after the documented smoke failed.

## SLICE-PF-06: Performance Claims Outrun Sized Fixtures

**Status:** OPEN

**Pattern:** SLICE is described as fast enough for large artifact scans or hot
consumer paths without sized JSON, JSONL, FLETCH partition, or SQLite fixtures
and benchmark evidence.

**Domain:** performance claims, compiled selectors, CLI scans, mock client,
FLETCH adoption, and backend fold planning.

**Detection difficulty:** Small fixtures pass instantly, so performance debt is
invisible until a real manifest or table becomes large.

**Structural solution:** Add sized fixtures and bounded-memory/performance
checks before making speed claims or replacing hot consumer paths.

**Evidence:** `docs/reviews/formalism-role-review.md` and
`docs/reviews/slice-plan-role-review.md` performance follow-ups.
