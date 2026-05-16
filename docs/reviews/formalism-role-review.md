# SLICE Formalism Role Review

Scope: `docs/specs/formalism.md` after adding the ICELINES and FLETCH layering
model.

## Verdict

PASS. This is the right direction if SLICE stays a typed selector kernel and
does not become the owner of every downstream plan.

## Role findings

| Role | Finding | Direction |
|---|---|---|
| Expression Kernel Keeper | PASS. The formalism names a shape-by-construction IR and keeps ranking, fetching, cache folding, graph cuts, and domain windows out of core. | Add range/set/pattern predicates only with tests and compatibility notes. |
| Adapter Boundary Keeper | PASS. FLETCH cacheline folding is explicitly above SLICE; ICELINES hockey semantics stay in ICELINES. | Put product field catalogs in adapters or consumers, not `slice-core`. |
| Diagnostics Auditor | PASS WITH FOLLOW-UP. The formalism includes explain output and diagnostics as first-class passes. | Next implementation wave should add machine-readable parse/type diagnostics. |
| Performance Engineer | PASS WITH FOLLOW-UP. Reusable compiled selectors and FLETCH row scans are the right performance shape. | Add sized JSON/JSONL/FLETCH partition fixtures before claims about speed. |
| Scope Keeper | PASS. The doc says SLICE should not know everything. | Keep that sentence as a gate for future adoption PRs. |
| Contract Checker | PASS. The formalism is consistent with README and product plan. | Add tests as each target IR node becomes real code. |
| Validation Checker | PASS. Docs-only change validated with existing cargo checks. | Future formalism changes should include code or golden examples. |

## Consumer direction

| Consumer | Role verdict |
|---|---|
| CROP | Best first runtime migration candidate because its current frontmatter query is simple and parity-testable. |
| PEBBLE | Best schema/fixture candidate because metadata selection is product-neutral. |
| FLETCH | Best planning candidate because SLICE can select rows, then FLETCH can fold selected rows into cacheline/quiver plans. |
| PROOF | Later consumer for report filters; avoid coupling to rendering. |
| ICELINES | Best architecture reference now; later adapter experiment for simple bio/stat filters only. |

## Layer decision

SLICE should not know about every system. SLICE should know how to represent,
type-check, evaluate, and explain selector intent. CROP, PEBBLE, FLETCH, PROOF,
and ICELINES should know how selector intent maps to their artifacts and
execution plans.

For FLETCH specifically: SLICE can choose the rows; FLETCH decides how those
rows fold into cachelines, active partitions, rollups, and quivers.
