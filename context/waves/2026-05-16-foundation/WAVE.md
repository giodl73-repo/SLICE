# Wave: Foundation

## Goal

Create SLICE as a low-layer, product-neutral Rust query/expression dependency
with a first working selector contract, CLI smoke path, docs, and repo
operations scaffolding.

## Thesis

If FLETCH, CROP, PEBBLE, PROOF, and domain CLIs share one expression kernel,
they can exchange filters and artifact slices without each repo inventing a
new predicate language.

## Pulse table

| Pulse | Title | Status | Outcome |
|------:|-------|--------|---------|
| 01 | Workspace foundation | done | Created repo skeleton, docs, skills, and first tested contract. |
| 02 | Pebble/CROP/FLETCH examples | done | Added `slice-mock-client` to prove selectors over representative artifact shapes and downstream FLETCH folding. |
| 03 | Typed path catalogs | done | Added numeric comparisons and field catalogs so mock adapters validate selector paths and value types. |
| 04 | Explain reports | done | Added machine-readable explain reports for compiled selectors and mock-client output. |
| 05 | Consumer migration plan | done | Chose CROP frontmatter-query parity as the first real adoption surface and recorded downstream boundaries. |
| 06 | Diagnostic reports | done | Added machine-readable diagnostics with byte offsets and catalog/type context. |
| 07 | Requirement reports | done | Added deduplicated typed field requirements for adapter materialization. |
| 08 | CROP parity mock | done | Proved CROP frontmatter-query behavior in the mock client without moving CROP policy into core. |
| 09 | First consumer adoption | done | CROP adopted `slice-core` for `frontmatter_query` while keeping view and graph policy local. |
| 10 | PEBBLE selector examples | done | PEBBLE added dev-only SLICE tests for document and section metadata selectors. |
| 11 | FLETCH selector examples | done | FLETCH added dev-only SLICE tests for cache-index and active-partition selectors. |
| 12 | PROOF selector examples | done | PROOF added dev-only SLICE tests for artifact manifest row selectors. |
| 13 | ICELINES selector examples | done | ICELINES added dev-only SLICE tests for simple player bio/stat row predicates while keeping hockey query semantics local. |
| 14 | PEBBLE optional runtime selectors | done | PEBBLE added a feature-gated SLICE selector helper for document and section metadata rows. |
| 15 | PROOF artifact runtime selectors | done | PROOF added a SLICE-backed artifact manifest selector helper over prepared compile report rows. |
| 16 | FLETCH row runtime selectors | done | FLETCH added SLICE-backed helpers for cache-index and active-partition rows while keeping fetch/cache and quiver policy local. |
| 17 | ICELINES prepared-row runtime selectors | done | ICELINES added SLICE-backed helpers for prepared player bio/stat rows while keeping hockey query semantics local. |

## Success criteria

- README explains SLICE's low-layer dependency role and first command.
- Product plan records dependency placement and non-goals.
- Wave/pulse scaffolding exists.
- Skills exist for future wave, pulse, and research execution.
- `slice-core` parses and evaluates the first selector grammar.
- Validation commands pass.
