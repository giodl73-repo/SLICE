# SLICE Product Plan

## Thesis

Portfolio repos already need reusable selectors: CROP view filters, Pebble
metadata predicates, FLETCH manifest slices, PROOF report filters, and ICELINES
query surfaces. SLICE extracts the shared expression kernel so each repo can
reuse one low-layer contract while keeping product semantics local.

## Dependency placement

SLICE is a shared dependency below FLETCH, CROP, PEBBLE, PROOF, and domain CLIs.
It should depend only on stable parsing/evaluation crates and common data types.
Consumers adapt their own records into SLICE values.

## Waves

1. **Foundation** - create the Rust workspace, first selector grammar, CLI smoke
   path, docs, and repo operations scaffolding.
2. **Adapters** - add optional adapters or examples for Pebble documents, CROP
   units/views, and FLETCH manifests without pulling product policy into core.
3. **Diagnostics and planning** - improve parser spans, actionable diagnostics,
   and reusable explain output for agents and CLIs.
4. **Adoption** - migrate first consumer filters from one-off parsing to SLICE.

## Non-goals

- No product-owned ranking, graph selection, source fetching, cache policy, or
  document rendering in `slice-core`.
- No private implementation copy; SLICE is clean-room OSS infrastructure.
- No broad language features until a consumer has a measured need.
