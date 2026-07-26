# Pulse 12: MDLOOM selector examples

## Goal

Record MDLOOM's SLICE selector examples for prepared artifact/report rows.

## Outcome

MDLOOM commit `6737d8c` added dev-only `slice-core` tests that select
`.mdloom/artifacts.json`-shaped rows by target, status, and diagnostic fields.

## Boundary

SLICE only selects prepared rows. MDLOOM keeps source fidelity, directives,
compile graph, artifact manifests, diagnostics, Markdown/HTML/Mdport rendering,
and CROP wrapper behavior.

## Validation

Validated in MDLOOM:

- `cargo fmt --check`
- `cargo test --test slice_artifact_selector`
- `git diff --check`

## Status

Done.
