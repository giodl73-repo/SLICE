# Consumer migration plan

## Goal

Move one real downstream predicate surface onto SLICE without turning
`slice-core` into a product engine. The first migration should prove that SLICE
can replace local low-level predicate parsing while each consumer keeps schema,
policy, rendering, ranking, cache folding, and user-facing command ownership.

## Migration order

1. **CROP frontmatter query parity**
   - Target: a narrow adapter for CROP metadata/frontmatter clauses that already
     look like row predicates.
   - SLICE owns: parsing, typed path validation, operator compatibility,
     evaluation over adapter-provided values, and explain reports.
   - CROP keeps: view recipes, graph cuts, link health, corpus status policy,
     prefix caches, output formats, and compatibility flags.
   - Gate: CROP's existing frontmatter-query fixtures pass through a SLICE
     adapter with no behavior drift for supported clauses.
   - Local proof: `slice-mock-client` includes a CROP frontmatter parity adapter
     for top-level fields, array-like tag strings, and missing-field `ne`
     semantics.

2. **PEBBLE metadata selector examples**
   - Target: document/section metadata filters used by context-pack consumers.
   - SLICE owns: reusable metadata selector evaluation against Pebble-shaped
     values.
   - PEBBLE keeps: `pebble.v1` schema, chunking, provenance, and pack emission.
   - Gate: examples demonstrate identical selected document/section IDs before
     and after the adapter.

3. **FLETCH manifest and partition selectors**
   - Target: filter cacheline/partition manifest rows before FLETCH folds them.
   - SLICE owns: selecting manifest rows and explaining the required fields.
   - FLETCH keeps: cacheline profiles, active partition sets, rollups, quiver
     candidates, fetch/cache execution, and policy gates.
   - Gate: selected partition rows are stable, and folded quiver output remains
     produced by FLETCH-side code.

4. **PROOF report and CROP-backed slice filters**
   - Target: report-side filtering after PROOF has artifacts or CROP side-info
     rows in hand.
   - SLICE owns: low-level predicates over prepared rows.
   - PROOF keeps: Markdown/source fidelity, directives, compile graph,
     rendering, and artifact manifests.
   - Gate: report output remains byte-stable except for intentionally selected
     rows.

5. **ICELINES simple bio/stat adapter**
   - Target: only simple player bio/stat filters that map cleanly to typed row
     fields.
   - SLICE owns: the low-level predicate kernel.
   - ICELINES keeps: hockey query UX, stat IDs, aliases, windows, career
     aggregation, leaderboards, similarity, ranking, percentiles, and data
     requirements.
   - Gate: adapter tests prove parity for simple filters while advanced query
     features stay in ICELINES.

## Readiness checklist

- A consumer-owned field catalog maps every supported path to a `ValueType`.
- Unsupported legacy clauses fail before evaluation with an explicit diagnostic.
- Existing consumer fixtures run in both old and SLICE-backed modes until parity
  is proven.
- Explain reports are surfaced in debug/report output so maintainers can inspect
  fields, operators, and literals.
- No downstream policy, ranking, rendering, fetching, graph selection, or cache
  folding enters `slice-core`.

## Role review

- **Expression Kernel Keeper:** Pass if each migration starts with existing
  predicate-shaped clauses and avoids adding broad language features without
  fixture pressure.
- **Adapter Boundary Keeper:** Pass if CROP, Pebble, FLETCH, PROOF, and ICELINES
  each keep their domain policy and expose only field catalogs plus row values to
  SLICE.
- **Diagnostics Auditor:** Needs the next pulse to improve byte-span and
  compatibility diagnostics before a real CROP adapter is merged.
- **Performance Engineer:** Pass for fixture-scale adapters; larger manifests
  should benchmark selection over prepared rows before replacing hot paths.
- **Stakeholder POV:** CROP is the first useful migration because its
  frontmatter predicates are already the closest match to SLICE's current
  grammar. ICELINES should remain later because its high-value query layer is
  domain semantics, not low-level row filtering.
