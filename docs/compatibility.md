# SLICE compatibility policy

SLICE is a pre-1.0 shared selector foundation. Compatibility is deliberate
because product repositories depend on its grammar, typed catalogs, evaluation
semantics, machine-readable reports, and backend fold plans.

## Protected contract

The protected surface includes:

- public `slice-core` APIs, AST types, operators, literals, catalogs, compiled
  selectors, and errors;
- selector grammar, precedence, path handling, literal escaping, missing-field
  behavior, and typed validation;
- matching semantics for equality, ordering, ranges, membership, null,
  containment, quantifiers, prefixes, suffixes, and boolean composition;
- deterministic requirement ordering and explain-tree structure;
- `slice.diagnostic.v1`, `slice.explain.v1`, `slice.parse_explain.v1`,
  `slice.requirements.v1`, and `slice.fold.v1` report schemas;
- SQLite and OData predicate text, parameter ordering, source partitioning,
  residual filters, and diagnostic meanings; and
- the boundary that consumers own schemas, joins, ranking, and domain policy.

Internal refactoring is compatible only when these observable contracts remain
stable.

## Versioning rules

- Additive operators, fields, or report data may remain within the current
  `0.y` line when existing selectors and consumers retain their behavior.
- Breaking APIs, grammar, precedence, matching semantics, validation, report
  schemas, diagnostic meanings, fold output, or deterministic ordering require
  a minor-version bump while the affected crate is below `1.0`.
- Prefer deprecation plus migration notes before removing a public item.
- A breaking change must identify affected consumers and include selector,
  catalog, or adapter migration guidance.
- Downstream repositories should pin commits for reproducible evidence.
  Branch consumers must run the downstream rehearsal before updating.

## Foundation tests

From the SLICE repository:

```powershell
cargo test -p slice-core -p slice-sqlite -p slice-cli -p slice-mock-client
```

The core tests protect grammar, matching, typing, reports, diagnostics, and fold
plans. The mock client protects the documented MDPORT, MDCROP, FLETCH, and
ICELINES adapter boundaries.

## Downstream breakage rehearsal

FLETCH is the required first external consumer rehearsal because it compiles
selectors over real cache-index and active-partition reports before applying
FLETCH-owned policy gates and quiver folding.

From the FLETCH repository:

```powershell
python tools\repo_map.py write-cargo-config
cargo test -p fletch-core slice_selects_cache_index_rows_before_fletch_policy_gates
cargo test -p fletch-core slice_selects_active_partitions_before_quiver_folding
```

The generated, ignored Cargo config patches `slice-core` to the sibling SLICE
checkout. A compile failure exposes API breakage. Selection or ordering
failures expose grammar, typing, evaluation, or adapter-boundary drift.

SLICE foundation changes are not ready until the affected foundation tests and
the FLETCH rehearsal pass.

## Performance Claim Boundary

The performance claim boundary is part of the compatibility gate.

`SLICE-PF-06` keeps speed, large-scan, hot-path, FLETCH partition, SQLite table,
and production-readiness claims out of the protected contract until sized
evidence exists. Use [`performance-claim-boundary.md`](performance-claim-boundary.md)
before replacing consumer hot paths or publishing performance language. The
current `tests/check-performance-claim-boundary.ps1` guard proves a 1,000-row
JSONL smoke path only; it is not benchmark evidence.
