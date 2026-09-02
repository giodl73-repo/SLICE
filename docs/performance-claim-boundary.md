# SLICE Performance Claim Boundary

This boundary closes `SLICE-PF-06`.

SLICE can be described as a reusable typed selector kernel, but not as fast
enough for large artifact scans, FLETCH partitions, SQLite tables, or hot
consumer paths unless sized evidence exists for the claimed surface.

## Promotion Rule

A SLICE performance claim or hot-path replacement must name:

- the consumer and path being replaced;
- the input shape, such as JSON array, JSONL rows, Markdown table, FLETCH
  partition rows, or SQLite source rows;
- the fixture size and row width;
- the selector expression and typed catalog, when a catalog is used;
- the command that generated or evaluated the fixture;
- the measured result or explicit smoke threshold;
- the memory posture, including whether the check is streaming, bounded, or
  allowed to materialize rows;
- residual predicates, downstream joins, ranking, auth, fetching, rendering,
  cache folding, and product policy that stay outside SLICE;
- Performance Engineer, Adapter Boundary Keeper, Contract Checker, and
  Validation Checker dispositions.

If any field is missing, the wording must stay at fixture-scale or
functionality-only scope. A tiny fixture can prove semantics; it cannot prove
large-scan performance or production readiness.

## Current Check

`tests/check-performance-claim-boundary.ps1` generates a 1,000-row temporary
JSONL fixture, runs `slice eval --jsonl --count`, and checks the expected
selection count. This is a smoke-sized guard, not a benchmark. Larger speed,
memory, hot-path, FLETCH, SQLite, or production claims still require the
promotion rule above.
