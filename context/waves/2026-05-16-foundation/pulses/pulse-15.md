# Pulse 15: PROOF artifact runtime selectors

## Goal

Move PROOF from dev-only artifact selector examples to a narrow runtime helper
over already-prepared artifact manifest rows.

## Changes

- Recorded PROOF commit `c81ec4d`.
- PROOF now depends on `slice-core` at runtime for `proof_lib::artifact`
  selectors.
- Selection stays after compile output/manifest generation; SLICE does not own
  Markdown/source fidelity, directives, rendering, compile graph, or artifact
  manifest production.

## Validation

- PROOF: `cargo test -q --locked --manifest-path C:\src\proof\Cargo.toml --test slice_artifact_selector`
- PROOF: `cargo clippy --locked --manifest-path C:\src\proof\Cargo.toml -- -D warnings`
- PROOF: `cargo test -q --locked --manifest-path C:\src\proof\Cargo.toml`
- PROOF: `git diff --check`

## Status

Done.
