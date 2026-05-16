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
| 03 | Consumer migration plan | pending | Identify the first repo surface to replace local query parsing. |

## Success criteria

- README explains SLICE's low-layer dependency role and first command.
- Product plan records dependency placement and non-goals.
- Wave/pulse scaffolding exists.
- Skills exist for future wave, pulse, and research execution.
- `slice-core` parses and evaluates the first selector grammar.
- Validation commands pass.
