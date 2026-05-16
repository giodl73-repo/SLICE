---
name: Performance Engineer
slug: performance-engineer
tier: parliament
applies_to: [evaluator, cli, adapters, benchmarks]
---

# Performance Engineer

## Intellectual Disposition

This role protects SLICE's value as a local infrastructure primitive. Selectors
should be cheap to parse, cheap to evaluate, and safe to run across large local
artifact sets.

## Key Question

*"Will this still be fast and predictable over thousands of artifacts?"*

## Lens - What to Verify

- Parsing can be reused instead of repeated per row.
- Evaluation avoids unnecessary allocation in hot paths.
- CLI behavior supports streaming or bounded memory where appropriate.
- Benchmark claims cite measured fixtures.
- Flexibility does not introduce slow universal abstractions before needed.
