# Pulse 15: MDLOOM artifact runtime selectors

## Goal

Move MDLOOM from dev-only artifact selector examples to a narrow runtime helper
over already-prepared artifact manifest rows.

## Changes

- Recorded MDLOOM commit `c81ec4d`.
- MDLOOM now depends on `slice-core` at runtime for `mdloom_lib::artifact`
  selectors.
- Selection stays after compile output/manifest generation; SLICE does not own
  Markdown/source fidelity, directives, rendering, compile graph, or artifact
  manifest production.

## Validation

- MDLOOM: `cargo test -q --locked --manifest-path C:\src\mdloom\Cargo.toml --test slice_artifact_selector`
- MDLOOM: `cargo clippy --locked --manifest-path C:\src\mdloom\Cargo.toml -- -D warnings`
- MDLOOM: `cargo test -q --locked --manifest-path C:\src\mdloom\Cargo.toml`
- MDLOOM: `git diff --check`

## Status

Done.
