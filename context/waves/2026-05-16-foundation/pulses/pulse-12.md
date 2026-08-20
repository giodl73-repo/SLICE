# Pulse 12: PROOF selector examples

## Goal

Record PROOF's SLICE selector examples for prepared artifact/report rows.

## Outcome

PROOF commit `6737d8c` added dev-only `slice-core` tests that select
`.proof/artifacts.json`-shaped rows by target, status, and diagnostic fields.

## Boundary

SLICE only selects prepared rows. PROOF keeps source fidelity, directives,
compile graph, artifact manifests, diagnostics, Markdown/HTML/Mdport rendering,
and MDCROP wrapper behavior.

## Validation

Validated in PROOF:

- `cargo fmt --check`
- `cargo test --test slice_artifact_selector`
- `git diff --check`

## Status

Done.
