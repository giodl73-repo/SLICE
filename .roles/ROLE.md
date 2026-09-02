# SLICE - Role Index

Roles for keeping SLICE low-layer, product-neutral, and useful as a shared
query/expression dependency. Read this before opening any role file.

---

## Parliament roles

Adversarial infrastructure voices for SLICE design reviews. They keep
`slice-core` reusable without letting a downstream repo turn it into product
logic.

| File | Voice | Primary tension |
|---|---|---|
| `parliament/expression-kernel-keeper.md` | Expression Kernel Keeper | Small, stable selector grammar vs. general-purpose language creep |
| `parliament/adapter-boundary-keeper.md` | Adapter Boundary Keeper | Product-neutral evaluation vs. MDCROP/FLETCH/MDPORT schema leakage |
| `parliament/diagnostics-auditor.md` | Diagnostics Auditor | Actionable parse/eval errors vs. opaque parser failures |
| `parliament/performance-engineer.md` | Performance Engineer | Fast local artifact scans vs. slow universal abstractions |

---

## Editorial roles

Quality gates before a wave or pulse is considered ready. Run after parliament,
not instead of it.

| File | Role | Checks |
|---|---|---|
| `editorial/scope-keeper.md` | Scope Keeper | SLICE stays a selector/expression kernel |
| `editorial/contract-checker.md` | Contract Checker | Grammar, operators, examples, and compatibility stay coherent |
| `editorial/validation-checker.md` | Validation Checker | Pulses include concrete fmt/test/smoke commands |

---

## Stakeholder roles

Consumer lenses for understanding how a low-layer expression primitive serves
real downstream tools.

| File | Stakeholder | Primary concern |
|---|---|---|
| `stakeholders/mdcrop-view-author.md` | MDCROP View Author | Reusable metadata/frontmatter predicates without losing MDCROP policy |
| `stakeholders/mdport-pack-consumer.md` | Mdport Pack Consumer | Select document/section metadata without changing Mdport schema |
| `stakeholders/fletch-manifest-user.md` | FLETCH Manifest User | Slice manifests/cachelines/partitions with stable predicates |
| `stakeholders/proof-report-author.md` | PROOF Report Author | Filter generated reports without moving rendering into SLICE |
| `stakeholders/icelines-query-user.md` | ICELINES Query User | Keep domain-friendly query UX while reusing low-level expressions |

---

## PITFALL gates

| Pitfall | Gate | Required roles |
|---|---|---|
| `SLICE-PF-06` | Performance claim boundary. Blocks large artifact scan, FLETCH partition, SQLite table, production-speed, and hot-path replacement claims until fixture size, row width, command, selector, catalog posture, memory posture, downstream responsibilities, measured result or smoke threshold, and review dispositions are visible. | Performance Engineer; Adapter Boundary Keeper; Contract Checker; Validation Checker |
