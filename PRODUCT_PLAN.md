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

## Formalism

SLICE's formalism is a typed selector pipeline:

`surface syntax -> parsed expression -> normalized expression -> resolved typed IR -> planned requirements -> reusable evaluator -> explain output`

The core owns boolean composition, paths, typed predicates, diagnostics, and
evaluation over adapter-provided values. Consumers own field catalogs, aliases,
data requirements, ranking, rendering, and domain policy. See
[`docs/specs/formalism.md`](docs/specs/formalism.md).

## Consumer points of view

- **CROP:** replace local metadata/frontmatter predicate parsing only after
  parity is proven; CROP keeps graph cuts, corpus health, and view policy.
- **PEBBLE:** make document and section metadata easy to select without changing
  `pebble.v1` schema ownership.
- **FLETCH:** slice manifests, cachelines, partitions, and quivers locally
  without implying fetch/cache execution; SLICE selects rows, FLETCH folds them
  into cacheline/quiver plans.
- **PROOF:** filter generated reports and CROP-backed slices without moving
  Markdown rendering or source fidelity into SLICE.
- **ICELINES:** keep domain-friendly hockey query commands and aliases while
  reusing low-level expression pieces where they help.

## Waves

1. **Foundation** - create the Rust workspace, first selector grammar, CLI smoke
   path, docs, and repo operations scaffolding.
2. **Adapters** - add optional adapters or examples for Pebble documents, CROP
   units/views, and FLETCH manifests/partitions without pulling product policy
   or FLETCH cacheline folding into core.
   - Current local validation: `slice-mock-client` exercises Pebble-shaped,
     CROP-like, FLETCH-like, and ICELINES-like rows while keeping FLETCH folding
     in the client layer.
3. **Typed catalogs** - add field catalogs and numeric comparisons so adapters
   can validate paths and operator/type compatibility before evaluation.
4. **Explain reports** - emit machine-readable field/operator/literal summaries
   for compiled selectors so agents and downstream CLIs can show what a selector
   depends on.
5. **Diagnostics and planning** - improve parser spans, actionable diagnostics,
   and reusable requirement output for agents and CLIs.
6. **ICELINES-shaped IR experiment** - prototype an adapter for simple
   ICELINES bio/stat filters while keeping hockey commands, stat catalogs,
   windows, career aggregation, similarity search, and ranking in ICELINES.
7. **Adoption** - migrate first consumer filters from one-off parsing to SLICE.
   The first candidate is CROP frontmatter-query parity; see
   [`docs/plans/consumer-migration.md`](docs/plans/consumer-migration.md).

## Non-goals

- No product-owned ranking, graph selection, source fetching, cache policy, or
  document rendering in `slice-core`.
- No private implementation copy; SLICE is clean-room OSS infrastructure.
- No broad language features until a consumer has a measured need.
