# SLICE Plan Role Review

Scope: `README.md`, `PRODUCT_PLAN.md`, and
`context/waves/2026-05-16-foundation/WAVE.md`.

## Verdict

PASS WITH FOLLOW-UPS. The plan correctly positions SLICE as a low-layer,
product-neutral query/expression dependency. The main follow-up is to keep
consumer adoption evidence-driven: prove CROP/Pebble/FLETCH fixtures before
making runtime dependencies broad.

## Parliament review

| Role | Finding | Action |
|---|---|---|
| Expression Kernel Keeper | PASS. The first grammar is intentionally small: paths, `eq`, `ne`, `has`, `contains`, and `and`. | Do not add broader language features until at least two consumers need them. |
| Adapter Boundary Keeper | PASS. The plan keeps product schemas in adapters/downstream repos and keeps `slice-core` product-neutral. | Keep Pebble/CROP/FLETCH examples as fixtures or optional adapters, not core policy. |
| Diagnostics Auditor | NEEDS FOLLOW-UP. Parser errors have offsets, but the plan does not yet require invalid-example docs or machine-readable explain output. | Wave 03 should add invalid examples and explain output acceptance criteria. |
| Performance Engineer | NEEDS FOLLOW-UP. The plan names local artifact scans, but no benchmark fixtures or streaming targets exist yet. | Wave 02 should include JSON/JSONL fixture sizes and a bounded-memory CLI check. |

## Editorial review

| Role | Finding | Action |
|---|---|---|
| Scope Keeper | PASS. Non-goals explicitly exclude fetching, caching, graph cuts, rendering, and domain ranking. | Keep this as a release gate for every grammar expansion. |
| Contract Checker | PASS. README syntax matches the first implemented parser and crate/command names align with SLICE. | Add compatibility notes before changing operator semantics. |
| Validation Checker | PASS. Pulse 01 lists `cargo fmt --check`, `cargo test`, CLI smoke, and `git diff --check`. | Add adapter contract tests when Wave 02 examples become supported surfaces. |

## Stakeholder review

| Stakeholder | Finding | Action |
|---|---|---|
| CROP View Author | PASS WITH RISK. CROP is the strongest first adopter because it already has `frontmatter_query`. | First adoption should be parity-only before expanding syntax. |
| Pebble Pack Consumer | PASS. Pebble metadata selectors are a clean example as long as SLICE does not own `pebble.v1`. | Add document and section metadata fixtures in Wave 02. |
| FLETCH Manifest User | PASS. Manifest/cacheline slicing fits SLICE's low-layer role. | Start with JSON fixtures before linking FLETCH runtime crates. |
| PROOF Report Author | PASS. Report filtering is a plausible later use; rendering stays out of SLICE. | Keep PROOF adoption later than CROP/Pebble/FLETCH examples. |
| ICELINES Query User | PASS. There is an explicit ICELINES POV: keep `icelines query` domain-friendly and reuse only low-level expression pieces. | Do not replace ICELINES command UX with raw SLICE expressions; use ICELINES as an ergonomics reference. |

## ICELINES POV

Yes, SLICE has an ICELINES point of view. The dedicated role is
`.roles/stakeholders/icelines-query-user.md`, and the product plan now records
that ICELINES should keep its domain-friendly hockey query commands, aliases,
and metrics while reusing SLICE only for low-level expression mechanics where
that helps.

Follow-up clarification: `docs/specs/formalism.md` now makes this concrete.
SLICE should extract the reusable typed selector pipeline underneath
ICELINES-style query engines, not replace `icelines query` or move hockey
semantics into `slice-core`.

Second clarification: FLETCH cacheline/partition intelligence sits above SLICE.
SLICE can select manifest or partition rows; FLETCH owns folding those rows into
cacheline, rollup, quiver, and gate plans.
