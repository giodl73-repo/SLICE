# Consumer migration plan

## Goal

Move one real downstream predicate surface onto SLICE without turning
`slice-core` into a product engine. The first migration should prove that SLICE
can replace local low-level predicate parsing while each consumer keeps schema,
policy, rendering, ranking, cache folding, and user-facing command ownership.

## Migration order

1. **MDCROP frontmatter query parity**
   - Status: adopted in MDCROP commit `02006c9`; `frontmatter_query` now uses
     `slice-core` for parsing, catalog validation, requirements, and evaluation.
   - Target: a narrow adapter for MDCROP metadata/frontmatter clauses that already
     look like row predicates.
   - SLICE owns: parsing, typed path validation, operator compatibility,
     evaluation over adapter-provided values, and explain reports.
   - MDCROP keeps: view recipes, graph cuts, link health, corpus status policy,
     prefix caches, output formats, and compatibility flags.
   - Gate: MDCROP's existing frontmatter-query fixtures pass through a SLICE
     adapter with no behavior drift for supported clauses. This gate is met for
     the current `eq`, `ne`, `has`, and `and` surface.
   - Local mdloom: `slice-mock-client` includes a MDCROP frontmatter parity adapter
     for top-level fields, array-like tag strings, and missing-field `ne`
     semantics.

2. **MDPORT metadata selector examples**
   - Status: optional runtime adoption in MDPORT commit `9239143`; the
     feature-gated `slice` helper selects documents and sections with
     `slice-core` while preserving MDPORT's product-neutral schema boundary.
   - Target: document/section metadata filters used by context-pack consumers.
   - SLICE owns: reusable metadata selector evaluation against Mdport-shaped
     values.
   - MDPORT keeps: `mdport.v1` schema, chunking, provenance, pack emission, and
     the choice of whether consumers enable selector helpers.
   - Gate: examples demonstrate identical selected document/section IDs before
     and after the adapter. This gate is met by MDPORT's feature-enabled
     document and section selector tests.

3. **FLETCH manifest and partition selectors**
   - Status: runtime adoption in FLETCH commit `b7ddbe7`; `fletch-core`
     exposes SLICE-backed helpers for cache-index and active-partition row
     selectors.
   - Target: filter cacheline/partition manifest rows before FLETCH folds them.
   - SLICE owns: selecting manifest rows and explaining the required fields.
   - FLETCH keeps: cacheline profiles, active partition sets, rollups, quiver
     candidates, fetch/cache execution, and policy gates.
   - Gate: selected partition rows are stable, and folded quiver output remains
     produced by FLETCH-side code. This gate is met by FLETCH's cache-index and
     active-partition runtime selector tests.

4. **MDLOOM report and MDCROP-backed slice filters**
   - Status: runtime adoption in MDLOOM commit `c81ec4d`; `mdloom_lib::artifact`
     exposes a SLICE-backed helper for filtering prepared artifact manifest
     rows.
   - Target: report-side filtering after MDLOOM has artifacts or MDCROP side-info
     rows in hand.
   - SLICE owns: low-level predicates over prepared rows.
   - MDLOOM keeps: Markdown/source fidelity, directives, compile graph,
     rendering, and artifact manifests.
   - Gate: report output remains byte-stable except for intentionally selected
     rows. This gate is met because selection happens after MDLOOM has already
     compiled and produced manifest-shaped rows.

5. **ICELINES simple bio/stat adapter**
   - Status: runtime adoption in ICELINES commit `3848d51`; `icelines-query`
     exposes SLICE-backed helpers for simple prepared player bio/stat row
     predicates.
   - Target: only simple player bio/stat filters that map cleanly to typed row
     fields.
   - SLICE owns: the low-level predicate kernel.
   - ICELINES keeps: hockey query UX, stat IDs, aliases, windows, career
     aggregation, leaderboards, similarity, ranking, percentiles, and data
     requirements.
   - Gate: adapter tests prove parity for simple filters while advanced query
     features stay in ICELINES. This gate is met by ICELINES' prepared-player
     selector tests for simple player position, nationality, and stat row
     filters.

6. **TRACKER Markdown table selectors**
   - Status: CLI/tool adoption in pulse 18; `slice eval --markdown-table`
     selects rows from TRACKER and wave Markdown tables.
   - Target: repo planning and dependency-tracker rows that are already
     represented as Markdown tables.
   - SLICE owns: table-row projection, selector parsing, and row predicate
     evaluation.
   - TRACKER keeps: tracker file layout, dependency taxonomy, status meanings,
     and planning policy.
   - Gate: a real TRACKER table can be selected from the CLI and emitted as
     JSONL rows without requiring TRACKER-specific code.

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
- **Adapter Boundary Keeper:** Pass if MDCROP, Mdport, FLETCH, MDLOOM, and ICELINES
  each keep their domain policy and expose only field catalogs plus row values to
  SLICE.
- **Diagnostics Auditor:** Pass for current examples because `slice-core` now
  exposes diagnostic and requirements reports; downstream adapters still own how
  those reports are surfaced to authors.
- **Performance Engineer:** Pass for fixture-scale adapters; larger manifests
  should benchmark selection over prepared rows before replacing hot paths.
- **Stakeholder POV:** MDCROP is the first useful migration because its
  frontmatter predicates are already the closest match to SLICE's current
  grammar. ICELINES should remain later because its high-value query layer is
  domain semantics, not low-level row filtering.
